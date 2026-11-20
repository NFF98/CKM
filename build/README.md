# CAKAMI｜建置工作區（Build）

**目前狀態：B01–B05 已有部分 Coding 與真機證據；US-01「理解→意圖→追問→確認」真機 NG（2026-11-19），US-02／03 NOT_RUN。**

## 首先閱讀

**從 [User Story Sprint Plan](SPRINT_PLAN.md) 開始：先選 US-01／US-02／US-03，再查 [B01–B21 Backlog](BACKLOG.md) 的工程任務及原依賴。**

**2026-11-19 Sprint 已更正為 User Story 驅動。** Coding 前查看 [US→BXX 追溯矩陣](TRACEABILITY.md)，再對照 [GUI Spec](../design/GUI_UX_SPEC.md)／[Functional Spec](../design/FUNCTIONAL_SPEC.md)／[Technical Spec](../design/TECHNICAL_SPEC.md)／[Acceptance Spec](../test/ACCEPTANCE_SPEC.md)。執行入口：[Sprint 1／US-01 本地意圖](sprints/SPRINT-01_LOCAL_AI.md)、[Sprint 2／US-02 找真人＋US-03 完成合作](sprints/SPRINT-02_MATCHING.md)、[Sprint 3／US 全流程回歸與發布](sprints/SPRINT-03_RELEASE.md)。**現在先修 [US-01A 純文字→候選→本人確認](sprints/SPRINT-01_PROGRESS.md)**，不再重做 B01。三個 Sprint 不改既有 Bible 功能、依賴、P0/P1，也不需額外批准才能 Coding。

## 設計依據

- [Business Proposal v1.2](../design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)：商業與 MVP 範圍。
- [System Design v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)：技術規格、Rust 驗證與 8-slot。
- [Functional Design v1.8](../design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)：S01–S09 及內外部 Sequence Diagram。

## 現有與規劃的工程區域

- `app/`：既有 Tauri v2／TypeScript 使用者介面，Rust Core 位於 `app/src-tauri/src/`。
- `app/src-tauri/plugins/`：既有 Android 原生 OCR／Gemma 模型 Bridge。
- Cloud B10～B18：尚未實作的授權、匹配、聊天和成果服務（目錄名稱待工程設計）。

每項功能必須包含測試與驗收證據；依賴套件應鎖定版本；**不得將 API Key、私人圖片或使用者資料提交到 GitHub**。

## 強制溝通規則

使用者可以使用英文提出需求；所有回答、文件、Issues、程式變更說明與交接，均使用**繁體中文**。參見 [AGENTS.md](../AGENTS.md)。
