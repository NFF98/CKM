//! B03 Gemma 4 E2B：Rust 控制的裝置端 LiteRT-LM 插件。
//! Rust 從本地 SQLite 提供已驗證內容，Kotlin 只在使用者裝置推論。
//! 桌面／iOS Adapter 尚未實作時明確回報 MODEL_UNAVAILABLE。
use serde::{Deserialize, Serialize};
use tauri::{plugin::{Builder, TauriPlugin}, Runtime};

#[cfg(target_os = "android")]
use tauri::{plugin::PluginHandle, Manager};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelRequest {
    text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareRequest {
    allow_metered: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub model_id: String,
    pub runtime: String,
    pub bytes: u64,
    pub integrity_verified: bool,
    pub download_state: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub download_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeReply {
    pub raw_text: String,
    pub model_id: String,
    pub runtime: String,
    pub backend: String,
    pub elapsed_ms: u64,
}

#[cfg(target_os = "android")]
struct AndroidModel<R: Runtime>(PluginHandle<R>);

/// 只接受已知的原生錯誤碼；不將路徑、堆疊、私人文字暴露給 WebView。
#[cfg(target_os = "android")]
fn native_error(error: impl std::fmt::Display) -> String {
    let message = error.to_string();
    for code in [
        "MODEL_NOT_INSTALLED", "MODEL_IMPORT_CANCELLED", "MODEL_INVALID_FILE",
        "MODEL_HASH_MISMATCH", "MODEL_NO_SPACE", "MODEL_IMPORT_FAILED",
        "MODEL_LOAD_FAILED", "MODEL_INFERENCE_FAILED", "MODEL_TIMEOUT",
        "MODEL_OUT_OF_MEMORY", "MODEL_BUSY", "MODEL_INPUT_INVALID",
        "MODEL_DOWNLOAD_FAILED", "MODEL_DOWNLOAD_NETWORK",
    ] {
        if message.contains(code) {
            return code.to_owned();
        }
    }
    "MODEL_INFERENCE_FAILED".into()
}

pub fn status<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<ModelStatus, String> {
    #[cfg(target_os = "android")]
    {
        return app.state::<AndroidModel<R>>().0
            .run_mobile_plugin("modelStatus", ())
            .map_err(native_error);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Ok(ModelStatus {
            installed: false, model_id: "gemma-4-E2B-it".into(),
            runtime: "litert-lm".into(), bytes: 0, integrity_verified: false,
            download_state: "unavailable".into(), downloaded_bytes: 0,
            total_bytes: 2_588_147_712, download_error: None,
        })
    }
}

/// 初次開啟 App 即啟動模型準備。Android 原生插件立即回覆，
/// 傳輸由 app-private worker 執行，Wi-Fi 預設保護且可從 .partial 續傳。
pub fn prepare<R: Runtime>(
    app: &tauri::AppHandle<R>,
    allow_metered: bool,
) -> Result<ModelStatus, String> {
    #[cfg(target_os = "android")]
    {
        return app.state::<AndroidModel<R>>().0
            .run_mobile_plugin("prepareModel", PrepareRequest { allow_metered })
            .map_err(native_error);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, allow_metered);
        Err("MODEL_UNAVAILABLE".into())
    }
}

/// 使用者可暫停大型下載；模型未經 checksum 驗證不會顯示 Ready。
pub fn pause<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<ModelStatus, String> {
    #[cfg(target_os = "android")]
    {
        return app.state::<AndroidModel<R>>().0
            .run_mobile_plugin("pauseModelDownload", ())
            .map_err(native_error);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("MODEL_UNAVAILABLE".into())
    }
}

/// SAF 選檔由 Android 原生界面完成；前端不能指定任意本地路徑。
pub fn import_model<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<ModelStatus, String> {
    #[cfg(target_os = "android")]
    {
        return app.state::<AndroidModel<R>>().0
            .run_mobile_plugin("importModel", ())
            .map_err(native_error);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        Err("MODEL_UNAVAILABLE".into())
    }
}

pub fn propose<R: Runtime>(app: &tauri::AppHandle<R>, text: String) -> Result<NativeReply, String> {
    if text.trim().is_empty() || text.len() > 8 * 1024 {
        return Err("MODEL_INPUT_INVALID".into());
    }
    #[cfg(target_os = "android")]
    {
        return app.state::<AndroidModel<R>>().0
            .run_mobile_plugin("propose", ModelRequest { text })
            .map_err(native_error);
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app, text);
        Err("MODEL_UNAVAILABLE".into())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("cakami-model")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            {
                let handle = api.register_android_plugin("com.nff98.cakami.model", "CakamiModelPlugin")?;
                app.manage(AndroidModel(handle));
            }
            #[cfg(not(target_os = "android"))]
            let _ = (app, api);
            Ok(())
        })
        .build()
}
