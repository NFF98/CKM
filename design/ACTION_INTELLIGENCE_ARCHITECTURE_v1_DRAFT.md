# CAKAMI｜Action Intelligence Architecture v1 — BUSINESS-ALIGNED DRAFT

> **狀態：PROPOSED／待 User 審核；未實作、未通過驗收、不可宣稱 MOAT 已形成。**
> **日期：2026-11-19。** 上位 SOURCE：[`Business Proposal v1.2`](doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)（P1 MVP LOCKED）；[`System v1.7`](doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)、[`Functional v1.8`](doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)、[`Technical Spec`](TECHNICAL_SPEC.md)、[`Build Bible`](../build/BACKLOG.md)。
> 此文件只提出現有 B02～B05 到 B08～B18 缺少的智能與資料流契約；**不變更 P1 核准產品範圍，也不宣稱已核准任何新型資料分享／機器學習方式。**

## 1. 北極星：不是背單字 App，也不是單機 Action Checklist

**CAKAMI = Drop Anything → Discover People → Accomplish Together。**

本地資料產生**經本人確認的合作 Intent** → 在本人明確逐項授權後，雲端找同目標或互補 Need/Offer 的真人 → 雙方同意 → 15 分鐘文字共同練習 → 雙方確認完成 → 合法、最少必要的 Outcome 回到各自手機，協助未來做更好的意圖判斷／少問不必要的問題。這是一個**本地→雲端→真人成果→本地**的閉環；不是讓雲端替使用者推理私人原圖。

P1 限 **18+ JLPT N2 Study Match**；P2 Map、P3 Travel、P4 Skill/Create、P5 Intent Network 僅為後續有條件擴張。不能為了 Action Intelligence 偷加地圖、Dating、AI Tutor、購物等新 MVP 範圍。

## 1A. 跨領域共用的真正大腦：Universal Intent Engine（2026-11-19 User 方向；設計草案）

**核心要求：Study 是第一個發布的領域，不是模型的世界觀。** 日後 Fashion、Car、Map/Travel、Skill/Create 應可重用同一套「理解來源 → 多種可能意圖 → 補最少關鍵資訊 → 本人確認 → 雲端真人配對 → Outcome 回本地」引擎，不必每次重寫一套 AI 大腦。新增領域要有經審核的 Domain Pack，但不能讓未上線領域偷渡 P1 Cloud 配對。

### 1A.1 把容易混淆的五件事分開

| 層級 | 用戶看得懂的意思 | 嚴格規則 |
|---|---|---|
| **ContentFact** | 「圖片／文字真正寫了什麼、畫了什麼？」 | 可核對來源；沒有視覺輸入時不能宣稱看懂視覺 |
| **DomainHypothesis** | 「可能在談學習、時尚、汽車還是其他？」 | 可以多選或 UNKNOWN，不能由目前只開放 Study 就強制判成 Study |
| **IntentHypothesis** | 「這個人究竟想學、尋找、比較、求助、提供協助還是共同完成？」 | 內容主題不等於意圖；模型只能產生 `suggested_private`，不產生合法 Goal |
| **ExecutionCapability** | 「CAKAMI 現在真的能幫使用者完成哪件事？」 | 必須對應已實作 Action／Cloud Capability；不顯示按了卻不會做的空按鈕 |
| **ConfirmedMatchIntent** | 「本人確認、符合規則且同意分享的合作需求」 | B04 Goal 明確確認→B05 Registry→B06 8-slot→B08 Consent→B12 Server validation，才能雲端匹配 |

**同一句 `パクる` 的正確行為：**
- 只有 `パクる` 三個字 → **Domain=UNKNOWN，Intent=UNKNOWN**；可客觀解釋詞義，不能直接推定「想學日語／想找學伴」。
- 日文教材中出現 `パクる` → Domain 可能 Study；可提出「你是想保存詞彙，還是想找人練習？」**但 Goal 未經本人確認仍 UNKNOWN**。
- 時裝參考圖與「設計をパクる」上下文 → Domain 可偏 Fashion／設計模仿話題；**不能**直接推定要購買、抄襲設計或找合作對象。
- 汽車改裝文章與「パクる」 → Domain 可偏 Car／改裝參考；**不能**直接推定要找技師或買零件。
- 含多領域線索或證據不足 → 保留多個假設，最多問一個能真正區別意圖的關鍵問題，或先只收藏。

### 1A.2 五段可替換的本地決策 Pipeline

```text
B02 原始 Drop / 文字 / 圖片 / OCR + 實際來源版本
  ↓
(1) EVIDENCE：可驗證內容與來源模態（TEXT / OCR / VISION / UNKNOWN）
  ↓
(2) HYPOTHESES：Gemma 只推測少數 Domain / Subject / Potential Goal / Unknown
  ↓
(3) CONTEXT：Rust 查本地「本人曾確認的 Goal、偏好、有效期、過往 Outcome」
  ↓
(4) DECISION：Rust 在現有能力清單中選擇：建議 / 問 1 個問題 / 只保存
  ↓
(5) HUMAN GATE：本人選擇、可編輯、確認；B04/B05/B06/B08 驗證
  ↓
Cloud：經授權的跨人配對 + 真實合作
  ↓
Verified Outcome → 本地記憶 → 影響下次的 CONTEXT / DECISION
```

**不是再跑一個昂貴 Agent 循環：** 先做輕量規則／OCR 標籤／當前已確認上下文，再**必要時**用一輪受限的 Gemma JSON 推論；只在無法區分的關鍵狀況問使用者。SQLite 保存可重用的已處理 Fact 和結果，避免每開一頁就重新叫 Gemma。推論快慢、記憶體與正確率須 Android 實測，不承諾 E2B 必定勝任所有領域。

### 1A.3 通用資料結構（PROPOSED typed contract，不代表已 Coding）

```json
{
  "assetId": "local-id",
  "sourceRevision": "source-content-hash",
  "evidence": [
    {"ref": "local-evidence-id", "modality": "ocr", "quote": "パクる", "verified": true}
  ],
  "domainCandidates": [
    {"domainId": "unknown", "supportingEvidence": []}
  ],
  "intentCandidates": [
    {
      "candidateId": "local-candidate-id",
      "domainId": "unknown",
      "goalKind": "unknown",
      "subject": "パクる",
      "need": [],
      "offer": [],
      "unknowns": ["user_goal"],
      "sourceEvidenceRefs": ["local-evidence-id"],
      "state": "suggested_private"
    }
  ],
  "nextStep": {"kind": "ask_or_save", "question": "你想用這個內容做什麼？"}
}
```

此例僅用來**說明不知道的情況**，不是要求對單字必定追問。結構化解析要有 schema 版本、大小限制、UNKNOWN、來源 hash、證據與合法 state；不要把小模型輸出的「信心 97%」當成校準過的真實機率。模型產出的候選資料**不得包含 confirmed=true、Consent=true、visibility=published 或 Server Match ID**。

### 1A.4 核心共用 + Domain Pack：擴張不是重做

| 元件 | 永久共用 Core | Domain Pack（需正式審核、版本化、測試） |
|---|---|---|
| 證據 | OCR／本地視覺輸入、引用、去重、來源版本 | 名詞別名與 domain-specific 線索 |
| 意圖 | UNKNOWN／多候選、本人確認、Goal／Need／Offer、有效期 | Study 的學習目標／Fashion 的設計合作／Car 的改裝求助等**候選類型** |
| 行動 | `SAVE_PRIVATE`、`CLARIFY`、`CONFIRM_GOAL`、`PREVIEW_SHARE`、`FIND_MATCH`（只有能力真的存在才提供） | 領域專用行動與必需資料，不允許 Remote Pack 決定自動公開 |
| 配對 | 合法 Goal/Need/Offer/Feasibility + 硬條件排除 + 真雙向 Consent + 可驗證 Outcome | 每個領域必需的安全／資格／時間／位置等硬限制 |
| 記憶 | 本地已確認偏好、改錯、失敗／完成、來源有效期 | 各領域決策依據，不跨領域硬套既有假設 |

**B05 目前 `study.jlpt_level=N2` 是一個 Domain Pack 的 Concept，不應直接寫死在 Core。** B05 的 Parameter Registry（配對欄位）與 Action Capability Registry（可執行動作）是**不同兩個 Registry**；後者必須檢查 handler 真實存在與必要授權。

Cloud 可有共用 `PublishedIntent` 投影，如 `domain_id`、`goal_kind`、`need/offer`、`required_constraints`、`optional_preferences`、`expires_at`、`consent_receipt`、`version`；但任何 domain 專用欄位必須先經 B05 Registry、Rust 白名單及 Server 再驗證。**P1 只能發布 `study_n2`，Fashion/Car 可做離線辨識 Spike，但不得連到真人配對或公佈新類型直到該領域正式核准。**

### 1A.5 關鍵硬體現實：目前 B03 的「看圖」其實是「讀圖上的字」

`local_store::model_input_for_asset` 對圖片只取 `corrected_text/raw_text` OCR 結果，而 Android `CakamiModelPlugin.propose` 的參數只有 `text`；**原始圖片像素目前沒有進入 Gemma 推論**。因此現在不能聲稱「只 Drop 沒有文字的服裝照片／汽車照片，就能辨識剪裁、車型、設計特色」。跨領域圖片理解需要驗證 Gemma 4 E2B + LiteRT-LM 是否可用圖片輸入及裝置記憶體／準確率，或另做受核准的 On-device Image Encoder/VLM Adapter；沒有該能力時回 `vision_unknown`，只用已取得的文字證據，不猜。這是**獨立技術 Gate，不可用新 Prompt 偽裝完成**。

### 1A.6 Accuracy = 有證據地判斷 + 不確定時不亂決定

- **優先第一手內容**：圖片文字、使用者這次說明；次要才用已確認的本地歷史。歷史只當 prior，**不能**因上次 Study，就把這次 Fashion 強制判成 Study。
- **多種假設而非硬分類**：允許 Study/Fashion/Car/Unknown 同時作候選；只有有足夠證據時推薦 domain-specific 行動。低證據、相互矛盾或未知新領域 → `UNKNOWN`／`CLARIFY`；不要把「模糊」當作模型失敗。
- **明確的 Goal 界線**：看到物品、單字或商標，只是內容；要跟人做什麼需本人選擇／確認。模型擅自建立 Need、Offer、購買意向或公開資料一律禁止。
- **按成本與必要性提問**：只問答案真正影響「下一步可執行性／是否可合法配對」的問題；P1 B09 問題總預算最多 3 題（含 Goal 確認），Pair-only 最多 2 題。沒有資訊就「先收藏」不是強迫填表。
- **Outcome 驗證回饋**：由 Cloud B18 提供可信、授權可見的合作結果回本地；本地只更新「哪些建議被接受／更改／完成」，不能把單次成功當普遍規律。跨使用者的 Outcome 聚合須另經同意與資料治理決策。
- **可測試的選擇邏輯（初版）**：對每個候選先以 Rust 硬規則排除「證據不足／能力不存在／需授權但未同意／資料已失效」，再根據本地**已確認**歷史、內容線索與 Unknown 進行可追溯排序；不必一開始訓練私有權重。當不確定性影響選擇時退回提問或只收藏。

### 1A.7 Cloud 不只有「找同類」：通用可行性 Matching

使用者的 Domain 只說明內容屬於哪個領域；**能不能配對**需要另外根據**本人同意公開的** Goal、Need、Offer、執行條件判斷：

1. **Same Goal**：兩人有相同已確認目標，例如想一起練 JLPT N2。
2. **Complementarity**：A 需要某技能，B 自願提供且範圍、條件相容；單純相似不等於能互相幫忙。
3. **Feasibility / Hard Gates**：時間、期限、安全、角色、有效授權、封鎖、版本、必要資格全部合法；硬衝突必須排除，向量距離不能凌駕它們。
4. **Compatible Cross-domain**（未來提案）：跨領域能否互補取決於**版本化、經人工審核的能力／需求對照規則**，而不是兩個人都寫「設計」就自動媒合。沒有核准跨領域對照時只提供合法同領域匹配。
5. **Outcome**：排名要用可信的合作完成率測試；不能拿點擊或兩人剛聊天冒充完成。

雲端只收 Rust/Server 驗證且使用者**逐項同意分享**的 typed `PublishedIntent`，不拿使用者私人圖片／OCR／完整筆記來做雲端語意理解。若使用 pgvector，Embedding 必須由經核准的**裝置端模型**產生並單獨取得 Consent；否則先以 typed Goal/Need/Offer + SQL hard filter 建立有效配對基線。

### 1A.8 Outcome 循環有兩條，不能混在一起

**路線 A：個人回饋（P1 閉環的必要方向；詳細實作待核定）**

```text
雙方真人完成 → B18 Server 驗證 Session
  → 只將「本人有權看到的 Session ID / 結果 / 日期 / 可用的已授權特徵」回自己手機
  → SQLite OutcomeMemory → 下次少問已知問題、不重複推薦已完成事項
```

**路線 B：Circle Intelligence／群體效果（後續探索，必須另核准）**

```text
各人各自選擇加入結果改進計畫
  → Cloud 只保留經同意的、最少必要的合作成果特徵
  → 僅在足夠樣本量、存取控制、污染/重放防護及重新辨識風險評估過關後聚合
  → 版本化的「某種匹配規則／問題順序是否更有效」的非個人化統計
  → 各手機下載合法的統計參數，本地調整候選排序與追問策略
```

**禁止把「去識別化」當成零風險**：小群體、獨特時間地點或稀有需求仍可能重新辨識；聚合粒度、最小樣本門檻、保留時間、撤銷與資訊揭露需正式設計和驗收。Cloud 可做非 AI 的統計或規則，但 Phase 1 不得偷偷加入 Cloud AI 推論或未授權的跨使用者 Profile。「群體智慧」不能讓 A 取得 B 的私人歷史、原圖或未授權的個人分數。

### 1A.9 如何證明「跨域準確」而不是另一個死板 App？

正式設計驗收先跑 [AI-INTENT-01～12](../test/ACCEPTANCE_SPEC.md)，特別是：
- **語境實驗**：同一個 `パクる`，分別放在 Study、Fashion、Car、不明語境。只知道單字時應 UNKNOWN；證據充分才產生相關候選。
- **負例實驗**：用戶過往常學日語，但這次 Drop 明顯是 Fashion；系統不能永遠沿用 Study。
- **視覺 Gate**：沒有文字的服飾／汽車圖片，在實作真正影像 Adapter 前應 `vision_unknown`，絕不亂猜。
- **端到端實驗**：本人確認 Goal，經過 Consent 才能搜尋真人；雲端正確查 Need↔Offer/條件；合作結束後回本地可信 Outcome。
- **競品基線**：同一資料集對比「固定 Study 按鈕」「只讓 Gemma 解釋內容」「沒有個人記憶的通用意圖系統」與 CAKAMI；只看真正確認意圖的品質、完成率、問題數、假意圖率、誤配率。

**設計不等於保證：** 跨域可擴充性來自抽象資料契約與 Domain Pack；準確率必須用未參與設計的多領域樣本、Android 真機和真實使用者驗證，未達標就不能聲稱已建立技術 MOAT。

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
