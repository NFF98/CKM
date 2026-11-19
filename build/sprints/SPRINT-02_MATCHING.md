# Sprint 2｜US-02 找到合適真人 + US-03 真正合作完成（使用 B08／B10–B18）

**目的**：從手機端確認分享條件之後，能在 Supabase 找到**真正的另一個已授權使用者**，完成邀請、雙方接受、文字練習與兩方確認結果。這個 Sprint 不使用任何雲端 AI；Supabase 只做帳號、資料驗證、條件查詢及聊天。

- **範圍**：跨 Sprint 1 的 B08／B07 權限／撤銷契約，以及 B10–B18（全部 P0）；BXX 保留模組驗收，但以 US-02／US-03 真 Cloud 使用者流程交付。
- **狀態**：PLANNED；開始前不需要額外 Sprint Activation PR。
- **依賴**：B10 可與 Sprint 1 獨立模組並行設計；B11 需 B06、B10；B12 需 B08、B11；B13–B18 依照 [Bible](../BACKLOG.md) 排序；任何依賴未通過，不可宣稱下游端到端已驗證。
- **Demo**：A/B 兩個**真實測試帳號**，接到 US-01 真正確認的 N2 Intent → 本人同意分享 → 真 Cloud 同目標／互補可行配對（US-02）→ 雙方邀請接受 → Chat → 15 分鐘 → 兩方各確認 → Server 只有 1 筆可信 COMPLETED → 本人合法結果回手機（US-03；OutcomeReceipt 新契約需批准）。
- **不假設**「建好資料表」就是有真人匹配；可使用受控、已同意的**合成測試帳號資料**，不得宣稱有人真的在使用產品。


## 0. Sprint 2 的兩條 User Story（優先於逐個 BXX Done）

**US-02：『我確認想找 N2 學伴，CAKAMI 經我同意後，幫我找到真的合適的人。』**

| Slice | 一次走完的真人操作 | 需要一起接通 | PASS Gate |
|---|---|---|---|
| **US-02A** | US-01 的已確認私有 Intent → 顯示實際將分享的欄位 → 本人授權 → Cloud 收到且 Server 批准 **最少合法摘要** | B07/B08 + B10/B11/B12 | 雙帳號／Server API 真的運行，未同意時 Cloud 無紀錄；過期、撤銷、偽造版本、8-slot 越權被拒；無雲端 AI |
| **US-02B** | A 查詢已同意的真人 B → 能看到具體相容理由；0 人顯示真空狀態；只有真正必要時才做 Pair-only 追問 | B12→B13→B14→B15，B09 問題契約 | 有效 Need↔Offer／共同時間才候選，封鎖／到期／硬條件衝突必被排除；Pair-only ≤2 題，結果實際更新 |

**US-03：『我們真的合作完成，CAKAMI 記住可信的成果。』**

| Slice | 一次走完的真人操作 | 需要一起接通 | PASS Gate |
|---|---|---|---|
| **US-03A** | A 邀請 B，B 接受 → 才開 Chat；拒絕／封鎖／過期不能進入 | B14/B15→B16→B17 的權限入口 | A/B/C 多帳號確認 Server 安全；重播、雙擊、越權不會生成非法 Session |
| **US-03B** | 雙方聊天並開始 15 分鐘練習 → 各自確認 → Server 記錄**唯一真正完成**的 Outcome | B17→B18 | ≥15 分鐘及雙方確認（依待核准計時規則），少於 15 分鐘／單方確認／No-show／取消不可算 COMPLETED |
| **US-03C** | 本人手機從可信 Server 取回自己有權看到的合作結果，重新打開 App 仍可查；供下一次本地判斷參考 | B18→本地 SQLite（OutcomeReceipt 設計草案；不得擅自新增未核准公開欄位） | Server 權限／版本／重試／離線＋本機資料驗證；不可拿對方私人歷史或未授權 Outcome；跨使用者聚合另需新 Consent 決策 |

**US-02 要真的找到合法候選才 PASS；US-03 要真的雙方完成且可信結果回存才 PASS。** B11 資料表完成、B13 SQL 回 1 筆、B17 Chat 畫面能開都不等於故事完成。原有 S2-AC10～18 是保留的**模組檢查表**，不是另一套驗收主線。

## Coding 前應讀的來源規格

這份 Sprint **不是新的配對演算法／Server Contract**。B10–B18 的依據為 [Bible](../BACKLOG.md)＋[逐項追溯表](../TRACEABILITY.md)：
[Functional Spec](../../design/FUNCTIONAL_SPEC.md) S03–S09／S04a、
[Technical Spec](../../design/TECHNICAL_SPEC.md) §4–5（Consent／RLS／Matching／Session）、
[GUI Spec](../../design/GUI_UX_SPEC.md) S04／S04a／S05–S09、
[Acceptance Spec](../../test/ACCEPTANCE_SPEC.md) B10–B18／T07–T13／E2E-01、03–05。
**正式 E2E-01～05 的名稱與內容只能由 Acceptance Spec 定義；本 Sprint 自訂的開發劇本一律改稱 S2-FLOW。**

## 1. B10–B18 的可操作驗收

| AC-ID／B-ID | 要做出來的功能 | 具體 PASS／必須拒絕的情況 | Evidence |
|---|---|---|---|
| **S2-AC10／B10** | 登入／登出、18+ 使用條件、基本帳號權限、封鎖與檢舉入口 | 可用兩個隔離帳號登入；登出後不可讀私有資料；未符合**實際採用**的成年確認規則者不得發布／聊天；封鎖與檢舉可操作且不暴露檢舉者資料 | 真 Supabase Auth 測試、兩帳號操作、身份權限 Test |
| **S2-AC11／B11** | PostgreSQL Schema、RLS、Server 端 Payload／狀態驗證、邀請／聊天／成果資料表 | A **不可**讀／寫 B 的私人 Intent、聊天及授權；匿名者無法繞過；Client 傳入偽造 owner、過期 consent、舊 version、超過 8 概念均拒絕；Client `is_confirmed=true` 不被當作 Server 可信真相 | Migration、RLS／RPC Tests（含非法直接 API）／T08/T10 |
| **S2-AC12／B12** | 使用者同意後才同步必要配對資料，撤銷及到期下架 | 未同意時 Cloud 沒有公開紀錄；同意後只有許可欄位可見；撤銷／到期候選立即從**伺服器查詢**消失；離線撤銷即本地排除＋排隊，連線重試直到 Server ACK，UI 不提前顯示成功 | A/B Cloud 狀態、網路 Payload Trace、Outbox／Retry、T07 |
| **S2-AC13／B13** | Same Goal＋Need↔Offer 候選搜尋，硬條件先篩選 | 相同 N2 Goal、需求／可提供互補、可練時間相容才有候選；**時間衝突、過期、撤銷、封鎖、scope 不符**，即使語意向量極相近也不得出現 | 合成候選池、可重現 Ground Truth、API/RLS Tests |
| **S2-AC14／B14** | S04／S05 候選卡、清楚的匹配理由、S04a 無人分支 | 真正 1 名合法候選會顯示簡單、可驗證的匹配理由；0 人顯示「目前沒有符合條件的人」＋返回／修改條件／重新搜尋；不捏造真人或成功率 | 雙帳號 E2E、零候選 Fixture、UI 狀態截圖 |
| **S2-AC15／B15** | Pair 專屬缺失問題及重排 | Pair-only 最多**2 題**；只問可改變配對判斷的問題；回答只影響指定 Pair、限期有效，不偷寫進 Profile；超時或已封鎖不得沿用答案 | B09 Budget＋Pair Scope Test、T13 |
| **S2-AC16／B16** | 邀請、接受／拒絕／取消／失效、封鎖與檢舉 | A 邀 B，只有**雙方有效接受**可產生 Chat 權限；拒絕／取消／封鎖／失效後不能開 Chat；重複點擊不會產生兩個 Session；邀請 TTL 待產品決定，不能自行把提案 24 小時冒充既定規則 | Multi-user API/E2E、Race/Idempotency、非法寫入 Test |
| **S2-AC17／B17** | 安全的文字聊天、練習 Session、15 分鐘計時與頻率限制 | 只有雙方已同意才能互發訊息；第三人／未同意者被 RLS 擋下；正常可開始並維持 Session；Server 端時長資料不能由手機改寫；中途斷線和背景行為按**經確認的計時規則**處理 | 雙帳號 Chat／Server Time Test、拒絕第三人、網路錯誤重試 |
| **S2-AC18／B18** | 完成、取消、No-show、雙方結案確認 | 計時不足／A 單方確認／聊天有訊息但無完成，全部**不能**算 COMPLETED；合格 15 分鐘且雙方各確認 → 恰好 1 場成功；重放／連點不會重複計算；取消／No-show 依確認的規則區分 | 雙方 Outcome E2E、Server 狀態機／重試、資料庫核對 |

**S2 同時驗 US-02 和 US-03：前者是真合法候選，後者是雙方可信完成並回存本人 Outcome；兩條故事分別標 PASS/FAIL/NOT_RUN，不以 BXX CI 當產品成功。** 正式市場完成率仍要真實使用者研究，不能靠合成帳號宣稱達標。

## 2. 合成測試帳號／候選池（最小）

| ID | 設定 | 預期 |
|---|---|---|
| USER-A | N2 Goal，Need=文法講解，Offer=閱讀練習，晚上有空 | 可與符合互補條件的 B 匹配 |
| USER-B | N2 Goal，Need=閱讀練習，Offer=文法講解，晚上有空；同意分享 | A/B 可互相找到，雙方同意後才可聊天 |
| USER-C | 同 Goal 但時間完全不重疊 | **不匹配** |
| USER-D | 非同 Goal，向量極高相似或資料曾有效但已過期 | **不匹配** |
| USER-E | 被 A 封鎖，或撤回授權 | **不匹配**、不能互相聊天 |
| ATTACKER | 未登入／登入的非相關第三人，直接呼叫 API | 不能查到 A/B 私人資料、Chat、未授權 Consent |

所有帳號均為測試用；如需要驗證真 Email 發送，先準備可合法使用的測試信箱，不把帳號密鑰提交 Public Repo。

## 3. 必須實際執行的完整操作劇本

**S2-FLOW-01 正常（對應正式 E2E-01）**：A、B 登入與符合成年確認 → A/B 授權發布 → A 查詢找到 B → A 邀請 → B 接受 → 開 Chat → 依 Server 計時練習 → 各自確認 → 查 Server 僅 1 筆 COMPLETED。

**S2-FLOW-02 無候選（對應正式 E2E-03）**：A 的硬條件與所有人衝突 → S04a 出現；不虛構或建議不存在的對象。

**S2-FLOW-03 隱私／攻擊（對應正式 T08、E2E-04／05 的部分負向案例）**：未經同意發佈；跨帳號讀取；A 偽造 B 的確認事件；重播過期邀請；第三人傳訊 → 全部被 Server 拒絕。

**S2-FLOW-04 撤銷／離線（對應正式 T07、E2E-04）**：A 分享後離線撤銷 → 手機立即停止本地搜尋、顯示 Cloud 待撤銷 → 重連 Server ACK → 其他帳號候選再查詢不可見。

**S2-FLOW-05 未達完成（對應正式 E2E-05）**：只有 A 確認、未達 15 分鐘、取消或 No-show → 不計入 COMPLETED。

以上每個 E2E 的設定、時間、帳號資料、HTTP 狀態、Server 端事實、UI 結果均需留下可重現 Evidence；不要只靠 UI 截圖宣稱後端 RLS 安全。

## 4. 不能直接擅自決定的產品選項

在**受影響的 B-ID 開工前**提出最小化決策，不必卡住 B01 或其他無依賴開發：
- **B10**：成年人採何種確認方式、未登入可走到哪一步；自我聲明並非強式驗齡。
- **B08／B12**：用戶逐項確認的可分享欄位、原圖「額外主動分享」是否有 MVP 需求。**禁 Cloud AI 不等於禁資料同步**。
- **B16**：邀請保存多久及過期行為。
- **B17／B18**：15 分鐘如何起算、斷線／背景何時計時、No-show／取消定義。

**不影響使用者或資料安全的工程選擇**（資料表名、Rust Command 名、RPC 命名、Idempotency Key 格式），由 Coding Agent 設計、執行契約測試並在 PR 中記錄，不反覆要求人工微管理。

## 5. Sprint 2 Debug 與完成條件

PR 中註明 `S2-ACxx / Bxx` 與環境：Supabase 專案識別（不得公開秘密）、Migration 版本、RLS policy、Request／Response（去識別）、重現步驟、預期／實際、修正 commit 與測試結果。Server 業務規則失敗不得只靠 Client UI 關閉按鈕修飾；必須在資料／Server 邊界擋下。

**全部完成**＝**US-02A/B、US-03A/B/C 各自在真 Cloud／雙帳號 PASS**，加上 S2-AC10～18 必需模組測試、安全／撤銷／期限／Chat 失敗案例 PASS。任一項尚未驗證，標 FAIL/BLOCKED/NOT_RUN；仍可做不依賴它的功能，但不可假稱 Sprint 2 Done。

接續：[Sprint 3｜測試與發布](SPRINT-03_RELEASE.md)。
