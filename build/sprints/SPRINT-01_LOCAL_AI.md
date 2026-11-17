# Sprint 1｜本地 AI + 私人意圖（B01–B09）

**目的**：使用者可在真正的 Android App 放入 JLPT N2 圖片／文字，離線由本地 AI 協助理解，自己確認找學伴目標，儲存並預覽分享條件。**iOS 從第一天保留平台 Adapter，並非聲稱 iOS 已通過測試。**

- **範圍**：B01、B02、B03、B04、B05、B06、B07、B08、B09（全部 P0）。
- **開始狀態**：READY_TO_CODE；目前全部沒有 App PASS Evidence。
- **依賴**：以 [Bible B01–B09](../BACKLOG.md) 的逐項依賴為準；B09 需 B05–B07，B08 需 B07。
- **可見 Demo**：開啟 Android App → 選入含「パクる」的圖片 → 本地 OCR／E2B 呈現理解（不擅自判斷找人）→ 本人設定 N2 練習 Goal → 確認配對概念 → 查看將分享的摘要（**未經同意仍不公開**）。

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

**驗收單位是 B-ID，不必為了 Sprint 名稱而一起判 PASS。** B02、B03 等可逐個提交 PR；每次變更要跑相應的已存在自動化測試。圖像推論不可用時，先修 B03；可並行獨立工作，但依賴它的內容不得假裝已完成。

## 2. Day 1 技術風險優先查證

**第一個要跑的 Vertical Slice：** B01 真機啟動 → 使用者點按鈕 → Tauri invoke Rust → 顯示 Rust 回傳。只有這個真成功，才宣稱 B01 Demo 有效。

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

Sprint 1 可以宣稱 **全部完成** 的前提：S1-AC01～09 對應 B01–B09 各有真正的 PASS Evidence；本地 AI 必須真機跑起來，SQLite 約束需 DB 實測，分享前不可上傳。缺少裝置或模型的項目標 **BLOCKED/NOT_RUN**，其他項目仍可開發。不得因 Web 模擬畫面、靜態 Mock、僅 CI 綠燈就宣稱完成。

下一段：[Sprint 2｜真人配對](SPRINT-02_MATCHING.md)。
