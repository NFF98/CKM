# Sprint 1｜實際 Coding／驗收進度

> GitHub 追蹤用，不是新 Governance Gate。**只有具備真實 Evidence 才可標 PASS**；上層依據 [Build Bible](../BACKLOG.md)／[Spec Traceability](../TRACEABILITY.md)／[Sprint 1 AC](SPRINT-01_LOCAL_AI.md)。

| Task | 開發狀態 | 測試狀態 | 目前能客觀證明的交付／下一步 |
|---|---|---|---|
| **B01** | IN_PROGRESS | Android 啟動＋Bridge：使用者回報成功；飛航模式 NOT_RUN | PR #6：TS／Rust Unit Tests PASS，Android APK 成功打包。使用者回報 Android 0.1.0 上可顯示 `Rust Bridge 已回應`、`平台：android`。未有飛航模式／adb 日誌；S1-AC01 尚未全部 PASS。 |
| **B02** | IN_PROGRESS（刪除功能已 Coding，待 Android 真機回歸） | **最終 CI：TS 16/16、Rust 21/21、Harness、Android APK PASS**；舊版使用者真機：選圖／顯示／關閉重啟／飛航模式讀回 PASS；**新版刪除真機 NOT_RUN**；拍照／損毀檔／權限拒絕 NOT_RUN | PR #7 同分支：刪除確認、Rust SQLite／圖片檔清理／中斷恢復均已提交，最終可安裝 APK Artifact 見下。F-01 OPEN；B02 尚未完成。 |
| **B03** | IN_PROGRESS（ML Kit OCR／人工修正＋Gemma 4 E2B LiteRT-LM 端到端 Coding；Native 尚待驗證） | **最新 TS 30/30、Rust 31/31、Harness PASS**；Android OCR／Gemma Kotlin **compile-only PASS**（未打包 APK）；手機 OCR／Gemma：NOT_RUN | PR #8 疊於 PR #7；Gemma 插件採 Android SAF 匯入 `.litertlm`（SHA-256 校驗），Rust 審核原文證據／僅私有候選，SQLite 保存、來源更正後失效；不含 Cloud AI／自動模型下載；Gemma 真機推論尚未 PASS。 |
| **B04** | IN_PROGRESS（Rust Goal Gate、SQLite 私有草稿／確認事件與 S02 UI 已 Coding） | **TypeScript 37/37、Rust 41/41、Harness PASS**；Android 真機 NOT_RUN | [PR #9](https://github.com/NFF98/CKM/pull/9) 疊於 PR #8；「只收藏」完全不建立 Goal，「本人二次確認」才原子建立／修訂私有草稿；沒有公開／Consent／Matching。 |
| B05 | NOT_STARTED | NOT_RUN | Parameter Registry |
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
| B03－Gemma 4 E2B 本地模型 | Kotlin LiteRT-LM CPU + SAF 匯入／SHA-256 驗證 + Rust 受控 JSON/Evidence + SQLite/GUI Coding；TypeScript/Rust CI PASS；**Android Kotlin Gradle compile-only PASS** | NOT_RUN | 匯入 2.59GB 正式模型、Hash 不匹配拒絕、RAM/載入/回應時間、飛航模式、真實文字推論、拒絕偽造 Goal/Consent、重啟保存、修改 OCR 後舊建議失效，證明絕無 Cloud fallback |
| B04－私人 Goal Gate／手動確認 | Rust／SQLite／TS／S02 UI 已 Coding；TypeScript 37/37、Rust 41/41、Harness PASS | NOT_RUN | 只收藏「パクる」與餐廳廣告 → 沒有草稿；使用者手動輸入 N2 找學伴 Goal → 二次確認後才建立本機草稿／事件；返回修改／取消不可寫入；改 Goal 需新事件／版本；殺掉 App 重啟仍在；刪來源級聯；離線確認；無公開／配對／Cloud 上傳 |

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
