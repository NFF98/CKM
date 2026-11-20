# CAKAMI｜E2B 使用者意圖建議不應被阻擋｜手機驗收

2026-11-21。US01-E2B-04：僅改提示詞、Rust 私人候選驗證、GUI 主顯示；**不換模型、不重新下載 2.59 GB E2B**。

## 手機驗收（離線）

1. **不要卸載 CAKAMI**。用與現有 CLEAN 版本相同 App ID／簽章的更新 APK 覆蓋。
2. 新增文字 **JLPT** → 儲存 →「處理內容 → 2 AI 理解」：必須在主畫面看到按鈕「請 E2B 推薦 2～3 個可能方向」，不在進階摺疊裡。
3. 點按鈕 → 藍色「系統執行」→ 顯示「我理解的內容」及 E2B 的 2～3 個假設（例如考試級別、考試準備、找練習夥伴），**但內容由模型實際產出，不能偽裝成固定選項**。每個選項都可帶入下一頁「3 我的目標」編輯；未經本人確認不建立 Goal。
4. 固定選項必須標記「不是 AI 推測」；有 AI 候選時預設收起，但仍可展開。沒有候選則固定選項保持可見。整頁不應把 AI 推論藏在進階，也不應同時堆兩套內容。
5. 以 **パクる、JLPT、JLPT N2、日文廣告、汽車照片 OCR、服裝設計** 等主題測試：模型不能被硬限制成只有找人合作的目標。使用者可只收藏／修改，而不是被強迫接納模型猜想。
6. 飛航模式執行、完全關閉／開啟 App：E2B 模型及私人內容仍保留，不重新下載 2.59 GB。真正的 AI 回覆可持久化。
7. 模擬惡意候選：證據片段不存在於原文、超出 3 個、模型回傳 consent/visibility/confirmed 欄位，Rust **不得存入**；AI 不能越權建立本人確認事件或公開資料。

## 程式驗收

- E2B `maxOutputToken` 512，短 JSON schema；最多 3 個有效來源候選，允許只有主題而未包含「我想找人」。
- Rust 保留來源字串相符與唯一枚舉欄位檢查；私人候選的狀態只可為 `suggested_private`；沒有本人確認事件，`GoalStore.load` 必須是 `None`。
- CI：TypeScript 型別及 Vitest、Rust 單元測試 PASS；Native Android Kotlin 編譯 PASS；再交付有同簽章的 Android ARM64 APK，**不是 CI 隨機簽章 RAW APK**。
- **品質評分**：JLPT 回答真的具有可選擇的用途、正確理解原文、避免自信胡說；若答案仍糟糕，才依同一批測項考慮換模型。
- 手機測試尚未跑，實機品質狀態 **NOT_RUN**。即使 CI PASS 也不能說 E2B 準確率 PASS。


## 2026-11-21 建置／固定簽章交付證據

- [TS/Vitest、Rust 測試 PASS](https://github.com/NFF98/CKM/actions/runs/44826867063)：Rust 包含 `JLPT` 回傳三個私人候選但不建立 Goal；TypeScript 編譯成功。
- [Android 原生 Kotlin 編譯／Unit tests PASS](https://github.com/NFF98/CKM/actions/runs/44826867046)。
- [Android ARM64 v0.1.6 APK 建置 PASS](https://github.com/NFF98/CKM/actions/runs/44826888807)：`com.nff98.ckm.dev`、`versionCode 1006`、`versionName 0.1.6` 經 GitHub Actions Android SDK `aapt` 直接確認。
- 最終私有 ChatGPT 安裝附件：`CAKAMI-v0.1.6-E2B-OUTPUT-FIX-ARM64.apk`，大小 **169,398,260 bytes**，SHA-256 `1999e359d6d5aeb10933014b6b8523a70bdde19b4499add95074097e15754a26`。
- 已用與 CLEAN v0.1.4 **同一把固定私鑰**離線簽署；APK v2/v3 皆 PASS；憑證 SHA256 `a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5`。
- 簽署前後 **961 個非 META-INF ZIP 條目 CRC／長度完全一致、AndroidManifest.xml 二進位完全一致**。模型私有檔 `gemma-4-E2B-it.litertlm` 未打入 APK，沒有更換 E2B 格式／下載來源。
- **手機實際安裝、E2B 模型保留、AI 產出內容品質：NOT_RUN**；確認結果前 PR 保持 Draft，絕不將 Kotlin/Rust/APK PASS 冒稱真人效果 PASS。
- **注意**：本 APK 源自原本正常的 `v0.1.4`，**不包含**尚未驗收的 PP-OCRv6 替換工作；目的是只測 E2B 改造，避免把兩種缺陷混在一起。
