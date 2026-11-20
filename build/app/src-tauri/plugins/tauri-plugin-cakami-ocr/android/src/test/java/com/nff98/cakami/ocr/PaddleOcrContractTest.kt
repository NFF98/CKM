package com.nff98.cakami.ocr

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PaddleOcrContractTest {
    @Test fun engineIdIsPinnedAndNotLegacyMlKit() {
        assertEquals("paddle_ocrv6_small_onnx", PaddleOcrContract.ENGINE)
        assertEquals("6.0.0", PaddleOcrContract.VERSION)
    }
    @Test fun unsupportedOlderPhonesFailClosedWithoutCloudFallback() {
        assertFalse(PaddleOcrContract.supportsDevice(23))
        assertFalse(PaddleOcrContract.supportsDevice(25))
        assertTrue(PaddleOcrContract.supportsDevice(26))
        assertTrue(PaddleOcrContract.supportsDevice(35))
    }
}
