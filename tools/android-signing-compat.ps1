# CAKAMI: safe Android certificate diagnostic / optional APK re-sign on Windows.
# This script NEVER uninstalls an app, clears data, reads private app data,
# or installs an APK. It only reads the installed app's public APK and checks
# whether the owner's local debug keystore matches its signing certificate.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $NewApk,
    [string] $PackageId = "com.nff98.ckm.dev",
    [string] $Keystore = "$env:USERPROFILE\.android\debug.keystore",
    [switch] $Resign
)

$ErrorActionPreference = "Stop"

function Find-Program([string] $Name) {
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($null -ne $command) { return $command.Source }

    if ($Name -eq "apksigner") {
        foreach ($root in @($env:ANDROID_HOME, $env:ANDROID_SDK_ROOT,
                            "$env:LOCALAPPDATA\Android\Sdk")) {
            if (-not $root -or -not (Test-Path $root)) { continue }
            $tools = Get-ChildItem (Join-Path $root "build-tools") -Directory -ErrorAction SilentlyContinue |
                Sort-Object Name -Descending
            foreach ($folder in $tools) {
                $exe = Join-Path $folder.FullName "apksigner.bat"
                if (Test-Path $exe) { return $exe }
            }
        }
    }
    throw "找不到 $Name。請先安裝 Android Platform Tools / Build Tools / Java JDK。"
}

function Apk-Certificate([string] $Signer, [string] $Apk) {
    $output = (& $Signer verify --print-certs $Apk 2>&1) -join [Environment]::NewLine
    if ($LASTEXITCODE -ne 0) { throw "APK 簽章驗證失敗：$Apk; $output" }
    $match = [regex]::Match($output, "Signer #1 certificate SHA-256 digest:\s*([0-9A-Fa-f]{64})")
    if (-not $match.Success) { throw "無法從 APK 取得 SHA-256 簽章指紋：$Apk" }
    return $match.Groups[1].Value.ToLowerInvariant()
}

if (-not (Test-Path -LiteralPath $NewApk -PathType Leaf)) {
    throw "找不到新 APK 檔案：$NewApk（請先解壓 GitHub 下載的 ZIP）"
}
$NewApk = (Resolve-Path -LiteralPath $NewApk).Path
$adb = Find-Program "adb"
$apksigner = Find-Program "apksigner"
$keytool = Find-Program "keytool"

$devices = (& $adb devices) -join [Environment]::NewLine
if ($devices -notmatch "(?m)^[^\r\n\s]+\s+device\s*$") {
    throw "找不到已授權的 Android USB 偵錯裝置，請接上手機並允許 USB 偵錯。"
}

$paths = @(& $adb shell pm path $PackageId)
if ($LASTEXITCODE -ne 0) { throw "無法查詢舊 App：$PackageId" }
$installed = $paths | Where-Object { $_ -match "^package:.*base\.apk\s*$" } |
    Select-Object -First 1
if (-not $installed) {
    $installed = $paths | Where-Object { $_ -match "^package:" } | Select-Object -First 1
}
if (-not $installed) {
    throw "手機目前找不到 $PackageId；可能 App ID 與預期不同。程式不會解除安裝任何東西。"
}

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("cakami-signature-" + [Guid]::NewGuid().ToString("N"))
[System.IO.Directory]::CreateDirectory($tmp) | Out-Null

try {
    $oldApk = Join-Path $tmp "installed-base.apk"
    $remote = ($installed -replace "^package:", "").Trim()
    & $adb pull $remote $oldApk | Out-Null
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $oldApk)) {
        throw "無法讀取已安裝 App 的公開 base APK；沒有變動手機資料。"
    }

    $oldFingerprint = Apk-Certificate $apksigner $oldApk
    $newFingerprint = Apk-Certificate $apksigner $NewApk
    Write-Host "目前手機 App 簽章 SHA-256：$oldFingerprint"
    Write-Host "GitHub 新 APK 簽章 SHA-256：$newFingerprint"

    if ($oldFingerprint -eq $newFingerprint) {
        Write-Host "簽章相同。若仍有衝突，原因不在簽章；請保留資料並回報安裝錯誤。"
        exit 0
    }

    Write-Warning "已確認簽章不同，Android 不允許用新 APK 直接覆蓋且保留舊資料。"

    if (-not (Test-Path -LiteralPath $Keystore -PathType Leaf)) {
        Write-Warning "Windows 找不到原始 debug.keystore：$Keystore"
        Write-Warning "不能從舊 APK 反推私鑰。不要解除安裝、不要清除資料。"
        exit 2
    }

    $keyOutput = (& $keytool -list -v -keystore $Keystore -storepass android -alias androiddebugkey 2>&1) -join [Environment]::NewLine
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "本機 keystore 不是標準 Android Debug Key，或密碼／alias 不同。不可猜測原始金鑰。"
        exit 2
    }
    $keyMatch = [regex]::Match($keyOutput, "SHA256:\s*([A-Fa-f0-9:]{95})")
    if (-not $keyMatch.Success) { throw "無法解析本機 keystore 憑證 SHA256。" }
    $keyFingerprint = $keyMatch.Groups[1].Value.Replace(":", "").ToLowerInvariant()
    Write-Host "本機 Windows Debug Key 指紋：$keyFingerprint"

    if ($keyFingerprint -ne $oldFingerprint) {
        Write-Warning "本機私鑰不是手機已安裝版本的原始簽章，無法用它安全覆蓋。"
        Write-Warning "請保留舊 App；資料備份／平行測試方案詳見 test/ANDROID_SIGNING_RECOVERY.md"
        exit 2
    }

    Write-Host "已找到與舊 App 完全相同的原始 Debug 簽章！"
    if (-not $Resign) {
        Write-Host "加上 -Resign 參數可重新簽名新 APK；不會解除安裝或自動安裝。"
        exit 0
    }

    $name = [System.IO.Path]::GetFileNameWithoutExtension($NewApk)
    $dest = Join-Path ([System.IO.Path]::GetDirectoryName($NewApk)) ("{0}-compatible.apk" -f $name)
    if (Test-Path $dest) { throw "輸出檔已存在，為避免覆寫請先改名：$dest" }

    & $apksigner sign --ks $Keystore --ks-key-alias androiddebugkey --ks-pass pass:android --key-pass pass:android --out $dest $NewApk
    if ($LASTEXITCODE -ne 0) { throw "使用原始 Debug Key 重簽失敗。" }

    $compatible = Apk-Certificate $apksigner $dest
    if ($compatible -ne $oldFingerprint) {
        Remove-Item -LiteralPath $dest -Force
        throw "重簽後指紋仍不同，已刪除不相容的輸出檔。"
    }

    Write-Host "成功產生同簽章 APK：$dest"
    Write-Host ("請先保留舊 App，再執行 adb install -r " + '"' + $dest + '"' + " 進行覆蓋安裝。")
    Write-Host "本腳本從未解除安裝或清除手機資料。"
}
finally {
    if (Test-Path -LiteralPath $tmp) {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}
