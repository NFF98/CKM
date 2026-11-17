# CAKAMI｜測試工作區

**[MVP Acceptance Spec](ACCEPTANCE_SPEC.md) 已納入 main**，內容涵蓋 B01–B21 的測試設計。App 自動化測試／真機與雲端 E2E 仍未執行；**測試規格存在 ≠ 測試 PASS**。Coding 時須逐項加入可執行測試、合成資料、驗收紀錄。

**最低必要測試範圍：**

- E2B 只能提出候選 Parameter，無法跳過 Rust 直接寫入已確認資料。
- 每個 Intent 最多 **8 個有效配對概念**，並發寫入也不能超過上限。
- Parameter 的來源、scope、確認事件、期限、可見性與撤銷必須有強制驗證。
- 每次 Intent 啟用／配對嘗試最多 **3 題主動追問**。
- OCR 不確定或使用者目標不明時，不能編造已確認 Intent。
- 未授權使用者不能讀到其他人的私人意圖及聊天。
- **真正的完成場次須有雙方確認**，不能只靠聊天或單方打勾。
- 與固定表單和純向量配對進行公平基準比較；量測成功率、延遲與成本。
- Android 實機測試本地模型相容性、效能與耗電。

依 [追溯矩陣](../build/TRACEABILITY.md) 由每個 B-ID 查到 Sprint AC、[Acceptance Spec](ACCEPTANCE_SPEC.md) 與原始 [System v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx) 的 T01–T13。Sprint 不可重新定義 E2E ID。

**只使用合成測試資料，勿提交任何可識別真人的截圖或個資。** 測試說明及報告一律繁體中文。
