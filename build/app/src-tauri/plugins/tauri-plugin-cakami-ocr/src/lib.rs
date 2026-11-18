//! Rust 唯一控制的 Android 原生 OCR 插件。所有輸入由 App-private SQLite Asset ID 對應，
//! 插件從來不接觸網路；非 Android 平台明確回報未提供 Adapter。
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(target_os = "android")]
use tauri::{
    plugin::PluginHandle,
    Manager,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRequest {
    pub base64: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrSegment {
    pub text: String,
    pub bounds: [i32; 4],
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeOcrResult {
    pub text: String,
    pub segments: Vec<OcrSegment>,
    pub engine: String,
    pub engine_version: String,
    pub elapsed_ms: u64,
}

#[cfg(target_os = "android")]
struct AndroidOcr<R: Runtime>(PluginHandle<R>);

/// Rust↔Kotlin bridge；不向前端提供原生插件直連命令。
pub fn recognize<R: Runtime>(app: &tauri::AppHandle<R>, base64: String) -> Result<NativeOcrResult, String> {
    #[cfg(target_os = "android")]
    {
        return app.state::<AndroidOcr<R>>()
            .0
            .run_mobile_plugin("recognize", OcrRequest { base64 })
            .map_err(|_| "OCR_RECOGNITION_FAILED".to_string());
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, base64);
        Err("OCR_MODEL_UNAVAILABLE".into())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("cakami-ocr")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            {
                let handle = api.register_android_plugin("com.nff98.cakami.ocr", "CakamiOcrPlugin")?;
                app.manage(AndroidOcr(handle));
            }
            #[cfg(not(target_os = "android"))]
            let _ = (app, api);
            Ok(())
        })
        .build()
}
