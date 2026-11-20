package com.nff98.cakami.ocr

import android.app.Activity
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Matrix
import android.os.Build
import android.os.SystemClock
import android.util.Base64
import androidx.exifinterface.media.ExifInterface
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.paddle.ocr.PaddleOCR
import com.paddle.ocr.util.OpenCVUtils
import java.io.ByteArrayInputStream
import java.util.concurrent.Executors
import kotlinx.coroutines.runBlocking

@InvokeArg
class RecognizeArgs {
    var base64: String = ""
}

/**
 * CAKAMI B03. All OCR is local: official PaddleOCR v6 Small ONNX models are
 * bundled inside the APK; the private picture never leaves Android storage.
 * Rust still owns the SQLite OCR record, revisions and provenance.
 */
@TauriPlugin
class CakamiOcrPlugin(private val activity: Activity): Plugin(activity) {
    private val executor = Executors.newSingleThreadExecutor()

    companion object {
        const val OCR_ENGINE = "paddle_ocrv6_small_onnx"
        const val OCR_ENGINE_VERSION = "6.0.0"
        const val MAX_IMAGE_BYTES = 8 * 1024 * 1024
        const val MAX_IMAGE_PIXELS = 16_000_000L
    }

    @Command
    fun recognize(invoke: Invoke) {
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
            var bitmap: Bitmap? = null
            var rotated: Bitmap? = null
            try {
                if (Build.VERSION.SDK_INT < 26) {
                    invoke.reject("OCR_MODEL_UNAVAILABLE")
                    return@execute
                }
                val bytes = Base64.decode(args.base64, Base64.NO_WRAP)
                if (bytes.isEmpty() || bytes.size > MAX_IMAGE_BYTES) {
                    invoke.reject("OCR_INVALID_INPUT")
                    return@execute
                }
                val info = BitmapFactory.Options().apply { inJustDecodeBounds = true }
                BitmapFactory.decodeByteArray(bytes, 0, bytes.size, info)
                if (info.outWidth <= 0 || info.outHeight <= 0 ||
                    info.outWidth.toLong() * info.outHeight.toLong() > MAX_IMAGE_PIXELS) {
                    invoke.reject("OCR_IMAGE_TOO_LARGE")
                    return@execute
                }
                bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
                if (bitmap == null) {
                    invoke.reject("OCR_IMAGE_INVALID")
                    return@execute
                }
                val degrees = try {
                    ExifInterface(ByteArrayInputStream(bytes)).rotationDegrees
                } catch (_: Exception) {
                    0
                }
                val input = if (degrees == 90 || degrees == 180 || degrees == 270) {
                    Matrix().apply { postRotate(degrees.toFloat()) }.let { m ->
                        Bitmap.createBitmap(bitmap!!, 0, 0, bitmap!!.width, bitmap!!.height, m, true)
                    }.also { rotated = it }
                } else bitmap!!

                // OpenCV and ONNX Runtime both run inside this Android process.
                if (!OpenCVUtils.init(activity)) {
                    invoke.reject("OCR_RECOGNITION_FAILED")
                    return@execute
                }
                // PaddleOCR.create/recognize are suspend functions. Do not block
                // the Tauri UI thread: this entire path runs in an executor.
                val result = runBlocking {
                    val engine = PaddleOCR.create(activity)
                    try {
                        engine.recognize(input)
                    } finally {
                        engine.release()
                    }
                }
                val segments = JSArray()
                for (item in result.results.take(500)) {
                    val p = item.box.points
                    val left = p.minOf { it.x }.toInt()
                    val top = p.minOf { it.y }.toInt()
                    val right = p.maxOf { it.x }.toInt()
                    val bottom = p.maxOf { it.y }.toInt()
                    val row = JSObject()
                    row.put("text", item.text)
                    val bounds = JSArray()
                    bounds.put(left)
                    bounds.put(top)
                    bounds.put(right)
                    bounds.put(bottom)
                    row.put("bounds", bounds)
                    segments.put(row)
                }
                val response = JSObject()
                response.put("text", result.results.take(500).joinToString("\n") { it.text })
                response.put("segments", segments)
                response.put("engine", OCR_ENGINE)
                response.put("engineVersion", OCR_ENGINE_VERSION)
                response.put("elapsedMs", SystemClock.elapsedRealtime() - started)
                invoke.resolve(response)
            } catch (_: OutOfMemoryError) {
                invoke.reject("OCR_OUT_OF_MEMORY")
            } catch (_: Exception) {
                invoke.reject("OCR_RECOGNITION_FAILED")
            } finally {
                if (rotated != null && rotated !== bitmap) rotated?.recycle()
                bitmap?.recycle()
            }
        }
    }
}
