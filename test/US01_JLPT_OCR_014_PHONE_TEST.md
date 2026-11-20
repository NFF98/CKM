# CAKAMI v0.1.4｜JLPT 空推薦、日文 OCR 40% 真機修復驗收

**2026-11-21**。v0.1.3 因低信心 OCR 回歸測試 FAIL 已作廢，請只安裝通過修復測試的 v0.1.4。SSOT：`build/BACKLOG.md` US01-JLPT-01 / US01-OCR-02。真機結果：**NOT_RUN**，不得把 CI 說成 OCR 準確率 PASS。

## 用戶手機一步步測

1. **不可先解除安裝現有 CLEAN CAKAMI**。直接安裝相同簽章、原包名 `com.nff98.ckm.dev` v0.1.4（versionCode 1004）更新 APK；驗證原收藏、E2B 保留，沒有重抓 2.59 GB。
2. 「新增」輸入 **JLPT** → 儲存。直接進「處理內容 → 2 AI 理解」看到三個可選下一步：**看 JLPT 級別／建立 JLPT 複習卡／找人一起練日文**。無須按 Gemma、不須額外輸入「我想找人」、網路可離線。從卡點「查看」應顯示 N5–N1；點「儲存到收藏」應真的新增一張 SQLite 複習卡；返回收藏應可開啟。點「填入我的目標」應切至 Goal 編輯欄，但**未經本人確認不得擅自儲存或公開 Goal**。
3. 輸入 **JLPT N2**：級別說明與新複習卡應保留 N2，不能擅自猜其他級別；只輸入 **JLPT** 時標明級別未指定。
4. 用**上次準確率約 40% 的同一張日文照片**，打開「內容 → 辨識」。新版本機 ML Kit 會讀取 EXIF 方向，不再硬編碼 0°；若是較小、低信心圖片，會在時限內嘗試對比增強再辨識，且僅較可靠時採用第二次結果。人工修正仍應能保存並在重啟後保留。
5. 比較**相同 5～10 張照片**的原文／修改前後 OCR 輸出。分別記錄：文字總數、正確字數、實測字元正確率；**沒有照片和人工標準答案就不能宣稱準確率已達成或從 40% 升到某百分比**。重拍時務必文字清晰、避免反光；無法辨識時顯示需要人工修正，不能把錯 OCR 當已確認的本人意圖。

## 程式防護及退出條件

- **推薦**：使用本機可驗證的 `suggestLocalNextActions()`，包含 JLPT、單字及一般內容的保守選項；與 E2B 模型候選不同，**不是已確認 Intent**。
- **OCR**：仍使用打包的 `com.google.mlkit:text-recognition-japanese:16.0.1`；新的 `androidx.exifinterface:exifinterface:1.4.2` 只在手機內解 EXIF，不外傳照片。低信心對比重試每筆最多一次，僅原圖小於 2.5 MP、時間剩餘超過 6 秒時執行；不做無限制多次模型推論。
- **測試**：Vitest 應驗證單詞 JLPT 必有三個可點的可執行行為及安全約束；Kotlin 應編譯並執行 OCR enhancement policy tests。型別、Rust、Gradle 原生編譯及 APK build 全通過才提供使用者。
- **升級簽章**：v0.1.4 APK 必須憑證 SHA256 `a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5`（與 CLEAN 0.1.2 完全一致），Android versionCode 1004 且 APK v2/v3 簽章有效。
- 不改 App ID、不卸載、不清資料、不重下載 Gemma E2B；不用雲端 AI/OCR。

**目前待確認**：使用者真機上實際日文 OCR 修正前／後的文字準確率、JLPT 三種選項是不是有幫助，兩者仍需以使用者回報判斷 PASS / FAIL。

## v0.1.4 發版檢查（2026-11-21）

- [TypeScript／Vitest、Rust PASS](https://github.com/NFF98/CKM/actions/runs/44817321437)：JLPT 本地候選和既有 Rust 安全功能通過。
- [Android Kotlin OCR/Model 編譯及單元測試 PASS](https://github.com/NFF98/CKM/actions/runs/44818390276)：包括 EXIF 方向處理相關 Plugin 編譯、低信心重試與第二次太差時不覆蓋原文的測試。
- [Android 0.1.4 ARM64 建置 PASS](https://github.com/NFF98/CKM/actions/runs/44818462381)：`aapt` 真實確認 `com.nff98.ckm.dev`、versionCode `1004`、versionName `0.1.4`；source commit `48b5cf9aa0d8f6b71b96819cfa27b213a27cf0ab`。
- 本次簽署的使用者版本：`CAKAMI-v0.1.4-JLPT-OCR-UPDATE-ARM64.apk`，169,398,260 bytes，APK SHA256 `0f1627378d54905bacca55bdcfb53aaa63c24f383433a1f16ad7aef8e3bb31f3`，存放於使用者當前 ChatGPT 對話附件。
- APK 簽章 SHA256 `a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5` 與 CLEAN 0.1.2 完全相同；Android SDK 35 apksigner v2/v3 驗證 PASS。
- RAW APK 和簽署版 AndroidManifest 完全一致，961 個非 META-INF ZIP assets CRC 與長度一樣。這份 APK **可以直接升級 CLEAN 固定簽章 CAKAMI**，不需重新下載完整 E2B；若舊手機仍安裝其他簽章則無法保證直接覆蓋。
- **真 Android 使用者「JLPT→三個可操作選項」與「OCR 40% 下降/提高」 E2E：NOT_RUN**；模型/隱私資料是否保留仍需手機實測。
- 初版 v0.1.3 OCR JUnit 測出低信心覆寫缺陷，不應提供用戶驗收；v0.1.4 已增加最低 0.45 confidence 門檻並通過新測試。
