package com.nff98.cakami.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assert.assertThrows
import org.junit.Test
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection

class ModelAutoDownloaderTest {
    @Test
    fun freshDownloadIsNotAppend() {
        assertFalse(ModelDownloadWirePolicy.shouldAppend(
            0L, HttpURLConnection.HTTP_OK, null, ModelAutoDownloader.GEMMA_SIZE
        ))
    }

    @Test
    fun validByteRangeResumeAppendsToPartial() {
        assertTrue(ModelDownloadWirePolicy.shouldAppend(
            1024L, HttpURLConnection.HTTP_PARTIAL,
            "bytes 1024-2047/" + ModelAutoDownloader.GEMMA_SIZE,
            ModelAutoDownloader.GEMMA_SIZE
        ))
    }

    @Test
    fun serverIgnoringRangeRestartsFromZeroRatherThanDuplicating() {
        assertFalse(ModelDownloadWirePolicy.shouldAppend(
            2048L, HttpURLConnection.HTTP_OK, null, ModelAutoDownloader.GEMMA_SIZE
        ))
    }

    @Test
    fun badResumeRangeNeverCorruptsPartial() {
        for (bad in listOf(
            "bytes 0-1023/" + ModelAutoDownloader.GEMMA_SIZE,
            "bytes 2048-2047/" + ModelAutoDownloader.GEMMA_SIZE,
            "bytes 2048-4095/1000000000",
            "bytes 2048-999999999999/" + ModelAutoDownloader.GEMMA_SIZE,
            "invalid",
        )) {
            assertThrows(IOException::class.java) {
                ModelDownloadWirePolicy.shouldAppend(
                    2048L, HttpURLConnection.HTTP_PARTIAL, bad,
                    ModelAutoDownloader.GEMMA_SIZE,
                )
            }
        }
        assertThrows(IOException::class.java) {
            ModelDownloadWirePolicy.shouldAppend(2048L, 416, null, ModelAutoDownloader.GEMMA_SIZE)
        }
    }

    @Test
    fun exactChecksumMustMatchEvenWhenFileLooksComplete() {
        val file = File.createTempFile("cakami-model-test", ".partial")
        try {
            file.writeText("hello")
            assertTrue(ModelDownloadWirePolicy.matchesSha256(
                file, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
            ))
            assertFalse(ModelDownloadWirePolicy.matchesSha256(file, "0".repeat(64)))
            assertFalse(ModelDownloadWirePolicy.matchesSha256(file, "not-a-hash"))
        } finally {
            file.delete()
        }
    }

    @Test
    fun officialArtifactIsPinnedToKnownVersionAndExactSize() {
        assertTrue(ModelAutoDownloader.GEMMA_URL.contains(
            "/resolve/6e5c4f1e395deb959c494953478fa5cec4b8008f/"
        ))
        assertTrue(ModelAutoDownloader.GEMMA_URL.startsWith("https://huggingface.co/"))
        assertEquals(2_588_147_712L, ModelAutoDownloader.GEMMA_SIZE)
    }
    @Test
    fun updateReusesExistingVerifiedE2BWithoutDownload() {
        val dir = java.nio.file.Files.createTempDirectory("cakami-reuse").toFile()
        val model = File(dir, "gemma-4-E2B-it.litertlm")
        val marker = File(dir, "gemma-4-E2B-it.litertlm.sha256")
        val hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        try {
            model.writeText("hello")
            marker.writeText(hash)
            assertEquals(
                ModelBootAction.USE_INSTALLED,
                ModelReusePolicy.decide(model, marker, 5L, hash),
            )
        } finally {
            dir.deleteRecursively()
        }
    }

    @Test
    fun updateWithMissingMarkerVerifiesLocalModelRatherThanRedownloading() {
        val dir = java.nio.file.Files.createTempDirectory("cakami-recover").toFile()
        val model = File(dir, "gemma-4-E2B-it.litertlm")
        val marker = File(dir, "gemma-4-E2B-it.litertlm.sha256")
        val hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        try {
            model.writeText("hello")
            assertEquals(
                ModelBootAction.VERIFY_EXISTING,
                ModelReusePolicy.decide(model, marker, 5L, hash),
            )
            assertTrue(ModelDownloadWirePolicy.matchesSha256(model, hash))
        } finally {
            dir.deleteRecursively()
        }
    }

    @Test
    fun updateNeverSilentlyOverwritesUnexpectedExistingModel() {
        val dir = java.nio.file.Files.createTempDirectory("cakami-guard").toFile()
        val model = File(dir, "gemma-4-E2B-it.litertlm")
        val marker = File(dir, "gemma-4-E2B-it.litertlm.sha256")
        val hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        try {
            assertEquals(
                ModelBootAction.DOWNLOAD_IF_NEEDED,
                ModelReusePolicy.decide(model, marker, 5L, hash),
            )
            model.writeText("bad")
            marker.writeText(hash)
            assertEquals(
                ModelBootAction.VERIFY_EXISTING,
                ModelReusePolicy.decide(model, marker, 5L, hash),
            )
        } finally {
            dir.deleteRecursively()
        }
    }

}
