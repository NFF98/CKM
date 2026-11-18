package com.nff98.cakami.ocr

import android.app.Activity
import android.graphics.BitmapFactory
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
                val image = InputImage.fromBitmap(bitmap, 0)
                val settled = AtomicBoolean(false)
                val handler = Handler(Looper.getMainLooper())
                val timeout = Runnable {
                    if (settled.compareAndSet(false, true)) {
                        invoke.reject("OCR_TIMEOUT")
                    }
                }
                handler.postDelayed(timeout, 20_000L)
                client.process(image)
                    .addOnSuccessListener { result ->
                        try {
                            if (!settled.compareAndSet(false, true)) return@addOnSuccessListener
                            handler.removeCallbacks(timeout)
                            val segments = JSArray()
                            result.textBlocks.forEach { block ->
                                block.lines.forEach { line ->
                                    val b = line.boundingBox
                                    val seg = JSObject()
                                    seg.put("text", line.text)
                                    val rect = JSArray()
                                    rect.put(b?.left ?: 0)
                                    rect.put(b?.top ?: 0)
                                    rect.put(b?.right ?: 0)
                                    rect.put(b?.bottom ?: 0)
                                    seg.put("bounds", rect)
                                    segments.put(seg)
                                }
                            }
                            val response = JSObject()
                            response.put("text", result.text)
                            response.put("segments", segments)
                            response.put("engine", "mlkit_japanese_bundled")
                            response.put("engineVersion", "16.0.1")
                            response.put("elapsedMs", SystemClock.elapsedRealtime() - started)
                            invoke.resolve(response)
                        } catch (_: Exception) {
                            invoke.reject("OCR_RECOGNITION_FAILED")
                        } finally {
                            client.close()
                            bitmap.recycle()
                        }
                    }
                    .addOnFailureListener {
                        handler.removeCallbacks(timeout)
                        client.close()
                        bitmap.recycle()
                        if (settled.compareAndSet(false, true)) {
                            invoke.reject("OCR_RECOGNITION_FAILED")
                        }
                    }
            } catch (_: OutOfMemoryError) {
                invoke.reject("OCR_OUT_OF_MEMORY")
            } catch (_: Exception) {
                invoke.reject("OCR_RECOGNITION_FAILED")
            }
        }
    }
}
