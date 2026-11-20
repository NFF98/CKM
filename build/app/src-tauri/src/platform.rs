//! B01：跨平台 Adapter 邊界。
//! Android Kotlin／iOS Swift 的本地 OCR／E2B Adapter 在 B03 實作，
//! 在本檔所屬的共用 Rust 核心之外處理平台權限與模型生命週期。

pub trait NativeAdapter {
    fn platform_name(&self) -> &'static str;
    fn native_ai_state(&self) -> &'static str {
        "not_installed"
    }
}

pub struct CurrentPlatform;

impl NativeAdapter for CurrentPlatform {
    fn platform_name(&self) -> &'static str {
        std::env::consts::OS
    }
}

pub fn current_platform() -> impl NativeAdapter {
    CurrentPlatform
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b01_platform_adapter_is_local_and_has_no_fake_ai() {
        let adapter = current_platform();
        assert!(!adapter.platform_name().is_empty());
        assert_eq!(adapter.native_ai_state(), "not_installed");
    }
}
