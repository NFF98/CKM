# CAKAMI｜Phase 1 MVP 三個 Sprint 執行總覽

> **建立日期：2026-11-17**；由使用者明確要求將 B01–B21 分成 **3 個 Sprint**，作為 Coding、Debug、驗收的共同參考。  
> **目前狀態：PLAN READY／程式尚未完成**。本文件與分篇 Sprint 文件是執行安排，**不另立產品需求**；正式功能／依賴／P0 與 P1 以 [Build Bible B01–B21](BACKLOG.md) 和既有 Business v1.2／System v1.7／Functional v1.8 為準。  
> 此計畫 **不是新的 Governance Gate**：CKM Harness 已容許開始 Coding；**不需要每個 Sprint 再次 Activation、Freeze 或批准**。人類只決定有爭議的產品行為，以及真正發布。


## 0. 先讀規格，再依 Sprint Coding（不能只讀此計畫）

**先讀 [四份細部設計對照／B01–B21 追溯矩陣](TRACEABILITY.md)。** 每個 Coding Task 都必須同時對照 [正式 Bible](BACKLOG.md)、[GUI Spec](../design/GUI_UX_SPEC.md)、[Functional Spec](../design/FUNCTIONAL_SPEC.md)、[Technical Spec](../design/TECHNICAL_SPEC.md)、[Acceptance Spec](../test/ACCEPTANCE_SPEC.md) 中受影響的章節。**Sprint 是工作排程，不是新的產品 Spec，也不能改寫 E2E Test 的原始 ID／期望結果。**

## 1. 三個 Sprint 的交付

| Sprint | 對應 Bible | 真正可見成果 | 交付門檻 |
|---|---|---|---|
| [Sprint 1｜本地 AI 與私人意圖](sprints/SPRINT-01_LOCAL_AI.md) | **B01–B09（均 P0）** | Android App 能 Drop N2 圖片／文字；本地 OCR＋E2B 提議內容；本人確認 Goal；8-slot、授權預覽、離線管理 | 9 個 B-ID 均有真實測試；Rust／SQLite 邊界可信；**B03 必須真機推論** |
| [Sprint 2｜真人配對與合作](sprints/SPRINT-02_MATCHING.md) | **B10–B18（均 P0）** | 2 個合法測試帳號：同意分享 → 找到真候選／無候選 → 雙方邀請 → Chat → 15 分鐘練習 → 雙方完成確認 | 真實 Supabase Auth／RLS／Server Validation；非法資料無法跨帳號讀寫 |
| [Sprint 3｜測試與 Android Pilot](sprints/SPRINT-03_RELEASE.md) | **B19（P0）、B20（P1）、B21（P0）** | 全流程回歸；T01–T13；安全／離線／效能證據；可安裝 Android 測試 APK | **B19 P0 全數 PASS 才允許 B21 發布**；B20 Benchmark 是 P1，**不是**發布阻擋條件 |

**工期**：先以 7 天為挑戰目標，**非承諾**；模型、雲端權限與真機速度需要實測。分 Sprint 不代表只能在某一天做該項工作；優先依 Bible 依賴和可運作的 Vertical Slice 開發。

## 2. 第一個可見成果與開發順序

1. **B01（立即開始）**：Tauri v2／TypeScript／Rust 骨架，Android 真機啟動，畫面能呼叫 Rust Command；同時架好 iOS Adapter 邊界。
2. **B02 → B03（早期高風險技術驗證）**：選圖／文字／拍照 → 真機本地 OCR／E2B。模型不可用就如實回報 BLOCKED，**不可拿假的模型輸出取代 B03**；可並行做獨立基礎工作。
3. **B04–B09**：內容與本人意圖分離、Concept Registry、交易式 8-slot、Intent 生命週期、分享授權、合理追問。
4. **B10–B18**：帳號安全→RLS→授權同步→真人候選→邀請→Chat／Session→雙方完成。
5. **B19 → B21**：完整 P0 驗收及 Android 測試版；B20 P1 比較可平行或稍後完成，不能把它偷改成 B21 必要 Gate。

## 3. 開發從第一天就遵守的硬規則

- **Phase 1 沒有 Cloud AI**：OCR、E2B、任何 AI 推理與向量產生（若需要）在手機本地；Supabase 可用於經同意後的帳號、資料查詢、配對、邀請及聊天，但不能呼叫雲端模型。
- **Android 優先驗證、iOS 從 Day 1 可移植**：共用 TypeScript／Rust 的資料、規則、商業邏輯；選圖、權限、OCR、本地模型等能力以平台 Adapter 封裝。**iOS 實際編譯／運行尚未驗證**，不能冒稱已支援。
- **Code-first GUI**：不做 High-fi；直接在 App 裡確認 S01–S09、S04a 的流程、觸控、錯誤／空／離線狀態。
- **Rust + SQLite 是可信邊界**：8 個有效 Matching Concepts，slot 0–7、確認事件、provenance、scope、expiry、撤銷；AI 不得直接寫出已確認的 Goal／Need／Offer。
- **雲端資料要有 User Consent**：原圖、私人 OCR／筆記不會**自動**上傳；實際上傳欄位需使用者確認；Embedding 也不能當匿名資料。
- **真人學習 ≠ 假成功**：候選非成功、聊天非成功；15 分鐘與兩方完成確認才算完成。
- **最小化治理**：沒有 Cursor、沒有二次 Sprint/Task 開工許可。Harness 形式性提醒不阻擋 Coding；真正的程式、安全與測試失敗要修正，不能假 PASS。
- **不可提交私人資料或密鑰**：Repo 是 Public；測試帳號、配對資料及截圖使用合成／專用測試資料。

## 4. 每項 B-ID 的完成定義（Definition of Done）

- 對照對應 Sprint 文件的 **AC（Acceptance Criteria）** 和 Bible 原始驗收；兩者不一致時以 Bible 與已批准規格為準。
- 必須有**可執行的程式**、正常案例＋關鍵失敗案例、可重現測試指令及結果（而不是只有文案／Mock／靜態圖）。
- 有 GitHub PR／Commit、使用的環境與依賴版本、證據位置；裝置相關項目要有實機／adb 日誌（不包含私密資料）。雲端安全需實際 RLS／API 測試。
- 狀態固定使用 **NOT_STARTED / IN_PROGRESS / PASS / FAIL / BLOCKED / NOT_RUN（測試尚未執行）**，只有實測 PASS 才說完成。Mock 可輔助開發，但不能冒充 Android／E2B／Supabase 真實 PASS。
- **Sprint 不用等全綠才允許做無依賴的其他 Coding**。某項受阻，只標示該 B-ID 和受其依賴的下游功能 BLOCKED，不得讓治理流程造成全案死鎖。
- 每個 PR 自動跑當時已實際存在的 CI（Harness、相關 TypeScript／Rust／測試）；新增的測試需明確記錄。**CI 的範圍以實際執行結果為準**，不宣稱目前具備原生 iOS/Android 全自動測試。

## 5. Debug 如何運作（直接在 Coding PR 內處理）

```text
可重現的錯誤 → 記錄 B-ID／AC-ID → 最小失敗測試
→ 找根因 → 修改 → 重跑受影響測試 → GitHub PR 記 Evidence
```

每個失敗至少記錄：`B-ID + AC-ID`、平台與版本、重現步驟、預期／實際、錯誤日誌、根因、修正 Commit、重測結果與尚存風險。先在同一個 Coding PR 中完成一般 Bugfix；若牽涉未決定的產品規則才請使用者決策，**不為普通 Bug 新增治理審批**。同一修復方法連續無效，改用新的可驗證手段，不無限重試。

## 6. 未決事項不阻擋 B01，但相關功能不能擅自定案

| 類型 | 尚待確認的產品細節 | 影響 | 工作方式 |
|---|---|---|---|
| 帳號及成年 | 免登入可用範圍、18+ 是自我聲明還是真驗齡 | B10／B16 | 在 B10 前確認，不影響 B01 |
| 分享 | 公開欄位範圍；是否容許使用者**額外主動**分享原圖 | B08／B12 | 先採最小私有資料原則，具體 Payload 經 Human 確認才發布 |
| 邀請 | 逾時、取消與重試規則 | B16 | 在 B16 前確認 |
| Session | 15 分鐘起算、離線／背景中斷、No-show | B17／B18 | 在相應任務前確認 |
| 模型 | E2B 版本、runtime、機型／記憶體可行性 | B03 | **立即實測**，屬技術驗證非先決產品審批 |

不把未批准的建議數值寫成已核准事實。具體欄位／API 命名由 Coding Agent 提出並做 Contract Test，不需要 Human 審每個欄位；會改變私隱、安全與使用者體驗的選擇才由 Human 決策。

## 7. 參考與進度證據

- [Bible：B01–B21](BACKLOG.md)（唯一建置工作總表）。
- [Sprint 1 驗收](sprints/SPRINT-01_LOCAL_AI.md)｜[Sprint 2 驗收](sprints/SPRINT-02_MATCHING.md)｜[Sprint 3 驗收](sprints/SPRINT-03_RELEASE.md)。
- [單一 AI Harness](../harness/README.md)；[PR #4 已合併](https://github.com/NFF98/CKM/pull/4)。
- **四份詳細規格已合併至 main**：[GUI／UX](../design/GUI_UX_SPEC.md)、[Functional](../design/FUNCTIONAL_SPEC.md)、[Technical](../design/TECHNICAL_SPEC.md)、[Acceptance](../test/ACCEPTANCE_SPEC.md)。它們是 Sprint Coding 的**實際工作依據**；其中標記 PROPOSED／OPEN 的產品行為仍不代表已批准。
- **[B01–B21 完整追溯矩陣](TRACEABILITY.md)**：每個 B-ID → 四份 Spec 章節 → Sprint AC → 正式 Test IDs（T01–T13、E2E-01～05）。避免 Sprint 規格另創定義。
- 開工當下 B01–B21 **沒有任何 App 功能被標記 PASS**。進度／測試結果依後續 Coding PR 及實際 Evidence 更新，不在此偽造。
