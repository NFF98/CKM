# Sprint 1｜實際 Coding／驗收進度

> GitHub 追蹤用，不是新 Governance Gate。**只有具備真實 Evidence 才可標 PASS**；上層依據 [Build Bible](../BACKLOG.md)／[Spec Traceability](../TRACEABILITY.md)／[Sprint 1 AC](SPRINT-01_LOCAL_AI.md)。

| Task | 開發狀態 | 測試狀態 | 目前能客觀證明的交付／下一步 |
|---|---|---|---|
| **B01** | IN_PROGRESS | Android 啟動＋Bridge：使用者回報成功；飛航模式 NOT_RUN | PR #6：TS／Rust Unit Tests PASS，Android APK 成功打包。使用者回報 Android 0.1.0 上可顯示 `Rust Bridge 已回應`、`平台：android`。未有飛航模式／adb 日誌；S1-AC01 尚未全部 PASS。 |
| **B02** | IN_PROGRESS | CI／Android 真機待驗證 | 已提交 Rust SQLite App-private 儲存、圖片／拍照／文字入口、私人清單及讀回；10 項 Rust 測試、8 項 TS Contract 測試。暫定 8MB／64KB 防護屬 F-01 待確認。Android 重啟／離線／相機尚未真機驗收。 |
| B03 | NOT_STARTED | NOT_RUN | 真機本地 OCR／Gemma E2B；最大技術風險 |
| B04 | NOT_STARTED | NOT_RUN | 內容 ≠ Goal |
| B05 | NOT_STARTED | NOT_RUN | Parameter Registry |
| B06 | NOT_STARTED | NOT_RUN | SQLite 原子交易＋8-slot |
| B07 | NOT_STARTED | NOT_RUN | Intent 生命週期 |
| B08 | NOT_STARTED | NOT_RUN | Consent／分享預覽 |
| B09 | NOT_STARTED | NOT_RUN | 追問限制 |

**當前實作 PR**：[PR #6](https://github.com/NFF98/CKM/pull/6)，分支 `feat/s1-b01-tauri-mobile-shell`。

**實際測試 Evidence**
- TypeScript：[`B01 App` Run #44024038403](https://github.com/NFF98/CKM/actions/runs/44024038403)；TS Typecheck、Vitest **5/5 PASS**、Vite Build **PASS**。
- Rust：最初因 App Icon 不是 RGBA 編譯失敗；已修正，[`B01 App` Run #44024343756](https://github.com/NFF98/CKM/actions/runs/44024343756) **Rust 3/3 PASS**。
- Android ARM64 Debug APK：[APK Smoke Workflow](https://github.com/NFF98/CKM/actions/workflows/b01-android-apk.yml) 已建立；曾因同一個 PNG 問題失敗，修正後 [Run #44024584452](https://github.com/NFF98/CKM/actions/runs/44024584452) 正在打包。**尚未確認 APK 可安裝**。
- Android 實體裝置離線 UI↔Rust 驗收：**NOT_RUN**；無硬體 Evidence，不可把雲端 CI 算成真機 PASS。

**2026-11-17 使用者實測回報**：在 Android 裝置上看到 `Rust Bridge 已回應`，平台 `android`、版本 `0.1.0`、`雲端 AI：禁用`、`本地 AI：尚未接入`。此證據支援實機啟動與 Rust Bridge 功能；尚未驗證飛航模式，所以不能說 B01 全部驗收完成。後續 B02 以獨立堆疊 PR 實作。

**規則**：每次繼續 Sprint 1，先看此表與最新 PR，從未完成的 AC 續做；遇到真機環境不足要標 NOT_RUN／BLOCKED，不得把 Harness CI 綠燈算成 Android 功能完成。


## B02 Coding Evidence（2026-11-17）

- B02 分支：`feat/s1-b02-local-drop-storage`（疊在 PR #6 B01 上，避免未通過 B01 就直接假裝已在 main）。
- 有效功能：App 本地圖片、拍照與文字上傳入口；Rust `save_drop`、`list_drops`、`load_drop`；App-private SQLite + 檔案持久化。原圖不送往網路／雲端，且**不推斷 Goal**。
- 實測：CI 尚待結果；Android 手機上需測試重新啟動、飛航模式、拍照／選圖與圖片讀回。
- 未定：F-01 的最終圖片格式、容量、批次和重複策略，目前 8MB／64KB 僅工程安全界線；顯示重複但保留兩筆，不宣稱為最終產品策略。
