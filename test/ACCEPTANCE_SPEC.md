# CAKAMI｜MVP 驗收測試參考規格（B01–B21）

> **狀態：作為 Sprint AC 與 Coding 測試的參考依據；SOURCE 測試保持原定義，新增提案仍需核對。已有部分 Android 真機測試：B02 保存與 B03 OCR／詞義理解子路徑使用者回報可用，但 **US-01 跨模組 Action Intelligence 真機 NG**；未執行的項目不得宣稱 PASS；Coding 不需要等待整份文件 LOCKED。**  
> 日期：2026-11-17；適用：Android-first，18+ JLPT N2 Study Match，S01–S09＋S04a，B01–B21。  
> 唯一 Build Backlog：../build/BACKLOG.md；測試基準：../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx §15.5／§16.5 的 T01–T13、Functional v1.8 的四道 Gate；GUI Draft：../design/GUI_UX_SPEC.md；Technical Draft：../design/TECHNICAL_SPEC.md。  
> **SOURCE** 表示已由設計鎖定的必測行為；其餘 Test Case IDs 與 Test Mechanism 是提案，需批准後方為正式驗收矩陣。**未執行 ≠ 通過**。

## User Story End-to-End Gate（2026-11-19 User 決策：以使用者旅程交付）

**新開發準則：** 模組 BXX 可以分別 Coding/CI，但只有使用者從起點操作到最後有**真狀態**，才能把 User Story 判為 PASS。既有 `T01–T13`、`E2E-01～05`、`AI-LOCAL` 不改名、不刪除；新 `US-` IDs 是**跨 BXX 整合驗收**，不取代安全單元測試。詳細順序見 [User Story Sprint Plan](../build/SPRINT_PLAN.md)。

### US-01：文字／圖片 → 本地理解 → 可能意圖 → 追問 → 本人確認（**目前 FAIL/NG**）

| Test ID | 使用者真實操作 | PASS 必須包含 |
|---|---|---|
| **US-01-01** | Android App 直接輸入純文字：「我想找人一起練 JLPT N2 口說」 | **不依賴 OCR**，手機 Gemma 實際提出基於原文的私人 Goal 候選；使用者可一鍵採納或修改→本人確認→Rust SQLite 保存有來源、確認事件與版本的私人 Goal；重啟仍在，**不能要求從零自己重寫 Goal** |
| **US-01-02** | 純文字「パクる」或其他欠缺情境的短語 | 誠實顯示不知道真正 Goal（UNKNOWN）、允許只收藏或問真正能區分意圖的問題；**不能強迫歸類成 Study／找學伴、不能自動公開** |
| **US-01-03** | Drop 有日文的 N2 圖片→真正 OCR→Gemma→選/改私人 Goal | 和純文字**同一套 Intent Pipeline**；更正 OCR 後舊候選失效；原圖、OCR 沒有未授權 Cloud 呼叫 |
| **US-01-04** | 輸入 Fashion/Car/混合語境的文字或截圖，手機歷史主要是 Study | 能保留非 Study／UNKNOWN，不強制 N2 目標；現有 OCR-only 不可宣稱理解無文字照片像素；Fashion/Car 不可於 P1 對外配對 |
| **US-01-05** | 使用者確認 Goal 後，AI 提出合法 Need/Offer/期限等**僅已批准 Registry Concepts** 候選 | 本人可核對／編輯後才確認；B05 Rust 強制來源、scope/expiry/版本；未核准 Concepts 維持 UNKNOWN，**不得 AI 自創已合法欄位** |
| **US-01-06** | 追問缺資訊及實測 8 個活躍 Concepts 上限 | B09 每次主動 **0～3 題（含 Goal 確認）**，多次重試不重問；B06 SQLite/Rust 硬限制 **最多 8 種已核准有效 Matching Concepts**；第 9 個必要概念不能悄悄覆蓋既有硬條件（T10–T13） |
| **US-01-07** | 使用者按「只收藏」、取消確認，或離線後修改／撤銷 | 無自動 Goal/Consent/Cloud；已確認版本/事件真實、狀態可回讀；B07/08 只提供合法私人操作／預覽，不假裝 Cloud 已同步 |

**US-01 PASS 條件：** US-01-01～07 的真 Android User Journey（含整合的 Rust/SQLite／錯誤測試）均有 Evidence，相關 BXX P0 Gate 也 PASS。**現在整條 US-01 = FAIL**，因 Gemma 真機只解釋文字、沒有可用意圖候選→確認的連接。既有 B03 OCR/詞義及 B05 Registry CI PASS 不能抵銷這個 NG。

### US-02：本人授權 → Cloud 真正找到相容真人（**NOT_RUN**）

| Test ID | 使用者真實操作 | PASS 必須包含 |
|---|---|---|
| **US-02-01** | 真 A/B 帳號有各自已確認的私人 N2 Intent → 在手機逐項預覽與同意分享 | 同意前 Cloud 不存在公開 Intent；同意後只送核准最少欄位，Server 驗證 ownership/8-slot/expiry/version/Consent；Cloud 無模型推論 |
| **US-02-02** | A 找相容的 B，以及沒有符合者情境 | 相同 Goal／互補 Need↔Offer 和時間／硬條件都正確；真的找到合法候選或真 0 人；不能虛構真人／相似度假成功 |
| **US-02-03** | B 封鎖、撤銷、過期、條件不合、跨帳號直打 API | RLS／Server 拒絕；離線撤銷本機排除、重連後 Server ACK、其他人不可再搜尋 |
| **US-02-04** | Pair-only 必要追問 | 最多 2 題，答案僅 Pair scope、限時有效；不得改永久 Profile 或繞過硬限制 |

**US-02 PASS 條件：** US-02-01～04 有雙帳號＋真 Supabase E2E/攻擊負例、Server 實際紀錄、合法輸入輸出 Evidence；不是 UI 的候選 mock。

### US-03：真合作 → Server 可信完成 → 自己手機留下結果（**NOT_RUN**）

| Test ID | 使用者真實操作 | PASS 必須包含 |
|---|---|---|
| **US-03-01** | A 邀 B，B 接受／拒絕／封鎖不同情境 | 只有雙方合法接受才啟 Chat；第三人和不合法狀態呼叫 API 被拒，重放/並發不能重複 Session |
| **US-03-02** | A/B 真文字 Chat 並完成 15 分鐘 Session → 各自確認 | Server 以已核准計時規則及兩份不同使用者確認恰建立 **1** 筆可信 COMPLETED |
| **US-03-03** | 少於 15 分鐘／單方確認／No-show／取消／越權/重試 | 不計 COMPLETED、不重複計算；Chat 訊息數／點擊不等於完成 |
| **US-03-04** | 本人登入手機同步/重試已完成 Outcome，重新開啟 App | **僅本人合法可讀且事前核准的** OutcomeReceipt 入本地 SQLite，供下一次已確認上下文使用；另一人的私人資料/跨人聚合不得偷傳。具體分享欄位未核准則 BLOCKED 而非偽裝 PASS |

**US-03 PASS 條件：** US-03-01～04 + B16～18 Server/RLS/SQL/Android 真 E2E。未核准的 OutcomeReceipt 權限與保留期須正式決策；不得以合成手機結果代替可信 Server 完成。

### 數據準確率與模型比較（設計假說，不是當前實際 PASS）

- **不能**以 E2B 自報「60–90% confidence」當作意圖準確率，也不能因用戶按確認就宣稱 AI 預測 90%。應比較「AI 初次意圖 Top-1 與獨立真人 Ground Truth 一致率」「應為 UNKNOWN 時的正確拒判率」「經少量追問後與真人意圖一致率」，另報不合適配對率／問題數／完成合作比率。
- 使用者提出的 **60～90% 是欲驗證的目標區間**，尚非已完成數據；目前 [AI-TEXT-8](#ai-text-8純文字--意圖候選--b04b05-確認--b06-最多-8-slots設計草案not_run) 提的開發 ≥60%／確認後 ≥90% 只屬建議門檻，待真人標註集及 User 最終決策。分純文字、OCR、Study、Fashion、Car、UNKNOWN 報告；低證據不亂猜也算成功的安全性能力。
- **B20 仍 P1**，不因對照研究變成 B21 必過的硬性 Gate；但所有已聲稱的準確率必須有測試 Evidence。

## 已核准驗收紅線：Phase 1 不使用雲端 AI（2026-11-17）

- 所有 OCR、E2B、AI 提議、Embedding 生成（若有）必須在 Android 裝置本地完成；禁止遠端推理與失敗時 Cloud AI fallback。
- **追加測試 AI-LOCAL-01（B03／B13／B19）**：Android 斷網後可對已安裝且已準備好的本地模型完成 OCR／E2B 推論；必須記錄實際使用的 runtime／模型，不能拿靜態假結果代替。
- **追加測試 AI-LOCAL-02（B03／B12／B19）**：模型故障／超時／無法載入時，監控網路與伺服器端日誌，確認沒有轉呼叫任何外部 OCR／LLM／VLM／Embedding 服務；UI 明示失敗並允許人工修正。
- **追加測試 AI-LOCAL-03（B11–B13／B19）**：Cloud API／Supabase 僅處理已授權 Payload、資料庫查詢／向量距離／邀請與聊天，不在 Server 或 Edge Function 呼叫模型推論；本地產生的 Embedding 若要上傳，必須有明確 Consent。
- 以上測試目前均為 **NOT_RUN**，不宣稱模型已在 Android 驗證。

## B03／B21 模型更新生命週期驗收（User 已核准產品規則，2026-11-19）

**以下新測試均為 NOT_RUN；新增設計不等於完成新版更新器。** 以 Android 真機 + App-private 模型檔 + 網路／儲存故障注入為準。

| ID | 操作／故障注入 | 必須驗收 |
|---|---|---|
| AI-MODEL-01 | Gemma 模型已完成 2.59 GB 下載，安裝同簽章／同 applicationId 的新 APK（只改 UI／Rust／Prompt） | 模型版本與 SHA 不變、不觸發 2.59 GB 重下載、私人 SQLite 資料保留；飛航模式仍可推論 |
| AI-MODEL-02 | 發佈新模型版本，有 Wi-Fi／無 Wi-Fi 各測一次 | 有版本說明／大小和可稍後更新的選項；無 Wi-Fi 或暫時拒絕時舊版正常離線運作，不強迫重下載 |
| AI-MODEL-03 | 新版下載中斷、App 中止、重啟或 CDN 不可用 | 舊版不被刪除；可從有效部分續傳或安全重試，不能把殘檔當已安裝 |
| AI-MODEL-04 | 新模型檔 SHA 錯誤、來源異常、Manifest 不受信任／不相容 | 新模型拒絕啟用；舊模型與資料不變；錯誤可辨識，沒有遠端 AI fallback |
| AI-MODEL-05 | 新版本完整下載且校驗通過，但 LiteRT-LM 載入失敗或最小推論失敗 | 回退舊版仍可使用；不能把下載完成或 SHA 驗證當成推論 PASS |
| AI-MODEL-06 | 模擬新舊兩模型同時保留時手機剩餘空間不足；切換中途 crash | 不刪舊版以騰空間；重啟後仍能辨識並使用最後成功的版本，無破損啟用指標 |
| AI-MODEL-07 | 新模型啟用及本地 Smoke Test 成功 | 才可標示已升級、按政策安全回收舊版；記錄 modelId、modelVersion、hash、runtime 與切換／回退證據 |
| AI-MODEL-08 | 解除安裝 App 後重新安裝（負例） | UI／文件必須警示 App-private 模型通常遺失；不可把此情況算成「APK 更新造成重下載」Bug |
| AI-MODEL-09 | Rust Bridge、模型已下載、模型推論三狀態交叉驗證 | 狀態互不假冒：模型校驗通過不等於推論已通過；更新失敗不顯示為 Rust 連線失敗 |

**Gate：** B03-M1 與 B21-M1 的 P0 驗收需要上列必測情境的可重現真機 Evidence；尚未實作／尚未測到時一律 NOT_RUN。規格依 [Technical](../design/TECHNICAL_SPEC.md)／[Backlog](../build/BACKLOG.md)。

## B03→B04 Action Intelligence 真機產品目標 NG（2026-11-19）

- **使用者提供真機結果：NG／FAIL**。輸入日文詞彙 `パクる`，目前 Android CAKAMI 已能 OCR 與 E2B 解釋詞義，但**沒有提供真正可執行的下一步，使用者仍不知道要做什麼**。這個新端到端產品價值驗收不得標示 PASS。原 S1-AC03 的「詞義可理解／不能把內容強行變成 Goal」與此新要求不同；前者可單獨 PASS，不等於新要求已完成。
- **ACT-01（目前 FAIL；下次必須復測）**：Drop `パクる` → 顯示有根據的內容解釋 + 少量相關行動候選（例如複習、練用法、本人確認找學伴或只收藏）→ 使用者**不必重新打字**，選擇一個動作 → App 真正執行並保存可驗證的結果。**只顯示選項但按下無效果也算 FAIL**。
- **ACT-02（NOT_RUN）**：無上下文的字詞不能猜出唯一 Goal、不能自動確認學伴／分享／公開；使用者只收藏或取消時，不建立意圖。
- **ACT-03（NOT_RUN）**：完整句子、資訊複雜截圖、OCR 更正後仍能提出適合的有限選項；不確定時誠實顯示；飛航模式本地執行，且重開 App 保留已選的本地結果。
- **狀態語義**：B03／B04 既有 Coding 仍在，但 Action Intelligence E2E **FAIL，不得稱為已完成產品目標**；修復方案尚未 Coding／APK 驗收。B05 Registry CI PASS 不等於本項通過。

## AI-INTENT：跨領域本地意圖引擎驗收（DESIGN PROPOSAL；2026-11-19）

**狀態：NOT_RUN。** 產品核心須有通用領域識別、Intent/Content 分離、可拒絕強制分類、經本人確認的合作需求及安全 Cloud 投影。P1 只上線 Study N2，Fashion/Car 是**離線跨領域泛化／負例測試**，不准因此暗中啟用 P2+ 雲端配對。

| Case | 輸入／條件 | 必須發生的結果 |
|---|---|---|
| AI-INTENT-01 | 只有 `パクる` | 可解釋詞義；`domain=unknown`／`goal=unknown`；不得擅自建立 Study Goal 或傳 Cloud |
| AI-INTENT-02 | JLPT N2 教材中的 `パクる`，有真實教材來源證據 | 可建議 Study 相關私人合作意圖；需要本人挑選／修改確認才建立 Goal |
| AI-INTENT-03 | 時裝設計討論 `パクる`，與日文學習無關 | 不得強制歸 Study；可辨識 Fashion 語境／保留 UNKNOWN；不得自動假設要購買、模仿或找合作人 |
| AI-INTENT-04 | 汽車改裝文章 `パクる`，與日文學習無關 | 不得強制歸 Study；可辨識 Car 語境／保留 UNKNOWN；不得自動推定求助或要買零件 |
| AI-INTENT-05 | 同時含 Study 與 Fashion/Car 線索，或證據互相衝突 | 保留多個假設、尋找最小關鍵區別問題／允許只收藏；模型不能偽造已確認 Goal |
| AI-INTENT-06 | 手機過往 20 筆 Study，但本次是明確 Fashion 截圖 | 不能用歷史覆蓋本次證據；不自動把 Fashion 映射為 N2 |
| AI-INTENT-07 | 沒有文字的服飾／汽車照片，當前只裝文字 OCR 模型 | 誠實顯示 `vision_unknown`，不能宣稱看懂剪裁、車型或設計；未有真實 VLM Adapter 前不得算跨領域圖片理解 PASS |
| AI-INTENT-08 | B03 候選缺 source、混入 confirmed/consent/visibility，或更正 OCR 後重讀舊候選 | Rust 擋下偽造狀態／缺來源／過期候選；原文已改則舊建議失效 |
| AI-INTENT-09 | 合法候選但用戶按「只收藏／稍後」 | 無 Goal、無配對、無雲端同步；私人資料仍存在手機 |
| AI-INTENT-10 | 用戶確認 Study Goal，B05 尚無正式 Need/Offer Concept | 保留 UNKNOWN，不用模型自創 Registry／Share 白名單；僅合法 B06/08/12 契約可同步 |
| AI-INTENT-11 | 未上線 Fashion/Car Domain Pack 或未開放 Match Capability | 雲端拒絕發布/搜尋該領域；已核准 P1 Study 路徑仍正常 |
| AI-INTENT-12 | B18 兩方有效完成，回傳本人可見 OutcomeReceipt；或只單方確認／低於 15 分鐘 | 前者同步回本地並可作下次可解釋的回饋；後者絕不當 COMPLETED；不得洩露另一方私人內容 |

**建議評估方法**（非已達成結果）：用至少 100 筆 Study、100 筆 Fashion、100 筆 Car/Other／跨域、歧義與新類別保留測試集；區分「有充分證據的 domain 命中率」和「證據不足時有沒有選 UNKNOWN」，不可只靠總 accuracy；逐筆標記 AI 猜的 Goal 是否被本人更改，跨域錯誤率、誤發布率、第一次有效 Intent 所需操作、Android 推論耗時與離線可用。與「Study 寫死規則」「固定幾個動作按鈕」做公平對照，未通過之前不得宣稱跨域 MOAT。

此節對應 [Universal Intent Engine 設計草案](../design/ACTION_INTELLIGENCE_ARCHITECTURE_v1_DRAFT.md#1a-跨領域共用的真正大腦universal-intent-engine2026-11-19-user-方向設計草案)，並需在正式 domain 開放前逐領域增加真實視覺與多人端到端驗收。

## AI-TEXT-8｜純文字 → 意圖候選 → B04/B05 確認 → B06 最多 8 slots（設計草案，NOT_RUN）

| Case | 操作 | 必須驗證 |
|---|---|---|
| AI-TEXT-01 | **純文字**「想找人一起練 JLPT N2 口語」 | 不經 OCR；B03 產生有證據的私人 Study/Goal 候選；B04 可直接選擇/編輯並本人確認；B05 僅核准 Concept 能確認；不能自動 Consent |
| AI-TEXT-02 | **純文字**「パクる」 | 詞義可解釋；意圖/領域保持 UNKNOWN；不能依 P1 市場或過往偏好強制認定找 N2 學伴；不填滿 8 slots |
| AI-TEXT-03 | **純文字**「想找人一起研究 BMW E46 改裝，週末有空」 | Domain 可為 Car；Goal/Need/主題/時間作獨立候選，Offer/精確時間/地點/技能 UNKNOWN；不得把 8 個展示面向直接算作 8 個合法 Matching slots；P1 不對外配對 Car |
| AI-TEXT-04 | B09 初次 Goal 確認及追問 | 最多 **3 題** AI 主動詢問（含 Goal 確認），一個答案可以確認多個合法概念；已足夠可 0～1 題，不得要求逐一回答 8 題 |
| AI-TEXT-05 | B06 已有 8 種有效 Matching Concepts，嘗試新增第 9 種 | SQL/Rust 拒絕或請本人明確決定，絕不能自動刪掉硬條件；UNKNOWN/私人 Content 不占活躍 slot |
| AI-TEXT-06 | 將 Gemma 報告的「85% confidence」當作可發布意圖 | Rust 必須拒絕用模型自報信心替代 B04/B05 本人確認與 B08 Consent；未有經驗校準不得把 85% 稱為準確率 |
| AI-TEXT-07 | 用獨立標註純文字、OCR、跨領域語境與 UNKNOWN 測試集衡量意圖 | 分別報告**確認前 top-1 正確率、UNKNOWN 正確拒判率、確認後意圖一致率、誤確認率、問題數**；開發期 60%／確認後 90% 僅為候選目標、**尚未測得／待核定**，不得借用單一 N2 測試推論跨域成功 |

所有 Cases **NOT_RUN**。這裡的**8 slots**依既有 Business v1.2 / System v1.7 是每個 Intent 最多 8 種**已核准、有效、可參與匹配的 Parameter Concepts**；8 個展示面向**不等於**固定 8 個必填欄位、8 題 AI 問題或 8 種一律要公開的屬性。見 [Action Intelligence Architecture v1 Draft](../design/ACTION_INTELLIGENCE_ARCHITECTURE_v1_DRAFT.md)。

## 1. Done 的唯一判準

每個 B-ID 須同時具備：
1. 對應已核准的 Functional／GUI／Technical Contract、明確必要依賴、可追溯 Commit／PR。
2. 自動化測試結果與真機／跨端測試（如適用）；包含正常、異常、越權、邊界、離線及重試。
3. Evidence：測試命令、環境／模型／版本、測試資料、實際結果、失敗紀錄及修正 commit；人工測試附可重現步驟或不含私密資料的畫面。
4. 實際結果與預期相同才 PASS。若因環境無法跑，標示 BLOCKED；測試未寫／未跑則 NOT_RUN；不能靠 mock 替代最終 Android／Supabase 驗收。
5. 只有所有 P0 Gate PASS，才能 B21 Android Pilot Release。B20 屬 P1、依 Bible 為 B21 參考，不私自調整為硬性依賴。

### 狀態語彙（PROPOSED）

- NOT_RUN：尚未執行；FAIL：實際結果不符合；BLOCKED：因環境／規格阻擋；PASS：真實執行並驗證；RETEST_REQUIRED：有改動需重跑。
- 這些 Test Case 狀態不能與 Build Bible 的「未開始／進行中／完成」混用；只有完成 Evidence Review 才變更 B-ID 的正式進度。

## 2. v1.7 P0 Parameter Integrity：T01–T13 原則對照（SOURCE）

| Case | 測試方法／輸入 | 預期結果 | 關聯 |
|---|---|---|---|
| **T01** | E2B 提議含 CONFIRMED／假的本人確認事件 | Rust 拒絕；只有合法 UI 操作才產生可確認的事件 | B03–B06 |
| **T02** | 缺 source／scope／expiry_policy，分別呼叫 Rust 與直接違規 DB INSERT | Rust 拒絕，SQLite NOT NULL／CHECK 也阻擋 | B05–B06 |
| **T03** | 一次性「週六有空」，無明確日期／有明確日期各一筆 | 必須是固定日期的 fixed；不能變成每週六；日期未知不得捏造 | B05–B07 |
| **T04** | N2／JLPT N2／日語二級，同範圍重複輸入與多值條件 | 相同 concept 歸一、單值不重複；合法多值保留不同 value | B05–B06 |
| **T05** | Drop 餐廳優惠圖但 Goal UNKNOWN | 保存本地內容，不生成／發布假配對需求 | B02–B04 |
| **T06** | expires_at 已過、Intent 已 CANCELLED | Matching 搜尋／排序不可把它視為有效條件／候選 | B07／B11–B14 |
| **T07** | 撤銷後斷網、同步失敗、恢復連線 | 本地立即排除；Outbox 重試直至 Cloud ACK；UI 能看到尚未同步風險 | B07–B08／B12 |
| **T08** | 用他人帳號及舊版本直接呼叫 Cloud API | Server 拒絕越權、失效／撤銷、版本不一致的公開資料 | B10–B12 |
| **T09** | 使用者連續 Drop 100 張日文單字截圖 | 內容保留；不建立 100 種相同或無意義 Matching Concept；不變更 Schema | B02–B06 |
| **T10** | 已佔 slot 0–7 再加入第 9 種有效概念，含並發寫入／直接 DB 操作 | Rust 與 SQLite 同時限制；Cloud 也拒絕 >8 的不合法 Payload | B05–B06／B11 |
| **T11** | 已有 study.jlpt_level，再 Drop N2 別名與 100 張「パクる」 | 合法 concept 重用 slot；私人單字內容不佔 Matching slots | B03–B06 |
| **T12** | 8 slots 已滿，第 9 個為必要安全硬條件 | 不得無聲刪除；BLOCKED_BY_BUDGET，禁止 publish／propose，顯示需本人決定 | B05–B08／B15 |
| **T13** | 各種重複提問、不同 Pair、隔日與過期答案 | 單次 AI 主動 ≤3 題（含 Goal 確認）、Pair-only ≤2、每日未請求 ≤3；Pair-only TTL 清理且不升 Profile | B09／B15 |

**關鍵**：T07／T08／T10 需要真 Cloud 測試，T03／T04／T10／T11／T12 要直接執行 DB Constraint／Transaction 的失敗案例；只有畫面測試不夠。上述測試保留 v1.7 原始 ID 與語意，不重新編號。

## 3. B01–B21 功能驗收矩陣

| B-ID | 可觀察的成功條件 | 最少失敗／安全案例 | Evidence 類型 |
|---|---|---|---|
| **B01** | Android 真機啟動；Tauri UI → Rust 正確呼叫；離線可載入 | 中斷 Bridge／權限錯誤不 crash | APK／裝置影片／日誌／版本 |
| **B02** | 圖片、拍照與文字可 Drop，本地重啟後仍可讀 | 儲存失敗／離線／沒有權限／不發原圖 | Asset Test／網路觀察 |
| **B03** | 真機本地 OCR＋E2B；「パクる」只被當內容；未知不捏造 | 模型未載入、失敗 JSON、Timeout、人工修正 | 100 張 N2 測試集、裝置效能 |
| **B04** | Content 與 Goal 分離，只有本人確認後可建 Intent Draft | 沒有 Goal 不可 publish | T05／Rust/UI Test |
| **B05** | Registry normalize；來源、scope、type、expiry、events 正確 | AI 偽造 confirmed、未知概念不能公開 | T01–T04 |
| **B06** | SQLite 8-slot Atomic Write + provenance | 第九種／並發／直接 SQL／硬條件誤刪被拒 | T02／T10–T12、故障注入 |
| **B07** | Intent 編修、撤銷、過期與離線可運作 | 一次性時間不變永久；已失效不匹配 | T03／T06／T07 |
| **B08** | 逐項預覽分享、可追溯 Consent、預設私有 | 沒有同意不可發布；撤回同步狀態真實 | Local＋Cloud Payload Evidence |
| **B09** | UNKNOWN 只問重要缺口、不強迫湊滿 3 題 | 3 題超限、重試重問、Pair scope 汙染 | T13 |
| **B10** | 成年條件與帳號登入、封鎖／檢舉入口 | 未驗證／未授權帳號不跨帳號讀寫 | Auth／Security Test |
| **B11** | Cloud Schema + RLS + Server Validation | A 讀 B 私密資料；非法版本／8-slot 被拒 | RLS／API Test、T08/T10 |
| **B12** | 明確授權後最少同步、撤銷及到期下架 | 同意前不上傳，撤銷離線重試並示警 | T07／Network Trace |
| **B13** | Same Goal／Need↔Offer 召回；硬條件先決 | 時間／封鎖／期限衝突，即使高向量仍排除 | 合成候選池 Ground Truth |
| **B14** | S05 真候選與可解釋理由；S04a 無候選 | 0 候選不虛構；相似度不當成功百分比 | UI／API Fixture／E2E |
| **B15** | 需要才問 Pair-only，回答後重新驗證排序 | Pair-only 超 2、過期、跨 Pair 用答案 | T13＋Pair E2E |
| **B16** | 邀請／接受／拒絕／逾時／封鎖合法轉移 | 單方接受或拒絕後不可讀／開 Chat | 多帳號 E2E／狀態競爭 |
| **B17** | Mutual Accepted 的 A、B 進 Chat + 15 分鐘 Session | C 讀聊天、限流、斷線、封鎖 | Chat Server E2E／計時 Evidence |
| **B18** | 有效時長 + 雙方確認才 COMPLETED | 單方確認／時間不足／No-show／重複提交 | Outcomes DB／雙帳號 E2E |
| **B19** | T01–T13、Android S01–S09＋S04a、安全／撤銷／期限回歸全部通過 | 任何 P0 失敗不得 release | 全套 Test Report |
| **B20** | 同候選池公平比較固定表單、單向量、多向量 Constraints、CAKAMI | 分母／曝光不一致、真 Outcome 尚無不能造數 | Benchmark 報告（P1） |
| **B21** | Android Pilot 可安裝、效能／記憶體／電池已記錄、可回滾 | 安全 Gate 未過／APK 無法安裝不得發布 | Release Checklist、Rollback Drill |

> B19 不應等所有功能寫完才第一次建立：每個 B-ID 開發時即新增對應測試，B19 只完成全串接 Regression 與 Release Gate。

## 4. GUI 驗收（Code-first：由已核准操作規則 + Android 實作證據鎖定；不需 High-fi）

| GUI-ID | 畫面／範圍 | 預期結果 |
|---|---|---|
| GUI-01 | 全畫面 Android 窄寬度與鍵盤 | 主要 CTA 不被鍵盤或系統導航遮擋；≥44px touch target |
| GUI-02 | 全畫面焦點／無障礙 | 重要操作有文字／accessible name，Focus 可見；色彩非唯一提示 |
| GUI-03 | S01–S02 | 來源、人工修正、UNKNOWN、辨識失敗與「只保存」清楚；不假確認 |
| GUI-04 | S03 | 私有與公開摘要明顯不同；8-slot 滿額、Consent 版本、撤銷未同步可見 |
| GUI-05 | S04／S04a | 0 候選、離線、伺服器錯誤明確區分；不假裝找到真人 |
| GUI-06 | S05／S06 | 可解釋理由、缺失條件、Pair 問題數與候選失效狀態清楚 |
| GUI-07 | S07 | 邀請單方／雙方接受／拒絕、過期、封鎖資訊明確；Chat CTA 按合法狀態控制 |
| GUI-08 | S08–S09 | Server 真實練習時長／雙方確認進度可見；未達門檻不展示「已完成」 |
| GUI-09 | Async loading／retry | 真 checkpoint 才有 %；否則只有階段文字＋活動指示，不 fake progress |
| GUI-10 | Cross-screen | 返回／取消／Dialog focus／overlay、同一元件狀態、隱私揭露與正式功能一致 |

目前 GUI-01–GUI-10 全部 **NOT_RUN**；以真正 Android App 的逐頁功能／操作／狀態驗收，不要求 High-fi PNG／Mockup。未有真機 Evidence 不得標為 PASS。

## 5. Android 端到端正式腳本（PROPOSED）

### E2E-01｜正常雙人閉環

**前置**：兩台 Android 真機或已核准的雙裝置測試配置、測試帳號 A／B（成年測試條件）、相容的 N2 學習需求、測試用假資料、Cloud Test Environment。

1. A Drop N2 日文圖片，OCR／E2B 產生候選；A 親自確認想找學伴 Goal。
2. A 在 S03 看到最少必要分享欄位、確認 Consent，Server 接受合法公開 Intent。
3. B 同樣確認可匹配的有效 Intent。
4. S04 找到真人 B；S05 顯示可解釋匹配理由；必要時 S06 追問並記錄 Pair-only 確認。
5. A 發邀請，B 接受；只有在雙方皆同意且未封鎖後，Chat 才可使用。
6. 雙方啟動符合已核准計時規則的 15 分鐘文字練習；Server 具可重現時長 Evidence。
7. A 與 B 各自確認完成，S09 顯示完成；雲端 Outcome 恰有一個有效 COMPLETED Session。

**PASS**：上述每一步有真實伺服器／本地狀態與裝置畫面佐證；私人原圖未出現在 Cloud。

### E2E-02｜只存「パクる」，不找學伴

Drop 單字 → 可修正、保存本地 → 選只收藏。**PASS**：無公開 Intent／無發佈請求／Slot 不因單字內容暴增。

### E2E-03｜沒有真人候選

使用有效、合法但沒有相容候選的 Intent 搜尋。**PASS**：S04a 真實空狀態；不出現虛構真人或成功機率；可保留本地學習路徑。

### E2E-04｜拒絕／封鎖／撤銷

A 邀請 B；B 拒絕／封鎖；A 嘗試直接呼叫 Chat API → **必須拒絕**。A 離線撤銷 → 本地立即排除、UI 顯示雲端尚未 ACK，重連後 Cloud 下架且候選不可再召回。

### E2E-05｜Completion 負向案例

一方確認、未達 15 分鐘、重複提交、取消、No-show、多帳號越權提交，均不應產生合法 COMPLETED。15 分鐘計時政策須先通過 Functional F-09 決策。

## 6. 測試資料與方法

- **資料集 D1**：不少於 100 份合法取得的 N2 圖片樣本（句子／單字／文法、OCR 失敗、模糊圖、上下文不明）；對每張保留本地人工標註 Ground Truth。若樣本不足，標記 BLOCKED，不能將 100 張同一圖片視為 100 種辨識驗證。
- **資料集 D2**：合成合法／非法 Intent：Goal UNKNOWN、同義詞、scope、fixed／persistent、各類 expiry、0／8／9 slots、版本衝突；沒有真人敏感資料。
- **資料集 D3**：至少三個隔離測試帳號 A、B、C，加上有權限／無權限／撤銷／封鎖／到期候選；用於 RLS 和 State Machine。
- **資料集 D4**：固定且可重現的 Matching 候選池與 Ground Truth；每種 Baseline 使用相同輸入、候選曝光與安全條件。
- **B20 四種方法**：固定表單、單 Goal Vector、多向量＋Constraints、CAKAMI 8-slot＋Dynamic Questions。真實完成場次必須來自合法、經雙方確認的 Sessions；若尚無真實 Outcome，**只能發表離線代理指標與「尚未驗證真實完成率」**。
- **安全資料**：Repo 為 Public，絕不提交真實私人圖片、訊息、PII、token、API secret、未授權真實用戶記錄。

## 7. 測試執行與 Evidence 模板（PROPOSED）

每個執行紀錄至少含：

~~~yaml
case_id: T10
b_id: [B06, B11]
result: NOT_RUN
test_type: rust_sqlite_cloud
environment:
  git_commit: PENDING
  android_device: PENDING
  android_version: PENDING
  model_id: PENDING
  db_migration_version: PENDING
dataset: synthetic-8-slot-overflow
command_or_steps: PENDING
expected: "第九個有效 concept 於 Rust、SQLite、Cloud 皆被拒絕"
actual: NOT_RUN
evidence_paths: []
defect_issue: null
reviewer: PENDING
reviewed_at_utc: null
~~~

- 每個 PASS 必須附真實可重現紀錄；不要用預先填好的 PASS JSON 製造假綠燈。
- 「效能可接受」需在 B03 技術 Spike 結束後，核准裝置範圍與門檻；測到的數字是 Evidence，不把目標當作結果。
- B21 只針對 Android Pilot 發布，**100 位成年測試者／20 場完成／D7 留存**是 Business Proposal 的 Pilot 成效目標，不因 APK 可安裝就宣稱達成。

## 8. B19／B21 Release Gate

| Gate | 必須滿足 |
|---|---|
| G-R1 本地安全 | T01–T13 對應的 Rust／SQLite 測試完整 PASS，包含故障注入與實體約束 |
| G-R2 Cloud 安全 | A/B/C 越權、Consent、8-slot、失效版本、撤銷／過期、Chat 封鎖測試 PASS |
| G-R3 Android | 真機 S01–S09／S04a 主流程及關鍵異常 E2E PASS，記錄 OCR／E2B 真機效能 |
| G-R4 隱私 | 測試監測確認原圖／私人 OCR 不被未授權上傳；撤銷待同步的風險如實呈現 |
| G-R5 發布 | 可安裝測試版、環境設定、發布與回滾演練 Evidence 齊備 |
| G-R6 B20（P1） | Benchmark 能重現；真 Outcome 尚缺時如實標未驗證；非 Bible B21 硬前置 |

任一 P0 Gate FAIL／BLOCKED → **NO-GO**，不能以「AI 已寫完」當作通過。

## 9. 相應驗收前必須解決的 Acceptance Decision

| Decision | 需要鎖定 |
|---|---|
| A-01 | B03 的 100 張測試集來源、標註規則、裝置及準確率／效能門檻 |
| A-02 | Android 可操作 GUI 的驗收方式、目標畫面尺寸與測試設備（**不需高保真圖**） |
| A-03 | 15 分鐘從哪個 Event 起算，背景／離線／No-show 的測試預期 |
| A-04 | Auth／成年驗證與跨帳號存取拒絕的精確政策 |
| A-05 | Chat 訊息限流、封鎖後歷史可見性、資料保存 |
| A-06 | Matching Baseline 固定候選池／統計口徑／真正的 Outcome 最小觀察量 |
| A-07 | 發布目標裝置與 APK／AAB、安裝／回滾流程及 P0 裝置門檻 |

**鎖定條件**：以上未定事項不得默默設計新功能；決策記於對應 Functional／Technical／GUI Source，User 確認，再同步 Case、Issue 與 GitHub Bible 的工作狀態。
