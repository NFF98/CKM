# Sprint 1｜US-01：從任何輸入到本人確認有效 Intent（使用 B01–B09）

**目的**：使用者可在真正的 Android App 放入 JLPT N2 圖片／文字，離線由本地 AI 協助理解，自己確認找學伴目標，儲存並預覽分享條件。**iOS 從第一天保留平台 Adapter，並非聲稱 iOS 已通過測試。**

- **範圍**：B01、B02、B03、B04、B05、B06、B07、B08、B09（全部 P0）。
- **目前 User Story 狀態：FAIL（2026-11-19 真機）**；B03 Gemma 只能解釋詞義，B03→B04 沒有完成私人意圖候選／追問／確認整條流程。模組既有 Coding/CI 成果保留；詳細看 [Sprint 1 Progress](SPRINT-01_PROGRESS.md)。
- **依賴**：以 [Bible B01–B09](../BACKLOG.md) 的逐項依賴為準；B09 需 B05–B07，B08 需 B07。
- **可見 Demo（完成條件）**：Android 輸入純文字或選入 N2 圖片 → Gemma 提供有證據的內容／可能 Goal（可 UNKNOWN）→ 最多 3 題必要確認（含 Goal）→ 本人選擇／修改／確認 → Rust + SQLite 保存合法私有 Goal 和 Parameter Concepts（最多 8 種活躍）→ 只有本人額外同意才可進一步雲端發布；本 Sprint 不假裝雲端已有真人。


## 0. 先交付 User Story，不再逐個 B-ID 宣稱產品 Done

**US-01：『我輸入一段文字或丟入圖片，CAKAMI 不只解釋內容，而是幫我確認可以找誰做什麼。』**

| Slice／優先序 | 從使用者角度走完整一段 | 需要一起接通的 BXX | PASS 條件 |
|---|---|---|---|
| **US-01A / NEXT** | **純文字**「想找人一起練 JLPT N2 口說」→ 有證據的私人 Goal 候選 → 點選／修改 → 本人確認 → 重新打開仍在 | B02→B03→B04，B05 基礎契約 | Android 真機 + Rust／SQLite 跨模組測試；不能只給詞義、空白 Goal 表單或假建議 |
| **US-01B** | 圖片／OCR／修正 → 使用同一意圖引擎；「パクる」無上下文保持 UNKNOWN；Fashion／Car 文字負例不被強制變 N2 Goal | B02→B03→B04 | 真機 OCR + 確認 + 修改來源使舊候選失效；沒有像素視覺能力不能聲稱看懂衣服／汽車 |
| **US-01C** | AI 從已確認 Goal 提配對參數候選；本人確認重要條件，不足時問**真正有用的問題** | B04→B05→B06→B09 | T01–T04／T09–T13，B06 Rust+SQLite 同時限 8 個合法活躍 Concept；問 0～3 題含 Goal 確認；不得自填 UNKNOWN |
| **US-01D** | 保存／只收藏／修改／撤銷／過期／重啟；預覽哪些內容將來可能分享，但未確認絕不上傳 | B02→B04→B07→B08 | 本地持久性、確認事件、期限／撤銷／離線安全；無 Consent 雲端沒有上傳；正常與負例都過 |
 
**工作方式：** 每一個 Slice 同時修改所需 BXX，交付可走到終點的真實狀態；BXX 的 Coding／單元測試 PASS 只當工程證據。**Sprint 1 / US-01 只有四個 Slice 與相關安全案例全部真機 PASS 才能標 PASS。** 不能把「Gemma 認得字」或「B05 Registry CI PASS」當成 US-01 完成。

**先測再打 APK：** 用合成純文字 `我想找人一起練 JLPT N2 口說`、單字 `パクる`、Fashion／Car 非學習文字、N2 圖片／人工更正建 Rust+TS Contract/SQLite 測試；按照已核准的批次 APK 策略合併真機測試。**B09 不等到 B06 完成才開始思考：US-01A 先把 Goal 確認的問題和問答資料流設好，完整提問預算／8-slot 硬限制在 US-01C 接通。**

## Coding 前應讀的來源規格

這份 Sprint 只分配 B01–B09 的工作和可驗收輸出。每一項以 [Bible](../BACKLOG.md) 為主，依 [B01–B09 逐項追溯表](../TRACEABILITY.md) 查看：
[Functional Spec](../../design/FUNCTIONAL_SPEC.md) 的 S01–S03／S06、
[Technical Spec](../../design/TECHNICAL_SPEC.md) 的 §1–3、
[GUI Spec](../../design/GUI_UX_SPEC.md) 的 S01–S03、
[Acceptance Spec](../../test/ACCEPTANCE_SPEC.md) 的 B01–B09／T01–T13／AI-LOCAL。
**不准只看本頁就 Coding；若本頁與正式規格不一致，先修本頁。**

## 1. 工作與可操作驗收（AC）

| AC-ID／B-ID | 要做出來的功能 | 具體通過標準／失敗案例 | Evidence |
|---|---|---|---|
| **S1-AC01／B01** | Tauri v2 + TypeScript + Rust，Android 真機可啟動且 UI↔Rust 呼叫成功；建立共用 Core／Android-iOS Adapter 介面 | **飛航模式**可啟動、按操作後 Rust 回傳可驗證的真實結果；Bridge 錯誤不閃退；不得把 Web 模擬當真機結果 | Commit、APK、裝置型號／OS／adb log、Bridge Test |
| **S1-AC02／B02** | 照相、選圖、貼入文字；本地保存與讀取 | N2 圖／文字存入 App 私有空間，**殺掉 App 再開仍在**；拒絕權限、損毀檔、重複／空內容有清楚狀態；沒有自動上傳原圖 | 實機重啟流程、Local storage Test、網路觀察 |
| **S1-AC03／B03** | 真正本地 OCR + Gemma E2B 候選推論、未知與人工修正 | 以日文圖片辨識「パクる」為**內容**而非已確認 Goal；模型離線可推論；回傳無效、無法載入／逾時時顯示錯誤且可手修；不得 Cloud AI fallback | 模型 ID／版本、runtime、真機時間與記憶體、測試圖及 AI-LOCAL-01/02 |
| **S1-AC04／B04** | Content 和 Intent Draft 明確分離，只有使用者能確定 Goal | Drop 日文字、餐廳廣告但**不確認 Goal**：只保存私人內容，不會產生可公開 Intent／真人配對；手動確認後才產生 Intent 草稿 | UI／Rust 測試、T05 |
| **S1-AC05／B05** | Parameter Concept Registry：欄位型別、來源、scope、確認事件、期限、版本、可見性 | N2／JLPT N2 同義合併；AI 傳來假 `CONFIRMED` 或缺 provenance／scope／expiry 應拒絕；未知型別不能自行升格已確認配對條件 | Registry／Rust Unit Tests、T01–T04 |
| **S1-AC06／B06** | SQLite Schema、原子交易與 **8 個有效配對概念 slot 0–7** | 已有 8 類再寫第 9 類，Rust 和 SQLite 均阻擋；並發／直接 SQL 不能繞過；同一概念重用 slot；不會因 100 張圖片佔滿 8 slots；第 9 個**必要安全硬條件**則禁止發布而非自動刪條件 | SQLite Migration、Transaction／Concurrency Tests、T02/T09–T12 |
| **S1-AC07／B07** | Intent 清單、編輯、取消、有效期、離線讀寫 | 單次星期六可用不會變成每週六；過期／撤銷不再有效；離線撤銷本地立即生效，且不謊稱雲端同步完成 | State Machine／DB Tests、T03/T06/T07 的本地部分 |
| **S1-AC08／B08** | 分享 Preview／Consent 紀錄／預設私有 | 未點同意不能上傳／公開；Preview 逐項列出要傳的配對欄位；原圖／OCR／私人筆記**不會自動上傳**；撤銷狀態透明；上傳欄位未確定前先做本地流程，不假裝 Cloud 已安全 | Consent UI、Payload Schema Test、網路監控；真正 Cloud 驗收留 B12 |
| **S1-AC09／B09** | 只問影響決策的 UNKNOWN、保存問題預算 | 每次啟用／配對嘗試**主動 ≤3 題（含 Goal 確認）**，充分資訊時可以 0 題；重試不重複濫問；每日未請求主動 ≤3，Pair-only 題數在 B15 補完 | Budget Unit Tests、T13 的 Intent 部分 |

**驗收分兩層：每個 B-ID 可獨立做 CI／安全測試，但產品是否交付只由 US-01A～D 的真 User Journey PASS 判定。** 一段 NG 就修跨 BXX 的整合，不能靠 BXX 綠燈結案；無依賴的基礎 Coding 可並行。

## 2. Day 1 技術風險優先查證

**歷史已做的基礎 Slice（不代表 US-01 PASS）：** B01 真機 Bridge → B02 SQLite → B03 OCR／Gemma。現在下一個開發起點是 **US-01A：純文字→意圖候選→本人確認→SQLite**，不是再重頭開發 B01 或只測 Gemma 詞義。

**第二個高風險查證：** B02 選圖／儲存 → B03 本地 OCR／E2B。先在 Android 取得模型版本、runtime、ABI、模型大小／記憶體／延遲證據。若需要模型檔下載，只能下載權重／資料，**推論不得交給遠端**。Windows 可以做 Android 開發；iOS 打包／真機需另行提供 macOS＋Xcode。

**工程隔離約束：**共用 TypeScript 畫面、Rust 參數驗證／SQLite、模型輸入輸出契約；原生影像權限、OCR、模型載入、檔案 URI 必須放 Android/iOS Adapter。Android-specific 程式不得直接滲入共用配對規則。

## 3. 最小合成測試集（先做，逐步擴充）

| Fixture | 操作 | 預期 |
|---|---|---|
| JP-01 含「パクる」的自製圖片 | Drop → OCR／E2B | 顯示單字作為內容；Goal 未確認不可發布 |
| JP-02 自製 N2 文法截圖 | Drop → 修改 OCR | 人工修正可保存，來源仍可追溯 |
| NON-GOAL-01 自製餐廳廣告 | Drop | 不自動建立聚餐意圖 |
| PAR-8 / PAR-9 合成概念 | 加入 8 種，再第 9 種 | 第 9 種被拒，舊 8 種未被刪除 |
| TIME-ONEOFF | 輸入一次性星期六可用 | 缺日期不補造；確認後是 fixed，不是 recurring |
| OFFLINE-REVOKE | 已有 Intent，關網後撤銷 | 本地排除、顯示同步待處理；正式雲端 ACK 在 B12 驗證 |

測試不得上傳真實他人截圖、私人聊天或個資至 Public Repo。

## 4. Sprint 1 Debug 與完成條件

每項 Bug 在實作 PR 描述或 `debug/` 留下：`S1-ACxx / Bxx`、手機型號／Android、版本、輸入 Fixture、預期／實際、錯誤記錄、修正 SHA、回歸結果。問題僅影響某 B-ID 時，**不把其他獨立 Coding 全數鎖死**。

Sprint 1 可以宣稱 **全部完成** 的前提：**US-01A～D 真機使用者流程 PASS**，且 S1-AC01～09 的必要 BXX 安全／功能測試均有 Evidence；本地 AI 必須真機跑起來，SQLite 約束需 DB 實測，分享前不可上傳。缺少裝置或模型的項目標 **BLOCKED/NOT_RUN**，其他項目仍可開發。不得因 Web 模擬畫面、靜態 Mock、僅 CI 綠燈就宣稱完成。

下一段：[Sprint 2｜真人配對](SPRINT-02_MATCHING.md)。
