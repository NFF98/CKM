# CAKAMI｜功能行為詳細規格（審核草案）

> **狀態：DRAFT／待 Human Review，未 LOCKED、未授權 Coding。**  
> 日期：2026-11-17；範圍：Phase 1，18+ JLPT N2 Study Match，Android First，S01–S09／S04a。  
> 正式來源：../build/BACKLOG.md（B01–B21）；doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md；doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx（尤其 §3、§8、§15–16）；doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx。  
> 用法：**SOURCE** 代表有正式來源的既有規則；**PROPOSED** 代表有待批准的細節；**OPEN** 代表仍須 Human 決策。不得把 DRAFT 當成最新已核准 Product Truth。

## 已核准的 Phase 1 原則：所有 AI 在手機本地執行

- 2026-11-17 User 明確決定：**Phase 1 MVP 完全不使用雲端 AI**，包含遠端 OCR、雲端 LLM／VLM、雲端 Embedding 生成、AI Agent 服務及失敗時偷轉 Cloud AI。
- 影像／文字理解、Goal／Concept 候選與必要 AI 特徵生成均在裝置執行。模型無法運作時，只能如實提示並允許人工修正／保存私人內容；不得將這種 fallback 宣稱已符合 B03 本地 AI 驗收。
- Supabase／PostgreSQL 的非模型服務仍用於經使用者同意的配對資料儲存、搜尋／向量距離運算、身分驗證、邀請、聊天及結果；公開欄位與授權仍以設計規格為準。
- **此原則已決定**；具體本地 Embedding 模型是否需要、硬體 runtime 與效能仍須技術驗證。

## 1. 不變的功能邊界（SOURCE）

- 使用者 Drop 圖片／文字，裝置端 OCR／E2B **只提議**內容與意圖候選；Content ≠ Goal／Need／Offer／Consent。
- Rust 是唯一可信本地 Parameter 寫入入口；SQLite 實施欄位完整性與 0–7 bounded slots；雲端以登入身分與 Server-owned 狀態再驗證。
- **每個 Intent 最多 8 種同時有效的 Matching Concepts**；內容數、歷史、私人詞彙不受 8 張／8 筆限制；UNKNOWN 若參與配對判斷也占 slot。
- 每次啟用／配對嘗試主動提問 ≤3（含 Goal 確認）；同 Pair 專屬提問 ≤2；每天未請求主動提問 ≤3。資訊足夠時不強迫發問。
- 任何時候只要缺合法確認事件、有效期限、scope、來源或分享授權，就不能視為已發布／可參與配對。
- 原圖、私人 OCR、私人筆記和完整本地歷史預設不離機；Embedding 本身也不能預設匿名。
- 兩個有效成年帳號且雙方接受才可 Chat；只有滿足 15 分鐘與雙方完成確認才能標記 COMPLETED。
- 未知不代表符合；向量高度相似不得繞過明確硬條件衝突、封鎖、撤銷、到期。
- P1 不做 Map、Travel、Dating、餐廳訂位、完整 Creature 遊戲或通用 App Builder。

## 2. 逐畫面的行為契約

### S01 — Drop（B01／B02）
- **前置**：裝置具備可運作的本地儲存；使用者可先保存私人內容，不以登入、雲端為第一價值必要條件（符合 Local-first 方向）。
- **輸入**：拍照／選圖／文字；保存來源、時間與可追溯來源 ID；既有資料與 Intent 不被新 Drop 無聲覆蓋。
- **成功**：保存私人原始內容，進入 S02 理解；重新啟動仍可讀取。
- **異常**：權限拒絕、損毀檔案、容量不足、空白文字、重複檔案、使用者取消，須有明確回覆，不可聲稱已儲存。
- **OPEN F-01**：批次圖片數、支援的影像格式／最大檔案、重複檔案處理政策、是否允許免帳號完整 S01–S03 路徑。

### S02 — Understand／Goal（B03／B04／B09）
- **前置**：有可讀私人 Content，或可人工輸入文字。
- **執行**：本地 OCR → E2B 產出候選 → 顯示來源與不確定性 → 讓使用者修正；若無可靠辨識，UNKNOWN，不捏造。
- **重要 Gate G1**：一張「パクる」圖片只能證明看到單字；Goal 未確認不能發布找人的 Intent。餐廳廣告亦同。
- **成功路徑 A**：使用者只要保存字詞／內容，留在私人本地流程，不自動前往 S04。
- **成功路徑 B**：使用者明確表達找學伴 Goal，Rust 記錄使用者確認事件，建立／更新私人 Intent Draft，再可前往 S03。
- **主動追問**：Goal 確認若屬 AI 主動詢問，消耗同次嘗試 1 題預算；答案不清楚維持 UNKNOWN。
- **異常**：OCR 未辨識、模型載入失敗、E2B 超時或錯誤格式，允許人工輸入及修正；這是保留既有本地內容的安全操作，不代表已替換本地 E2B 必須實作的 P0 要求。
- **OPEN F-02**：使用者「只收藏」後的返回目的地、AI 問題是否使用固定文字／模板。

### S03 — Intent／8-slot／Consent（B05–B08）
- **前置**：由使用者確認的 Goal；尚未確認的概念不能自動發佈。
- **參數來源**：每個概念具 concept_id、value_type、source/evidence_ref、scope/owner、state、expiry_policy、version、visibility、consent／confirmed event；受控 Registry／別名歸一，不允許模型任意新增正式 schema。
- **Slot**：只對同時在單一 Intent 參與匹配的 concept 計數；重用同義概念不新增 slot；超出 8 應阻擋；第 9 個必要硬條件進 BLOCKED_BY_BUDGET，停止 publish／propose，讓使用者明確處理，不能自動丟棄。
- **授權 Gate G2/G3**：顯示將分享的最少必要欄位；使用者主動確認後才發起同步；一次性資訊不升成永久 Profile。
- **生命週期**：DRAFT、ACTIVE、REVOKED、EXPIRED／CANCELLED 依正式 Intent Lifecycle 管理；過期、撤銷本地即時排除配對。離線撤銷進同步重試佇列，介面顯示尚未被雲端確認的風險。
- **異常**：必要條件缺失、version 衝突、授權未完成、過期時間不明、同步失敗 → 保留本地資料、禁止不合法公開。
- **OPEN F-03**：Consent 各欄位的預設選取與已核准的最少欄位清單、Consent version schema、8-slot 衝突互動決策。

### S04 — 找真人（B12／B13）
- **前置**：有效且已明確授權、Server 接受的公開 Intent；伺服器須再次核對版本、期限、匹配權限。
- **查詢**：先排除已撤銷／過期／封鎖／明確衝突的候選；再執行 Same Goal 與 Need↔Offer 候選召回。
- **成功**：顯示有效候選，進 S05；若無候選，S04a；不造假真實人數、成功率或候選。
- **異常**：未登入、授權拒絕、Intent 尚未同步、離線、雲端 401/403/5xx、被封鎖或版本過期 → 對應明確 UI 狀態，不回假候選。
- **OPEN F-04**：候選上限、搜尋刷新／分頁、搜尋等待政策、向量版本與來源。

### S04a — 無候選分支（B14）
- **來源已定**：無真人候選時可單人練習／選擇新候選通知；不可顯示假真人。
- **成功**：可返回本地學習／調整有效條件／稍後重試；若日後要通知，必須在明確 opt-in 後才啟用。
- **OPEN F-05**：「單人練習」具體行為（不能自行創建完整 AI Tutor）、通知管道／允許通知的條件、背景執行限制。

### S05 — 候選卡／排序理由（B14）
- **輸入**：伺服器返回、確實有效、經分享授權的最少必要候選欄位。
- **成功**：以可解釋原因呈現 Same Goal／雙向可行性／明確符合的條件；可略過、封鎖、檢舉；需要 Pair 問題時導向 S06。
- **不能做**：用相似度當作成功百分比；公開原圖／私人 OCR；展示不存在的真人。
- **異常**：候選後來過期、被封鎖、變更 Intent 版本，行動前由 Server 再驗證並顯示失效。
- **OPEN F-06**：候選公開欄位與排序權重、同分處理。

### S06 — Pair-specific 最少必要追問（B09／B15）
- **前置**：已存在特定有效候選 Pair，且關鍵 UNKNOWN 會影響可行性。
- **行為**：優先查既有同義 concept；一次提出真正能改變決策的問題。答案寫為 pair scope，限該 Pair 有效期限；必要公開條件變更須再次授權。
- **限制**：同一次啟用／配對嘗試主動問答總數不超過 3；單 Pair 問題不超過 2；剩餘關鍵 UNKNOWN 不可假設已滿足。
- **成功**：Rust 接受答案，重新驗證 → 重排／邀請；資訊已足夠可跳過 S06。
- **異常**：Pair 被封鎖、失效、答案過期、超限、版本衝突、Slot 預算滿 → 禁止不合法提案。
- **OPEN F-07**：全域啟用／嘗試 Question Budget 與單 Pair Budget 跨重新整理／重試如何計次，哪種問題是使用者主動而非 AI 主動。

### S07 — Mutual Invitation（B16）
- **前置**：兩方 Intent 有效，雙方可互相看到已授權摘要，無封鎖，通過必要硬條件。
- **行為**：A 發邀請→待 B 決定；接受／拒絕／逾時／取消／封鎖事件由 Server 按已登入身分記錄；**只有雙方有效接受後**才能建立可使用 Chat。
- **禁止**：接受前從其他 API 偷讀 Chat、拒絕後建立 Session、重放舊邀請繞過期限。
- **異常**：雙方同時操作、重複接受、版本變化、網路斷線，須 idempotent／重新查 Server 狀態。
- **OPEN F-08**：邀請 TTL、是否接受後立即建立 Session、通知方式及重複邀請節流值。

### S08 — Chat＋15 分鐘練習（B17）
- **前置**：Server 已確認 MUTUAL_ACCEPTED，且未封鎖、取消、撤銷。
- **行為**：僅 Pair 的兩名合法用戶可讀寫該 Chat；Session 時間由可信 Server 時鐘或 Server 承認的狀態轉換計算，不信任手機上報 duration；需頻率限制。
- **成功**：雙方可正常文字交流並開始／結束練習，時長達到門檻後前往 S09。
- **異常**：斷網、App 背景化、對方離開、封鎖、刪訊／檢舉、頻率限制、重複訊息，顯示真實 Server 狀態。
- **OPEN F-09（P0 阻擋）**：計時從「雙方接受」還是「明確開始 Session」起算？雙方必須同時在線嗎？背景／斷線是否扣時？累計或連續 15 分鐘？訊息保存期限與最多頻率。

### S09 — Completion／No-show（B18）
- **前置**：合法 Session，且符合已核准計時條件。
- **行為**：雙方以各自登入身分提交是否完成；完成 Gate G4 為：有效 ≥15 分鐘 + A CONFIRM + B CONFIRM；取消／No-show 另存結果，不得當 COMPLETED。
- **異常**：一方不回應、時間不足、重複確認、確認後另一方封鎖、版本爭議，須可重試、安全且不得重複計成功次數。
- **OPEN F-10（P0 阻擋）**：完成確認的期限、如何對待不同意、No-show 判斷條件、結果修正政策、取消後是否可重開 Session。

## 3. 跨畫面狀態與 Gate

| Gate | 時點 | 不符合時必須如何處理 |
|---|---|---|
| G1 Goal | S02 | 保持私人內容；不產生可發佈意圖 |
| G2 Parameter | S03／S06 | Rust 拒絕違規寫入；SQLite 再限制；第 9 硬條件 BLOCKED_BY_BUDGET |
| G3 Consent & Safety | S03／S07 | 不分享未批准欄位；伺服器拒絕失效／越權／未雙方接受請求 |
| G4 Outcome | S09 | 只記錄真實結果；不能將點擊／聊天／單方確認當成完成 |

### Intent 與配對生命週期（SOURCE，非完整轉移規格）

- Intent 由 PRIVATE DRAFT → 有明示同意與驗證的 ACTIVE；可被修改、撤銷、過期／取消。
- v1.7 高層 Flow：DRAFT → ACTIVE → CANDIDATE → CLARIFYING → PROPOSED → MUTUAL_ACCEPTED → SESSION → COMPLETED；可進 REJECTED／EXPIRED／CANCELLED。
- **PROPOSED IMPLEMENTATION**：將 Intent Lifecycle、Pair Lifecycle、Session Lifecycle 分別保存並連結，避免把「候選」誤認為全域 Intent 只有一個狀態；資料模型仍以 v1.7 Server／Local 跨層校驗為準，待 Technical Spec 與 Human Review 確認。

### 使用者與模型權限責任（SOURCE）

| 角色 | 允許 | 不允許 |
|---|---|---|
| OCR／E2B | 提出有 evidence 的候選內容／概念 | 自稱 CONFIRMED、公開、繞過 Rust／SQLite |
| UI | 呈現資訊；由真人觸發確認、授權、撤銷請求 | 自己寫 Parameter／提供模型任意 event ID |
| Rust | 驗證、標準化、Atomic Transaction、來源與事件管理 | 把模型猜測當人類操作 |
| Cloud | 以 Auth + Server-side 權限／版本／Consent／TTL 判斷公開與匹配 | 只相信手機傳的 confirmed flag、分數與時間 |

## 4. 決策清單與 Freeze 前條件

| ID | 需確認 | 主要 B-ID | 等級 |
|---|---|---|---|
| F-01 | Drop 格式／大小／批次上限 | B02 | 執行細節 |
| F-02 | S02 只保存的返回路徑與主動提問判定 | B04／B09 | 使用者流程 |
| F-03 | Consent Payload 欄位、版本與第 9 條件的實際處理方式 | B05–B08／B12 | **P0** |
| F-04 | Matching Top-K／刷新策略／向量生成來源 | B13–B14 | **P0** |
| F-05 | S04a 單人練習與通知精確範圍 | B14／B21 | **P0** |
| F-06 | 候選摘要欄位與 ranking baseline 權重 | B14 | **P0** |
| F-07 | 3 題與 2 題跨裝置／跨重新嘗試的計數策略 | B09／B15 | **P0** |
| F-08 | 邀請期限／去重／取消／通知 | B16 | **P0** |
| F-09 | 15 分鐘 Session 時計算與斷線規則 | B17–B18 | **P0** |
| F-10 | Completion、No-show／取消細節 | B18 | **P0** |
| F-11 | 18+ 自我聲明與實際驗證機制、濫用處理的最低門檻 | B10 | **P0** |
| F-12 | 離線撤回後「本地即時停止」與「雲端確認下架」的產品文案、同步狀態 | B12 | **P0** |

**Human Freeze 規則**：每項決策以「選項 → 取捨 → User Approval → Commit SHA → 影響 S/B/T」記錄；未決事項留 DRAFT，不能讓 AI 直接定案。已核准的行為若與 v1.7／v1.8 不同，先正式修改來源設計與 Bible 引用關係。

## 5. 與其他規格的責任分工

- GUI/UX：GUI_UX_SPEC.md（只擁有 layout／文案呈現／狀態與跳轉提案）。
- Technical：TECHNICAL_SPEC.md（資料契約、資料表、Server 驗證邊界）。
- Acceptance：../test/ACCEPTANCE_SPEC.md（測試預期與 Evidence）。
- Bible：../build/BACKLOG.md（唯一 Build Backlog 與實際進度；本文件不另建 B 項或變更優先級）。
