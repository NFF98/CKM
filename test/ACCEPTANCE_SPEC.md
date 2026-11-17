# CAKAMI｜MVP 驗收測試規格（審核草案）

> **狀態：DRAFT／待 Human Review，未 LOCKED；本文件沒有任何已執行的 PASS。**  
> 日期：2026-11-17；適用：Android-first，18+ JLPT N2 Study Match，S01–S09＋S04a，B01–B21。  
> 唯一 Build Backlog：../build/BACKLOG.md；測試基準：../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx §15.5／§16.5 的 T01–T13、Functional v1.8 的四道 Gate；GUI Draft：../design/GUI_UX_SPEC.md；Technical Draft：../design/TECHNICAL_SPEC.md。  
> **SOURCE** 表示已由設計鎖定的必測行為；其餘 Test Case IDs 與 Test Mechanism 是提案，需批准後方為正式驗收矩陣。**未執行 ≠ 通過**。

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

## 9. 開發前必須解決的 Acceptance Decision

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
