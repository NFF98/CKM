# CAKAMI｜設計工作區

**設計來源要可追溯，不任意改動使用者上傳的原始文件。**

## 目前最新設計

- [Business Proposal v1.2](doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)：Phase 1 MVP、市場、成長飛輪與驗收指標。
- [System Design v1.7](doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)：8-Parameter、Rust／SQLite 驗證、Matching Algorithm。
- [Functional Design v1.8](doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)：S01–S09 畫面、功能對照、外部／內部 Sequence Diagram。

## 已合併至 main 的 Coding 詳細參考

- [GUI／UX Spec](GUI_UX_SPEC.md)：Android／iOS 共用畫面行為、S01–S09／S04a、Code-first GUI。
- [Functional Spec](FUNCTIONAL_SPEC.md)：逐畫面的可觀察行為、Intent／Consent／Matching／Session Gate。
- [Technical Spec](TECHNICAL_SPEC.md)：Rust／SQLite、裝置端 OCR／E2B、Supabase 資料契約與信任邊界。
- [Acceptance Spec](../test/ACCEPTANCE_SPEC.md)：B01–B21、T01–T13、E2E、P0 Evidence。

**實作唯一總表：[Build Bible B01–B21](../build/BACKLOG.md)。** 要開工時先查 [Spec↔Sprint↔Test 追溯矩陣](../build/TRACEABILITY.md) 及 [三 Sprint 計畫](../build/SPRINT_PLAN.md)。四份規格的 SOURCE 與已明確確認的原則可作依據；PROPOSED／OPEN 不代表已被使用者批准。Coding 可直接進行，不需整份文件 Freeze。

如需修改設計，應註記影響的畫面、參數規則、權限、資料流及測試項目，避免 Business、System、Functional 三份文件互相矛盾。

所有新增說明與審核意見皆使用**繁體中文**，即使原始需求用英文提出。
