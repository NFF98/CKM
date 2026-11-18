# CAKAMI｜Sprint 1 B01–B02：跨平台 App 與私人 Drop

**唯一依據**：[`build/BACKLOG.md` B01](../BACKLOG.md)、[`build/TRACEABILITY.md`](../TRACEABILITY.md)、[Technical Spec](../../design/TECHNICAL_SPEC.md)、[Functional Spec](../../design/FUNCTIONAL_SPEC.md)、[GUI Spec](../../design/GUI_UX_SPEC.md)、[Acceptance Spec](../../test/ACCEPTANCE_SPEC.md)、[S1-AC01](../sprints/SPRINT-01_LOCAL_AI.md)。

## 已寫程式／尚未證明

- 已寫：Tauri v2 設定、Vanilla TypeScript UI、可導航的 B01 首頁／診斷頁、Rust `bridge_health` Command、跨平台 Native Adapter 邊界、TypeScript／Rust Unit Tests。
- **已見使用者實測回報**：Android 裝置能開 App 並顯示 Rust Bridge 回應（平台 android、版本 0.1.0）。B01 **飛航模式尚未實測**，因此尚未全部 PASS。
- **B02 已寫程式**：選圖、拍照、文字 Drop、App-private SQLite 和圖片檔案持久化、清單／讀回、重複標示、錯誤防護；CI 已通過，Android 真機驗收尚未執行。
- **未開始**：B03 本地 OCR／E2B、B04～B09。保存內容不等於確認 Goal，也不會發布配對 Intent。
- Phase 1 禁 Cloud AI；本功能不包含 Cloud API，也沒有偷偷 Fallback。

## 開發環境

Node.js 22.16+、npm、Rust stable、Tauri v2；Android 需 Android SDK／NDK、JDK、Rust Android targets，並將 Android Device／Emulator 連線。參考 [Tauri Android 官方 prerequisites](https://v2.tauri.app/start/prerequisites/#android)。Windows 可做 Android 開發；iOS 需 macOS + Xcode。

~~~sh
cd build/app
npm install
npm run check
npm test
npm run build
npm run tauri android init
npm run tauri android dev
~~~

Android 發行版（需對應 SDK／簽署環境）：
~~~sh
cd build/app
npm run tauri android build -- --apk
~~~

macOS iOS（架構保留，**尚未經真機測試**）：
~~~sh
cd build/app
npm run tauri ios init
npm run tauri ios dev
~~~

**真機 B01 驗收**：飛航模式中開啟安裝好的 App → 「裝置狀態」→「執行連線檢查」→ 確實顯示 Rust 平台與 App 版本 → 來回切換首頁不閃退 → 保存 Android 裝置型號、OS、App Version、adb log／不含私密資料的截圖。

**Web 瀏覽器不會模擬成功**：按連線檢查會明確告知目前不是 Tauri App。

**技術暫定值**：`com.nff98.ckm.dev` 是開發期 Bundle ID，正式上架前必須決定正式識別字；`icons/icon.png` 是暫定程式用識別圖，不是正式品牌設計。

## B02 Android 真機驗收（S1-AC02）

**下載**：[GitHub Actions B02 Android APK Run](https://github.com/NFF98/CKM/actions/runs/44048635171) → Artifacts → `cakami-b01-android-arm64-debug` → 解壓縮取得 APK。Artifact 名稱仍為 B01（沿用建置 Workflow），**請以 Run #44048635171 確認這是含 B02 的版本**。

**注意 Debug 簽章**：GitHub Runner 的 Android Debug Key 可能不同。若手機顯示與已安裝舊版簽名不相容，需先解除安裝舊開發 APK 再安裝新版；這會刪除先前的 App 私有資料。B02「重啟資料還在」的測試必須在**同一次新版安裝**之內執行，不可以用解除安裝來測持久化。

1. 安裝新版 APK，開啟「開始」，應看到「選擇圖片」「拍照」「保存私人文字」「我的私人收藏」；若仍是舊 B01 畫面，代表裝到舊版。
2. 在文字框輸入 `パクる`，按「保存私人文字」；應顯示保存成功、清單出現文字，點開可見原文。再貼相同文字一次，應提示重複，且原紀錄仍在。
3. 點「選擇圖片」，選一張沒有隱私的測試 PNG／JPEG，確認預覽及清單。點「拍照」，拍一張測試紙張；若權限拒絕／取消，應有明確結果。
4. **完全關閉 App（從最近使用的 App 列表劃掉）**，重新開啟；兩種資料仍在，點開與原文／原圖一致才算持久化 PASS。
5. 啟用**飛航模式**並強制關閉再開啟，確認私人收藏仍可查看；同時可在「裝置狀態」驗證 B01 Rust Bridge。
6. 嘗試只填空白文字、選 8 MB 以上圖片、取消相機／選圖；不得顯示成功。最後確認沒有自行發布 Goal／找學伴 Intent。

**如失敗**：記錄手機型號、Android 版本、操作步驟、畫面錯誤文字；不需上傳你的私人圖片、文字或 SQLite。

**測試狀態**：TypeScript 13/13、Rust 13/13、APK Build／Harness CI PASS；**Android 真機 B02 NOT_RUN**。程式雖有 Magic Bytes＋WebView 圖片解碼檢查，仍需用真損壞圖片／權限拒絕情境實測。Android OS 備份／資料加密政策屬後續隱私審查，不能從目前 Unit Tests 推定已完全驗證。

**F-01 未決**：PNG/JPEG/WebP、8 MB 單張、64 KB 文字是暫定技術上限，不是已批准的最終格式／容量／批次或重複策略。

## Coding/Debug

在 Coding PR 記錄 `S1-AC01/B01`、測試命令、真實結果、失敗根因、修正 SHA、裝置／OS／runtime。CI PASS 僅能證明有實際執行的 TS／Rust Test，不代表 Android 原生可用。


## B02 刪除收藏補強（2026-11-18，PR #7 同分支）

先前 Android 真機已確認選圖、顯示、重啟與飛航模式下私人圖片留存；「パクる」**尚未經本地 OCR 辨識**，B03 仍未實作。先前 APK **沒有刪除功能**，新版加入單筆「刪除」按鈕及「確認刪除／取消」對話框。

- 「取消」：原紀錄與預覽都應保留。
- 「確認刪除」：由 Rust 依 UUID 刪 App-private SQLite 紀錄及對應 App-private 圖檔；不觸碰手機相簿原圖。若圖片缺失或磁碟／SQLite 操作失敗，不顯示成功；中斷刪除在下次開啟 App 自動檢查恢復／清理。
- **新版回歸測試**：保存一張合成測試圖片與一筆測試文字 → 點刪除再取消，確認仍在 → 再點刪除並確認，清單消失 → 完全關閉重開 → 兩筆都沒有；重複內容各存一筆時，刪掉一筆不應刪另一筆；飛航模式重複測試。
- **回報方式**：提供是否成功及 Android 版本／操作情境即可，不要上傳真實私人圖片。
- **狀態**：新 Rust／TS 測試和 Android APK CI 以此 PR 新 commit 的 workflow 為準；新版 Android 刪除真機驗收仍 NOT_RUN。不可沿用舊版 CI 當作新版 PASS。

## B03｜Gemma 4 E2B 真正 On-device 推論（2026-11-18）

B03 目前已提交 Rust ↔ Android Kotlin LiteRT-LM ↔ SQLite 的 Coding，但**Android 原生編譯／裝置推論驗收與記憶體效能尚未通過**。本地模型不會像 Cloud AI 一樣自動可用。

- **Runtime**：`com.google.ai.edge.litertlm:litertlm-android:0.16.1`，Android CPU-first；鎖版是為了相容 Tauri 模板所使用的 Kotlin 2.2.x 編譯器。不得換成 `latest.release` 造成不可重現建置。
- **來源**：[Hugging Face 官方 LiteRT Community / Gemma 4 E2B](https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/blob/main/gemma-4-E2B-it.litertlm)；正確檔名：`gemma-4-E2B-it.litertlm`，約 2.59 GB。SHA-256：`181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c`。
- **不含在 APK 中、但也不需手動下載**：首次開啟 Android App 時會先檢查模型是否已存在；缺少時自動經非計量網路（通常 Wi-Fi）從官方固定版本 HTTPS 下載。使用者不必自己找檔案、註冊 Hugging Face 或用 SAF 選檔。若只有行動數據，App 等待 Wi-Fi，使用者也可明確點選「改用行動網路下載 2.59 GB」。模型依序寫入 App-private `.partial`、支援 HTTP Range 續傳；完成後驗證精確 2,588,147,712 bytes 與 SHA-256，成功才標為已安裝。未經驗證絕不推論；舊 SAF Command 僅保留向後相容，不在消費者畫面顯示。
- **AI 使用入口**：收藏圖片 → 真正 Android 本地 OCR 或手動修正；或直接保存文字 → 點「用手機 Gemma 理解這筆內容」。僅將該收藏在本機 SQLite 的已保存文字交給 Kotlin LiteRT-LM；原圖不會傳給模型以外的端點，不會上傳任何私人內容。
- **防 AI 捏造**：只接受嚴格 JSON（`evidenceQuote/contentHint/uncertainty`），其中 `evidenceQuote` 必須逐字存在於原文。Rust 拒絕其他欄位、假 `Goal`／`Consent`／`CONFIRMED`，建議只存 SQLite `local_model_suggestions` 且 `status=suggested_private`，不會發布配對意圖。
- **資料失效**：更正 OCR 或刪除原圖後，舊模型建議不再可見；刪圖時 SQLite 外鍵級聯清除模型建議。
- **平台限制**：Android API 26 以上支援自動下載。iOS／desktop 尚無原生 LiteRT-LM Adapter；Gemma CPU 真機 RAM、初始化時間與輸出品質尚未驗證。下載目前是 App 進程內執行：**App 被系統結束時不保證背景繼續，但下次打開會從已保留的 `.partial` 續傳**。不會偷換 Cloud AI。
- **目前待驗證**：Kotlin/Gradle 編譯及 Range/SHA 單元測試、首次從官方 HTTPS 下載完整 2.59 GB、Wi-Fi/行動網路限制、斷線／重啟續傳、空間不足、錯誤 Hash、下載完成後關網再推論、模型 RAM/時間／JSON 輸出。**Android 實機執行前保持 NOT_RUN**。
- **批次策略**：這輪不打包、不提供新 APK；維持「至少兩個可驗收功能一起打包一次」原則。Android Gradle 編譯檢查也不會產生 APK Artifact。

## B04｜Content ≠ Goal 私人草稿 Gate（S1-AC04，PR #9）

**不會自動建立 Goal**：只保存私人圖片、文字、OCR 或 Gemma 的 `contentHint` 都不是「我要找真人學伴」。系統絕不能在使用者沒有明確選擇的情況下，自動建立配對意圖。即使已確認 Goal，B04 也**只保存私人 Intent Draft**，不含 B05 Concept Registry、B06 8-slot、B07 Intent Lifecycle、B08 Consent、雲端同步或任何真人搜尋。

**兩條實際畫面路徑**：點開任一私人收藏後，可選「只收藏內容」或親自輸入「你想找真人學伴做什麼？」。前者不執行 Goal Rust 寫入。後者先輸入例如「我想和真人一起練 JLPT N2 聽力」，點「確認 Goal，建立私人草稿」，再於對話框讀取實際文字、按「確認我的 Goal」才呼叫 Rust。取消／返回修改不應有任何事件或草稿。

**Rust 與 SQLite**：本人確認會產生 Rust-owned UUID event、版本號與 `local_goal_drafts`／`local_goal_confirmation_events`。草稿一律 `goalState=confirmed_by_user`、`intentState=draft_private`、`visibility=private`、`shareConsent=false`、`publishable=false`。修改已確認草稿必須再次確認，傳入既有 revision；過期畫面拒絕覆寫。刪除來源收藏時，草稿及事件會隨 SQLite Foreign Key cascade 一併刪除。**人工確認 Goal ≠ 分享同意**。

**待手機驗收（NOT_RUN）**：新增「パクる」或餐廳優惠文字 → 只收藏 → 沒有 Goal、Intent／配對；手動確認一筆 N2 Goal → 私人草稿可查看，重啟仍在；修改 Goal 產生新版本與新確認事件；取消 dialog 不改資料；刪除來源後草稿／事件消失；飛航模式仍可執行，沒有任何外傳。所有狀態與結果保存至 `build/sprints/SPRINT-01_PROGRESS.md`，只以真實操作 Evidence 標 PASS。

**APK 批次規則**：單筆 B04 Commit 不觸發 Android APK，先用 TS／Rust／Harness 自動化測試。至少兩個可驗收功能成批、且程式與相關 CI 穩定時再安排一次 APK／真機測試。

## B05｜受控 Parameter Concept Registry 與真人確認（2026-11-18）

B05 是 S1-AC05 / T01–T04 的「**概念與私人參數 Gate**」，**不是** B06 的 Matching 8-slot，也沒有分享或搜尋真人的功能。先開啟一筆私人收藏，手動確認 B04 的找學伴 Goal，才能在下方看到「私人條件（B05）」。

- **已知概念**：Registry v1 目前只建立 SSOT 明確列出的一項 `study.jlpt_level`；`N2`／`JLPT N2`／`日語二級`（日文二級等同義別名）會變成一個 `N2` 值。同 Goal 更新同概念只增加版本，不建新的 Slot。
- **未知概念**：T-01 完整白名單／型別／硬條件仍 OPEN，其他名稱只能記為 `UNKNOWN`／`private`，**沒有確認按鈕**，不會送進配對。當前也不把 100 張單字截圖當作 100 個參數。
- **人工提案與確認**：輸入條件 → Rust `preview_parameter_concept` 標準化 → `save_parameter_proposal` 寫入 App-private SQLite `OBSERVED`／`UNKNOWN`；已知 `OBSERVED` 另由使用者開啟確認視窗，按下「確認此私人條件」後 `confirm_private_parameter` 才建立 Rust UUID 事件／`CONFIRMED`。**CONFIRMED 只代表本人確認這個私人值；沒有分享同意、沒有 Active Intent**。
- **來源、權限與版本**：`source=user_explicit` 和 `evidence_ref` 由 Rust 依本人輸入控制；外部不可傳入假 `confirmed_event_id`、`source=model_inferred`、`visibility=published`、`consent=true`；每筆保存需正確 `goal revision` 和 `parameter version`，過期畫面不會覆寫新資料。SQLite `CHECK / UNIQUE / FK` 額外防繞過。
- **Scope 與時效**：此版本只允許 `intent`、`intent_lifecycle` 或一次性 `fixed`。一次性「星期六有空」必須**親自選擇完整未來日期與時間**，絕不自動推成每週重複；`profile`、`pair`、`persistent_review` 尚未提供合法生命週期／持有者資訊，會直接拒絕。
- **Model Boundary**：OCR／Gemma 仍只有私有內容建議；模型輸出的 `CONFIRMED`、假 event／Consent 欄位不會透過 B05 API 寫入合法確認事件。模型不存在／無法載入時可以純手動輸入條件，不宣稱 B03 AI 驗收完成。
- **批次真機待測**：三個 N2 同義詞只成一個，UNKNOWN 無法確認、版本衝突、過去時間與缺日期拒絕、取消不寫、真正本人確認／重開保留、Goal 修改後候選失效、刪來源清空 Evidence／Confirmation、飛航模式、本機無網路上傳。結果登記至 `build/sprints/SPRINT-01_PROGRESS.md`，Android 手機未跑前一律 `NOT_RUN`。
- **APK**：不因這次 B05 Coding 單獨建新 APK，至少兩個獨立功能與單元測試一起具備驗收條件才打包。

## B03 UX 改進｜Gemma 一站式自動下載（2026-11-18）

- **使用者流程**：安裝輕量 APK → 第一次開啟時有 Wi-Fi 就自動從 [LiteRT Community 官方倉庫](https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm) 安全下載 2.59 GB 模型 → 顯示已下載 MB / % → 驗證 SHA-256 → 啟用「用手機 Gemma 理解這筆內容」。不需帳號、不需找下載連結、不需手動匯入。
- **可靠性**：使用不可變 revision `6e5c4f1e395deb959c494953478fa5cec4b8008f`；檔案長度 `2,588,147,712` bytes；驗證 SHA-256 `181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c`；HttpURLConnection HTTPS + TLS、HTTP Range 和 Content-Range 驗證；CDN 不支援續傳回 200 則安全重下，不把資料 append 錯位；.partial 留在手機私有空間；SHA 不符刪除部分檔案，絕不標為 Ready。
- **隱私與流量**：網路連線只從手機 **下載公開模型**，不把任何用戶照片、OCR、文字、Goal 或 prompt 上傳。預設只在非計量網路自動進行；行動網路有單獨按鈕，使用者自行明確決定；可暫停。模型下載完成後，原有推論完全離線。
- **真機限制**：目前使用 App process 內的單獨執行緒；若 App 被系統終止，下載可能停止，下次開啟仍可續傳（不宣稱背景下載一定成功）。Android API 26+；需要足夠可用空間。Android WebView／HTTPS 重導向／官方伺服器完整行為需真機驗證。
- **單元測試**：Kotlin 原生 JUnit 覆蓋正常分段續傳／不合法 Content-Range／不支援 Range 安全重啟／SHA256；TS Contract 覆蓋自動下載觸發、行動網路 opt-in、暫停與假 Ready 拒絕。真實 2.59 GB 下载與 LiteRT CPU 初始化必須等 Android 真機驗收，不可只算 Kotlin 編譯。
- **批次 APK**：與 B02 刪除、B03 OCR／本地推論、B04 Goal Gate、B05 Registry 一起測試，不為這一個功能單獨重打 APK。
