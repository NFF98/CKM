# CAKAMI｜Sprint 1 B01：跨平台 Tauri App 骨架

**唯一依據**：[`build/BACKLOG.md` B01](../BACKLOG.md)、[`build/TRACEABILITY.md`](../TRACEABILITY.md)、[Technical Spec](../../design/TECHNICAL_SPEC.md)、[Functional Spec](../../design/FUNCTIONAL_SPEC.md)、[GUI Spec](../../design/GUI_UX_SPEC.md)、[Acceptance Spec](../../test/ACCEPTANCE_SPEC.md)、[S1-AC01](../sprints/SPRINT-01_LOCAL_AI.md)。

## 已寫程式／尚未證明

- 已寫：Tauri v2 設定、Vanilla TypeScript UI、可導航的 B01 首頁／診斷頁、Rust `bridge_health` Command、跨平台 Native Adapter 邊界、TypeScript／Rust Unit Tests。
- **未實測**：Android 真機啟動／離線 Rust 回應、iOS 真機、完整 Android APK。沒有真機證據不能標 B01 PASS。
- **未開始**：B02 Drop 持久化、B03 本地 OCR／E2B、B04～B09 功能；不能把目前 UI 當作已具備它們。
- Phase 1 禁 Cloud AI；本功能不包含 Cloud API，也沒有偷偷 Fallback。

## 開發環境

Node.js 22.16+、npm、Rust stable、Tauri v2；Android 需 Android SDK／NDK、JDK、Rust Android targets，並將 Android Device／Emulator 連線。參考 [Tauri Android 官方 prerequisites](https://v2.tauri.app/start/prerequisites/#android)。Windows 可做 Android 開發；iOS 需 macOS + Xcode。

~~~sh
cd build/app
npm install
npm run check
npm test
npm run build
npm run tauri android init
npm run tauri android dev
~~~

Android 發行版（需對應 SDK／簽署環境）：
~~~sh
cd build/app
npm run tauri android build -- --apk
~~~

macOS iOS（架構保留，**尚未經真機測試**）：
~~~sh
cd build/app
npm run tauri ios init
npm run tauri ios dev
~~~

**真機 B01 驗收**：飛航模式中開啟安裝好的 App → 「裝置狀態」→「執行連線檢查」→ 確實顯示 Rust 平台與 App 版本 → 來回切換首頁不閃退 → 保存 Android 裝置型號、OS、App Version、adb log／不含私密資料的截圖。

**Web 瀏覽器不會模擬成功**：按連線檢查會明確告知目前不是 Tauri App。

**技術暫定值**：`com.nff98.ckm.dev` 是開發期 Bundle ID，正式上架前必須決定正式識別字；`icons/icon.png` 是暫定程式用識別圖，不是正式品牌設計。

## Coding/Debug

在 Coding PR 記錄 `S1-AC01/B01`、測試命令、真實結果、失敗根因、修正 SHA、裝置／OS／runtime。CI PASS 僅能證明有實際執行的 TS／Rust Test，不代表 Android 原生可用。
