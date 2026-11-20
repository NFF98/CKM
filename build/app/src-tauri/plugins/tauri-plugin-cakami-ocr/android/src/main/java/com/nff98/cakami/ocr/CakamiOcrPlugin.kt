package com.nff98.cakami.ocr

import android.app.Activity
import android.graphics.BitmapFactory
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.ColorMatrixColorFilter
import com.google.mlkit.vision.text.Text
import androidx.exifinterface.media.ExifInterface
import java.io.ByteArrayInputStream
import android.os.SystemClock
import android.os.Handler
import android.os.Looper
import android.util.Base64
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.google.mlkit.vision.common.InputImage
import com.google.mlkit.vision.text.TextRecognition
import com.google.mlkit.vision.text.japanese.JapaneseTextRecognizerOptions
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

@InvokeArg
class RecognizeArgs {
    var base64: String = ""
}

@TauriPlugin
class CakamiOcrPlugin(private val activity: Activity) : Plugin(activity) {
    private val executor = Executors.newSingleThreadExecutor()

    @Command
    fun recognize(invoke: Invoke) {
        // Kotlin Plugin commands start on the UI thread; decoding must happen in worker.
        val args = try {
            invoke.parseArgs(RecognizeArgs::class.java)
        } catch (_: Exception) {
            invoke.reject("OCR_INVALID_INPUT")
            return
        }
        if (args.base64.isEmpty() || args.base64.length > 12_000_000) {
            invoke.reject("OCR_INVALID_INPUT")
            return
        }
        executor.execute {
            val started = SystemClock.elapsedRealtime()
            try {
                val bytes = Base64.decode(args.base64, Base64.NO_WRAP)
                if (bytes.isEmpty() || bytes.size > 8 * 1024 * 1024) {
                    invoke.reject("OCR_INVALID_INPUT")
                    return@execute
                }
                val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
                BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
                if (bounds.outWidth <= 0 || bounds.outHeight <= 0 ||
                    bounds.outWidth.toLong() * bounds.outHeight.toLong() > 16_000_000L) {
                    invoke.reject("OCR_IMAGE_TOO_LARGE")
                    return@execute
                }
                val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
                if (bitmap == null) {
                    invoke.reject("OCR_IMAGE_INVALID")
                    return@execute
                }
                val client = TextRecognition.getClient(
                    JapaneseTextRecognizerOptions.Builder().build()
                )
                // BitmapFactory does NOT apply the JPEG/HEIC/PNG/WebP EXIF rotation.
                // The old hard-coded 0 degrees caused sideways phone photos to be
                // recognized as horizontal Japanese text. Read only local image bytes.
                val degrees = try {
                    ExifInterface(ByteArrayInputStream(bytes)).rotationDegrees
                } catch (_: Exception) {
                    0
                }
                val safeDegrees = if (degrees in listOf(0, 90, 180, 270)) degrees else 0
                val image = InputImage.fromBitmap(bitmap, safeDegrees)
                val settled = AtomicBoolean(false)
                val handler = Handler(Looper.getMainLooper())
                val timeout = Runnable {
                    if (settled.compareAndSet(false, true)) invoke.reject("OCR_TIMEOUT")
                }
                handler.postDelayed(timeout, 20_000L)

                fun confidence(text: Text): Float {
                    val lines = text.textBlocks.flatMap { it.lines }
                    val reliable = lines.filter { it.confidence > 0f }
                    if (reliable.isEmpty()) return 0f
                    val weighted = reliable.sumOf { it.confidence.toDouble() *
                        it.text.length.coerceAtLeast(1) }
                    val chars = reliable.sumOf { it.text.length.coerceAtLeast(1) }
                    return (weighted / chars).toFloat()
                }

                fun contrast(original: Bitmap): Bitmap {
                    val enhanced = Bitmap.createBitmap(
                        original.width, original.height, Bitmap.Config.ARGB_8888)
                    val filter = ColorMatrixColorFilter(floatArrayOf(
                        1.35f, 0f, 0f, 0f, -30f,
                        0f, 1.35f, 0f, 0f, -30f,
                        0f, 0f, 1.35f, 0f, -30f,
                        0f, 0f, 0f, 1f, 0f
                    ))
                    val paint = Paint(Paint.FILTER_BITMAP_FLAG).apply {
                        colorFilter = filter
                    }
                    Canvas(enhanced).drawBitmap(original, 0f, 0f, paint)
                    return enhanced
                }

                fun finish(text: Text?, extra: Bitmap? = null) {
                    val firstFinish = settled.compareAndSet(false, true)
                    if (firstFinish) {
                        handler.removeCallbacks(timeout)
                        try {
                            if (text == null) {
                                invoke.reject("OCR_RECOGNITION_FAILED")
                            } else {
                                val segments = JSArray()
                                text.textBlocks.forEach { block ->
                                    block.lines.forEach { line ->
                                        val rect = JSArray()
                                        val bounds = line.boundingBox
                                        rect.put(bounds?.left ?: 0)
                                        rect.put(bounds?.top ?: 0)
                                        rect.put(bounds?.right ?: 0)
                                        rect.put(bounds?.bottom ?: 0)
                                        val segment = JSObject()
                                        segment.put("text", line.text)
                                        segment.put("bounds", rect)
                                        segments.put(segment)
                                    }
                                }
                                val response = JSObject()
                                response.put("text", text.text)
                                response.put("segments", segments)
                                response.put("engine", "mlkit_japanese_bundled")
                                response.put("engineVersion", "16.0.1")
                                response.put("elapsedMs", SystemClock.elapsedRealtime() - started)
                                invoke.resolve(response)
                            }
                        } catch (_: Exception) {
                            invoke.reject("OCR_RECOGNITION_FAILED")
                        }
                    }
                    client.close()
                    extra?.recycle()
                    bitmap.recycle()
                }

                client.process(image)
                    .addOnSuccessListener(executor) { first ->
                        val firstConfidence = confidence(first)
                        val pixels = bitmap.width.toLong() * bitmap.height.toLong()
                        val remaining = 20_000L - (SystemClock.elapsedRealtime() - started)
                        val retry = OcrImagePolicy.shouldRetryContrast(
                            firstConfidence, pixels, remaining)
                        if (!retry || settled.get()) {
                            finish(first)
                        } else {
                            val enhanced = try { contrast(bitmap) }
                                catch (_: OutOfMemoryError) { null }
                                catch (_: Exception) { null }
                            if (enhanced == null) {
                                finish(first)
                            } else {
                                client.process(InputImage.fromBitmap(enhanced, safeDegrees))
                                    .addOnSuccessListener(executor) { second ->
                                        val selected = if (OcrImagePolicy.useEnhanced(
                                            confidence(first), confidence(second),
                                            first.text.isBlank(), second.text.isNotBlank()
                                        )) second else first
                                        finish(selected, enhanced)
                                    }
                                    .addOnFailureListener(executor) {
                                        finish(first, enhanced)
                                    }
                            }
                        }
                    }
                    .addOnFailureListener(executor) { finish(null) }
            } catch (_: OutOfMemoryError) {
                invoke.reject("OCR_OUT_OF_MEMORY")
            } catch (_: Exception) {
                invoke.reject("OCR_RECOGNITION_FAILED")
            }
        }
    }
}

/** Pure thresholds for an optional offline low-contrast second pass.
 * The original result always wins unless the retry is materially more confident. */
internal object OcrImagePolicy {
    fun shouldRetryContrast(confidence: Float, pixels: Long, remainingMs: Long): Boolean =
        confidence < 0.65f && pixels in 1..2_500_000L && remainingMs > 6_000L

    fun useEnhanced(firstConfidence: Float, secondConfidence: Float,
                    firstEmpty: Boolean, secondNonEmpty: Boolean): Boolean =
        (firstEmpty && secondNonEmpty && secondConfidence > 0.40f) ||
        (secondConfidence > firstConfidence + 0.08f && secondNonEmpty)
}
