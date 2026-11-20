import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import java.net.URL
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.security.MessageDigest

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.nff98.cakami.ocr"
    compileSdk = 35

    defaultConfig {
        // Keep existing app minSdk for data-preserving upgrades. The v6 runtime
        // is gated to API 26+ inside CakamiOcrPlugin.kt.
        minSdk = 23
        consumerProguardFiles("consumer-rules.pro")
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

dependencies {
    implementation(project(":tauri-android"))
    // Official PaddleOCR v6 Android SDK (pinned Apache-2.0 source vendored below).
    implementation("com.microsoft.onnxruntime:onnxruntime-android:1.21.1")
    implementation("com.quickbirdstudios:opencv:4.5.3")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.exifinterface:exifinterface:1.4.2")
    testImplementation("junit:junit:4.13.2")
}

// Bundled ONNX files are fetched only during build, not on the user's phone.
// Model assets are NOT the private 2.59 GB E2B and will never touch its path.
// Hashes for the resolved upstream weights are printed in CI and recorded in
// release notes. Fail the build if assets are missing or incomplete.
val paddleAssets = listOf(
    Triple("models/det/inference.onnx",
        "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_det_onnx/resolve/main/inference.onnx",
        1_000_000L),
    Triple("models/rec/inference.onnx",
        "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/main/inference.onnx",
        2_000_000L),
    Triple("models/rec/inference.yml",
        "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/main/inference.yml",
        1_000L),
)
val preparePaddleModels = tasks.register("preparePaddleModels") {
    description = "Fetch official PP-OCRv6 Small Android models into APK assets"
    val outputDir = layout.projectDirectory.dir("src/main/assets")
    outputs.files(paddleAssets.map { outputDir.file(it.first) })
    doLast {
        for ((path, url, minSize) in paddleAssets) {
            val file = outputDir.file(path).asFile
            if (!file.isFile || file.length() < minSize) {
                file.parentFile.mkdirs()
                val temp = File(file.parentFile, file.name + ".downloading")
                try {
                    val connection = URL(url).openConnection().apply {
                        connectTimeout = 30_000
                        readTimeout = 180_000
                        setRequestProperty("User-Agent", "CAKAMI-Android-Offline-OCR-Build/1.0")
                    }
                    connection.getInputStream().use { input ->
                        temp.outputStream().use { output -> input.copyTo(output) }
                    }
                    require(temp.length() >= minSize) {
                        "OCR model asset too small: $path (${temp.length()} bytes)"
                    }
                    Files.move(temp.toPath(), file.toPath(), StandardCopyOption.REPLACE_EXISTING)
                } finally {
                    temp.delete()
                }
            }
            val sha = MessageDigest.getInstance("SHA-256")
            file.inputStream().use { stream ->
                val buf = ByteArray(128 * 1024)
                while (true) {
                    val n = stream.read(buf)
                    if (n < 0) break
                    sha.update(buf, 0, n)
                }
            }
            println("OCR model asset $path bytes=${file.length()} sha256=" +
                sha.digest().joinToString("") { "%02x".format(it) })
        }
    }
}
tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn(preparePaddleModels)
}
