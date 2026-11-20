mod local_store;
mod goal_draft;
mod parameter_registry;
mod local_consent;
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

/// B01 Rust Bridge 現在讀 B03 真實 Android Model Adapter；不再硬寫
/// 「尚未接入」。model_verified 僅代表下載＋驗證，**不代表推論已 PASS**。
/// 無法取得狀態時只能回 status_error，不可假報模型 ready。
fn bridge_health_from_model(
    model: Option<&tauri_plugin_cakami_model::ModelStatus>,
) -> BridgeHealth {
    let adapter = platform::current_platform();
    let native_ai = match model {
        None => "status_error",
        Some(m) if m.installed && m.integrity_verified => "model_verified",
        Some(m) if m.download_state == "downloading" || m.download_state == "verifying" =>
            "downloading",
        Some(m) if m.download_state == "waiting_wifi" => "waiting_wifi",
        Some(m) if m.download_state == "paused" => "paused",
        Some(m) if m.download_state == "error" => "download_error",
        Some(m) if m.download_state == "unavailable" => "unavailable",
        Some(_) => "not_installed",
    };
    BridgeHealth {
        contract_version: 1,
        bridge: "ready",
        platform: adapter.platform_name(),
        app_version: env!("CARGO_PKG_VERSION"),
        network_required: false,
        cloud_ai_enabled: false,
        native_ai,
    }
}

/// Rust Bridge 真正呼叫模型原生插件，診斷與 Gemma 狀態同源。
#[tauri::command]
fn bridge_health(app: tauri::AppHandle) -> BridgeHealth {
    let model = tauri_plugin_cakami_model::status(&app).ok();
    bridge_health_from_model(model.as_ref())
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

/// 可觀測的本地模型狀態；模型未匯入不會假顯示 Ready。
#[tauri::command]
fn local_model_status(app: tauri::AppHandle) -> Result<tauri_plugin_cakami_model::ModelStatus, String> {
    tauri_plugin_cakami_model::status(&app)
}

/// 首次開啟由 UI 呼叫：App 會自動取得官方固定版本的 Gemma 檔。
/// 預設只允許非計量網路；使用者可明確允許行動網路。
/// 僅下載公開模型權重，不傳送任何私人照片、OCR、文字或 AI prompt。
#[tauri::command]
fn prepare_local_model(
    app: tauri::AppHandle,
    allow_metered: bool,
) -> Result<tauri_plugin_cakami_model::ModelStatus, String> {
    tauri_plugin_cakami_model::prepare(&app, allow_metered)
}

#[tauri::command]
fn pause_local_model_download(
    app: tauri::AppHandle,
) -> Result<tauri_plugin_cakami_model::ModelStatus, String> {
    tauri_plugin_cakami_model::pause(&app)
}

/// 舊版 SAF 選檔相容入口，正常使用者流程完全不需要手動匯入。

#[tauri::command]
async fn import_local_model(app: tauri::AppHandle) -> Result<tauri_plugin_cakami_model::ModelStatus, String> {
    tauri::async_runtime::spawn_blocking(move || tauri_plugin_cakami_model::import_model(&app))
        .await.map_err(|_| "MODEL_IMPORT_FAILED".to_string())?
}

/// 僅由 Rust 從本地資料構造 prompt；結果在 Rust 驗證 evidence 並存私有 SQLite。
#[tauri::command]
async fn local_model_propose(app: tauri::AppHandle, id: String) -> Result<local_store::ModelSuggestion, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let store = private_store(&app)?;
        let input = store.model_input_for_asset(&id)?;
        let response = tauri_plugin_cakami_model::propose(&app, input.clone())?;
        private_store(&app)?.save_model_suggestion(&id, &input, response)
    }).await.map_err(|_| "MODEL_INFERENCE_FAILED".to_string())?
}

/// B09: explicit user's private response to one pending source-versioned question.
#[tauri::command]
fn answer_intent_question(
    app: tauri::AppHandle, asset_id: String, ordinal: i64, answer: String,
) -> Result<(), String> {
    private_store(&app)?.answer_intent_question(&asset_id, ordinal, &answer)
}

#[tauri::command]
fn load_model_suggestion(app: tauri::AppHandle, id: String) -> Result<Option<local_store::ModelSuggestion>, String> {
    private_store(&app)?.load_model_suggestion(&id)
}

/// B04 S1-AC04：讀取僅本人明確確認後才存在的私人 Intent 草稿。
/// Drop / OCR / E2B 不會寫入此表；未確認時回 None。
#[tauri::command]
fn load_private_goal_draft(
    app: tauri::AppHandle,
    asset_id: String,
) -> Result<Option<goal_draft::PrivateGoalDraft>, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    goal_draft::GoalStore::open(&root)?.load(&asset_id)
}

/// B04：獨立的人類確認操作；Rust 產生 event ID，原子保存為 private draft。
/// 不接受模型寫入確認事件／分享欄位；不會發佈或啟動配對。
#[tauri::command]
fn confirm_private_goal(
    app: tauri::AppHandle,
    input: goal_draft::ConfirmGoalInput,
) -> Result<goal_draft::PrivateGoalDraft, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    goal_draft::GoalStore::open(&root)?.confirm(input)
}

/// US-01D/B07: user-only local withdrawal/deadline; still no Cloud sync.
#[tauri::command]
fn update_private_goal_lifecycle(
    app: tauri::AppHandle,
    input: goal_draft::LifecycleUpdateInput,
) -> Result<goal_draft::PrivateGoalDraft, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    goal_draft::GoalStore::open(&root)?.update_lifecycle(input)
}

/// US-01D/B08: show private fields; final Cloud publish consent belongs to Sprint 2.
#[tauri::command]
fn preview_private_goal_share(
    app: tauri::AppHandle, asset_id: String,
) -> Result<goal_draft::PrivateSharePreview, String> {
    let root=app.path().app_data_dir().map_err(|_|"STORAGE_UNAVAILABLE".to_string())?;
    goal_draft::GoalStore::open(&root)?.share_preview(&asset_id)
}

/// B08: inspect which approved field identifiers the person may explicitly review.
/// Local review is never Cloud publish authority.
#[tauri::command]
fn load_local_share_review(
    app: tauri::AppHandle, asset_id: String,
) -> Result<local_consent::LocalShareReview, String> {
    let root=app.path().app_data_dir().map_err(|_|"STORAGE_UNAVAILABLE".to_string())?;
    local_consent::LocalConsentStore::open(&root)?.load(&asset_id)
}

#[tauri::command]
fn record_local_share_review(
    app: tauri::AppHandle, input: local_consent::ReviewInput,
) -> Result<local_consent::LocalShareReview, String> {
    let root=app.path().app_data_dir().map_err(|_|"STORAGE_UNAVAILABLE".to_string())?;
    local_consent::LocalConsentStore::open(&root)?.review(input)
}

/// B05: versioned allowlisted Concept Registry; T-01 complete whitelist remains OPEN.
#[tauri::command]
fn list_concept_registry() -> Vec<parameter_registry::RegistryConcept> {
    parameter_registry::registry()
}

/// Read-only Rust normalization; front-end never decides N2 concept identity.
#[tauri::command]
fn preview_parameter_concept(
    concept_alias: String,
    value: Option<String>,
) -> Result<parameter_registry::ParameterPreview, String> {
    parameter_registry::preview_parameter(concept_alias, value)
}

/// Only a confirmed B04 private Goal draft can own these private proposals.
/// No model, source, event, visibility, consent or matching fields are writable.
#[tauri::command]
fn save_parameter_proposal(
    app: tauri::AppHandle,
    input: parameter_registry::SaveProposalInput,
) -> Result<parameter_registry::PrivateParameter, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    parameter_registry::ParameterStore::open(&root)?.save(input)
}

#[tauri::command]
fn list_parameter_proposals(
    app: tauri::AppHandle,
    draft_id: String,
) -> Result<Vec<parameter_registry::PrivateParameter>, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    parameter_registry::ParameterStore::open(&root)?.list(&draft_id)
}

/// Explicit user action only: Rust makes its own confirmation event and increments version.
/// Still private: B06 slots never authorize Cloud sharing; B08 review is local only.
#[tauri::command]
fn confirm_private_parameter(
    app: tauri::AppHandle,
    input: parameter_registry::ConfirmParameterInput,
) -> Result<parameter_registry::PrivateParameter, String> {
    let root = app.path().app_data_dir().map_err(|_| "STORAGE_UNAVAILABLE".to_string())?;
    parameter_registry::ParameterStore::open(&root)?.confirm(input)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_cakami_ocr::init())
        .plugin(tauri_plugin_cakami_model::init())
        .invoke_handler(tauri::generate_handler![
            bridge_health,
            local_ocr_recognize,
            load_ocr,
            correct_ocr,
            local_model_status,
            prepare_local_model,
            pause_local_model_download,
            import_local_model,
            local_model_propose,
            load_model_suggestion,
            answer_intent_question,
            load_private_goal_draft,
            confirm_private_goal,
            update_private_goal_lifecycle,
            preview_private_goal_share,
            load_local_share_review,
            record_local_share_review,
            list_concept_registry,
            preview_parameter_concept,
            save_parameter_proposal,
            list_parameter_proposals,
            confirm_private_parameter,
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

    fn sample_model(installed: bool, integrity_verified: bool, download_state: &str)
        -> tauri_plugin_cakami_model::ModelStatus {
        tauri_plugin_cakami_model::ModelStatus {
            installed,
            model_id: "gemma-4-E2B-it".to_string(),
            runtime: "litert-lm-android-0.16.1".to_string(),
            bytes: if installed { 2_588_147_712 } else { 0 },
            integrity_verified,
            download_state: download_state.to_string(),
            downloaded_bytes: if installed { 2_588_147_712 } else { 0 },
            total_bytes: 2_588_147_712,
            download_error: None,
        }
    }

    #[test]
    fn b01_bridge_is_local_but_reads_actual_model_state() {
        let model = sample_model(false, false, "waiting_wifi");
        let status = bridge_health_from_model(Some(&model));
        assert_eq!(status.contract_version, 1);
        assert_eq!(status.bridge, "ready");
        assert!(!status.platform.is_empty());
        assert!(!status.network_required);
        assert!(!status.cloud_ai_enabled);
        assert_eq!(status.native_ai, "waiting_wifi");
    }

    #[test]
    fn b03_bridge_reports_verified_local_model_not_inference_pass() {
        let ready = sample_model(true, true, "ready");
        let json = serde_json::to_value(bridge_health_from_model(Some(&ready)))
            .expect("應可序列化");
        assert_eq!(json["contractVersion"], 1);
        assert_eq!(json["networkRequired"], false);
        assert_eq!(json["cloudAiEnabled"], false);
        assert_eq!(json["nativeAi"], "model_verified");
    }

    #[test]
    fn b03_bridge_never_reports_ready_from_partial_or_missing_model() {
        for state in ["downloading", "verifying", "paused", "waiting_wifi", "error",
                      "unavailable", "idle"] {
            let model = sample_model(false, false, state);
            assert_ne!(bridge_health_from_model(Some(&model)).native_ai, "model_verified");
        }
        assert_eq!(bridge_health_from_model(None).native_ai, "status_error");
        let mismatch = sample_model(true, false, "ready");
        assert_ne!(bridge_health_from_model(Some(&mismatch)).native_ai, "model_verified");
    }
}
