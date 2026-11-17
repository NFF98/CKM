# CKM｜單一 AI Coding Agent Harness（草案）

> 來源參考：[CursorBuildMachine-Template](https://github.com/NFF98/CursorBuildMachine-Template)。本版本只抽取 CKM 第一階段必要的自動治理概念，**不是**宣稱整套母版已完整遷移或通過 Attack E2E。

## 角色：沒有 Cursor

- **Human（使用者）**：唯一 Product Decision、Build Activation、Spec 變更、PR Merge、Release Approval 權限。
- **ChatGPT（唯一執行 AI）**：Planner + Coding + Tests + Debug + Code Review 建議 + Evidence。Review 由同一個 AI 做，**不宣稱獨立第三方審核**。
- **Harness（GitHub CI）**：可重現的規則檢查，至少防止 HOLD 寫入、跨 B-ID 路徑漂移、同 PR 改產品規格、明顯的雲端 AI API 依賴。
- 不使用 Cursor lease、Cursor Skill actor、Cursor handoff、Cloudflare Release Adapter。Android 真機 E2E 與 Supabase RLS 驗證必須另建真正的 Test Gate，不可拿 Web 的 Playwright 模擬 Android 原生功能驗收。

## 目前狀態

\`harness/policy/ckm-policy.json\`：\`implementation_enabled=false\`、\`current_task=null\`。  
\`harness/tasks/B01.json\`：\`PLANNED\`，**不是 ACTIVE**。  
此 PR 合併前不會變更 main；即使合併也不能自動開始 B01 Product Code。

## 啟動流程

1. 先完成 Harness PR 的 Human 審核，確認最小治理方法可接受。
2. 單獨 PR 啟動 B01：在 policy 設 \`implementation_enabled=true\`、\`current_task="B01"\`，並把 B01 Manifest 標為 \`ACTIVE\`。**只能在 Human 核准下操作**；不能由 ChatGPT 自稱自己批准。
3. 啟動後，在獨立 [B01] PR 只修改 Manifest 許可路徑；使用既有 Bible 和 B01 驗收條件。所有程式輸出、測試、可重現日誌必須放在 PR。
4. 每個 B-ID 只在真實 Test Evidence PASS 後標完成。未通過 Android 真機驗收即 BLOCKED／NOT_RUN。
5. 後續 B02–B21 只增補任務清單與需要的測試 Gate；不必為單一創辦人複製母版全部管理層。

## 使用方法

從 CKM Repository 根目錄執行：

~~~sh
node harness/scripts/ckm-gate.mjs
~~~

GitHub PR 自動執行同一腳本；如果本機有該 PR base SHA，可：

~~~sh
CKM_BASE_SHA=<40位SHA> CKM_PR_TITLE='[B01] Android 基礎' node harness/scripts/ckm-gate.mjs
~~~

> **保證範圍**：只檢查列出的直接違規模式，不可能靠 regex 證明完全沒有 Cloud AI 網路請求；日後須搭配依賴清單、網路觀察及 Android 斷網模型測試。
> **GitHub 分支保護／required status check**：目前未設定，CI 有執行不代表未經核准的 Merge／Push 在 GitHub 層已被阻擋。
