import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.nff98.cakami.model"
    compileSdk = 35

    defaultConfig {
        minSdk = 24
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
    // SAF picker 的 @ActivityCallback 需要 AndroidX ActivityResult 型別。
    implementation("androidx.activity:activity-ktx:1.10.1")
    // 0.16.1 使用 Kotlin metadata 2.3；Tauri Android 模板 Kotlin 2.2.x 可讀。
    // 0.17+ 的 metadata 2.4 須 Kotlin 2.3+，不可無審核改成 latest.release。
    implementation("com.google.ai.edge.litertlm:litertlm-android:0.16.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
}
