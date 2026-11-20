# CAKAMI｜Phase 1 MVP 建置待辦總表（Build Backlog）

**狀態：B01–B05 已有 Coding／部分真機 Evidence，US-01 端到端產品測試 NG；其餘依 [實際進度](sprints/SPRINT-01_PROGRESS.md) 為準**  
**計畫更新日期：2026-11-21**  
**MVP 市場：18 歲以上 JLPT N2 學習者；Android 優先；S01–S09，以及 S04a 無候選人的分支**  
**語言原則：[CKM 繁體中文溝通規範](../communication/COMMUNICATION_RULES.md)**

> **此文件是 B01–B21 唯一工程任務目錄，不是產品交付順序。** 2026-11-19 起 [三個 Sprint 改為 User Story First](SPRINT_PLAN.md)：先定義 US-01／02／03 使用者完整成果，跨相應 BXX 實作／測試。BXX 原編號、工程責任、依賴、安全紅線及正式 Test IDs 保留；一個 BXX CI PASS 不表示任何 User Story PASS。

## 一、設計依據與產品邊界

建置前須共同參照以下三份最新文件：

1. [Business Proposal v1.2](../design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)：MVP 目標市場、商業假設、成果指標。
2. [System Design v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)：系統架構、Rust 驗證、SQLite 8-slot 與配對演算法。
3. [Functional Design v1.8](../design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx)：S01–S09 功能對應、畫面流程、內外部 Sequence Diagram。

### 技術骨架（規劃）

`Tauri v2 + TypeScript`（畫面）→ `Rust`（驗證與控制）→ 本地 OCR／Gemma E2B 候選推論方案 → `SQLite`（私人資料）。

使用者明確授權後，才將最少必要的配對 Intent 傳到雲端（規劃採 `Supabase Auth + PostgreSQL + pgvector`，並支援雙方邀請及聊天）。**本地模型在 Android 上的相容性、速度與正確率仍需實測，不得視為已驗證。**

### 開發中不得違反的原則

- **內容不等於意圖**：辨識到「パクる」或餐廳廣告，不代表使用者要找學伴或一起用餐。
- **AI 不得捏造事實**：OCR／E2B 建議不能直接當成使用者已確認的 Goal、Need、Offer、時間或分享同意。
- **每個 Intent 最多 8 個有效配對概念**：SQLite slot 0–7；這不是 8 張圖片、8 個單字、8 筆資料或每人 8 種屬性。第 9 個必要條件不得默默取代既有硬條件；應阻擋發布並要求使用者決定。
- **主動追問預算**：每次 Intent 啟用／配對嘗試最多 3 題（包含目標確認）；Pair-specific 追問最多 2 題；每日未請求主動追問最多 3 次。以上沿用目前設計，需以真實使用測試校正。
- **Rust 是本地唯一可信寫入入口**：驗證來源、範圍、確認狀態、資料型別、期限、版本、分享權限；SQLite 另設限制；雲端發布也需再次驗證。
- **私人資料預設留在裝置**：原圖、OCR、私人筆記、完整歷史與個人資料不因配對而自動上傳。Embedding 也可能包含敏感資訊，不視為天然匿名。
- **真正完成才算成功**：15 分鐘文字學習需要雙方確認，不能把候選卡點擊、聊天訊息或單方聲稱當成完成。
- **安全與同意**：第一階段只服務 18 歲以上；公開 Intent 須明示同意；雙方接受後才開放聊天；須支援封鎖、檢舉、撤銷及過期。
- **MVP 不做**：地圖與位置追蹤（Phase 2）、Travel Match（Phase 3）、餐廳訂位、供應鏈整合或完整遊戲養成。

## 開發交付改為 User Story（2026-11-19，使用者決策）

**US-01（Sprint 1 / 當前 NG，先修）：** 用戶輸入純文字或 Drop 圖片 → 本地理解 → 可能意圖（或 UNKNOWN）→ 少量必要追問 → 本人確認 Goal／合法參數 → 本地 SQLite 持久化。工程跨 **B02–B06、B09**，加上 B01 Bridge、B07 生命周期與 B08 本地分享預覽。**現在不是完成 B06，而是先完成 US-01A「純文字→候選→本人確認」並實測。**

**US-02（Sprint 2 / NOT_RUN）：** 確認 Intent → B08 本人同意 → B10–B12 合法最小同步 → B13–B15 真 Cloud Same Goal／Need↔Offer＋硬條件、候選／無候選。不得把 CI／SQL mock 當真人候選。

**US-03（Sprint 2 / NOT_RUN）：** B16 雙方邀請接受 → B17 Chat／15 分鐘 → B18 雙方確認可信結果 → 經授權的本人 Outcome 回本地 SQLite。跨使用者的聚合回饋尚待資料用途與同意決策，不能自動納入 MVP。

**B19／B21（Sprint 3）**：完整 US-01～03 和原 T01–T13／E2E-01～05 安全真機/真 Cloud PASS 方能 Pilot。B20 仍是 P1。每條 Story 的具體 Gate 以 [新 Sprint Plan](SPRINT_PLAN.md)、[Acceptance Spec](../test/ACCEPTANCE_SPEC.md) 為準。

**BXX 仍有不可跳過的安全先後依賴**，但實作應以同一 User Story 的前端 → AI → Rust → SQLite／Server 真狀態作一組整合，不再讓「B03 詞義解釋 PASS、B04 手填 PASS、B05 Registry PASS」冒充 US-01 PASS。P1 只上線 JLPT N2；Fashion/Car 用來驗證通用 Core 不誤分類，未批准不得雲端發布。

## 二、功能式建置待辦（依賴排序；Sprint 分組另見計畫）

優先級：**P0**＝MVP 必須完成；**P1**＝驗證品質優化。  
進度以 [Sprint 1 實際 Coding／真機紀錄](sprints/SPRINT-01_PROGRESS.md) 為準：B01–B05 已有部分 Coding，B03→B04 Action Intelligence 真機 **FAIL/NG**；B06–B09 尚未開始。其他 BXX 不以「表格預設未開始」覆蓋真實證據。

| ID | 優先 | 功能交付內容 | 畫面 | 依賴 | 主要驗收條件 |
|---|---|---|---|---|---|
| **B01** | P0 | 建立 Tauri v2／TypeScript／Rust Android 專案骨架與基本導航 | 全部 | — | 真正 Android 裝置能執行；UI↔Rust 指令正常；本地功能不依賴雲端 |
| **B02** | P0 | 拍照、圖片／文字 Drop、私人檔案保存 | S01 | B01 | 重啟後仍可讀取；原始內容不會被自動上傳 |
| **B03** | P0 | 本地 OCR 與 E2B 模型接入、錯誤處理、人工修正 | S01–S02 | B01–B02 | 可將「パクる」辨識為內容；無法辨識時標示未知，不編造資訊 |
| **B04** | P0 | 分離內容辨識與 Intent；建立可人工編輯的意圖草稿 | S02 | B03 | 單字或餐廳廣告在目標未確認前，不能產生可發布的配對 Intent |
| **B05** | P0 | Parameter Concept Registry 與 Rust 驗證：來源、類型、scope、確認狀態、期限、版本、可見性 | S02–S03／S06 | B04 | 同義概念合併；未知種類僅能暫存候選；AI 無法偽造確認事件 |
| **B06** | P0 | SQLite Schema、交易式 8-slot 約束、來源證據與確認事件 | S03／S06 | B05 | slot 0–7 有效；第 9 個衝突寫入被阻擋；並發測試通過；不能默默刪除硬條件 |
| **B07** | P0 | 本地 Intent 清單、編輯、撤銷、到期處理及跨範圍資料使用 | S03 | B06 | 單次有空不變成永久偏好；過期 Intent 不參與配對；離線讀寫有效 |
| **B08** | P0 | 分享授權、預覽公開內容及同意紀錄 | S03 | B07 | 使用者能看見將上傳哪些欄位；預設私人；未明確同意不可公開 |
| **B09** | P0 | 高價值缺失條件追問及提問預算 | S02／S06 | B05–B07 | 只針對會影響決策的 UNKNOWN 追問；每次最多 3 題；資訊足夠時不強迫提問 |
| **B10** | P0 | 成年用戶帳號、驗證、權限及濫用防護策略 | 雲端 | B01 | 不同帳號資料正確隔離；有 18+ 條件確認與封鎖／檢舉流程 |
| **B11** | P0 | 雲端 Intent、候選、邀請、聊天、結果資料表及伺服器驗證／RLS | S03–S09 | B06、B10 | 不能讀取其他人的私人資料；過期、撤銷或無效 8-slot Payload 被拒絕 |
| **B12** | P0 | 使用者同意後同步最少欄位；撤銷與到期下架 | S03／S04 | B08、B11 | 同意前不上傳；撤銷後不得繼續被搜尋；同步結果與失敗狀態可追蹤 |
| **B13** | P0 | Same Goal／Need↔Offer 候選搜尋與硬條件過濾 | S04 | B11–B12 | 只查真人有效候選；時間或權限不符，即使向量相似度高也須排除 |
| **B14** | P0 | 候選排序、可解釋原因與 S04a 無候選畫面 | S04–S05 | B13 | 呈現真實候選與匹配理由；無人時不虛構真人或成功機率 |
| **B15** | P0 | Pair-specific 追問、重新驗證與候選重排 | S06 | B09、B14 | 只追問關鍵缺失；答案限於該次 Pair 的有效範圍及期限 |
| **B16** | P0 | 雙方邀請、接受／拒絕、封鎖及檢舉 | S07 | B14–B15 | 雙方未接受前不能聊天；拒絕、逾時與封鎖均能生效 |
| **B17** | P0 | 文字 Chat 與 15 分鐘共同練習 | S08 | B16 | 兩個已同意帳號能安全聊天並啟動練習；基本頻率限制有效 |
| **B18** | P0 | 雙方完成確認、取消與 No-show 結果紀錄 | S09 | B17 | 只有符合時長與雙方確認的 Session 才算完成 |
| **B19** | P0 | Rust／SQLite 驗證、雲端授權及 Android 端到端測試 | S01–S09 | B01–B18 | v1.7 指定的 13 個 Parameter Integrity 案例，以及安全、撤銷、到期、Chat、E2E 全數通過 |
| **B20** | P1 | 配對 Benchmark：固定表單、純向量與 CAKAMI 的公平比較 | S04–S09 | B13–B18 | 相同候選池下比較每 100 個有效 Intent 的完成數、提問量、違反硬條件次數、延遲及單場成本 |
| **B21** | P0 | Android 測試發布、安全審核、部署與回滾程序 | 發布 | B19；參考 B20 | 有可安裝測試版；記錄 E2B 速度／記憶體／電池；P0 發布門檻全部通過 |

### B03-M1／B21-M1｜Gemma 模型更新生命週期（User 已核准設計；2026-11-19）

**開發狀態：NOT_STARTED；不得把目前 B03 固定版模型首次自動下載當作完成。** 已核准的功能需求對應 [Technical Spec](../design/TECHNICAL_SPEC.md)／[Functional Spec](../design/FUNCTIONAL_SPEC.md)／[GUI UX Spec](../design/GUI_UX_SPEC.md)。

| 子項 | 優先 | 實作項目 | 依賴 | 完成標準 |
|---|---|---|---|---|
| B03-M1 | P0 | APK 與本地模型版本解耦；同 ID、同簽章覆蓋安裝保留模型，不因 UI／Rust／Prompt 更新重下載 | B03 | 同 SHA／模型版本升級 APK 後不觸發下載，離線 AI 仍能使用；誤解除安裝的警告明確 |
| B21-M1 | P0 | 經信任的模型版本清單、可選升級、雙版本暫存／驗證／原子切換、首次新模型推論測試及失敗回退 | B03-M1、B21 | 新模型驗證前保留舊版；斷網／空間不足／壞檔／不相容／推論失敗仍可使用原模型 |
| B21-M2 | P1 | 更新通知頻率、備援 CDN／託管來源策略、磁碟保留與清理期限 | B21-M1 | 沒有多餘下載與 Cloud 推論；低成本、可回復；具體頻率、來源與保留期仍 OPEN |

**發版阻斷（SOURCE）**：正常 APK 更新不應不必要重下 2.59 GB；如發佈新模型則必須先下載＋校驗＋啟用成功才可刪舊模型；任何失敗都保持舊模型可用。不許把尚未實作的 B21-M1/M2 標 PASS。

### Action Intelligence Core 泛用化（User 2026-11-19 需求；PROPOSED，尚未實作）

**不是把 Study 改成 Fashion/Car MVP**。P1 仍只開放 N2 真人學伴；但 B02～B05 的 AI 引擎不得將所有內容硬分類為日語學習，通用 Core 要能保留非 Study 的 PRIVATE／UNKNOWN，且新領域只有正式批准的 Domain Pack 才能開放 Cloud Match。對應 [Universal Intent Engine Design](../design/ACTION_INTELLIGENCE_ARCHITECTURE_v1_DRAFT.md) 與 [AI-INTENT-01～12](../test/ACCEPTANCE_SPEC.md)。

| 子項 | 擬議優先 | 設計交付／實作責任 | 不得假裝已完成的地方 |
|---|---|---|---|
| B03-U1 | P0（P1 正確性） | B03 OCR／Gemma 抽取有來源的 ContentFact、多候選 Domain/Intent/UNKNOWN；與 B04 安全分離 | 現有僅 `contentHint`；不會判斷不同域；僅 OCR 文字輸入 Gemma，無原圖視覺理解 |
| B04-U1 | P0（P1 首次價值） | Rust + SQLite 用已確認私有 Context 排序私人 Goal 候選；本人選擇、編輯、確認才建立 Goal，低證據時只收藏／少問 | 現有空白文字 Goal 輸入，不會使用 B03 結果 |
| B05-U1 | P0（P1 正確性） | 由已確認 Goal 提出合法 Parameter 候選；Registry 未核准概念維持 UNKNOWN；與可執行 Action Capability Registry 分離 | 現有只認 N2，同義別名；無智能 Need/Offer/時間抽取 |
| B13-U1 | P1（後續領域擴張） | Cloud 共用 Goal/Need↔Offer/可行性與 Domain Pack 版本化規則；每個新域先完成安全、Consent、測試 | P1 Cloud 只允許 N2 Study；不把相似文字當互補需求 |
| B18-U1 | P1（成果閉環） | 經 Server 驗證且授權可見的 OutcomeReceipt 回本地 SQLite，改善下次本地選擇，跨人結果利用另需同意 | 目前尚未建 B10～B18；不得由 UI 點擊算完成 |
| B03-V1 | P1／後續跨域視覺 Gate | 評估經批准的 On-device Vision Adapter：真正圖片像素理解 Fashion/Car 且離線、隱私、手機效能可用 | **OCR-only 不能算看懂服裝或車輛**；尚無此模型接口 |

**次序：** 先驗 AI-INTENT-01～11（包含歧義和跨域負例），修 B03→B04→B05 再續 B06；Cloud 循環需 B08/B10～B18 的合法依賴，AI-INTENT-12 需真 Server Outcome Evidence。此表為設計需求提案，未有 User 核准的具體 Domain Pack、Cloud 欄位白名單或資料使用規則不得先行實作。

## 三、建置依賴流程

```text
基礎功能            B01 → B02 → B03 → B04
                              ↓
參數完整性          B05 → B06 → B07 → B08
                        ↓             ↓
動態追問             B09          使用者授權
                         ╲          ╱
帳號與雲端            B10 → B11 → B12
                                     ↓
配對與候選            B13 → B14 → B15
                                     ↓
真人合作              B16 → B17 → B18
                                     ↓
驗證與發布            B19 → B20（比較報告）→ B21
```

> 上圖是高層順序，實際多重依賴依「二、功能式建置待辦」表格為準。B10 可在本地核心完成之前展開設計，但真正對外發布需通過 B06／B08 等檢查。

## US01-E2B-04｜保留 E2B，解除建議輸出封鎖（2026-11-21）

**使用者真機問題：輸入單一主題時，E2B 無法提出可用建議。狀態：程式修正已提交，手機端品質 NOT_RUN。**

- **根因 A（本地 AI 提示詞）**：原本用超長 JSON＋找人合作判定的提示詞，讓小型 E2B 同時執行複雜格式/意圖/權限判斷；已縮短提示詞，允許最多三個不同的**私人假設**。範例 `JLPT` 可以推測了解級別、備考、找人練習；絕不等同本人已確認。
- **根因 B（Rust 本地過濾）**：原本 `explicit_collaboration_intent(original)` 的 `wants && together` 先決條件，把所有未明確要求找人的模型候選清空。已移除這個顯示限制，但保留**候選來源逐字證據、枚舉網域、最多 3 項、不得偽造同意／分享欄位、來源變更失效**等防護。沒有原文字面證據的模型推測仍會被拒絕。
- **根因 C（GUI）**：真正 E2B 分析按鈕及輸出原本藏在「進階」。現在是「處理內容 → AI 理解」的**主要可見按鈕與主要結果區**，先顯示 AI 理解、再顯示 2–3 個可編輯的可能方向，使用者選擇後進 Goal 本人確認；固定知識卡明確標示「不是 AI 推測」，AI 有結果時自動收合以避免重複。
- **保留紅線**：模型依然是本機 `gemma-4-E2B-it.litertlm`，不雲端、不重下載、不自動寫入 confirmed Goal／Consent／公開。只有按本人確認才建私人 Goal，配對功能不因本修正被提前啟動。
- **驗收**：Rust 單元測試以孤立字 `JLPT` 回傳 3 候選，必須能顯示與跨重啟持久化，但 `GoalStore` 仍為空；偽造 evidence／consent 和未經人確認依然必須 FAIL；GUI 主畫面按鈕可見；使用者真機核對 10 個輸入的實用性。此項與 PP-OCRv6 尚未完成的 OCR 更換分開評估。
- [實機驗收清單](../test/US01_E2B_GATE_016_PHONE_TEST.md)。

## US01-OCR-03｜直接替換日文 OCR 引擎（2026-11-21）

- **選型**：已開始以官方 **PaddleOCR PP-OCRv6 Small ONNX** 取代 `ML Kit Japanese`，官方 Android SDK 用 ONNX Runtime／OpenCV，離線執行。與另需更新的 Gemma 4 E2B **不同模型／路徑**，正常同簽章 APK 更新不得重新下載 2.59 GB。
- **不是 Medium**：PP-OCRv6 Medium 的辨識指標較高，但官方 Android 範例直接支援 Small，第一版選 Small 是手機可安裝性與效能折衷，並非宣稱它對所有日文圖片全球第一。
- **實作狀態**：26 個官方 Apache 2.0 Kotlin SDK 來源 pinned `dab3fe35379033fdcb2d0e9572fac0b36c9a9ebf`，完整模型以官方 HuggingFace 在 APK **建置時** bundle；Android 原生 Plugin 已改用 `paddle_ocrv6_small_onnx`；Rust 同時允許舊 ML Kit 的持久化紀錄與新 PaddleOCR 的紀錄。正在跑 Kotlin/native/APK 回歸驗證，**未出現可安裝且簽章驗證過的 0.1.5 APK 前不可宣稱發版完成**。
- **必須通過**：同一批使用者照片（至少 10 張）量測 CER 字元錯誤率；照片與私人 Goal 保留、錯誤可人工修正、飛航模式可辨識、不上傳、官方模型未遺漏。手機尚未實測，不能聲稱準確率已從 40% 提升至多少。
- [v0.1.5 真機驗收](../test/US01_PP_OCRV6_015_PHONE_TEST.md)。

## US-01 即時阻斷缺陷（2026-11-21｜真機回報）

**狀態：用戶真機 FAIL；程式修正提交中，真機重測 NOT_RUN。**
- **US01-JLPT-01 / B03→B04：輸入「JLPT」沒有任何推薦**。驗收：不等 E2B，也不需要輸入「我想找人」，立即有 3 個**可真正操作**的私人選項：看 JLPT N5–N1 級別、建立持久化本地學習卡、把找學伴的提案填進待本人確認的 Goal。預設不猜級別、不建立確認事件、不公開、不上傳；E2B 失敗仍可使用。
- **US01-OCR-02 / B03：日文 OCR 真機使用者估計僅 40% 準確率**。先修已查出的照片 EXIF 旋轉未處理；低信心短圖啟動裝置端對比增強再辨識，只有較可靠時才覆蓋第一次結果，否則保留原文／人工修正路徑。**不能只因 CI PASS 就宣稱 OCR 正確率已改善到某個數字**；需使用者實拍同一批照片進行修改前後核對。
- **版本與資料保留**：更新包原始 App ID `com.nff98.ckm.dev`，版本升到 `0.1.4`，必須使用 CLEAN 0.1.1/0.1.2 同一私鑰憑證簽章；APK 直接覆蓋，E2B 不重下載，私人收藏不遺失。
- **隱私紅線**：不改用雲端 AI／OCR、不傳私人照片；JLPT 行動卡不是已確認私人意圖或配對，不改 B05 正式概念白名單。
- [手機驗收與證據](../test/US01_JLPT_OCR_014_PHONE_TEST.md)；本條優先於目前沒有真機價值的其他 BXX 工作。

## 四、最小驗收測試案例

| 測試情境 | 必須發生的結果 |
|---|---|
| 只 Drop 日文單字「パクる」 | 本地內容可保存；未確認找人目標前，不建立公開配對意圖 |
| Drop 餐廳廣告但沒說要做什麼 | 保存內容，不擅自推論聚餐；折扣到期不代表使用者長期想去餐廳的興趣也被刪除 |
| E2B 回傳 `status=CONFIRMED` | 沒有對應的使用者確認事件，Rust 應拒絕 |
| 已有 8 個有效概念，AI 再提出第 9 個 | 阻擋寫入／發布；需使用者決定，絕不能偷偷替換硬條件 |
| 丟入 100 張單字截圖 | 可以累積內容，但不能無限制創造新的 Matching 概念或資料庫欄位 |
| 候選向量很相似，但時間或權限衝突 | 即使語意高度相似，也不可配對 |
| A 拒絕或封鎖 B | 不得開始聊天，也不能計算為合作完成 |
| 雙方共同練習滿 15 分鐘並各自確認 | 精確記錄一場完成的 Session |
| 裝置離線或撤回分享同意 | 本地資料可管理；撤銷 Intent 立即停止本地匹配，雲端撤銷需排隊重試並顯示尚未同步風險 |

## 五、如何管理 Backlog

- **先選 User Story，再映射 BXX**：現有三個 Sprint 保留，但 [Sprint 1 交付 US-01](sprints/SPRINT-01_LOCAL_AI.md)（跨 B01–B09）、[Sprint 2 交付 US-02＋US-03](sprints/SPRINT-02_MATCHING.md)（跨 B08/B10–B18）、[Sprint 3 做完整回歸與發布](sprints/SPRINT-03_RELEASE.md)。以真使用者流程 PASS 交付，不能用單個 BXX CI PASS 取代。
- 每項工作完成時須附上可重現的驗收證據：測試結果、Commit／PR、使用合成資料的畫面或裝置實測。
- 尚未動工的項目一律標示「未開始」；不得將資料夾或待辦清單存在，當成 App 已實作。
- 使用者已決定採三個 Sprint。這是開發分組，不代表承諾工期或新增 Sprint Activation／治理硬限制。
- 如產品決策變更，需同步指出受影響的畫面 S01–S09、參數規則、資料隱私及測試。
- **所有 GitHub 溝通、文件與後續報告均使用繁體中文**，即使需求是以英文提出。

---
本 Backlog 依最新三份設計規格重新整理；**尚無執行進度、測試結果或部署成果**。
