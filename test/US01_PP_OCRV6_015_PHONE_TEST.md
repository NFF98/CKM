# CAKAMI v0.1.5｜Google ML Kit → PP-OCRv6 Small 離線替換驗收

**決策日期：2026-11-21。狀態：Android SDK 接入／Build 檢查中；真機準確率 NOT_RUN。**

## 選型

- 使用者真機回報：原 Google ML Kit Japanese 文字辨識估計只有 **40% 準確率**，已要求直接替換。
- **選擇：PaddleOCR PP-OCRv6 Small ONNX**（不是聲稱世界最高準確率的 Medium）；官方支援日文與 Android SDK，與 ONNX Runtime + OpenCV 一同在手機端運行。
- 官方部署說明：<https://www.paddleocr.ai/latest/en/version3.x/inference_deployment/cross_platform/android_deployment.html>。原始碼 Apache-2.0，來源固定：PaddlePaddle/PaddleOCR commit `dab3fe35379033fdcb2d0e9572fac0b36c9a9ebf`，26 個上游 Kotlin 原始檔及版權標頭原樣保留。
- 官方模型：<https://huggingface.co/PaddlePaddle/PP-OCRv6_small_det_onnx>（~10 MB）及 <https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx>（~21 MB）。建置時放進 APK assets；**使用者手機不必另外下載 OCR 模型，也不會重抓 E2B（2.59 GB）**。
- 版本：`0.1.5`，Android versionCode `1005`，既有 application ID `com.nff98.ckm.dev`；僅能用 CLEAN 0.1.0–0.1.4 **同一把簽章**覆蓋更新，否則不可交付。
- 新辨識證據：Rust/SQLite 只接收 `engine=paddle_ocrv6_small_onnx`, `engineVersion=6.0.0` 或讀取舊 `mlkit_japanese_bundled:16.0.1` 紀錄；不能讓模型的文字成為已確認 Goal。
- 原 Android App 及私人 SQLite 不刪除，原圖仍留在 Android app-private；只更換 OCR 辨識插件，Rust Bridge 與收藏頁正常。
- 官方 v6 SDK 目前需要 Android 8（API 26）以上，新引擎對更舊裝置**明確錯誤、不做雲端 fallback**。最低 App 支援版本不因此默默提高。
- 注意：不同測試資料、環境與語言的單獨 benchmark 不可直接等同用戶照片的準確率；**不能在真機驗證前保證 90% 或其他數字**。

## 實機驗收（正式測試）

1. 不解除安裝原 CLEAN CAKAMI，直接安裝**固定簽章** v0.1.5 APK。確認 E2B 已準備好、私人收藏還在，不發起新模型下載。
2. 在「收藏」重開舊照片；點「1 內容 → 用手機辨識日文」讓新 PaddleOCR 重新辨識；過去存的舊版 OCR 仍可顯示，不自動被抹掉。
3. 以**同一批 10 張失敗的日文相片**（單字頁、印刷文字、橫／直排、反光／模糊）執行，保留標準答案，測算 **CER（字元錯誤率）**，比較舊 40% 與新結果；不能只靠 OCR 非空就判定成功。
4. 測試文字按「保存人工修正」，重新開啟 App 與飛航模式後確認文字持久化；「JLPT」這類常見關鍵字仍有離線推薦。
5. **FAIL 條件**：App 崩潰/無法建立 ONNX Session／模型缺失／照片送網路／E2B 再下載／私人收藏消失／主要日文樣本比舊版更差。
6. 若 PP-OCRv6 Small 的日文 CER 仍不達標，先依相同照片定位：小字、旋轉、縱書、手寫、混排，再決定更大模型或專用 OCR；不盲目換大模型。

## GitHub CI 與 APK 交付門檻

- Android Kotlin 原生編譯＋單元測試、Rust/TypeScript、APK 打包全部 PASS。
- 確認 APK assets 含 det / rec ONNX + rec YAML，並記錄文件大小和 SHA-256。
- 確認 `aapt` 原 package、versionCode `1005`；用 CLEAN 固定私鑰**離線重簽**並驗證 v2/v3、cert SHA256 =
`a1ec5464e7f66dcd20a2b0d65f77db2c63c2b0aa3e72fe04479878c54f2eedf5`。
- 真機結果在使用者驗證前都應標 `NOT_RUN`，不准把 APK 建置通過當作準確率 PASS。
