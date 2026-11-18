# Sprint 1｜實際 Coding／驗收進度

> GitHub 追蹤用，不是新 Governance Gate。**只有具備真實 Evidence 才可標 PASS**；上層依據 [Build Bible](../BACKLOG.md)／[Spec Traceability](../TRACEABILITY.md)／[Sprint 1 AC](SPRINT-01_LOCAL_AI.md)。

| Task | 開發狀態 | 測試狀態 | 目前能客觀證明的交付／下一步 |
|---|---|---|---|
| **B01** | IN_PROGRESS | Android 啟動＋Bridge：使用者回報成功；飛航模式 NOT_RUN | PR #6：TS／Rust Unit Tests PASS，Android APK 成功打包。使用者回報 Android 0.1.0 上可顯示 `Rust Bridge 已回應`、`平台：android`。未有飛航模式／adb 日誌；S1-AC01 尚未全部 PASS。 |
| **B02** | IN_PROGRESS（刪除功能已 Coding，待 Android 真機回歸） | **最終 CI：TS 16/16、Rust 21/21、Harness、Android APK PASS**；舊版使用者真機：選圖／顯示／關閉重啟／飛航模式讀回 PASS；**新版刪除真機 NOT_RUN**；拍照／損毀檔／權限拒絕 NOT_RUN | PR #7 同分支：刪除確認、Rust SQLite／圖片檔清理／中斷恢復均已提交，最終可安裝 APK Artifact 見下。F-01 OPEN；B02 尚未完成。 |
| **B03** | IN_PROGRESS（原生 ML Kit OCR 與人工修正已 Coding；Gemma E2B 尚未接入） | TypeScript 22/22、Rust 27/27、Harness PASS；Android 原生編譯與真機 OCR／Gemma：NOT_RUN | PR #8 疊於 PR #7；Rust 透過 App-private Asset UUID 呼叫 Android Kotlin bundled Japanese OCR，SQLite 保存辨識／更正／UNKNOWN。真正 Gemini/Gemma E2B 推論尚未實作，模型未就緒回 MODEL_UNAVAILABLE。 |
| B04 | NOT_STARTED | NOT_RUN | 內容 ≠ Goal |
| B05 | NOT_STARTED | NOT_RUN | Parameter Registry |
| B06 | NOT_STARTED | NOT_RUN | SQLite 原子交易＋8-slot |
| B07 | NOT_STARTED | NOT_RUN | Intent 生命週期 |
| B08 | NOT_STARTED | NOT_RUN | Consent／分享預覽 |
| B09 | NOT_STARTED | NOT_RUN | 追問限制 |

**目前 Coding PR**：[B01 PR #6](https://github.com/NFF98/CKM/pull/6)（基底 App）；[B02 PR #7](https://github.com/NFF98/CKM/pull/7)（疊在 B01 上）。尚未合併；以獨立 B-ID／AC 追蹤。

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
| B03－Gemma E2B | 未接入模型；有 MODEL_UNAVAILABLE 的明確失敗路徑 | NOT_RUN | 選定 LiteRT-LM/runtime/量化與裝置、導入實際權重、真機離線推論、超時與記憶體；不得把人工修正當成 Gemma PASS |

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
- **自動測試**：新增 Rust OCR 持久化、UNKNOWN、錯誤／偽來源、外部刪除級聯；TS Command Contract、錯誤輸入與保密語義。[TypeScript 22/22、Rust 27/27 CI PASS](https://github.com/NFF98/CKM/actions/runs/44245049332)，[Harness PASS](https://github.com/NFF98/CKM/actions/runs/44245049356)。**尚未跑 Android Gradle／原生 Kotlin 編譯及真機**。
- **待測**：Android Gradle plugin 原生編譯、離線 OCR／多行／錯字、部分或空白辨識、大小／記憶體限制、人工修正重啟保留、刪圖級聯、沒有網路請求、Gemma 本地推論。全部 NOT_RUN。
- **APK 策略**：本輪只累積 Coding 和 Test Memo，不新增 APK；待批次條件滿足且使用者需要真機回歸時再打包。
