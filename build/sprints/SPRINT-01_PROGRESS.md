# Sprint 1｜實際 Coding／驗收進度

> GitHub 追蹤用，不是新 Governance Gate。**只有具備真實 Evidence 才可標 PASS**；上層依據 [Build Bible](../BACKLOG.md)／[Spec Traceability](../TRACEABILITY.md)／[Sprint 1 AC](SPRINT-01_LOCAL_AI.md)。

| Task | 開發狀態 | 測試狀態 | 目前能客觀證明的交付／下一步 |
|---|---|---|---|
| **B01** | IN_PROGRESS | Android 真機 NOT_RUN | Coding PR：Tauri TypeScript 畫面、Rust Command、平台 Adapter、測試；待 CI 與 Android 真機驗收 |
| B02 | NOT_STARTED | NOT_RUN | 選圖／拍照／文字 Drop、本地保存 |
| B03 | NOT_STARTED | NOT_RUN | 真機本地 OCR／Gemma E2B；最大技術風險 |
| B04 | NOT_STARTED | NOT_RUN | 內容 ≠ Goal |
| B05 | NOT_STARTED | NOT_RUN | Parameter Registry |
| B06 | NOT_STARTED | NOT_RUN | SQLite 原子交易＋8-slot |
| B07 | NOT_STARTED | NOT_RUN | Intent 生命週期 |
| B08 | NOT_STARTED | NOT_RUN | Consent／分享預覽 |
| B09 | NOT_STARTED | NOT_RUN | 追問限制 |

**當前實作 PR**：在 `feat/s1-b01-tauri-mobile-shell` 分支，完成後以 GitHub PR／CI 作為證據。

**規則**：每次繼續 Sprint 1，先看此表與最新 PR，從未完成的 AC 續做；遇到真機環境不足要標 NOT_RUN／BLOCKED，不得把 Harness CI 綠燈算成 Android 功能完成。
