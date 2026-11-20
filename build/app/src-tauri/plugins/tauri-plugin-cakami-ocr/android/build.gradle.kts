import org.jetbrains.kotlin.gradle.dsl.JvmTarget

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

// Gradle Exec is configuration-cache-safe, unlike doLast closures capturing
// this build script object. The shell step fetches official public weights
// into APK assets during build, never on the user's phone.
val preparePaddleModels = tasks.register<Exec>("preparePaddleModels") {
    workingDir = projectDir
    commandLine("bash", "download_onnx_models.sh")
    outputs.files(
        "src/main/assets/models/det/inference.onnx",
        "src/main/assets/models/rec/inference.onnx",
        "src/main/assets/models/rec/inference.yml",
    )
}
tasks.matching { it.name == "preBuild" }.configureEach {
    dependsOn(preparePaddleModels)
}
