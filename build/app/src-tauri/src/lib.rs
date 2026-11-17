mod platform;
use platform::NativeAdapter;
use serde::Serialize;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![bridge_health])
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
