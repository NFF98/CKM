// Android ML Kit 隨 App 打包，不依賴 Play Services 下載模型。
fn main() {
    tauri_plugin::Builder::new(&["recognize"])
        .android_path("android")
        .build();
}
