# CAKAMI｜Action Intelligence Architecture v1 — BUSINESS-ALIGNED DRAFT

> **狀態：PROPOSED／待 User 審核；未實作、未通過驗收、不可宣稱 MOAT 已形成。**
> **日期：2026-11-19。** 上位 SOURCE：[`Business Proposal v1.2`](doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)（P1 MVP LOCKED）；[`System v1.7`](doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)、[`Functional v1.8`](doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)、[`Technical Spec`](TECHNICAL_SPEC.md)、[`Build Bible`](../build/BACKLOG.md)。
> 此文件只提出現有 B02～B05 到 B08～B18 缺少的智能與資料流契約；**不變更 P1 核准產品範圍，也不宣稱已核准任何新型資料分享／機器學習方式。**

## 1. 北極星：不是背單字 App，也不是單機 Action Checklist

**CAKAMI = Drop Anything → Discover People → Accomplish Together。**

本地資料產生**經本人確認的合作 Intent** → 在本人明確逐項授權後，雲端找同目標或互補 Need/Offer 的真人 → 雙方同意 → 15 分鐘文字共同練習 → 雙方確認完成 → 合法、最少必要的 Outcome 回到各自手機，協助未來做更好的意圖判斷／少問不必要的問題。這是一個**本地→雲端→真人成果→本地**的閉環；不是讓雲端替使用者推理私人原圖。

P1 限 **18+ JLPT N2 Study Match**；P2 Map、P3 Travel、P4 Skill/Create、P5 Intent Network 僅為後續有條件擴張。不能為了 Action Intelligence 偷加地圖、Dating、AI Tutor、購物等新 MVP 範圍。

## 2. 單一端到端循環與擁有者

```text
B02 Drop / 本地 SQLite（原圖、OCR、筆記不外傳）
  ↓
B03 ML Kit OCR + Gemma E2B：辨識有證據的內容 Fact（不製造 Goal／Consent）
  ↓
[缺失契約 A] LOCAL INTENT DISCOVERY
  融合原文證據 + 使用者曾確認的私人學習 Goal/偏好 + UNKNOWN
  → 提出少量「要不要找 N2 真人學伴／你想練什麼」的私人候選
  ↓
B04 本人選擇／修正／確認 Goal；Rust 產生合法 confirmation event
  ↓
[缺失契約 B] PARAMETER DISCOVERY → B05 Registry
  從已確認 Goal 中提出 Need/Offer/時間等候選，先驗證 Registry 與證據；
  未核准概念留 UNKNOWN，僅人類可確認，不自動成為 8 slots
  ↓
B06 最多 8 個有效 Matching Concepts + B07 Intent lifecycle
  ↓
B08 Consent：逐項預覽最少分享欄位並由使用者明確授權
  ↓
B12（經 B10/11 Auth、RLS、Server validation）只上傳合法摘要
  ↓
B13～B15 Cloud 同目標／互補需求／時間可行性過濾、排序與少問關鍵 UNKNOWN
  ↓
B16 雙方邀請接受 → B17 文字 Chat / 15 分鐘 Session
  ↓
B18 Server 驗證「有效時長 + A 確認 + B 確認」的真正 Outcome
  ↓
[缺失契約 C] OUTCOME RECEIPT / LOCAL FEEDBACK
  把本人有權取得的最少 Session 結果同步回本機 SQLite；
  用已驗證成功/失敗信號改進「下次問什麼／推薦什麼」（規則先行，評估後再學習）
  ↺ 回到下一次 Drop / Intent Discovery
```

**雲端負責**：經授權的真人需求索引、同目標/互補條件查詢、硬限制、雙方同意、Session 結果可信驗證、必要通知/結果同步。**不負責**：呼叫雲端 LLM、解讀使用者私人原圖或 OCR、未授權行為推理。雲端的 SQL/pgvector 比對不等於 Cloud AI 推論。

## 3. 三段欠缺的介面（PROPOSED，名稱／欄位不屬已核准 API）

### A. B03→B04：ContentFact → PrivateIntentSuggestion

- B03 原有 `evidenceQuote/contentHint/uncertainty` 只能說明內容；新增**獨立的、嚴格驗證的候選輸出**（而非把 AI 猜測塞進 `confirmed_goal`）。
- `ContentFact`：`asset_id`、`source_revision/hash`、`category`（例如 `japanese_vocab`）、`evidence_quote`、`unknowns`、`model_version`。沒有來源根據的事實拒絕；正確詞義本身**不證明**用戶要找學伴。
- `PrivateIntentSuggestion`：`candidate_id`、`asset_id`、`suggested_goal`（可空）、`reason_evidence_ids`、`unknowns`、`status='suggested_private'`、`source_revision`。只建議「可能想為 JLPT N2 詞彙找真人練習」，不能自動寫 Rust 確認事件。
- `パクる` 單字是**內容證據**，對真正 Goal 資訊不足；UI 提供「只收藏」或「我想找 N2 學伴練這類內容」等可選擇項，最多問真正必要的問題。使用者可以選、改或拒絕，B04 只在本人明確確認後建立 Goal。
- **P1 不要把單機複習卡視為核心 North Star**；可以只保存本地內容作備援價值，但必須驗收「一張圖到有效 Intent」能否更容易。

### B. B04→B05：ConfirmedGoal → PrivateParameterSuggestions

- 只從**已由本人確認**的 Goal 與已有證據提議已定義 Registry Concept；標記來源 `ocr/model_inferred/user_explicit`、evidence、scope、TTL/UNKNOWN，不能把 `N2` 單字自動當成用戶的 JLPT 等級。
- B05 現有 Registry v1 僅核准 `study.jlpt_level=N2` 及別名；Need/Offer、時間、可行性等**完整白名單 T-01 尚 OPEN**，不得先在模型或 SQLite 中假設已核准。未知保持私人 UNKNOWN。
- Rust 才能驗證、去重、建立本人 confirmation event。B06 對有效概念強制 8-slot；8-slot 不是圖片上限。
- B09 是每次 Intent/配對動態追問最多 3 題（含 Goal 確認），B15 Pair-only 最多 2 題；優先問最可能改變匹配可行性的缺口，不是每張圖一律展示三個選項。

### C. B18→本地：VerifiedOutcome → OutcomeMemory

- `OutcomeReceipt`（草案）：`session_id`、`intent_id/version`、`result=COMPLETED|CANCELLED|NO_SHOW|UNRESOLVED`、`server_confirmed_at`、`own_confirmation_state`、`verified_by_server`。`COMPLETED` 只有 Server 驗證有效 15 分鐘及雙方確認才成立；單方確認不算。
- 本機按收件人權限與同步版本保存自己的最少結果，用于避免推薦已完成/已失效 Intent，改善下次問題與候選選擇。不得憑手機 UI 點擊自行編造 `COMPLETED`。
- **Cross-user Outcome Learning 尚是商業論點，不是已核准同步範圍**。若要雲端蒐集、跨使用者聚合或再分發 Outcome 特徵，先定義用途、最少欄位、保留期、撤銷、使用者同意、授權邊界與隱私測試；不能把另一位參與者的私人內容回傳給本地模型。
- 初期先用 Rust 可解釋的規則/統計更新，做到確定比無記憶基線好再考慮增強學習；無需每次 Outcome 遠端 LLM 或模型權重更新。

## 4. 對應原先 BXX（保留編號、加上缺失 AC；不得假裝已 Coding）

| 既有項目 | 已有的 | 必須補的閉環 |
|---|---|---|
| B02 | App-private Drop/SQLite、去重／儲存 | Content source_revision/可追溯更新作為候選有效性依據 |
| B03 | OCR + Gemma `contentHint` + evidence | 有實測的 ContentFact 與私人 IntentSuggestion，證據和 UNKNOWN 校驗 |
| B04 | 手動 Goal 確認＋Rust 私人事件 | 接受／修改 B03 私人 Goal 建議；不能自動確認 |
| B05 | N2 Registry + 人工輸入／確認 | 已確認 Goal 後的候選參數抽取；完整概念白名單仍 OPEN |
| B06～B09 | 尚未建置 | 8-slot、安全有效期、Consent、僅必要問題 |
| B10～B18 | 尚未建置 | 經授權的 Cloud Matching、真人協作、可信 Outcome |
| 跨 B18→B03/B04 | 尚未設計成可執行契約 | 已驗證結果回本機的 OutcomeReceipt／個人化回饋；跨人聚合另需決策 |

### 不改動的紅線

- Content ≠ Goal ≠ Consent ≠ Verified Outcome。**Gemma 只提議；Rust 驗證；真人確認；Cloud 只驗證合法授權和多人結果**。
- 只傳經本人確認及逐項同意的最少配對摘要；原始圖片、OCR、私人筆記和完整歷史不外送。
- 本地 AI 可離線使用；離線時不能謊稱雲端已匹配、分享已生效或 Session 已完成。
- 不變更 P1 範圍，直到正式 User 商業決策批准。

## 5. 不是靠試錯：分三層可否證的測試

| Gate | 測試樣本／方法 | PASS 定義（目標，待正式測試） |
|---|---|---|
| **AI-A01 本地候選品質** | 100 筆自製 N2 字詞/句子/廣告/無關截圖；專人標記「可否合理提議 N2 Study Goal」及應 UNKNOWN | 與純詞義解釋／固定問題比較：少誤判、不虛構 Goal；具有證據鏈；記錄假陽性與拒絕率 |
| **AI-A02 一張圖→合法 Intent** | 使用者從真 Android Drop 到本人確認 Goal，並驗證 B05 Registry、B06 8-slot、Consent 邊界 | 參照 Business Proposal：至少 80% 測試者可從一張圖建立及確認學習 Intent；不因更少點擊而省略確認或多上傳 |
| **AI-A03 雙真人完整閉環** | A/B 雙帳號：合法共享最少摘要→Match→雙方接受→15 分鐘→雙方確認→OutcomeReceipt 回到雙方本地；不合法第三人權限測試 | Cloud 恰一筆可驗證 COMPLETED、本地無未授權來源；單方未確認／時間不足不可記成功；離線恢復可正確同步 |
| **AI-A04 Moat 對照** | 相同真人候選池，比「固定表單／單向量／多向量+條件」與 CAKAMI；統一曝光及安全條件 | 更少必要問題且 **每 100 個有效 Intent 的真正完成 Sessions** 更高；未驗證前不得聲稱 MOAT |
| **AI-A05 Outcome Feedback** | 本機有/無可信完成歷史兩組，重複相同 Intent／過期／已完成案件 | 本地判斷可解釋、不重複推薦已完成任務，少問已知條件；不得讀到對方私人資料 |

**Business Proposal 的 P1 北極星是：每 100 個有效 Intent 的真正完成 Session。** 初期探索目標：100 名 18+ 測試者至少完成 20 場；D7 回訪 ≥25%。Action Intelligence 的成功不得只用 OCR 正確率、詞義正確率、卡片點擊率或漂亮 UI 評估。

## 6. 實作順序建議（不是新增獨立產品）

1. **先核准本草案與最小共享 Payload／T-01 Registry 概念清單**；最少可先鎖 `N2 Study Goal` 的提議及 B04 本人確認，不鎖不可發布。
2. B03→B04 新增 `PrivateIntentSuggestion` 的 Rust contract + SQLite 持久化、來源版本失效與 UI 可選確認；不讓 E2B 直接確認。
3. B04→B05 補 `PrivateParameterSuggestion` 受控路徑，保留來源／未知；再續 B06～B09。
4. B10～B18 Cloud E2E 建立後，加入可信 OutcomeReceipt 同步回本機；從規則式回饋開始，不假稱 AI 已學會。
5. 每一步分別跑 Rust/TS 契約、失敗注入及真 Android；至少完成相關兩項功能才打批次 APK（沿用既有批次規則）。

**未決需 Human 核准**：T-01 的 Need/Offer/時間正式 Concepts、F-03 Cloud Share Scope、OutcomeReceipt 欄位/保留期與跨人聚合 Consent、B09 問題策略、成功/No-show 的 Server 細則。未核准前可先實作本地私人候選，**不可擴張公開權限**。
