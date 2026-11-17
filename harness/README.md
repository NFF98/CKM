# CKM｜單一 ChatGPT Coding Agent Harness

> 參考 [CursorBuildMachine-Template](https://github.com/NFF98/CursorBuildMachine-Template)，但此版為 **CKM 簡化模式**：保留會提高品質的 CI，不引入治理死鎖。使用者已於 2026-11-17 批准 PR #4 並指定「CI 不可以卡住 Coding」。

## 誰負責什麼？

- **使用者**：產品方向、重要的資料／安全行為決策、MVP 驗收及正式發布批准。
- **ChatGPT**：唯一規劃＋Coding＋測試＋除錯＋PR 說明 Agent；同一個 Agent 的自我 Code Review 不是獨立審查。
- **GitHub Actions**：執行能重現的測試及基本安全檢查；**不擔任產品批准機構**。
- **不需要 Cursor**，不引入其 lease、handoff、角色狀態機。

## 不阻擋開發（治理只提醒）

- `implementation_enabled=true`：使用者已授權進入 Coding；`current_task=null` 代表可直接按 Bible B01–B21 工作，不必另開 Activation PR。
- B-ID Manifest 用來說明目標和驗收，不用依賴 Task status 才允許寫程式。
- 不因缺少 Sprint、Task 宣告、PR 標題、不同目錄修改、或文件與程式同 PR 而讓 CI FAIL。
- 仍保留「Bible 優先、不要自行發明產品規則」的實際工作紀律；有影響隱私／安全的產品選擇才回報人類決定。

## 何時 CI 會 FAIL？

- **Phase 1 禁止雲端 AI** 是已確認原則：若 Policy 允許 Cloud AI 或源碼中直接引用已知 Cloud AI Endpoint／SDK，安全檢查 FAIL。這只是基本靜態檢查，不能涵蓋所有間接呼叫。
- GitHub Action 中的 Harness 自動測試真的出錯；未來新增的 TypeScript/Rust 編譯或真正功能測試有實際 FAIL。
- 缺少基本 GitHub Bible／Agent 說明等明確配置損壞，而非形式上尚未完成治理流程。

**不會把「未執行真機測試」當成程式碼 PR 的 CI 失敗**。未跑的 Android／iOS 原生測試、OCR／E2B 實測、Supabase RLS 等，明確顯示 `NOT_RUN`。但在發布 MVP 時，真正需要的 P0 測試必須完成，不能假稱 PASS。

## iOS 可移植性

Day 1 必須讓 TypeScript、Rust 與配對規則盡量共用；Android/iOS 的相機、相簿、OCR、E2B runtime 用平台 Adapter 區隔。**不能把 Android 專用實作當成 iOS 已通過測試**。

## 如何執行

```sh
node --test harness/tests/ckm-gate.test.mjs
node harness/scripts/ckm-gate.mjs
```

PR／main push 會跑相同檢查。沒有 GitHub Ruleset 保護的地方，CI 不等於伺服器端的強制禁止合併；是否必須通過哪些檢查才可合併由使用者自行設定。

這不是完整 CursorBuildMachine 模板遷移，也沒有宣稱通過它的所有 Attack／Release Gate。
