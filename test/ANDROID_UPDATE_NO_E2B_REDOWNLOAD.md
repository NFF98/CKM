# CAKAMI 0.1.1｜手機直接覆蓋更新，不重新下載 E2B

日期：2026-11-20。GitHub SSOT：Sprint 1 Android 發版補救驗收。

## 已修正項目

- 原 Android 包名維持 `com.nff98.ckm.dev`，非 `.s1trial` 獨立測試包。
- 版本從 `0.1.0` 升到 `0.1.1`（Android versionCode 必須從 1000 上升）。
- 使用者先前拿到的固定簽章 CLEAN APK 憑證 SHA256 為：
  `a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5`。
- **新 APK 一定要用完全相同的私人簽章金鑰重簽**，Android 才允許在原位更新且保留 `files/models/`、SQLite 與 App 私有內容。
- E2B 啟動時現在先由 `ModelReusePolicy` 看 `files/models/gemma-4-E2B-it.litertlm` 及 `.sha256` 標記：
  - 模型長度 2,588,147,712 bytes 且標記相符：**直接本地使用，不發 HTTP、不下載**。
  - 模型檔案存在、但標記遺失或不符：**只在裝置端驗證 SHA-256，驗證成功修復標記；不發 HTTP**。
  - 模型檔案存在、但毀損／大小錯誤：**停止並顯示錯誤；不自行覆寫或重下載**。
  - 只有模型檔案真的不存在，才依照原本 Wi-Fi ／使用者同意的行動網路設定進行首次下載／續傳。
- 新增 Kotlin 單元測試驗證以上三條保護策略。

## 手機驗收（僅需一次）

**A. 手機已經安裝固定簽章的 CAKAMI CLEAN 0.1.0，且 E2B 已下載完成：**

1. **不要解除安裝、不要清除 App 資料。** 用新版 0.1.1 APK 直接更新。
2. 開啟 CAKAMI → 裝置端 Gemma 4 E2B：應顯示模型已安裝，**不應出現 0 / 2,588 MB 下載進度**。
3. 開飛航模式，再關閉／開啟 App：E2B 應仍顯示已安裝，且原私人收藏仍存在。
4. 如果覆蓋更新被 Android 拒絕，**停止，不要解除安裝或清資料**：驗證 App 包名、簽章憑證和版本碼。

**B. 手機還裝著舊 GitHub 隨機簽章 CAKAMI，沒裝固定簽章 CLEAN 0.1.0：**

- 舊隨機簽章 APK 不可能直接升級成固定簽章 APK 並保留私有資料。
- 你已選擇一次性乾淨重裝的話，先確認不需保存舊私人收藏；再解除安裝舊 App 和 `CAKAMI TEST`（如有），安裝固定簽章 0.1.1；E2B 只需在這個固定簽章 App **首次下載一次**。
- 後續 0.1.2+ 仍必須使用完全相同簽章，屆時按 A 方式直接覆蓋，不需再次下載 E2B。

## 發版阻擋條件

- APK 原包名不是 `com.nff98.ckm.dev`：FAIL。
- APK 0.1.1 版本碼不大於 1000：FAIL。
- APK 簽章 SHA256 與既有 CLEAN 0.1.0 不同：FAIL。
- APK 非 v2/v3 簽章或驗證失敗：FAIL。
- 對已安裝、有效的 E2B 發起自動 HTTP 下載：FAIL。
- Android 直接覆蓋安裝後私人收藏或 E2B 消失：FAIL。
- 尚無使用者手機實測 evidence，不能把真機驗收寫成 PASS。

GitHub Actions 固定簽章 pipeline 的私密 GitHub Secrets 尚需配置；不得為了通過 CI 把 keystore／密碼提交公開 Repo。這次由離線保存的同一把簽章金鑰輸出成品。
