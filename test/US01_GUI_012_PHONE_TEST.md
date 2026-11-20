# CAKAMI v0.1.2｜手機四頁 UI 一次驗收

**狀態**：GUI coding ready；真機 UI／E2B 更新待使用者確認。2026-11-20。

## 這次只驗收四件事

1. **首頁／色彩**：畫面頂部只有四個頁籤「新增／收藏／處理內容／裝置」，且有三種附文字說明的色碼：綠＝你操作、藍＝系統執行、橘＝需要輸入。現在操作會顯示文字提示，不能只用色彩判讀。
2. **E2B 首次下載後隱藏**：手機原先已下載完整 Gemma E2B → 正常同簽章覆蓋更新到 v0.1.2 → 新增頁不應再出現 2.59GB 模型下載卡；「裝置 → 查看本機 AI 詳細狀態」仍可檢查模型。**不可重新下載，不可失去原有收藏。**
3. **只有一層流程**：新增頁選圖片或輸入文字 → 自動前往「處理內容」，四步「1 內容 → 2 AI 理解 → 3 我的目標 → 4 私人條件」。一次只顯示所選步驟，不會再把 OCR、AI、Goal、Registry 全部堆在同一頁。可上下步／任意步驟跳轉，從「收藏」可再次開啟。
4. **不重複確認**：「2 AI 理解」選擇「用這個目標」→ 自動前往「3 我的目標」填好候選；**只按本人確認才會儲存私人 Goal**。確認後才有私人條件列表；期限與本地分享審核都藏在「進階管理」摺疊區，原本的私有 SQLite、版本衝突檢查、撤銷和不公開承諾保持不變。

## 一次執行的手機步驟

1. **保留現有 CAKAMI，不要卸載！** 若已裝上一輪 CLEAN v0.1.1 固定憑證 APK，安裝本輪 **v0.1.2 相同 App ID、相同 signing cert** APK 覆蓋。
2. 開啟首頁，確認「新增」無巨大的 E2B 下載區，色碼三個有文字。若模型尚未準備好，才應在新增頁出現首次 E2B 準備區。
3. 到「裝置」確認「本機 AI：已準備好」，點展開才會看到詳細狀態；返回「新增」。
4. 選擇照片，檢查自動切到「處理內容 → 1 內容」，照片縮圖可見、OCR 可辨識／手動修正。
5. 點「下一步：AI 理解」，執行本地 AI；如果原文已有保存的模型建議，直接看到候選。點「用這個目標」進入第 3 步，確認欄位已填寫但未自動確認。
6. 在第 3 步親自確認 Goal（確認 dialog）；確認後第 4 步可以看到私人條件，但不會自動公開或分享。沒有 Goal 時第 4 步應顯示先確認提示。
7. 回「收藏」，確認舊內容還在；點回任一筆，跳轉 1／2／3／4 各步，不能出現內容一次全部展開。
8. 完全關閉 App、重新打開，飛航模式下檢查收藏和 E2B 依然存在，無重複下載。

## 發版前技術驗收

- `npm run check` + Vitest：PASS 才發包。
- Android native Kotlin compile + tests：PASS 才發包。
- Android package `com.nff98.ckm.dev`，version `0.1.2` / code **1002**。
- `apksigner verify` 及公鑰 certificate SHA256 必須等於上一版 CLEAN v0.1.1：
  `a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5`。
- Debug/RAW CI APK 有臨時簽章，**絕對不可提供直接安裝**；必須用固定私鑰重新簽署再提供。

**重要**：只是在 GitHub CI 成功不代表真機已驗收；真機 APK 更新與 UI 驗收維持 `NOT_RUN`，直到使用者回報。

## 2026-11-20：交付完成與證據

- 本輪 UI 原始碼 commit：`65869ef2d258c7191903a27f86424f21048605df`。
- [TypeScript／Vitest／Rust 完整 PASS](https://github.com/NFF98/CKM/actions/runs/44666375969)。
- [Android 原生 Kotlin 編譯／Unit test PASS](https://github.com/NFF98/CKM/actions/runs/44666375972)。
- [Android 0.1.2 ARM64 debug 原始編譯 PASS](https://github.com/NFF98/CKM/actions/runs/44666652984)。由 GitHub Actions `aapt` 真實核對 package `com.nff98.ckm.dev`, versionCode `1002`, versionName `0.1.2`。
- 使用上輪 CLEAN 0.1.1 **完全相同的離線保存私鑰**重新簽署，簽章 SHA-256 = `a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5`。Android SDK 35 `apksigner verify`: v2=true、v3=true。
- 已簽署使用者 APK：`CAKAMI-v0.1.2-UI-UPDATE-ARM64.apk`，169,332,724 bytes，SHA-256 `e4eac8205070fa5ef7e9e3720904e1c9ff5ba40e8f6cfd4d73f205f0a7f4074d`，放在 ChatGPT 對話附件。APK 非 META-INF 961 檔案 checksum/length 與 RAW 原始編譯一致，AndroidManifest binary 完全一致。
- 測試限制：沒有 Android 裝置可做真人操作。**實機四頁切換、E2B 更新不重抓、圖片／Goal UI 人工操作，均仍 NOT_RUN。**
- 版本更新適用於已裝 **固定憑證 CLEAN 0.1.0/0.1.1** 的裝置；若手機仍安裝更早 GitHub 隨機 Debug 憑證版本，Android 本身會拒絕不同簽章的原地更新。絕對不可謊稱可保留那版的私人 E2B。
- 未來 GitHub 自動打包尚需 repo secrets（固定 key 已被個別離線保留）；沒有私鑰的 Runner 產物只供離線重簽，不可直接給使用者安裝。
