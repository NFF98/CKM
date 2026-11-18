mod local_store;
mod platform;
use platform::NativeAdapter;
use serde::Serialize;
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeHealth {
    contract_version: u8,
    bridge: &'static str,
    platform: &'static str,
    app_version: &'static str,
    network_required: bool,
    cloud_ai_enabled: bool,
    native_ai: &'static str,
}

/// S1-AC01／B01：真實 Rust Command（無網路／無遠端推論）。
/// B03 才會接入本地 OCR／E2B；此回應不能假裝 AI 已可用。
#[tauri::command]
fn bridge_health() -> BridgeHealth {
    let adapter = platform::current_platform();
    BridgeHealth {
        contract_version: 1,
        bridge: "ready",
        platform: adapter.platform_name(),
        app_version: env!("CARGO_PKG_VERSION"),
        network_required: false,
        cloud_ai_enabled: false,
        native_ai: adapter.native_ai_state(),
    }
}

fn private_store(app: &tauri::AppHandle) -> Result<local_store::LocalStore, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    local_store::LocalStore::open(&root)
}

/// B02：由 Rust 控制的 App-private 儲存，前端不能指定儲存路徑。
#[tauri::command]
fn save_drop(
    app: tauri::AppHandle,
    input: local_store::IncomingDrop,
) -> Result<local_store::DropSummary, String> {
    private_store(&app)?.save(input)
}

#[tauri::command]
fn list_drops(app: tauri::AppHandle) -> Result<Vec<local_store::DropSummary>, String> {
    private_store(&app)?.list()
}

#[tauri::command]
fn load_drop(app: tauri::AppHandle, id: String) -> Result<local_store::DropDetail, String> {
    private_store(&app)?.load(&id)
}

/// 刪除操作只能使用既有 UUID，檔案和 SQLite 由 Rust 共同維護。
#[tauri::command]
fn delete_drop(app: tauri::AppHandle, id: String) -> Result<bool, String> {
    private_store(&app)?.delete(&id)
}

/// B03：只接受 Rust 私人 Asset UUID，模型回傳才可寫入 OCR；絕不經遠端 OCR。
#[tauri::command]
async fn local_ocr_recognize(
    app: tauri::AppHandle,
    id: String,
) -> Result<local_store::OcrRecord, String> {
    let detail = private_store(&app)?.load(&id)?;
    if detail.summary.kind != "image" {
        return Err("OCR_IMAGE_REQUIRED".into());
    }
    let base64 = detail.data_base64.ok_or("ASSET_FILE_MISSING")?;
    let native = tauri_plugin_cakami_ocr::recognize(&app, base64)?;
    private_store(&app)?.save_recognition(&id, native)
}

/// B03：重新開啟 App 時從私有 SQLite 讀回原 OCR 與更正文字。
#[tauri::command]
fn load_ocr(app: tauri::AppHandle, id: String) -> Result<local_store::OcrRecord, String> {
    private_store(&app)?.load_ocr(&id)
}

/// B03：更正是 UI 真人的明確操作；不代表 Goal／分享／模型確認。
#[tauri::command]
fn correct_ocr(
    app: tauri::AppHandle,
    id: String,
    corrected_text: String,
) -> Result<local_store::OcrRecord, String> {
    private_store(&app)?.correct_ocr(&id, &corrected_text)
}

/// E2B 尚未有已驗證可部署的 Gemma Runtime／模型，明確拒絕，絕不使用 Cloud fallback。
#[tauri::command]
fn local_model_propose(_app: tauri::AppHandle, _id: String) -> Result<(), String> {
    Err("MODEL_UNAVAILABLE".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_cakami_ocr::init())
        .invoke_handler(tauri::generate_handler![
            bridge_health,
            local_ocr_recognize,
            load_ocr,
            correct_ocr,
            local_model_propose,
            save_drop,
            list_drops,
            load_drop,
            delete_drop
        ])
        .run(tauri::generate_context!())
        .expect("無法啟動 CAKAMI Tauri App");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b01_rust_bridge_contract_is_local() {
        let status = bridge_health();
        assert_eq!(status.contract_version, 1);
        assert_eq!(status.bridge, "ready");
        assert!(!status.platform.is_empty());
        assert!(!status.network_required);
        assert!(!status.cloud_ai_enabled);
        assert_eq!(status.native_ai, "not_installed");
    }

    #[test]
    fn b01_ts_contract_serialization() {
        let json = serde_json::to_value(bridge_health()).expect("應可序列化");
        assert_eq!(json["contractVersion"], 1);
        assert_eq!(json["networkRequired"], false);
        assert_eq!(json["cloudAiEnabled"], false);
        assert_eq!(json["nativeAi"], "not_installed");
    }
}
