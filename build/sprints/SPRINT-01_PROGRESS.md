# Sprint 1｜實際 Coding／驗收進度

> GitHub 追蹤用，不是新 Governance Gate。**只有具備真實 Evidence 才可標 PASS**；上層依據 [Build Bible](../BACKLOG.md)／[Spec Traceability](../TRACEABILITY.md)／[Sprint 1 AC](SPRINT-01_LOCAL_AI.md)。

| Task | 開發狀態 | 測試狀態 | 目前能客觀證明的交付／下一步 |
|---|---|---|---|
| **B01** | IN_PROGRESS | Android 真機 NOT_RUN | PR #6：Tauri TypeScript 畫面、Rust Command、平台 Adapter、Unit Tests；TS Typecheck／5 Tests／Vite Build PASS；Rust 3/3 Unit Tests PASS；APK Build IN_PROGRESS；Android 真機仍 NOT_RUN |
| B02 | NOT_STARTED | NOT_RUN | 選圖／拍照／文字 Drop、本地保存 |
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

**規則**：每次繼續 Sprint 1，先看此表與最新 PR，從未完成的 AC 續做；遇到真機環境不足要標 NOT_RUN／BLOCKED，不得把 Harness CI 綠燈算成 Android 功能完成。
