package com.nff98.cakami.ocr

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class OcrImagePolicyTest {
    @Test fun lowConfidenceSmallImageTriggersOfflineRetry() {
        assertTrue(OcrImagePolicy.shouldRetryContrast(0.41f, 1_000_000L, 10_000L))
        assertTrue(OcrImagePolicy.shouldRetryContrast(0.0f, 1_000_000L, 10_000L))
    }

    @Test fun goodScanDoesNotWasteTimeOrMemory() {
        assertFalse(OcrImagePolicy.shouldRetryContrast(0.90f, 1_000_000L, 10_000L))
        assertFalse(OcrImagePolicy.shouldRetryContrast(0.30f, 12_000_000L, 10_000L))
        assertFalse(OcrImagePolicy.shouldRetryContrast(0.30f, 1_000_000L, 1_000L))
    }

    @Test fun failedContrastCannotReplaceBetterOriginal() {
        assertFalse(OcrImagePolicy.useEnhanced(0.82f, 0.72f, false, true))
        assertFalse(OcrImagePolicy.useEnhanced(0.70f, 0.76f, false, true))
        assertTrue(OcrImagePolicy.useEnhanced(0.40f, 0.70f, false, true))
        assertTrue(OcrImagePolicy.useEnhanced(0.0f, 0.61f, true, true))
        assertFalse(OcrImagePolicy.useEnhanced(0.0f, 0.10f, true, true))
    }
}
