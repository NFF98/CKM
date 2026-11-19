# Sprint 1｜實際 Coding／驗收進度

> GitHub 追蹤用，不是新 Governance Gate。**只有具備真實 Evidence 才可標 PASS**；上層依據 [Build Bible](../BACKLOG.md)／[Spec Traceability](../TRACEABILITY.md)／[Sprint 1 AC](SPRINT-01_LOCAL_AI.md)。

## 2026-11-19｜本輪補齊手機可測的 US-01C／D（PR #17）

**新增真實 Coding（非只改文件）：**
- **US-01C / B05：** 已確認 Goal 包含 N2／JLPT N2／日語二級時，畫面提供「採用 Goal 裡的 N2 條件候選」；只會保存 `OBSERVED` 私人候選，**第二次本人確認**才加入 B06 slot。Fashion／Car／`パクる` 等沒有 N2 證據時不自動出現；有新 TS regression。
- **US-01C / B06：** 抽出獨立 atomic slot 配置判斷，新增合成 **8 個已占→第 9 個拒絕、原本 8 個不被踢掉、空位重用** Rust 測試。**不是**虛構 8 個正式批准的 Registry 類別；完整 T-01 白名單仍 OPEN，真實 8 種合法概念的 E2E 保持 BLOCKED。
- **US-01D / B08：** 新增 Rust SQLite `local_share_preview_approvals` / `local_share_preview_events`、使用者逐項選擇「Goal 全文／已確認 N2」及明確二次操作、版本化、快照綁定、撤回、來源刪除級聯。來源 Goal／條件／生命週期改變時舊紀錄失效。新增 TS strict contract、Rust 威脅／負例測試，以及 Android UI 操作；**全部本地，`canUpload=false`**。這是本地逐項審核而非 Sprint 2 的正式 Cloud Consent，未核准欄位、私密 OCR/圖片/補問答案不能拿去公開。
- **本輪不新增** Cloud 連接、任意公開欄位、假真人或假的跨域視覺理解；先讓使用者測試真 Android APK。

**工程測試**：此段程式曾有 TypeScript unknown 型別與 Rust moved value 編譯錯誤，均已修復。最終程式及 APK 的 CI 以 PR #17 最新 Head run／GitHub Actions 為準；不可將此段 Coding 自動寫為 Android 真機 PASS。

**剩餘 Release Gate**：用最新 APK 真機驗證「文字／圖片 OCR→私人建議→補問→本人確認→已核准條件→8-slot→逐項本地審核→撤回／期限／離線」；T-01 八種正式概念仍需 Registry Decisions，未核准前不能以測試假白名單發布。

## 2026-11-19｜US-01B／C／D 後續實際 Coding（PR #17，全部仍待 Android 真機 E2E）

**Stack：** [PR #15](https://github.com/NFF98/CKM/pull/15) → [PR #16](https://github.com/NFF98/CKM/pull/16) → **[PR #17](https://github.com/NFF98/CKM/pull/17)**（`feat/us01b-us01c-local-flow`，Draft；**不是 main 合併或 APK 已發布**）。本段為 2026-11-19 新開發紀錄；下方先前 NOT_STARTED 表格是當時歷史狀態。

| Slice | 真正提交的程式與跨模組測試 | 工程 CI／真機驗收 |
|---|---|---|
| **US-01A** | 沿用 PR #16 的 B02→B03→B04 私人候選／本人確認。Rust／SQLite／TS 原本 CI PASS。 | **Android E2E NOT_RUN**；不宣稱初次候選命中率，PR #17 新增最多三個有原文證據的私人 `intentCandidates`，舊單一候選 SQLite JSON 可向後相容；仍未有真機準確率。 |
| **US-01B** | `main.ts` 將 OCR／更正後的有效文字送入原本 `local_model_propose`，不建立圖片專用 AI；Rust 整合測試驗證 OCR→模型候選→B04 本人確認、來源更正後舊候選失效、無文字為 `vision_unknown`。 | 新版 GitHub CI 見 PR #17 Checks，真機 **NOT_RUN**。**Gemma 沒有圖片像素輸入，不能說看到圖就理解圖像本身**。 |
| **US-01C／B05+B06** | 實際 SQLite `local_matching_slots`，0–7 容量、`UNIQUE(draft_id,slot_index)`、`UNIQUE(draft_id,concept_id)`、來源／確認事件／版本 Admission Trigger，B05 人工確認同交易分配 slot；修改 Goal／Parameter 或撤銷讓舊 slot 失效；TypeScript 嚴格驗證 `matchSlotIndex`。 | SQLite 直接 SQL 偽造／slot 8、來源刪除連動測試已提交。**T01 完整 Approved Registry 白名單未核准，現僅 N2 一種合法概念；8 種＋第 9 種不同合法概念的實際邊界測試 BLOCKED，不能用偽造 8 種類別代替。** |
| **US-01C／B09** | 本地模型可回單一可空 `clarifyingQuestion`；Rust 只記住來源 hash 對應的真人回答，SQLite 最多 2 題（含去重與同一時間一題），另保留 B04 本人 Goal 確認為第 3 步。回答後回到同一 Gemma 引擎重新推論；UNKNOWN／不必問時可 0 題。 | Rust／SQLite quota、重試、人工回答非模型證據、來源更正失效＋TypeScript Contract 已提交；**Android 問題是否真的有用、Gemma 能否穩定輸出題目：NOT_RUN**。不是固定 8 個問題。 |
| **US-01D／B07+B08 local** | Rust／SQLite 私人期限（真完整日期）、撤銷、版本與確認事件、重啟持久化，並與 B06 Slot 聯動；UI 增加本人操作和只讀 Share Preview，明確 `consentRecorded=false`／`canUpload=false`，排除圖片／OCR／私人筆記／追問答案。 | 跨模組 Rust／SQLite 期限／撤銷／B06 Slot 清理／預覽＋TS 測試已提交；**Android 與網路封包 E2E NOT_RUN**。**B08 真實逐項 Consent 與 Cloud 發布不在此宣稱已實作**；T01 與 Cloud Payload 定案前只允許本地 Preview。 |

**PR #17 的實際 CI Evidence（核對 2026-11-19 GitHub Actions Logs）：**
- [B01 TypeScript／Rust Run #44450590257](https://github.com/NFF98/CKM/actions/runs/44450590257) **PASS**：TypeScript **6 個測試檔／60 個測試 PASS**；Rust **80 個測試 PASS**，0 failed；TypeScript typecheck 與 Vite Build PASS（`c33daa59`，僅 docs 差異於程式頭 `201be725`）。
- [Harness Run #44450590265](https://github.com/NFF98/CKM/actions/runs/44450590265) **PASS**。
- [Android Kotlin compile-only Run #44450436567](https://github.com/NFF98/CKM/actions/runs/44450436567) **PASS**（`201be725`，目前最新版 Kotlin 程式與 TS/Rust）；不含安裝／Gemma 實機推論。
- [新版 Android ARM64 Debug APK 批次 Run #44450760331](https://github.com/NFF98/CKM/actions/runs/44450760331)：已依 `apk-batch-ready` 啟動，**建置結果待最終 GitHub workflow**；沒有裝置安裝或 E2E 證據。更新同 App ID 如發生簽章衝突，**禁止直接解除安裝或清除資料**，避免丟失已下載的 Gemma 權重與私人 SQLite。
- 這些 **Coding CI PASS 並非 US-01 Android E2E PASS**；此頁面仍以完整真人 Android 測試為 User Story Release Gate。

**安全界線：** 現在不呼叫 Supabase、不執行 Cloud AI、不聲稱找到真人。完全依照 SSOT 將 8 slots 視為合法匹配概念上限，而非 8 項固定表單；追問不會自動轉成已批准的 `Need`／`Offer`／時間欄位。

**最後驗收狀態：** `US-01 = NG／NOT_RUN`（產品真機）。PR #17 的 GitHub Harness、TS/Rust CI、Kotlin compile-only 是工程 Gate；任何未通過的檢查必須修復並記錄最後 SHA。**尚無 US-01A～D 新版 Android APK 或實體 E2E PASS，不可蓋掉 2026-11-19 使用者報告的前版 NG。**

**接續阻斷：** (1) PR #17 所有最新 Checks PASS；(2) T-01 完整 8+1 核准類別測試，需正式 Registry Decisions；(3) Android 批次 APK 用現有模型做文字、OCR、UNKNOWN、雙追問、本人確認、期限／撤銷、飛航模式；(4) 真機錯誤及模型能力用 log 記錄。**除非全部通過，不改 Sprint 1 狀態為 PASS。**

## 2026-11-19｜US-01A Coding 續作（PR #16；尚非真機 PASS）

**PR：** [US-01A 本地意圖候選 Coding #16](https://github.com/NFF98/CKM/pull/16)，疊在 [User Story Sprint #15](https://github.com/NFF98/CKM/pull/15) 之上；保留 B01～B05 既有程式，不開始第二套架構。

- **B02 → B03 → B04 首次真接線（Coding 已推）：** 純文字保存後自動呼叫裝置端 Gemma；Kotlin 的候選 JSON 可攜帶 `intentCandidate`（領域、可編輯 Goal、原文證據），Rust 對有本人合作需求線索的文字建立 `suggested_private` 私人候選；不明語境 `パクる` 不允許模型猜成找學伴。
- **SQLite**：新增外鍵級聯的 `local_intent_candidates` 私人表，跟 B03 模型解釋結果同一交易寫入，核對來源 SHA；更正 OCR／刪來源後，舊候選不得重用。
- **UI**：看到具體建議可以按「採用這個建議，先修改再確認」，轉入既有 B04 Goal 編輯欄；**只有本人再次按確認**才會有原本 Rust 的私人 Goal confirmation event，沒有 Cloud Consent／Match。
- **韌性**：若手機 Gemma 不輸出可解析的候選、但使用者**原文已明確寫出找人合作意圖**，Rust 可提供**純原文、可修改的私人候選**，不自行捏造目標；只有字詞／廣告／非合作描述仍 UNKNOWN。模型 unavailable 時如實報錯且不走 Cloud AI。
- **新增測試**：Rust 從 B02 text Drop→B03 proposal→B04 human-confirm→重新開啟 SQLite 的跨模組整合；純文字、無上下文、Fashion、壞證據、偽造 Consent、TS contract。**Android 真機完整 US-01A PASS Evidence 尚未取得，不宣稱測試通過。**
- **最新規劃**：US-01A 程式與 CI 修正 → US-01B 圖片共用意圖引擎 → **US-01C 包含 B05＋B06 真 8-slot＋B09 智能追問** → US-01D 私有生命週期與分享預覽。**B06 沒有被刪除／延期到 Sprint 2**；Sprint 1／US-01 必須全路徑真機與相關 DB 安全測試 PASS 方能交付。
- **APK**：遵守既有至少兩項獨立功能準備好再批次打包的決策；PR #16 僅 Coding/CI，暫無新版 APK。不可叫使用者用舊 APK 驗新功能。

## 2026-11-19 最新交付狀態（User Story 優先；舊 BXX 表是歷史工程證據）

| User Story／切片 | 真實測試狀態 | 根因／下一步 | 絕對不能算 PASS 的證據 |
|---|---|---|---|
| **US-01 整條：理解→推測→必要追問→本人確認→私人合法 Intent** | **FAIL / NG（使用者 Android 回報）** | Gemma 能解釋 `パクる`，但 B03 只回 contentHint；B04 空白 Goal 手填、B05 手填參數、B09 未做；**先修 US-01A** | B03 詞義正確、B04 Rust Gate/SQLite 單元測試、B05 Registry CI 綠燈 |
| **US-01A 純文字→私人候選→本人確認/保存** | **FAIL（缺整合；待新版真機復測）** | 在原本 B02-B05 代碼上加候選資料契約、UI 選擇／編輯與 Rust 確認；不需要再重做 B01 | Gemma 能生成一段解釋 |
| **US-01B 圖片 OCR→同一套 Intent 流程** | **FAIL（整合未實作，OCR/詞義子能力初步 PASS）** | 保留本地 OCR/來源驗證，接 US-01A 引擎；跨域歧義保留 UNKNOWN | OCR 辨識 `パクる` |
| **US-01C 智能追問／Concept Registry／8-slot** | **NOT_RUN（未完成整合）** | B05 現僅 N2 alias；B06/B09 NOT_STARTED；新增 Rust + SQLite 跨模組 Gate、B09 ≤3 題 | B05 既有 Unit Tests PASS |
| **US-01D 私有生命週期／分享預覽／離線** | **NOT_RUN** | B07/B08 尚未開始；不得在未授權情況上傳或假稱 Cloud Ready | 單張圖片可離線保存 |
| **US-02 Cloud 真配對** | **NOT_RUN** | Sprint 2；待合法 US-01 Intent 及 B08/12 同意 | UI 假候選／Cloud mock |
| **US-03 真人共同完成＋可信 Outcome 回本地** | **NOT_RUN** | Sprint 2；B16-B18 尚未實作，OutcomeReceipt 合法欄位待決 | 單方確認／只產生 Chat |

**目前下一個 Coding 任務＝US-01A**，輸入 `我想找人一起練 JLPT N2 口說` → 有根據的私人 Goal 候選 → 本人選/改/確認 → 本機 SQLite 真的保存，重開仍在；輸入 `パクる` 則未知／只收藏，不得猜唯一 Goal。跨 B03/B04/B05 完成 Contract Tests，再依現有批次 APK 政策做真機驗收。**任何 User Story FAIL，不因其中 BXX 的單元測試 PASS 而關閉。**

**新執行依據：** [User Story Sprint Plan](../SPRINT_PLAN.md)／[Sprint 1 US-01 切片](SPRINT-01_LOCAL_AI.md)／[User Story Acceptance](../../test/ACCEPTANCE_SPEC.md)。下方原始 B01-B09 舊紀錄保留供追溯，其中早期寫的 NOT_RUN 以當時版本／日期為準，**不能蓋過 2026-11-19 使用者已回報的真機結果**。

| Task | 開發狀態 | 測試狀態 | 目前能客觀證明的交付／下一步 |
|---|---|---|---|
| **B01** | IN_PROGRESS | Android 啟動＋Bridge：使用者回報成功；飛航模式 NOT_RUN | PR #6：TS／Rust Unit Tests PASS，Android APK 成功打包。使用者回報 Android 0.1.0 上可顯示 `Rust Bridge 已回應`、`平台：android`。未有飛航模式／adb 日誌；S1-AC01 尚未全部 PASS。 |
| **B02** | IN_PROGRESS（刪除功能已 Coding，待 Android 真機回歸） | **最終 CI：TS 16/16、Rust 21/21、Harness、Android APK PASS**；舊版使用者真機：選圖／顯示／關閉重啟／飛航模式讀回 PASS；**新版刪除真機 NOT_RUN**；拍照／損毀檔／權限拒絕 NOT_RUN | PR #7 同分支：刪除確認、Rust SQLite／圖片檔清理／中斷恢復均已提交，最終可安裝 APK Artifact 見下。F-01 OPEN；B02 尚未完成。 |
| **B03** | IN_PROGRESS（OCR＋Gemma 本地推論＋一站式模型自動下載已 Coding，尚待真機） | 新自動下載 TS／Rust／Kotlin CI 進行中；手機 OCR／Gemma／下載：NOT_RUN | PR #8 模型適配＋[PR #11](https://github.com/NFF98/CKM/pull/11) 一站式準備（疊 B05 PR #10）；首次 Wi-Fi 自動下載官方 2.59GB 模型、Range partial 續傳、SHA-256 驗證、原生進度；不需使用者手動匯入，也不使用 Cloud AI；未實機驗證。 |
| **B04** | IN_PROGRESS（Rust Goal Gate、SQLite 私有草稿／確認事件與 S02 UI 已 Coding） | **TypeScript 37/37、Rust 41/41、Harness PASS**；Android 真機 NOT_RUN | [PR #9](https://github.com/NFF98/CKM/pull/9) 疊於 PR #8；「只收藏」完全不建立 Goal，「本人二次確認」才原子建立／修訂私有草稿；沒有公開／Consent／Matching。 |
| **B05** | IN_PROGRESS（Rust 受控 Registry／SQLite 私人參數草稿／人工確認／UI 已 Coding） | TypeScript **46/46**、Rust **53/53**、Harness PASS；Android 真機 NOT_RUN | [PR #10](https://github.com/NFF98/CKM/pull/10) 疊於 B04 PR #9；N2／JLPT N2／日語二級歸一；UNKNOWN 留私人候選；固定期限必須本人指定；版本／來源／範圍／事件全由 Rust 控制，無公開／8-slot。 |
| B06 | NOT_STARTED | NOT_RUN | SQLite 原子交易＋8-slot |
| B07 | NOT_STARTED | NOT_RUN | Intent 生命週期 |
| B08 | NOT_STARTED | NOT_RUN | Consent／分享預覽 |
| B09 | NOT_STARTED | NOT_RUN | 追問限制 |

**目前 Coding PR**：[B01 PR #6](https://github.com/NFF98/CKM/pull/6)（基底 App）→ [B02 PR #7](https://github.com/NFF98/CKM/pull/7) → [B03 PR #8](https://github.com/NFF98/CKM/pull/8) → [B04 PR #9](https://github.com/NFF98/CKM/pull/9)（逐層 Stacked、Draft，尚未合併）。依獨立 B-ID／AC 追蹤。

**實際測試 Evidence**
- TypeScript：[`B01 App` Run #44024038403](https://github.com/NFF98/CKM/actions/runs/44024038403)；TS Typecheck、Vitest **5/5 PASS**、Vite Build **PASS**。
- Rust：最初因 App Icon 不是 RGBA 編譯失敗；已修正，[`B01 App` Run #44024343756](https://github.com/NFF98/CKM/actions/runs/44024343756) **Rust 3/3 PASS**。
- Android ARM64 Debug APK：[APK Smoke Workflow](https://github.com/NFF98/CKM/actions/workflows/b01-android-apk.yml) 已建立；曾因同一個 PNG 問題失敗，修正後 [B01 APK Run #44047818523](https://github.com/NFF98/CKM/actions/runs/44047818523) **Build PASS**；使用者已回報 Android 0.1.0 App 與 Rust Bridge 可操作，飛航模式仍待驗證。
- Android 實體裝置啟動＋UI↔Rust：**使用者實測回報成功**；飛航模式和 adb Evidence **NOT_RUN**。不要把雲端 CI 直接算為這些測試 PASS。

**2026-11-17 使用者實測回報**：在 Android 裝置上看到 `Rust Bridge 已回應`，平台 `android`、版本 `0.1.0`、`雲端 AI：禁用`、`本地 AI：尚未接入`。此證據支援實機啟動與 Rust Bridge 功能；尚未驗證飛航模式，所以不能說 B01 全部驗收完成。後續 B02 以獨立堆疊 PR 實作。

**規則**：每次繼續 Sprint 1，先看此表與最新 PR，從未完成的 AC 續做；遇到真機環境不足要標 NOT_RUN／BLOCKED，不得把 Harness CI 綠燈算成 Android 功能完成。


## Sprint 1｜Android 真機測試待辦與 APK 批次佇列（2026-11-18）

**使用者最新指令**：不再每完成一個功能就產出 APK。每項功能完成後只登記待測情境、跑必要的單元測試；累積至少 **2 項 Coding＋相關自動測試完成、可供同一輪真機驗收的功能**，再集中打包一次 APK，避免重複耗時編譯與安裝。此為開發節奏，不更動 Acceptance 或 B-ID Done 規則。只在明確準備好批次時使用 `apk-batch-ready` label／手動執行 Workflow；日常 PR 更新不再觸發 Android APK。P0 安全／啟動阻斷可記錄原因例外提前打包。

**下一批真機驗收佇列**（舊有 APK 已打包的紀錄保留，不重複發給使用者）：

| 功能／待測項 | Coding／單元測試狀態 | 真機狀態 | 待驗證內容 |
|---|---|---|---|
| B02－私人收藏刪除 | 已 Coding；TS／Rust CI PASS | NOT_RUN | 刪除前確認；取消保留；刪文字與圖片；刪重複圖其中一筆不影響另一筆；清單／預覽消失；強制關閉重開仍已刪除；離線刪除；原相簿照片不受影響 |
| B02－拍照與異常輸入回歸 | 功能入口已 Coding；既有部分單元測試 PASS | NOT_RUN | 拍照完成／取消、相機與相簿權限拒絕、損毀圖片、8MB 上限、空白文字、儲存錯誤、不自動外傳 |
| B03－日文 OCR 與人工修正 | Kotlin／Rust／TS Coding 已提交 PR #8；TS 22/22、Rust 27/27、Harness PASS（不等於原生真機 PASS） | NOT_RUN | 斷網 OCR「パクる」與多行日文、UNKNOWN、人工修正、重開保留、更正後刪圖級聯、無原圖外傳；Android 原生 Gradle 編譯待批次驗證 |
| B03－Gemma 4 E2B 本地模型／自動下載 | Kotlin LiteRT-LM CPU＋一站式自動下載+SHA256+Range 續傳、Rust 受控 JSON/Evidence + SQLite/GUI Coding；CI 進行中 | NOT_RUN | 連 Wi-Fi 後首次啟動自動取得官方 2,588,147,712 bytes 模型、進度／暫停／斷網續傳、行動數據需本人點選、Hash 不匹配拒絕、低磁碟空間、RAM/載入/回應時間、飛航模式真實推論、拒絕偽造 Goal/Consent、重啟保存、修改 OCR 後舊建議失效，證明無 Cloud AI fallback |
| B04－私人 Goal Gate／手動確認 | Rust／SQLite／TS／S02 UI 已 Coding；TypeScript 37/37、Rust 41/41、Harness PASS | NOT_RUN | 只收藏「パクる」與餐廳廣告 → 沒有草稿；使用者手動輸入 N2 找學伴 Goal → 二次確認後才建立本機草稿／事件；返回修改／取消不可寫入；改 Goal 需新事件／版本；殺掉 App 重啟仍在；刪來源級聯；離線確認；無公開／配對／Cloud 上傳 |
| B05－Parameter Registry／Rust 概念驗證 | Rust／SQLite／TS／私人 UI 已 Coding；TypeScript 46/46、Rust 53/53、Harness PASS | NOT_RUN | 確認 B04 私人 Goal 後輸入 N2、JLPT N2、日語二級 → 同一個 study.jlpt_level；同義值更新不能建立重複概念／需版本；未知概念只顯示 UNKNOWN 且不能確認／公開；固定日期未輸入拒絕、時間不能變每週；二次確認才有 Rust event；修改 Goal 後參數失效；離線與刪除級聯；原圖、OCR、條件均不外傳 |

**當前狀態**：待測佇列已記錄；**不得僅因 B02 刪除一項功能修改就再要求單獨安裝新版 APK**。待至少兩項功能在同一新批次達到可驗收狀態，再統一提供一份 APK 與整批驗收清單。先前 B02 的真機成功回報仍保留原始 Evidence。

## B02 Coding Evidence（2026-11-17）

- B02 分支：`feat/s1-b02-local-drop-storage`（疊在 PR #6 B01 上，避免未通過 B01 就直接假裝已在 main）。
- 有效功能：App 本地圖片、拍照與文字上傳入口；Rust `save_drop`、`list_drops`、`load_drop`；App-private SQLite + 檔案持久化。原圖不送往網路／雲端，且**不推斷 Goal**。
- **CI Evidence**：[TypeScript 13/13 + Rust 13/13 + Vite Build 全 PASS](https://github.com/NFF98/CKM/actions/runs/44048635199)；[Android ARM64 Debug APK 打包與 Artifact PASS](https://github.com/NFF98/CKM/actions/runs/44048635171)；[Harness PASS](https://github.com/NFF98/CKM/actions/runs/44048635155)。這些是 GitHub Runner 的實際測試，不是 Android 手機 E2E。
- **真機驗收**：新版 APK 的拍照／選圖／文字保存、殺掉 App 後重新讀取、飛航模式與沒有自動外傳**全部 NOT_RUN**。
- 未定：F-01 的最終圖片格式、容量、批次和重複策略，目前 8MB／64KB 僅工程安全界線；顯示重複但保留兩筆，不宣稱為最終產品策略。


## B02 真機回報與刪除補強（2026-11-18）

- **已回報真機 PASS（舊版 B02 APK / Run #44048635171）**：從手機選擇含「パクる」的圖片，圖片成功保存／顯示；完全關閉 App 再開收藏仍存在；飛航模式下依然能讀回收藏。這只能證明圖片保存和讀取，**不代表「パクる」已由 OCR 辨識**（B03 尚未實作）。
- **已發現 FAIL（舊版）**：無「刪除收藏」入口。於 PR #7 同一分支新增單筆刪除與二次確認；僅刪 App-private 複本與 SQLite 文字／圖片紀錄，不刪相簿原圖。
- **新增單元測試案例**：TS 刪除 Contract／非法 ID／失敗回應；Rust 文字與圖片刪除、重啟後消失、重複內容獨立刪除、路徑穿越拒絕、圖片缺失不刪 DB、SQL 故障回復、刪除中斷恢復。**最終 CI 確認通過，見下列 run 和 commit。**
- **仍待真機**：使用新版 APK「新增圖片／文字 → 點刪除 → 取消不刪 → 再點刪除並確認 → 清單和預覽消失 → 強制關閉 App → 重開後不再出現 → 飛航模式重測」。拍照、損毀圖片、權限拒絕、無自動外傳仍需另行驗收。
- **不得合併或宣稱 B02 完成**：PR #6、#7 仍為 Stacked PR；當前維持 B02 IN_PROGRESS，B03–B09 NOT_STARTED。

### B02 刪除最終 CI 與 APK（2026-11-18）

- **受測程式 commit**：`5ae27328629265dde63e3d3880eb813fe5a5f082`（PR #7 分支，含 SQLite 刪除／圖片暫存／中斷恢復／清理錯誤提示）。
- **[TypeScript／Rust CI #44209853504](https://github.com/NFF98/CKM/actions/runs/44209853504)**：TypeScript typecheck／Vitest 16/16／Vite Build PASS；Rust 21/21 PASS。
- **[Harness #44209853490](https://github.com/NFF98/CKM/actions/runs/44209853490)**：PASS。
- **[Android ARM64 Debug APK #44209853489](https://github.com/NFF98/CKM/actions/runs/44209853489)**：Build／Artifact PASS；Artifact ID `14384594084`、名稱 `cakami-b01-android-arm64-debug`（沿用舊名稱，此 run 是含刪除功能的 B02 新版）。
- **Android 真機新版刪除：NOT_RUN**。需測「新增 → 刪除按鈕 → 取消 → 再次確認刪除 → 強制關閉 → 重開已不存在」，另測文字、重複圖片及飛航模式。APK CI PASS 不代表此真機案例 PASS。
- 若新版 APK 與舊版 debug 簽章不符，卸載再安裝會清除原 App 資料；須用**新版內重新建立的測試資料**跑完整刪除回歸。

## B03 部分 Coding（2026-11-18；PR #8）

- **分支**：`feat/s1-b03-native-ocr-correction`，疊在 B02 PR #7 上；不合併 PR。
- **Rust／Android native OCR**：Tauri v2 plugin + Kotlin Android ML Kit bundled Japanese 16.0.1；Rust 僅依使用者 App-private 圖片 UUID 讀本地圖像資料，Android 模型在裝置端執行，不呼叫 Cloud OCR。
- **資料與權限**：SQLite `local_ocr` 保存 rawText、人工更正、UNKNOWN、來源／segments／engine／elapsedMs；Rust 在每個 SQLite Connection 啟用 FK，刪除圖片時級聯清除 OCR／更正；人工修正不是 Goal／Consent 確認。
- **GUI**：圖片預覽提供「用手機辨識日文」「保存人工修正」，可在未辨識／未知／錯誤時手動補文字，未連接的 E2B 顯示不可用。
- **模型現狀**：Gemma E2B runtime／權重尚未加入；`local_model_propose` 明確回 `MODEL_UNAVAILABLE`，禁止 Cloud fallback。不能宣稱 B03 完成。
- **自動測試**：新增 Rust OCR 持久化、UNKNOWN、錯誤／偽來源、外部刪除級聯；TS Command Contract、錯誤輸入與保密語義。[TypeScript 22/22、Rust 27/27 CI PASS](https://github.com/NFF98/CKM/actions/runs/44245049332)，[Harness PASS](https://github.com/NFF98/CKM/actions/runs/44245049356)。**尚未跑 Android Gradle／原生 Kotlin 編譯及真機**；[最新 Commit 的 TypeScript／Rust CI #44245448150 PASS](https://github.com/NFF98/CKM/actions/runs/44245448150)，[Harness #44245448135 PASS](https://github.com/NFF98/CKM/actions/runs/44245448135)。原生 Kotlin 20 秒超時處理已加入；最新 CI 仍不能代表 Android 編譯。
- **待測**：Android Gradle plugin 原生編譯、離線 OCR／多行／錯字、部分或空白辨識、大小／記憶體限制、人工修正重啟保留、刪圖級聯、沒有網路請求、Gemma 本地推論。全部 NOT_RUN。
- **APK 策略**：本輪只累積 Coding 和 Test Memo，不新增 APK；待批次條件滿足且使用者需要真機回歸時再打包。

## B03 Gemma 4 E2B Coding 增量（2026-11-18）

- **開發程式**：同 PR #8 增加獨立 `tauri-plugin-cakami-model` Android Kotlin Adapter，依官方 LiteRT-LM Kotlin API `Engine → createConversation → sendMessage` 實作 CPU-first 本地推論；Model file 不打包 APK、不使用 Cloud AI／Cloud OCR。
- **模型導入**：本人從 Hugging Face 官方來源下載 `gemma-4-E2B-it.litertlm`（約 2.59 GB），Android SAF 手動選檔；App-private streaming、`SHA-256=181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c` 檢查；錯檔／校驗失敗不回報已安裝。Android API 26+ 才允許匯入。LiteRT-LM 版本固定 `0.16.1` 以配合當前 Tauri Kotlin 2.2.x。
- **模型輸出保護**：Rust `local_model_propose` 只以 App-private UUID 對應已保存的文字／OCR／人工更正為輸入，僅接受 JSON `evidenceQuote/contentHint/uncertainty`、逐字來源存在、無未知鍵；任何 Model `goal`／`consent` 等偽造欄位被拒；只存 `suggested_private` 而不是本人確認。SQLite `local_model_suggestions` FK 級聯刪除，來源更正後 stale suggestion 不再可見。
- **UI**：新增模型狀態、手動匯入和「用手機 Gemma 理解這筆內容」；清楚告知模型並非一定可用、輸出僅為私人建議。
- **自動化 Evidence**：[TS 30/30 + Rust 31/31](https://github.com/NFF98/CKM/actions/runs/44253299393) PASS；Harness 在同系列 Run PASS。Android Kotlin/Gradle 專用 **compile-only** Workflow `.github/workflows/b03-native-compile.yml` 不產 APK；第一次執行因 `tauri android init` 不生成 `tauri.settings.gradle` 而在設定階段失敗，並非 Kotlin 程式錯誤；改成獨立 Library Gradle 驗證：[Run #44253782242](https://github.com/NFF98/CKM/actions/runs/44253782242)，結果以最新實際 CI 為準。
- **仍未驗證**：LiteRT-LM 0.16.1 Android 原生編譯及模型實際載入、2.59GB 匯入和 SHA 校驗、真機 OCR／Gemma 日語品質、120s 超時、低記憶體／效能／電池、飛航模式／無雲端呼叫。B03 **IN_PROGRESS / Android NOT_RUN**。
- **開發節奏**：只記錄待測與 CI，暫無新版 APK，不要求使用者現在測試；至少兩項已完成功能一次合併真機驗收。

### B03 原生 Android Kotlin CI Evidence（2026-11-18）

- **Android 原生 compile-only PASS**：[GitHub Actions Run #44254131510](https://github.com/NFF98/CKM/actions/runs/44254131510)，受測 Source SHA `9acdc8bea0e1f74a651a133b3f28f5df99785815`。Log 同時包含 `:tauri-plugin-cakami-ocr:compileDebugKotlin` 與 `:tauri-plugin-cakami-model:compileDebugKotlin`，`BUILD SUCCESSFUL`。
- 最初 Android Gradle job 因 `tauri.settings.gradle` 在 init 階段不存在而失敗；修改 CI 為臨時三個 Library 的 Gradle 設定，後續發現缺少 AndroidX ActivityResult import 依賴並已修正，**最終在不打包 APK 的前提下通過編譯**。
- **此 PASS 僅證明 Gradle Kotlin 編譯，不代表**：安裝後能開啟原生 Plugin、2.59GB 模型可成功匯入、模型可在目標手機推論、耗時／RAM／電池符合要求或 OCR 辨識正確。以上皆 `NOT_RUN`。
- TypeScript 30/30、Rust 31/31、Harness PASS 的最早完整 [Run #44253299393](https://github.com/NFF98/CKM/actions/runs/44253299393)；最新 Rust／TS 改動若有，須看相應 PR CI。B03 維持 `IN_PROGRESS`，B04 仍 `NOT_STARTED`。
- **APK**：沒有產出新 Artifact，等後續批次真機驗收才安排。

## B04 Content ≠ Goal 交付與待驗收（2026-11-18）

- **SSOT**：B04／S1-AC04；Functional S02 Gate G1；T05（餐廳廣告只有內容不等於需求）、E2E-02（只收藏「パクる」不配對）。
- **程式位置**：PR #9 `feat/s1-b04-private-goal-gate`，從 PR #8 B03 分支延伸。新增 `build/app/src-tauri/src/goal_draft.rs`、`build/app/src/platform/private-goal.ts`、對應 TS/Rust tests，改 `src/main.ts` 串接 UI，保留每層獨立 PR。
- **路徑 A：只收藏**：Drop、OCR、Gemma 只會存 private content／AI suggestion，`load_private_goal_draft` 為 `null`；「只收藏內容」按鈕不呼叫任何寫入命令。Goal 不會被 AI 捏造。
- **路徑 B：明確確認 Goal**：使用者手動輸入要找真人學伴的 Goal → 點「確認 Goal」→ 二次對話框「確認我的 Goal」→ `confirm_private_goal`；Rust 驗證 UUID、限長、`confirmationAction=confirm_goal`、舊 revision，生成 event UUID，使用 SQLite IMMEDIATE Transaction 同步建立 `local_goal_drafts`（`draft_private`、`private`、`share_consent=0`）及 `local_goal_confirmation_events`。修改也需重新確認並產生新事件，無意圖自動發布。
- **SQLite 保護**：`CHECK` 約束拒絕直接改成 published／active／consent=1，`FOREIGN KEY ON DELETE CASCADE` 隨來源刪除清空該私人草稿與確認歷史。完全不介入 B05–B08 的 Registry、8 slot、Lifecycle、Consent；保留後續工作。
- **測試**：Rust 單元涵蓋只收藏無 Goal、人工確認／重開、過期版本、來源不存在、偽造 AI event／Consent、SQLite direct UPDATE 拒絕與刪除級聯、外鍵不容許假確認事件、Gemma 候選不會自動建 Goal；TS 合約與輸入防護。[最終程式 TypeScript 37/37 + Rust 41/41 PASS (Run #44257621647)](https://github.com/NFF98/CKM/actions/runs/44257621647)，[Harness PASS (Run #44257621807)](https://github.com/NFF98/CKM/actions/runs/44257621807)。受測 Source SHA `d153962e6f7c280410f7b42456bcf1a6f434bba7`；Android 真機仍未執行。
- **真機待測**：文案／兩條入口／二次確認取消／真正事件／更新衝突／重啟保存／刪來源後全數消失／飛航模式／無資料上傳。全部 `NOT_RUN`。
- **APK 節奏**：持續開發與自動測試；**不在此 commit 建立或提供 Android APK**。先把待測情境加入批次佇列，待後續整批功能穩定才安排 Android 真機測試。

## B05 Parameter Concept Registry Coding 與真機待測（2026-11-18）

- **來源**：main SSOT `build/BACKLOG.md` B05、`design/TECHNICAL_SPEC.md` §3、`build/sprints/SPRINT-01_LOCAL_AI.md` S1-AC05、`test/ACCEPTANCE_SPEC.md` T01–T04、T11。疊於 B04 PR #9 的 [PR #10](https://github.com/NFF98/CKM/pull/10)，不影響 main，仍是 Draft。
- **邊界 T-01 OPEN**：只有 SSOT 確定的 `study.jlpt_level` N2／JLPT N2／日語二級 alias 進入目前受控 Registry v1；不自行發明完整欄位白名單、N1–N5 enum、硬條件優先級。其他字詞／臨時時間只能成為 `UNKNOWN` 私人候選，不會自動新增正式 Concept。
- **Rust 單一控制點**：`list_concept_registry`、`preview_parameter_concept` 為只讀；`save_parameter_proposal` 只接受 B04 已由本人確認的 `draftId`、`expectedDraftRevision`、字詞／值、`scope=intent`、期限與參數版本。不能由 UI／AI 指定 source、CONFIRMED、event ID、owner、visibility、consent 或 Matching Slot。只記錄 `source=user_explicit` 和 Rust evidence UUID。
- **人類確認**：按「保存私人條件候選」只會存 `OBSERVED`／`UNKNOWN`；對已知且仍有效的 `OBSERVED` 按「由本人確認此條件」並在第二個確認視窗再次確認，Rust 以 Immediate transaction 建 `local_parameter_confirmation_events`，把參數升成 `CONFIRMED`（僅**私人已確認**，未授權分享／不代表 Matching 生效）。UNKNOWN 無確認按鈕；重複版本或不同 Goal revision 會拒絕。
- **SQLite double gate**：`local_parameter_evidence`、`local_parameter_drafts`、`local_parameter_confirmation_events`；CHECK／NOT NULL／FK／索引拒絕非 intent scope、空來源、假公開／Consent、未審核概念升格、不合法值與過期版本；已知概念以 `UNIQUE(draft_id,concept_id)` 去重，參數確認事件由 Rust 產生、關聯原始證據。刪來源 → B04 Goal → B05 私人參數與事件同步級聯。B06 的 8-slot／並發 Match 概念表仍未實作。
- **期限**：只支援 `intent_lifecycle` 或 **明確本人指定日期+時間的 `fixed`**。一次性「週六有空」沒有日期就拒絕 fixed；不擅自假設重複每週，也不以 Pair 的一次性回答寫成 Profile。Profile／Pair scope 留給後續受控規格和功能。
- **自動測試**：T01 AI 假 CONFIRMED／Consent／source 欄位拒絕、真正確認 Event；T02 直接 SQLite NULL／非法來源／scope／expiry／State／visibility 修改失敗；T03 fixed 無日期拒絕；T04 N2 aliases 去重；T11 100 筆單字收藏不建立 100 個參數；UNKNOWN、Goal revision 變動、刪除級聯、重開後保存。[最終程式 TypeScript 46/46、Rust 53/53、typecheck／Vite build PASS（Run #44260591263）](https://github.com/NFF98/CKM/actions/runs/44260591263)；[Harness PASS（Run #44260591264）](https://github.com/NFF98/CKM/actions/runs/44260591264)。受測 B05 Source SHA `8da944139db99c82d81ab938c6a9b562462aecab`。早期 CI 有 Rust stmt E0597 與 SQLite CHECK NULL 三值邏輯的失敗；**都已修正**並加入反例測試，不能把失敗 run 當最終驗收。
- **真機 Batch NOT_RUN**：新增收藏只作 Content → 使用者確認 Goal → 輸入三個 N2 別名；未知類別、一次性星期六、手動固定日期、確認與取消、重啟持久化、修改 Goal 舊候選失效、刪除來源後參數全部消失、飛航模式；確認 UI 不會產生任何分享／配對 Intent／網路呼叫。
- **APK 規則**：只累積 Coding 和真機 Test Memo。不在單一 B05 commit 建 Android APK；至少兩項功能 Coding＋自動測試準備好再批次測。

### B05 最新 CI Evidence／待測狀態（2026-11-18）

- [B05 TypeScript／Rust Run #44260591263](https://github.com/NFF98/CKM/actions/runs/44260591263)：Vitest **46/46 PASS**（6 test files），Rust **53/53 PASS**，TypeScript Check／Vite Build PASS。基於 `8da944139db99c82d81ab938c6a9b562462aecab`。
- [B05 Harness #44260591264](https://github.com/NFF98/CKM/actions/runs/44260591264)：PASS。
- **B05 Coding＋單元測試 PASS；真機 NOT_RUN**。SSOT 的 T-01 全 Registry 名單／硬條件尚 OPEN，完整 Match Concept Registry **不得宣稱已定案**。目前受控範圍只有已確認規格 N2 同義詞，其他只保留私人 UNKNOWN。
- B06 8-slot／transaction match constraints、B07 lifecycle、B08 consent 仍各自 NOT_STARTED。此次只新增測試備忘錄，**沒有新版 APK**，不要求使用者單獨安裝測試。

## B03 UX 修正：Gemma 4 E2B 自動下載（2026-11-18）

- **新 PR**：[B03 Gemma 一站式下載 PR #11](https://github.com/NFF98/CKM/pull/11)，疊 B05 PR #10，保留 B02～B05 已完成 Coding，B06 暫不開始。現在使用者要求的是**不用手動找 2.59GB 檔案、不必手動匯入**，並準備進行一次整批 Android 真機驗收。
- **使用者動線**：開啟 App → 有非計量連線（通常 Wi-Fi）就自動啟動官方模型下載 → 頁面顯示 MB／% → 完整 SHA-256 驗證後才可離線推論。沒有 Wi-Fi 先等待；要使用行動數據必須本人另按明確按鈕；可暫停和續傳。Web／非 Android 環境顯示不可用，不用 Web mock 假裝完成。
- **來源固定**：LiteRT Community 官方 Hugging Face commit `6e5c4f1e395deb959c494953478fa5cec4b8008f`，2,588,147,712 bytes，SHA-256 `181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c`。SDK 未完成不宣稱可用；模型未進 APK 但使用者不需手動下載／匯入。
- **下載與隱私**：Android Kotlin App-private `.partial`、HTTPS、Content-Range 嚴格驗證、伺服器忽略 Range 回 200 時從 0 安全重下、完整 SHA 驗證、原子檔案 rename、校驗標記；只下載公開模型，不上傳照片、OCR、文字、Goal 或模型 prompt。網路只用於取得權重，AI 推論本身不發雲端 API。
- **已知限制**：目前下載工作執行於 App process，Android 系統終止 App 時不保證背景繼續；重新開啟會從有效 .partial 續傳。耗電、真機內存、外部來源 HTTPS/redirect/Range、Wi-Fi 中斷與空間容量需要手機實際驗收，不可假稱 PASS。
- **CI／真機**：Rust／TS 自動化與 Android Kotlin 編譯＋JUnit Range／SHA256 測試均已加到 CI；依最新結果標 PASS/FAIL。真正 2.59GB 下載和推論仍然 **NOT_RUN**。
- **APK 批次**：由先前測試備忘錄中的 B02 刪除、B03 OCR＋Gemma、自動下載、B04 目標確認、B05 同義概念一起驗收，**不是每一項重出一個 APK**。先確認本 PR CI、Kotlin 測試，再產出一次 Android ARM64 測試 APK。

## B03 Gemma 首輪手機推論 FAIL 修正（2026-11-18）

- **使用者回報的真實 Android 測試 Evidence**：已自動下載 Gemma 4 E2B；對私人日文單字「パクる」按「用手機 Gemma 理解」時，App 顯示 **「模型輸出無法核對原文」**（舊版錯誤碼 `MODEL_INVALID_OUTPUT`）。屬於 B03 模型回答／Rust Output Validation 真機 **FAIL**；不可標為 B03 PASS。此錯誤表示模型輸出通過 native 呼叫到 Rust save 階段，但無原始模型回答可判定具體哪個條件違反；不應宣稱已確定是 JSON fence 或 hallucination。
- **程式問題**：原 Rust `serde_json::from_str` 嚴格只接受裸 JSON、`evidenceQuote` 必填且逐字匹配；小型模型輸出 Markdown fence、普通敘述、漏掉 quote、日文引用標點都會無差別回 `MODEL_INVALID_OUTPUT`。
- **修正方案（PR #12）**：只有私人建議可以接受短普通文字及 fenced JSON；如果模型沒給 evidenceQuote，Rust 自私人收藏原文（或本人更正後的 OCR）擷取不超過 80 字原文。若模型**明確給錯引用**，仍拒絕；多餘未知 JSON 欄位（含 Goal、Consent）、偽造確認／公開欄位仍拒絕；OCR 修改後舊提示仍失效。明確區分格式錯誤／原文引用不符／越權欄位，方便下一次真機回報。Android Gemma 提示詞更短且預算 384 tokens，減少 JSON 被截斷的風險。所有候選仍是 `suggested_private`，不會自行建立 Intent／分享。
- **回歸測試**：保存單字「パクる」後接受合法 JSON、fenced JSON、普通說明、由 Rust 原文擷取 quote；錯的 quote／偽造 Consent 或 Goal／格式破損皆拒絕。先跑 Rust/TS 和 Android Kotlin compile-only CI；**新版 APK 尚未打包**，避免每修一個問題就產生一次 APK。手機修正結果保持 NOT_RUN。
- **模型檔處理**：此為 Rust/提示詞修正，不改官方模型 SHA、大小或路徑，不應要求使用者重新下載模型。下次使用同簽章同 application ID 升級安裝時，原 App 私人檔應保留；若解除安裝或清除 App 資料則可能被刪除。

## B03 Android 回歸：Gemma 已下載卻顯示「本地 AI 尚未接入」（2026-11-18）

- **用戶真機回報 FAIL**：Gemma 2.59 GB 已自動下載，B01 裝置狀態頁依舊顯示「本地 AI：尚未接入」。實際查碼發現有兩個硬編碼：B01 `platform::NativeAdapter::native_ai_state()` 永遠回 `not_installed`，首頁 `runBridgeCheck()` 又無視 `nativeAi`，固定輸出「尚未接入」。
- **與原本 B03 Gemma「パクる」原文輸出驗證 FAIL 一起修**：[PR #12](https://github.com/NFF98/CKM/pull/12)，Rust `bridge_health(app)` 呼叫真實 `tauri_plugin_cakami_model::status(&app)`；嚴格區分 `model_verified`、`downloading`、`waiting_wifi`、`paused`、`download_error`、`not_installed`、`unavailable`、`status_error`；TS Contract 和手機診斷頁用實際回傳狀態顯示。模型已校驗不等於推論通過，避免誤導。
- **回歸測試**：Rust 模擬各種本地模型狀態，要求已安裝經驗證才有 `model_verified`；TS 檢查真實型別並拒絕捏造 cloud_ready；既有 Bridge、B02–B05 和 B03 安全驗證全部要過。無 Cloud fallback。
- **新版 Android ARM64 APK**：本次為**兩項已發生的真機測試失敗的阻斷性修復**（模型 JSON／日文 quote + Bridge 錯誤狀態），明確依使用者要求立刻批次打包再測。Android 需同一 app ID／簽章**直接升級安裝**，不要先解除安裝或清除 App 資料，以免已下載 Gemma 2.59GB 丟失；若遇簽章衝突則需先處理既有資料保留，不可未經提醒就卸載。
- **下一步真機驗收**：首頁確認 Gemma 已安裝 → 裝置狀態重新執行檢查應顯示「Gemma 4 E2B 已安裝／校驗通過（推論仍須實測）」；收藏一筆純文字 `パクる` → 點理解，若回 JSON／fenced JSON／簡短敘述都應保存為 **私人候選**，不應越權變成 Goal。再測飛航模式，原模型檔無需重下。這些真機結果尚為 NOT_RUN。

## 使用者最新真機驗收：B03→B04「知道下一步做什麼」目標 NG（2026-11-19）

- **現場測試（使用者直接回報）**：修正版 Android APK 已能 OCR，Gemma E2B 也能對日文字詞 `パクる` 提供詞義；但使用者要求的目標是 App 懂得「接下來該做什麼」，實際結果 **0 個可執行的下一步**。使用者明確判定 **B03→B04 Action Intelligence = Test NG**。
- **精確分層**：B03 OCR／詞義理解可標記「此測項初步 PASS」，**不能**推出 Action Intelligence PASS；B04 已有「真人手動輸入／確認私人 Goal」的防護程式，**不等於** AI 能從內容提供恰當的可執行行動候選。現有 B03 Gemma Prompt 明確禁止產生 Goal，而 B04 不會自動從內容推薦下一步：這是**現有功能契約與使用者新驗收目標的缺口**，不是誤認模型無法理解單字。
- **狀態**：B03、B04 原始 Coding 實作存在；B03→B04 **產品價值整合／Action Intelligence E2E = FAIL／NG（待修復）**，不得宣稱已完成或進入 B06 取代此缺口。B05 Concept Registry Coding／CI 既有 PASS 不變，但正式真機驗收仍未全部完成。
- **修復方向（先設計／測試，再進 Coding）**：區分「客觀原文／詞義／內容類別」→「模型建議的下一步私人候選」→「本人選擇動作」→「實際執行並保存」。對 `パクる` 可以提出**複習、練用法、找學伴、只收藏**等動作選擇，**不能把 AI 猜測當作本人 Goal、Consent、CONFIRMED 或公開分享**。單一詞義不能可靠推定使用者真意，應給少量有用選擇而非造假自動 Goal。
- **要求的下一次真機測試**：Drop `パクる`，不用再打字即可看到至少一個明確、相關且可執行的下一步；使用者點選後 App **真的完成**（如建立本地複習項或引導經本人確認的私人 Goal），重新開啟仍保留必要結果；只收藏／取消不建立 Goal。離線可用、無雲端 AI、無自動公開。**當前 NG，下一版尚未修復。**
