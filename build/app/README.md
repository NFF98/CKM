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
- **不含在 APK 中**：因體積龐大且節省重複編譯／下載，使用者**自行從官方來源取得**檔案，於 Android App 點「從手機檔案匯入 Gemma 模型」。使用 Android SAF Picker，複製至 App-private storage 時串流計算 SHA-256；不符拒絕，無任何自動下載／雲端推論。
- **AI 使用入口**：收藏圖片 → 真正 Android 本地 OCR 或手動修正；或直接保存文字 → 點「用手機 Gemma 理解這筆內容」。僅將該收藏在本機 SQLite 的已保存文字交給 Kotlin LiteRT-LM；原圖不會傳給模型以外的端點，不會上傳任何私人內容。
- **防 AI 捏造**：只接受嚴格 JSON（`evidenceQuote/contentHint/uncertainty`），其中 `evidenceQuote` 必須逐字存在於原文。Rust 拒絕其他欄位、假 `Goal`／`Consent`／`CONFIRMED`，建議只存 SQLite `local_model_suggestions` 且 `status=suggested_private`，不會發布配對意圖。
- **資料失效**：更正 OCR 或刪除原圖後，舊模型建議不再可見；刪圖時 SQLite 外鍵級聯清除模型建議。
- **平台限制**：Android API 26 起允許匯入；iOS／desktop 暫未提供原生 LiteRT-LM Adapter，明確顯示不可用。真機 RAM、CPU 初始化時間、輸出品質尚未驗證。
- **目前待驗證**：Android Kotlin/Gradle compile-only CI、裝置 SHA256 匯入、首次／再次推論、飛航模式、錯誤 JSON、模型不完整與拒絕載入、離線無外傳、120 秒超時與低 RAM、模型結果／來源與 SQLite 重啟持久化。
- **批次策略**：這輪不打包、不提供新 APK；維持「至少兩個可驗收功能一起打包一次」原則。Android Gradle 編譯檢查也不會產生 APK Artifact。
