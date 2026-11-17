# CAKAMI｜Business Proposal v1.2 — MVP LOCKED

> **Canonical business design** | Source: `CAKAMI_Business_Proposal_v1.2_MVP_Aligned.docx` | Aligned with System Design v1.7 | Decision date: 2026-11-17  
> This is the **only current business proposal** in `design/doc/`. Older versions are not part of this repository.
> Business figures and VC terms are **scenarios**, not independently verified projections or committed funding.

## 1. 產品定位與 MVP

**CAKAMI = Drop Anything → Discover People → Accomplish Together.**

一句話：使用者丟入圖片、筆記或想法，CAKAMI 先在裝置上理解，找出目標相同或能互相協助的人，讓雙方真正完成一件事。

**Phase 1 唯一切入市場：** 18 歲以上 JLPT N2 學習者；Android 優先。使用者 Drop 學習圖片／文字，透過本地模型辨識、親自確認 Intent，再授權同步最少必要資訊，尋找真人學伴，共同完成 **15 分鐘文字練習**，由雙方確認結果。

**使用者價值循環：** DROP → UNDERSTAND → MATCH → ASK → MUTUAL CONSENT → CHAT → COMPLETE。

### Phase 1 鎖定範圍

| 決策 | MVP 規格 |
|---|---|
| 目標族群 | 18+ JLPT N2 Study Match；Android first |
| 核心操作 | Drop 圖片／文字、確認 Intent、找候選、雙方同意、文字 Chat、15 分鐘共練與完成確認 |
| Local-first | 原圖、OCR、完整私人 Intent 和歷史保存在手機 SQLite；使用者明確授權後才同步最少配對資訊 |
| 8-slot budget | 每個 Intent **最多 8 種同時有效、可參與配對的 Parameter concepts**；不是 8 次 Drop、8 個單字或每人最多 8 筆資料 |
| AI 問答 | 每次 Intent 啟用／配對嘗試 **最多 3 題 AI 主動提問**，包含必要的目標確認；答不出來保持 UNKNOWN |
| 安全 | Rust Controller + SQLite 強制來源、適用範圍、確認狀態、有效期、版本、授權與 8-slot 原子寫入；Server 端再次驗證 |
| 後續階段 | **P2 Map / Attraction 試點**；**P3 Travel Match 擴張** |
| 不在 Phase 1 | 地圖 UI、精確位置追蹤、餐廳訂位、供應鏈整合、完整 Creature 遊戲、Dating、任意 AI App 生成 |

### 重要區別

- 內容辨識 **不等於** 使用者意圖。例如圖片只有「パクる」時，可以本地保存單字，但不能直接假定使用者想找學伴。
- E2B 只能提出參數候選；**Rust 決定是否有效、可保存或可公開**。
- 一次性時間不能誤當永久偏好；優惠到期、Intent 到期及配對有效期需分開管理。
- AI 不得把未確認的猜測當成使用者同意。

## 2. Phase 1–5 商業路線

| Phase | 產品範圍 | 前進條件 |
|---|---|---|
| **P1 Study Match** | 18+ JLPT N2、Drop、8-slot Intent、最多 3 題、雙方邀請、Chat、15 分鐘學習 | 首輪 100 人完成至少 20 場雙方確認的練習；P0 測試與匹配基準測試通過 |
| **P2 Map / Attraction** | 在單一地區試行景點／活動位置卡、粗略距離與同行時間 | P1 取得可重複的合作成果；位置隱私、地理運算與安全可驗證 |
| **P3 Travel Match** | 基於 P2 Places 資料，試行旅伴、時間重疊、城市探索 | 城市配對密度、實際完成率、安全達標 |
| **P4 Skill / Create** | 技能交換、共同完成小作品，測試互補／加成／協同 | 跨領域仍有良好完成率和合理獲客成本 |
| **P5 Intent Network** | 多 Wishlist 的通用 Consumer Network，視需求研究第三方 API / Protocol | 持續自然成長、有效供需密度與可驗證付費模式 |

P2–P5 是附條件的探索方向，不是承諾的發布日期。**現階段只投資 P1 驗證。**

## 3. Phase 1 商業目標（探索性驗收）

| 指標 | 初期門檻 |
|---|---|
| 第一次價值 | ≥80% 測試者能從一張圖片建立並確認學習 Intent |
| 真實成果 | 100 名成年測試者中，至少完成 **20 場**雙方確認的學習練習 |
| 留存 | D7 回訪 ≥25%；記錄第二個 Intent 和自發邀請 |
| 演算法品質 | 相同候選池、安全要求下，動態方法比固定表單／純向量搜尋：提問更少且有效完成率更高 |
| 8-slot Integrity | 第 9 種有效概念須被拒絕或由使用者明確替換；硬條件不得被默默移除 |

**North Star：每 100 個有效 Intent 真正完成的 15 分鐘 Sessions。** 不把點擊、聊天訊息數、相似度分數當作成功。

## 4. SWOT

| | |
|---|---|
| **Strengths** | 一張圖片即可起步；可推遲不必要的表單；對相同與互補需求都能配對；本地推論減少雲端 Token 成本 |
| **Weaknesses** | 冷啟動需要同一需求池的活躍密度；小模型辨識可能出錯；陌生人配對需安全設計 |
| **Opportunities** | Study → Map/Travel → Skill/Create 共用動態意圖處理；成功合作產生可改善排序的結果資料 |
| **Threats** | 2Connect、Anuma、活動與學習配對產品已有部分能力；大型平台可複製 Local AI 和普通向量搜尋 |

## 5. 營收與三年假設試算

P1 先免費獲客。**2028 年起**假設引入 Freemium：基本配對／聊天免費；Pro 暫定 **¥980／月**，提供多 Wishlist、進階偏好與 AI 協作摘要。定價、付費率均未驗證。

| 年度 | 平均 MAU | 付費轉換 | 年收入 | 年成本 | 稅前損益 |
|---|---:|---:|---:|---:|---:|
| 2027 | 1,500 | 0% | ¥0 萬 | ¥460 萬 | **−¥460 萬** |
| 2028 | 12,000 | 5% | ¥706 萬 | ¥930 萬 | **−¥224 萬** |
| 2029 | 60,000 | 5% | ¥3,528 萬 | ¥2,600 萬 | **+¥928 萬** |

`年收入 = 平均 MAU × 付費率 × ¥980 × 12`。成本假設涵蓋創辦人工時估值、雲端／支付、客服、安全及行銷。未計融資與稅。2029 年若其他條件不變，損益兩平約需 **44,200 平均 MAU**。這是單一敏感度情境，不是市場預測。

## 6. Founder 決策

- **現在做：** P1 Study Match，證明真實共同練習需求、8-slot 的可靠性與動態配對效益。
- **現在不做：** 通用 Messenger、Dating、完整 Creature 養成、購票團購、任意 AI App 生成。
- **GO / NO-GO：** 若人們 Drop 很多但不完成練習，或動態追問無法勝過固定表單，停止擴張並重新設計。

## 7. VC Investment Thesis — 為什麼這可能有規模？

VC 投資的不是「便宜的 AI 學伴 App」，而是 **把模糊私人意圖轉化為真正完成的合作**、能跨領域擴展的 Consumer Network。

| 論點 | 可能形成的機制 | 需要證明 |
|---|---|---|
| 本地 AI 降低推論邊際費 | E2B 在手機上完成圖片理解、Intent 抽取和追問，減少逐次雲端 LLM API 成本 | 目標手機速度、耗電、辨識品質、雲端 fallback 頻率 |
| 低摩擦促進有效輸入 | 多次 Drop、修正和新增 Wishlist 不必為每次都付 Cloud LLM Token | 是否增加**有效且經本人確認**的 Intent，不只是噪音 |
| 配對網路可能複利 | 更多合法授權、有效且活躍的雙向供需，提高可行候選密度 | 完成率、留存、自然邀請是否相互促進 |
| Outcome data 的後期壁壘 | 根據真人是否真的完成合作而不是點擊，改進排序與提問 | 同等候選池下是否顯著勝過對照組 |

### 成長飛輪（待驗證）

**Local AI 低雲端推論費 → 更多經確認的 Intent → 更高有效配對密度 → 更多真正完成的合作 → 更高回訪／邀請 → 更多有效供需。**

這是**可測試的因果假說**，不是已證實的 Network Effect。用戶增加但分散在不同市場或時段，匹配品質仍可能下降。

### 不能只比較 Token 成本

Local AI 不代表全部成本為零：裝置耗電、模型下載、機型相容性、雲端向量搜尋、驗證、聊天、推播、隱私與安全運營都需要付費。

**每完成一場合作的完整成本**：

`(雲端搜尋 + 資料庫 + Chat/推播 + 驗證/安全 + Fallback LLM + 營運支援) / 確認完成場次`

VC 看的是**單次真實成果的單位經濟**，而非單次 AI 呼叫有多便宜。

## 8. 模擬 VC 投資情境（非正式承諾）

| 條件 | 假設 |
|---|---|
| 模擬投資意向 | 附里程碑的 Pre-seed YES（角色扮演；並非任何真實投資承諾） |
| 模擬投資額 | **US$500,000** |
| 模擬投前估值 | **US$4,500,000** |
| 模擬投後估值 | **US$5,000,000** |
| 取得股權 | **10%** |
| 擬議用途 | 裝置測試、基準配對測試、集中式 Study Match 社群、隱私和安全 |

### 三道投資門檻（候選驗證標準）

1. **Local AI：** 目標 Android 裝置上，主要學習意圖經人工驗證的辨識正確率 ≥90%，速度與耗電可接受。
2. **真人配對：** 100–200 名 18+ 測試者；雙方已同意的配對中，≥30% 完成至少一次 15 分鐘練習。此數字和「100 人完成 20 場」的分母不同。
3. **演算法：** 相同候選、安全與曝光規則下，比固定表單和純向量達到更多完成合作／100 有效意圖，同時控制問題量、延遲與單場成本。

**加註：** Rust 8-slot 原子寫入、SQLite slot 0–7 限制、Cloud 再驗證與全部 P0 測試都通過，才開放真人配對。

## 9. 潛在 Algorithm Moat

**可快速複製的工具：** Gemma E2B、SQLite、pgvector、一般向量匹配。  
**有機會形成防禦性的能力：** 動態參數挑選 + 最少必要追問 + 雙向可行性排序 + 真實 Outcome Learning，加上高密度且活躍的需求網路。

| Moat 組件 | 使用者能感受的價值 | 需要累積的證據 |
|---|---|---|
| Dynamic Parameter Discovery | 不先填 20 格，遇到真正可能匹配的人才補關鍵問題 | 少問仍能提升完成率 |
| Reciprocal + Feasibility Ranking | 不只相似；檢查 Need/Offer、共同時間及合作方式 | 同樣候選池下的結果提升與可解釋性 |
| Outcome Learning | 以完成 Session 的結果改善後續配對 | 合規授權且足量的可信結果資料 |
| Intent Network | 足夠密集的有效需求供給 | 留存和自然邀請，不能只靠註冊數 |

### 每週飛輪 KPI

1. 每位活躍用戶新增加、經本人確認的有效 Intent 數
2. 至少有一個雙向可行候選的 Intent 比例
3. 每 100 個有效 Intent 完成的 Sessions
4. 每場完成 Session 所需的 AI 提問數
5. 每場完成 Session 的雲端及營運成本

另追 D7/D30 留存、自然邀請率、位置／資料隱私事故與撤銷時效。

## 10. MVP 驗證與規格凍結

**8-slot 是 Phase 1 的產品預算，不是學術最優值。** 必須以相同資料與候選人池比較：
- 固定表單 Matching
- 單一向量 Matching
- 多向量 + Constraints
- CAKAMI 8-slot + Dynamic Questions

**新增概念規則：** E2B 可以提建議，但只有 Concept Registry 已定義且 Rust 驗證通過的概念才能進入 8 個 active slots。未知的新概念只能暫留私人候選區；不可自行升級為公開配對欄位。第 9 個必要概念不能偷偷刪除硬條件；需暫停發佈並讓使用者明確選擇。

**後續地圖順序：** P1 只預留 place/time/visibility 概念；**P2** 才加入 Map/Attraction 試點；**P3** 才擴張 Travel Match。

**規格配對：** Business Proposal **v1.2** ↔ System Design **v1.7**。最新 Functional Overview 曾為 v1.8；它不取代本文件的 Business Scope。

---

*This GitHub-native Markdown is a structured transcription of the latest approved Business Proposal v1.2. No older business proposal is included in this repository.*
