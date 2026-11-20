package com.nff98.cakami.ocr

/** Stable on-device model contract, never a cloud service or implicit migration. */
internal object PaddleOcrContract {
    const val ENGINE = "paddle_ocrv6_small_onnx"
    const val VERSION = "6.0.0"
    const val MIN_ANDROID_API = 26
    fun supportsDevice(api: Int): Boolean = api >= MIN_ANDROID_API
}
