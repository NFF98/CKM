# CAKAMI（CKM）｜MVP 專案工作區

**語言規則：使用者可以用英文輸入，但 CKM 的回覆、文件、Issues、PR、測試、除錯、部署與交接，全部使用繁體中文。**  
完整規範：[AGENTS.md](AGENTS.md)／[溝通規則](communication/COMMUNICATION_RULES.md)。

CAKAMI 的初期目標：**Drop 內容 → 確認意圖 → 找到真人 → 一起完成事情**。

**Phase 1 MVP：** 18 歲以上 JLPT N2 學習者，Android 優先。使用者放入圖片或文字，由本地 AI 協助理解，在明確確認意圖並同意分享必要資料後尋找學伴，進行 15 分鐘文字學習，最後由雙方確認是否完成。

> **目前狀態：已整理 Coding 參考規格與 3 Sprint；App 功能尚未完成或正式發布。** 商業模式與配對效益仍是待驗證假說。  
> **注意：本 Repository 目前為 Public。** 不得提交真實使用者資料、私人圖片、完整聊天、密鑰或未授權的敏感內容。

## Repository 導覽

| 資料夾 | 用途 |
|---|---|
| [design/](design/) | 最新商業、系統、功能設計文件。使用者上傳的原始 Word 規格不任意更動 |
| [build/](build/) | 建置任務、程式實作與進度；目前唯一工作清單為 [Build Backlog](build/BACKLOG.md) |
| [test/](test/) | 自動化測試、驗收案例與配對效果測量 |
| [debug/](debug/) | 問題重現、錯誤紀錄、根因分析與修正驗證 |
| [deploy/](deploy/) | Android 打包、發布檢查及回滾 |
| [communication/](communication/) | 繁體中文溝通規範、決策、進度與交接 |

## 核心技術架構（規劃）

`Tauri v2 + TypeScript UI` → `Rust` → 本地 `OCR / Gemma E2B` 候選模型 → `SQLite`。如使用者同意共享，才同步最少必要的 Intent 資料到雲端服務（規劃為 Supabase／PostgreSQL／pgvector）供配對、邀請與聊天。原圖、私人 OCR 與完整歷史預設留在本地。

**MVP 強制原則：** 每個 Intent 最多 **8 個有效配對 Parameter 概念**；每次 Intent 啟用／配對嘗試最多 **3 題主動追問**；所有值需具備來源、scope、確認狀態與期限；未授權不得公開，且不可默默移除使用者硬條件。

## Coding 入口（四份規格 ↔ 三個 Sprint，已整合）

**第一步讀 [B01–B21 規格追溯矩陣](build/TRACEABILITY.md)**，找到這項工作的上層產品需求、GUI／功能／技術規格及驗收測試。

- **正式產品來源**：[Bible B01–B21](build/BACKLOG.md)、[Business v1.2](design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)、[System v1.7](design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)、[Functional v1.8](design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)。
- **完整實作參考**：[GUI/UX](design/GUI_UX_SPEC.md)、[Functional](design/FUNCTIONAL_SPEC.md)、[Technical](design/TECHNICAL_SPEC.md)、[Acceptance](test/ACCEPTANCE_SPEC.md)；已定原則直接遵循，PROPOSED／OPEN 細節不可擅自定案。
- **Coding 排程**：[三 Sprint 總覽](build/SPRINT_PLAN.md) → [S1 本地 AI](build/sprints/SPRINT-01_LOCAL_AI.md) → [S2 真人配對](build/sprints/SPRINT-02_MATCHING.md) → [S3 測試／發布](build/sprints/SPRINT-03_RELEASE.md)。

**從 B01 開始 Coding；不額外需要 Sprint Activation／GUI High-fi。** Phase 1 禁用 Cloud AI；Day 1 保留 Android／iOS 共用架構。未有真機測試或 Server Evidence，不可宣稱 B-ID 已 PASS。
