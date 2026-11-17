# CAKAMI｜建置工作區（Build）

**目前狀態：尚未開始程式實作。**

## 首先閱讀

**[MVP Build Backlog（B01–B21）](BACKLOG.md)** 是建置優先順序及驗收規範的唯一清單。

此階段**只建立 Backlog，不預先安排 Sprint**。先檢查需求、依賴、Android 裝置上的本地模型可行性與安全驗收，再決定工作分期。

## 設計依據

- [Business Proposal v1.2](../design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)：商業與 MVP 範圍。
- [System Design v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)：技術規格、Rust 驗證與 8-slot。
- [Functional Design v1.8](../design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)：S01–S09 及內外部 Sequence Diagram。

## 預計實作區域（尚未建立）

- `app/`：Tauri v2／TypeScript 使用者介面。
- `core/`：Rust Dynamic Parameter Controller、8-slot 與配對規則。
- `mobile/`：Android 本地 OCR／模型接入。
- `cloud/`：使用者同意後的後端 API、資料庫及聊天服務。

每項功能必須包含測試與驗收證據；依賴套件應鎖定版本；**不得將 API Key、私人圖片或使用者資料提交到 GitHub**。

## 強制溝通規則

使用者可以使用英文提出需求；所有回答、文件、Issues、程式變更說明與交接，均使用**繁體中文**。參見 [AGENTS.md](../AGENTS.md)。
