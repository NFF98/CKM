# CAKAMI｜User Story 驅動的 3 Sprint 執行計畫（2026-11-19 修正版）

> **取代舊 B01→B18 逐項交付的開發順序。** 既有三個 Sprint、B01–B21 編號、System v1.7 T01–T13、Functional v1.8、Business Proposal v1.2、MVP Study N2 範圍及安全規則均保留。**BXX 是模組／工程任務，User Story 是開發與驗收主線。**
>
> **現況（真機 Evidence）：US-01 = FAIL**。Gemma 能解釋 `パクる`，但沒有完成「推測→追問→本人確認→可用私人 Intent」；B05 單元測試 PASS 不等於 US-01 PASS。US-02、US-03 = NOT_RUN。詳見 [進度表](sprints/SPRINT-01_PROGRESS.md)。此文件修改的是工作計畫，**不是已修好程式或重發 APK**。

## 0. Business North Star 與不可改的邊界

**Drop Anything → Discover People → Accomplish Together**。P1 只發布 **18+ JLPT N2 Study Match**：Android 上輸入文字或 Drop 圖片，本地 AI 協助理解與確認私有 Goal；本人授權後 Cloud 找到同目標／Need↔Offer 相容的真人；雙方接受、15 分鐘文字練習、雙方確認完成。後續 Fashion/Car/Travel 可沿用通用引擎，但**不可因做跨域測試就開放 P1 以外的雲端配對**。

保留 System v1.7：每個 Intent **最多 8 種已核准且有效的配對 Concept**（不是 8 個固定問題或固定欄位）；一次意圖/配對嘗試 AI **最多主動追問 3 題，包含 Goal 確認**；Pair-specific 最多 2 題。E2B 只提出候選，Rust/SQLite 確保合法，Goal 必須本人確認，對外分享另有 Consent。P1 **無 Cloud AI**：Supabase 只做 Auth／授權資料／匹配規則／Chat／結果，不對原圖、OCR、筆記做雲端 AI 推論。

## 1. 三條 User Story，才是開發／驗收交付

| User Story | 真正要讓使用者做到什麼 | 動用既有 BXX | 交付 Gate |
|---|---|---|---|
| **US-01：我 Drop 文字／圖片，CAKAMI 幫我搞清楚要找誰做什麼** | **理解→提出意圖候選→必要追問→本人確認→儲存合法私人 Intent／參數**；不知道時留 UNKNOWN 或只收藏 | **B01–B07、B09**，B08 先準備私有分享預覽；核心跨 **B02→B03→B04→B05→B06→B09** | 同一支 Android APK 的完整流程＋真 SQLite 狀態、8-slot/同意安全負例 PASS；不是單一 BXX CI PASS |
| **US-02：我同意後，CAKAMI 找到真正合適的人** | 顯示要分享內容→本人授權→真 Cloud 最少必要同步→Same Goal／互補 Need↔Offer＋硬條件→候選或真實無候選 | **B08、B10–B15**，依賴 US-01 合法 Intent；B07 撤銷／到期 | A/B 真測試帳號，授權前不得公開、合法候選能找到、不相容／封鎖／過期／未同意必被排除；Server/RLS PASS |
| **US-03：我和對方真正完成合作，結果會留下來** | 真人邀請→雙方接受→Chat→15 分鐘→雙方確認→Server 可信完成→**本人可見 Outcome 回私人本地歷史** | **B16–B18**，跨 B02/B07 的結果回存契約（新設計草案需批准資料欄位） | A/B 真 Cloud Session、合法只有 1 筆 COMPLETED、負例不計完成；結果回流/重試有證據（僅已核准資料） |

**最終使用者成功不是按了按鈕／模型答對單字；而是產生合法 Intent → 找到真正相容的人 → 真正完成合作。** Business Proposal North Star：每 100 個有效 Intent 的完成 Sessions；對照方法是驗證技術假說，不是已證實的 MOAT。

## 2. 仍保留三個 Sprint，但每個 Sprint 以 User Story 切片跑通

| Sprint | User Story／切片 | 交付成果 | 何時叫 PASS |
|---|---|---|---|
| [Sprint 1｜US-01 本地意圖](sprints/SPRINT-01_LOCAL_AI.md) | **US-01A** 純文字→模型→私人候選；**US-01B** 含 OCR 的圖片→同一引擎；**US-01C** 關鍵追問→Goal 本人確認→B05 合法參數／B06 上限；**US-01D** 版本／過期／只收藏／離線／分享預覽 | 可測試的完整本地體驗，支援 UNKNOWN，Study 不寫死成所有內容的 Goal | **US-01A～D** 真機流程、安全 DB/Contract、相關 T01–T13 子集逐條實測 PASS；BXX 子項仍各自留證據 |
| [Sprint 2｜US-02＋US-03 真人閉環](sprints/SPRINT-02_MATCHING.md) | **US-02A** Consent＋Auth＋Cloud 同步安全；**US-02B** 真候選/無候選；**US-03A** 邀請/雙方接受；**US-03B** Chat/Session/Outcome；**US-03C** Outcome 回本地 | A/B 雙真實測試帳號跑完整條路，Cloud 不跑模型 | 各完整 Story 的 Cloud E2E、權限與負例 PASS；**US-02 PASS 不代表 US-03 PASS** |
| [Sprint 3｜Release & Regression](sprints/SPRINT-03_RELEASE.md) | **US-01～03** 全流程回歸＋**B19** 安全/權限/效能，**B21** Android Pilot；**B20=P1** 公平比較 | 真機＋Cloud 一次通過：Input → 合法 Intent → 相容真人 → 雙方完成 → 私人可信結果 | **B19 所有 P0**、T01–T13、AI-LOCAL、E2E-01～05、User Story Gate 全 PASS 才 B21；B20 **不是**發版 P0 阻擋 |

**原 B-ID 可以獨立開 PR、獨立跑 CI，但不能單靠某個 PR、單元測試或模型詞義答案標記 User Story 完成。** 若某切片 NG，立刻跨相關 BXX 修，直到它從使用者入口真的走到結果。無依賴的工程工作可並行，不用讓全案停擺。

## 3. 從現況立即繼續的順序（不是重新搭 B01）

| 順序 | 實際工程工作 | 必須先看到什麼證據 |
|---|---|---|
| **NEXT-01 / US-01A** | 針對 **純文字** 修 B03→B04：Gemma 提有來源、可為 UNKNOWN 的 `PrivateIntentSuggestion`；UI 能選／修改→Rust 本人確認成私人 Goal | `我想找 JLPT N2 學伴練口說` 的 Android 真機 Input→私人候選→確認→SQLite；`パクる` 不能被強迫變學習意圖 |
| **NEXT-02 / US-01B** | 圖片經 B03 OCR/更正進**同一條** Intent 流程；跨域 Fashion/Car OCR 文字作負例，沒有真視覺輸入時不可亂猜 | N2 教材／Fashion／Car／空白/歧義真機案例，來源修正使舊建議失效 |
| **NEXT-03 / US-01C** | B05 已確認 Goal→受控參數候選，B06 真實 8-slot，B09 問最關鍵缺失、≤3 題（含 Goal） | 不是 AI 自填「CONFIRMED」；合法 Concept 可確認，第 9 必要條件拒絕，未知不亂配，問 0～3 題 |
| **NEXT-04 / US-01D** | B07 期限、修改撤銷、離線狀態，B08 本地共享預覽（**不偷同步**）；完整 US-01 Android 真機重測 | 只收藏無 Goal；私有資料重啟持久；無 Consent 無網路上傳；US-01 全部可重現 |
| **NEXT-05 / US-02→US-03** | Consent、Cloud Matching、真人合作與可驗證 Outcome，依原 B10–B18 安全依賴實作 | 每條 US 在真 Cloud 與雙帳號實測通過，負例/權限也過 |

**避免 Trial & Error 的先決條件：** 先訂定 US-01 資料契約與測試 fixtures（包含純文字、歧義詞 `パクる`、Study／Fashion／Car、已確認／拒絕），在 Rust/TS 跑跨模組 Contract Tests；再整合 Android 批次驗收。保留既有「至少 2 項獨立功能準備好才打包新 APK，P0 阻斷例外」節奏；打包與否**不能**改變 PASS 定義。

## 4. 每一條 User Story 的固定工作節奏

```text
選一條 User Story → 寫真人操作/預期結果/失敗例
→ 找出需要改的 BXX → 定義來源/候選/確認/權限/持久化契約
→ 同一個 Fixture 做跨 BXX 的 CI/SQLite 測試
→ Android／Cloud 真 E2E → 用戶實測 PASS 或 NG
→ 更新 User Story + BXX 兩層 Evidence，NG 直接修到通
```

**Gate 兩層不可混淆：**
- **模組 Gate**：BXX 的單元／契約／安全／實體測試通過，可稱「該 BXX 的相應 Coding/CI 完成」。
- **產品 Gate**：US-01／02／03 的用戶旅程在真機／真 Server 完成，且必要拒絕／撤銷案例通過，才能稱該故事 PASS。

## 5. 仍需守住的工程與產品決策

- 既有 B01～B05 程式和 Stacked PR 原封保留；不因換 Sprint 計畫重寫或合併，也不抹掉原有 PASS/FAIL Evidence。
- P1 仍 18+ JLPT N2；Cross-domain 研究 Fashion／Car 只檢查「不亂誤判」與 Core 可擴展，未批准不開放 Cloud。圖片無文字時，現有 B03 **只有 OCR→文字 Gemma，無真正 Pixel Vision**，應顯示 UNKNOWN。
- 8-slot 是有效 Matching Concept 的**容量上限**，不是預設固定八欄；詳細 Concepts 白名單 T-01 仍 OPEN，不能自行捏造 Need/Offer/時間是已核准 Registry。
- B09 主動追問每次最多 3 題（含 Goal）、B15 Pair-only 2 題；單一問題可確認多個合法候選，資料不足留 UNKNOWN。
- 雲端只得到經本人同意的必要摘要，Server 重新驗證；拒絕、過期、封鎖、撤銷一律不可配對。B18 Server 驗證兩方完成；Outcome 回私人手機的具體欄位／Consent 與跨人聚合仍需 Human 批准，不能偷加公開資料或 Cloud AI。
- P0 新使用者驗收案例詳見 [Acceptance Spec：US Gate](../test/ACCEPTANCE_SPEC.md)。**原 E2E-01～05 與 T01–T13 不改名、不刪除**。
- 遇到尚未核准的公開欄位、成年確認、Session 計時／Outcome 規則，只阻擋相關 Cloud 功能，不停掉本地 US-01 私人候選 Coding。

## 6. 執行文件與接棒

- [Build Bible B01–B21](BACKLOG.md)：任務目錄／原有依賴；[Traceability](TRACEABILITY.md)：**先選 User Story，再映射 BXX**。
- [Sprint 1：US-01](sprints/SPRINT-01_LOCAL_AI.md)｜[Sprint 2：US-02/03](sprints/SPRINT-02_MATCHING.md)｜[Sprint 3：Release](sprints/SPRINT-03_RELEASE.md)。
- [Sprint 1 Progress](sprints/SPRINT-01_PROGRESS.md)：**US-01 FAIL**，既有 B02/03/04/05 狀態保留，不能以舊表上方歷史 NOT_RUN 抹掉使用者最新 NG。
- [Action Intelligence Universal Intent Engine DRAFT](../design/ACTION_INTELLIGENCE_ARCHITECTURE_v1_DRAFT.md) 是新候選契約的**設計草案，不是已實作的正式公開規則**。詳細 SOURCE 仍以 Business v1.2／System v1.7／Functional v1.8 為準。
