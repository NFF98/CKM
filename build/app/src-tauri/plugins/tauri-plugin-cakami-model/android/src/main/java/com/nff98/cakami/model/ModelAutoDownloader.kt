package com.nff98.cakami.model

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import android.os.StatFs
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * CAKAMI one-stop Gemma provisioning.
 * HTTPS pinned model -> app-private partial file -> Range resume -> SHA-256 -> private install.
 * Only public model bytes are downloaded; no private image/OCR/prompt is uploaded.
 * Process death stops the in-app transfer; next launch resumes the saved partial.
 */
internal class ModelAutoDownloader(
    private val context: Context,
    private val model: File,
    private val checksum: File,
    private val expectedHash: String,
    private val expectedSize: Long,
    private val pinnedUrl: String,
) {
    companion object {
        // Immutable Hugging Face commit containing SHA256 181938... (not mutable main).
        const val GEMMA_URL =
            "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/" +
            "resolve/6e5c4f1e395deb959c494953478fa5cec4b8008f/gemma-4-E2B-it.litertlm"
        const val GEMMA_SIZE = 2_588_147_712L
        const val RESERVE_BYTES = 128L * 1024 * 1024
    }

    private val staging: File get() = File(model.parentFile, model.name + ".partial")
    private val executor = Executors.newSingleThreadExecutor()
    private val running = AtomicBoolean(false)
    private val pauseRequested = AtomicBoolean(false)

    @Volatile private var state: String = "idle"
    @Volatile private var lastError: String? = null
    @Volatile private var transferred: Long = 0L

    fun progress(ready: Boolean): DownloadProgress {
        val count = when {
            ready -> expectedSize
            running.get() -> transferred
            else -> staging.takeIf { it.isFile }?.length() ?: 0L
        }
        return DownloadProgress(
            state = if (ready) "ready" else if (!running.get() && state == "idle" && count > 0)
                "paused" else state,
            downloadedBytes = count.coerceIn(0L, expectedSize),
            totalBytes = expectedSize,
            error = lastError,
        )
    }

    /**
     * App updates MUST reuse the existing app-private E2B. If model bytes
     * already exist, repair the checksum marker offline before ANY network
     * request. A model whose hash is wrong is never silently overwritten.
     */
    fun prepare(allowMetered: Boolean, ready: Boolean): DownloadProgress {
        val action = ModelReusePolicy.decide(model, checksum, expectedSize, expectedHash)
        if (ready && action == ModelBootAction.USE_INSTALLED ||
            action == ModelBootAction.USE_INSTALLED) {
            state = "ready"
            lastError = null
            return progress(true)
        }
        if (Build.VERSION.SDK_INT < 26) {
            state = "error"
            lastError = "MODEL_UNAVAILABLE"
            return progress(false)
        }
        if (action == ModelBootAction.VERIFY_EXISTING) {
            if (!running.compareAndSet(false, true)) return progress(false)
            pauseRequested.set(false)
            state = "verifying"
            lastError = null
            executor.execute {
                try {
                    if (model.length() == expectedSize &&
                        ModelDownloadWirePolicy.matchesSha256(model, expectedHash)) {
                        writeMarker()
                        staging.delete()
                        transferred = expectedSize
                        state = "ready"
                    } else {
                        state = "error"
                        lastError = "MODEL_HASH_MISMATCH"
                    }
                } catch (_: Exception) {
                    state = "error"
                    lastError = "MODEL_IMPORT_FAILED"
                } finally {
                    running.set(false)
                }
            }
            return progress(false)
        }
        if (!hasNetwork(allowMetered)) {
            if (!running.get()) {
                state = "waiting_wifi"
                lastError = null
            }
            return progress(false)
        }
        if (!running.compareAndSet(false, true)) return progress(false)
        pauseRequested.set(false)
        state = "downloading"
        lastError = null
        transferred = staging.takeIf { it.isFile }?.length() ?: 0L
        executor.execute {
            try {
                downloadAndVerify(allowMetered)
                state = "ready"
                lastError = null
            } catch (_: DownloadPaused) {
                state = "paused"
                lastError = null
            } catch (_: InsufficientSpace) {
                state = "error"
                lastError = "MODEL_NO_SPACE"
            } catch (_: HashMismatch) {
                staging.delete()
                state = "error"
                lastError = "MODEL_HASH_MISMATCH"
            } catch (_: IOException) {
                state = if (hasNetwork(allowMetered)) "error" else "waiting_wifi"
                lastError = if (state == "waiting_wifi") null else "MODEL_DOWNLOAD_FAILED"
            } catch (_: Exception) {
                state = "error"
                lastError = "MODEL_DOWNLOAD_FAILED"
            } finally {
                running.set(false)
            }
        }
        return progress(false)
    }

    fun pause(ready: Boolean): DownloadProgress {
        pauseRequested.set(true)
        if (!running.get() && !ready) {
            state = "paused"
            lastError = null
        }
        return progress(ready)
    }

    private fun hasNetwork(allowMetered: Boolean): Boolean {
        val manager = context.getSystemService(Context.CONNECTIVITY_SERVICE)
            as? ConnectivityManager ?: return false
        val network = manager.activeNetwork ?: return false
        val capability = manager.getNetworkCapabilities(network) ?: return false
        if (!capability.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)) return false
        return allowMetered || !manager.isActiveNetworkMetered
    }

    private fun requireSpace(remaining: Long) {
        if (!model.parentFile.exists() && !model.parentFile.mkdirs()) {
            throw IOException("Cannot create private model directory")
        }
        if (StatFs(model.parentFile.absolutePath).availableBytes < remaining + RESERVE_BYTES) {
            throw InsufficientSpace()
        }
    }

    private fun downloadAndVerify(allowMetered: Boolean) {
        if (!model.parentFile.exists() && !model.parentFile.mkdirs()) {
            throw IOException("Private storage unavailable")
        }
        var start = staging.takeIf { it.isFile }?.length() ?: 0L
        if (start > expectedSize) { staging.delete(); start = 0L }
        // If the process died after installing the verified model but before the
        // tiny checksum marker was committed, recover it without re-downloading 2.59 GB.
        if (model.isFile && model.length() == expectedSize && !checksum.isFile) {
            state = "verifying"
            if (ModelDownloadWirePolicy.matchesSha256(model, expectedHash)) {
                writeMarker()
                transferred = expectedSize
                staging.delete()
                return
            }
        }
        requireSpace(expectedSize - start)
        if (start < expectedSize) {
            if (pauseRequested.get()) throw DownloadPaused()
            val url = URL(pinnedUrl)
            require(url.protocol == "https" && url.host == "huggingface.co")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                instanceFollowRedirects = true
                connectTimeout = 20_000
                readTimeout = 60_000
                setRequestProperty("Accept-Encoding", "identity")
                setRequestProperty("User-Agent", "CAKAMI-Android-GemmaModel/0.1")
                if (start > 0L) setRequestProperty("Range", "bytes=$start-")
            }
            try {
                val code = conn.responseCode
                if (conn.url.protocol != "https") throw IOException("Unsafe redirect")
                val append = ModelDownloadWirePolicy.shouldAppend(
                    start, code, conn.getHeaderField("Content-Range"), expectedSize,
                )
                if (!append) {
                    // A server can ignore Range. Discard partial first to reclaim disk
                    // space before checking capacity for a complete restart.
                    if (staging.exists() && !staging.delete()) {
                        throw IOException("Cannot reset partial")
                    }
                    start = 0L
                    requireSpace(expectedSize)
                }
                transferred = start
                conn.inputStream.use { input ->
                    FileOutputStream(staging, append).use { output ->
                        try {
                            val buf = ByteArray(256 * 1024)
                            while (true) {
                                if (pauseRequested.get()) throw DownloadPaused()
                                if (!hasNetwork(allowMetered)) throw IOException("Network changed")
                                val n = input.read(buf)
                                if (n < 0) break
                                if (transferred + n > expectedSize) throw HashMismatch()
                                output.write(buf, 0, n)
                                transferred += n
                            }
                        } finally {
                            // Keep resume bytes durable on network interruption/pause.
                            output.fd.sync()
                        }
                    }
                }
            } finally { conn.disconnect() }
        }
        if (staging.length() != expectedSize) throw IOException("Incomplete model")
        state = "verifying"
        if (!ModelDownloadWirePolicy.matchesSha256(staging, expectedHash)) throw HashMismatch()
        if (pauseRequested.get()) throw DownloadPaused()

        if (model.isFile && checksum.isFile && checksum.readText().trim() == expectedHash) {
            staging.delete()
            return
        }
        if (model.exists() && !model.delete()) throw IOException("Cannot replace invalid model")
        if (!staging.renameTo(model)) throw IOException("Cannot install model")
        writeMarker()
        transferred = expectedSize
    }

    private fun writeMarker() {
        val tmp = File(model.parentFile, checksum.name + ".tmp")
        tmp.writeText(expectedHash)
        if (checksum.exists() && !checksum.delete()) throw IOException("Cannot replace checksum")
        if (!tmp.renameTo(checksum)) throw IOException("Cannot mark model verified")
    }

    private class DownloadPaused : IOException()
    private class InsufficientSpace : IOException()
    private class HashMismatch : IOException()
}

internal data class DownloadProgress(
    val state: String,
    val downloadedBytes: Long,
    val totalBytes: Long,
    val error: String?,
)

/** Pure HTTP Range & SHA validation, unit-testable without downloading a 2.59GB model. */
internal object ModelDownloadWirePolicy {
    private val rangePattern = Regex("^bytes (\\d+)-(\\d+)/(\\d+)$")

    fun shouldAppend(start: Long, code: Int, range: String?, total: Long): Boolean {
        if (start < 0L || start >= total) throw IOException("Invalid resume offset")
        // CDN ignored Range. Restart from 0 rather than append duplicate bytes.
        if (code == HttpURLConnection.HTTP_OK) return false
        if (code != HttpURLConnection.HTTP_PARTIAL) throw IOException("Unexpected HTTP status")
        val parsed = rangePattern.matchEntire(range ?: "")
            ?: throw IOException("Missing Content-Range")
        val a = parsed.groupValues[1].toLongOrNull() ?: throw IOException("Bad start")
        val b = parsed.groupValues[2].toLongOrNull() ?: throw IOException("Bad end")
        val c = parsed.groupValues[3].toLongOrNull() ?: throw IOException("Bad size")
        if (a != start || b < a || b >= c || c != total) {
            throw IOException("Wrong Content-Range")
        }
        return start > 0L
    }

    fun matchesSha256(file: File, hex: String): Boolean {
        val md = MessageDigest.getInstance("SHA-256")
        file.inputStream().buffered().use { input ->
            val bytes = ByteArray(256 * 1024)
            while (true) {
                val n = input.read(bytes)
                if (n < 0) break
                md.update(bytes, 0, n)
            }
        }
        val expected = try {
            hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray()
        } catch (_: NumberFormatException) { return false }
        return MessageDigest.isEqual(md.digest(), expected)
    }
}


/** Pure, testable policy: an Android APK update never triggers a second E2B
 * download when the existing app's private model bytes are present. */
internal enum class ModelBootAction {
    USE_INSTALLED,
    VERIFY_EXISTING,
    DOWNLOAD_IF_NEEDED,
}

internal object ModelReusePolicy {
    fun decide(model: File, checksum: File, expectedSize: Long, expectedHash: String): ModelBootAction {
        if (!model.isFile) return ModelBootAction.DOWNLOAD_IF_NEEDED
        if (model.length() != expectedSize) return ModelBootAction.VERIFY_EXISTING
        val markerMatches = try {
            checksum.isFile && checksum.readText().trim() == expectedHash
        } catch (_: IOException) {
            false
        }
        return if (markerMatches) ModelBootAction.USE_INSTALLED
        else ModelBootAction.VERIFY_EXISTING
    }
}
