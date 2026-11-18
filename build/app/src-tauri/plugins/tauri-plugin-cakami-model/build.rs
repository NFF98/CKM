// Kotlin Android plugin commands，模型檔從使用者挑選後本地匯入。
fn main() {
    tauri_plugin::Builder::new(&["modelStatus", "importModel", "propose"])
        .android_path("android")
        .build();
}
