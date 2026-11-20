import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.nff98.cakami.ocr"
    compileSdk = 35

    defaultConfig {
        minSdk = 23
        consumerProguardFiles("consumer-rules.pro")
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_1_8
    }
}

dependencies {
    implementation(project(":tauri-android"))
    // bundled variant：辨識模型已包含在 App，飛航模式不需下載
    implementation("com.google.mlkit:text-recognition-japanese:16.0.1")
    // Read JPEG/PNG/WebP orientation offline; no external OCR service.
    implementation("androidx.exifinterface:exifinterface:1.4.2")
    testImplementation("junit:junit:4.13.2")
}
