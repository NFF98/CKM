# CAKAMI Android 安裝衝突｜保留原 App 資料的修復方式

## 已查明的打包缺陷（2026-11-19）

- Android App ID：\`com.nff98.ckm.dev\`。
- 舊的 \`.github/workflows/b01-android-apk.yml\` 每次在新的 GitHub Runner 執行 \`tauri android init --ci\` 和 \`tauri android build --debug\`，**沒有還原固定的 Debug keystore**。
- Android 預設 Debug 簽章是每台機器的 \`~/.android/debug.keystore\`；因此兩次 GitHub 打包，即使 App ID、版本相同，也可能使用**不同的私鑰**。
- Android 拒絕不同簽章的既有 App 更新。日文錯誤 \`パッケージが既存のパッケージと競合するため、アプリをインストールできませんでした\` 正符合此情況，但仍應用「舊 App 安裝憑證 vs 新 APK 憑證」確認，不能只從日文訊息保證只有此原因。

**切勿先解除安裝或清除資料！** 舊版 Gemma（約 2.59 GB）與私有 SQLite 存在 App 私有目錄，解除安裝可能將其一併刪除。

## 解法 A：原始簽章私鑰仍在你的 Windows（可保留資料、直接覆蓋）

1. 手機開啟「開發人員選項／USB 偵錯」，用 USB 接上 Windows，並安裝 Android Platform Tools（\`adb\`）、Android SDK Build Tools（\`apksigner\`）、Java JDK（\`keytool\`）。
2. 保留手機上舊 CAKAMI，不要解除安裝。
3. 將下載的**新版 APK 檔**（不是 ZIP）解壓至 Windows。
4. 在 Cursor/PowerShell 執行：

\`\`\`powershell
pwsh -File .\tools\android-signing-compat.ps1 -NewApk "C:\Users\<你>\Downloads\app-arm64-debug.apk"
\`\`\`

腳本從手機**只讀**抽出已安裝的 APK，比對其簽章與 Windows \`$env:USERPROFILE\.android\debug.keystore\`。若原本是由這台 Windows 開發、debug keystore 沒變，腳本會輸出「可安全重簽」；若找不到相同私鑰則停止，絕不解除安裝。

5. 如果報告顯示本機私鑰指紋與已安裝 App **相同**，可執行：

\`\`\`powershell
pwsh -File .\tools\android-signing-compat.ps1 -NewApk "C:\Users\<你>\Downloads\app-arm64-debug.apk" -Resign
\`\`\`

這會產生 \`*-compatible.apk\`，它以原本的 Windows Debug keystore 簽名，**不是從 APK 猜出私鑰**。原手機資料仍留在原 App 中。安裝產出的新檔，應可原位覆蓋（版本碼仍需符合 Android 限制）。

## 解法 B：舊 App 由先前 GitHub 臨時憑證簽署／私鑰已遺失

**不能只重打 APK 就恢復舊私鑰**，Android 也不允許沒有原始私鑰的其他簽章直接覆蓋。不能靠等 12 小時、改 APK 檔名、修改版本碼解決憑證不一致。

- 舊 App 若為 **debuggable**：可先用 Android \`adb shell run-as com.nff98.ckm.dev id\` 確認可否讀取自家私有目錄，然後**先做完整本地備份並驗證**，再討論一次性換簽章與資料回復。不能在沒有驗證備份之前解除安裝。
- 若 \`run-as\` 被拒絕，且舊版本沒有資料匯出功能，不能保證保留該版本的私有 DB 與 Gemma。請保留舊 App，不能宣稱有安全的一鍵覆蓋方法。
- 為了只測新 UI，也能另做**不同 App ID** 的並排測試包；舊 CAKAMI 不受影響，但私人資料與已下載模型**不共用**，新 App 需要另下載模型（2.59 GB），不應假裝能直接讀舊模型。

## 永久修正：所有未來的 GitHub Debug APK 固定一把金鑰

此 PR 已修工作流程：

- 未設定 GitHub Repository Actions Secret **\`CKM_DEBUG_KEYSTORE_BASE64\`** 時，拒絕出產新的、會造成衝突的臨時金鑰 APK。
- 設定後每次先還原 \`~/.android/debug.keystore\`，檢查 Debug alias，打包後輸出 APK 憑證 SHA-256 指紋（只輸出公鑰指紋，不輸出私鑰）。
- **第一把固定金鑰一定要與手機目前已裝 App 的憑證相同，才能無損覆蓋！** 如果原私鑰已遺失，先做安全備份／移轉，再使用新的固定金鑰。
- **私鑰檔和 Base64 不得 Commit 到 Public Repo。** GitHub Secret 無法透過本 Chat 自動取得／建立。請自行於 GitHub → NFF98/CKM → Settings → Secrets and variables → Actions → New repository secret 設定。

若已證實 Windows Debug key 與舊 App 相同，Windows 可用：

\`\`\`powershell
$keyFile = Join-Path $env:USERPROFILE ".android\debug.keystore"
if (-not (Test-Path $keyFile)) { throw "debug.keystore 不存在，請勿生成新的金鑰冒充舊金鑰" }
[Convert]::ToBase64String([System.IO.File]::ReadAllBytes($keyFile)) | Set-Clipboard
# 立即貼入 GitHub Secret CKM_DEBUG_KEYSTORE_BASE64；完成後清空剪貼簿
Set-Clipboard -Value ""
\`\`\`

**不要把任何 keystore、密碼、Base64 內容或私人備份貼進 Chat 或公開 GitHub Issue。**

## 驗收標準

- [ ] 手機**原已安裝 App** 和準備安裝 APK 的 SHA-256 簽章一致
- [ ] 完成 \`adb install -r new.apk\` 或 Android 檔案安裝覆蓋，沒有安裝衝突
- [ ] 舊 SQLite 私有收藏仍在，原有 Gemma 模型不需重新下載
- [ ] 更新後原手機飛航模式可操作 B01–B09 流程
- [ ] 往後兩次 GitHub APK 的簽章指紋一致

本項是 Android 發版／簽章阻斷；即使 Rust／TypeScript CI PASS，手機安裝 FAIL 就不得把 Sprint 1 Android 驗收標 PASS。
