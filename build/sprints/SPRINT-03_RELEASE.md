# Sprint 3｜完整測試、修 Bug 與 Android Pilot（B19–B21）

**目的**：把 Sprint 1、2 真正接起來，驗證 P0 安全、隱私、真機本地推論、雙方完成與發布。**程式碼 PR 的 CI 不使用治理手續卡住 Coding，但發布測試不能因此跳過。**

- **範圍**：**B19=P0** 完整測試；**B20=P1** 配對公平比較；**B21=P0** Android 測試發布。
- **狀態**：PLANNED；所有 P0 預設 NOT_RUN，沒有實際執行證據不得標 PASS。
- **發布依賴**：Bible 只要求 **B21 依賴 B19；參考 B20**。B20 可稍後完成、不可升格成發布硬門檻。
- **不誤導平台支援**：iOS 程式從 Day 1 保持可移植；本階段交付是 Android Pilot，不是已通過 iOS App／TestFlight。


## Coding 前必讀的唯一正式測試定義

先核對 [Acceptance Spec](../../test/ACCEPTANCE_SPEC.md) 第 2～5 節（**T01–T13／GUI-01～10／E2E-01～05**）和 [B19–B21 追溯表](../TRACEABILITY.md)，再依 [Functional Spec](../../design/FUNCTIONAL_SPEC.md)、[Technical Spec](../../design/TECHNICAL_SPEC.md)、[GUI Spec](../../design/GUI_UX_SPEC.md) 的實際契約做端到端測試。**下面是摘要，不能與 Acceptance Spec 產生另一組衝突的正式 E2E ID。**

## 1. B19–B21 的可操作驗收

| AC-ID／B-ID | 工作 | 必須 PASS 的實測條件 | Evidence |
|---|---|---|---|
| **S3-AC19／B19（P0）** | Rust／SQLite、Supabase RLS／Server Rules、Android E2E、全部 P0 Regression | v1.7 **T01–T13** 全 PASS；雲端越權、8-slot 並發、離線撤銷、期限／封鎖、聊天／完成完整跑過；No Cloud AI 網路審查＋斷網真機 OCR／E2B；失敗案例都有修正及重測 | 真執行命令、log、device／runtime／test account 環境、可重現 Fixture、PR／Commit |
| **S3-AC20／B20（P1）** | 公平配對 Benchmark：固定表單、單 Goal 向量、多向量＋Constraints、CAKAMI 8-slot＋Dynamic Questions | **相同候選池／Goal／時間／硬條件**比較每 100 個有效 Intent 的真正完成數、問題量、硬條件違規、延遲與成本；不足樣本要標示限制、不得編造效果 | 版本固定的合成測試池、方法、統計表、重現指令；**可以 NOT_RUN 且不阻礙 B21** |
| **S3-AC21／B21（P0）** | Android Pilot 打包、安裝、啟動、安全與發布／回滾說明 | **可安裝且實際跑得動的 APK**；真機 S01–S09＋S04a 主線能重現；B19 P0 全 PASS；有模型啟動時間／記憶體／耗電證據、權限／隱私／風險紀錄，以及回滾／停止測試版的明確辦法 | APK Hash／版本、測試手機型號、Android 版本、安裝與離線步驟、裝置測試紀錄、Release Notes |

Sprint 3 不表示測試可以到最後一天才開始。**S1／S2 每個 B-ID 的自動化測試應隨程式一起寫**；Sprint 3 執行跨模組 P0 回歸、補齊未通過的項目，並留下正式的 Release Evidence。

## 2. B19 必測：System v1.7 的 T01–T13（P0）

| 測試 | 注入的情況 | 正確結果 | 執行場所 |
|---|---|---|---|
| **T01** | E2B 提案冒用 `CONFIRMED`／偽造 User Event | Rust 拒絕，不能跳過本人確認 | Rust |
| **T02** | 缺來源／scope／expiry_policy | Rust 拒絕，直接 SQLite 寫入亦失敗 | Rust＋SQLite |
| **T03** | 一次性週六有空，日期缺失或已有日期 | 不捏造日期、不升成每週 recurring | Rust＋SQLite |
| **T04** | N2／JLPT N2 同義概念與多值 | 同義 concept 歸一；合法不同值保留 | Registry＋SQLite |
| **T05** | Drop 餐廳廣告，Goal 未確認 | 只保存本地資料，不能生成已發布 Intent | Android＋Rust |
| **T06** | Intent 到期或已取消 | 不進入任何有效候選／配對結果 | Rust＋Server |
| **T07** | 離線撤銷、同步失敗、重連 | 本地立即失效，Outbox 重試直到 Cloud ACK；UI 顯示尚未同步 | Android＋Supabase |
| **T08** | 他人帳號、舊版本或已撤銷 Payload 呼叫 API | Server 拒絕越權／失效／Version 衝突 | Supabase Auth／RLS／RPC |
| **T09** | 連續 Drop 100 張 JLPT 截圖 | 可以儲存內容，不產生 100 個 Matching Slots／Schema 變化 | Android＋SQLite |
| **T10** | 第 9 個概念、並發操作、直接 DB 與非法 Cloud Payload | Rust、SQLite、Cloud 都拒絕；不超過 slot 0–7 | Rust＋SQLite＋Supabase |
| **T11** | N2 同義＋大量「パクる」內容 | 合法概念重用 slot；私人單字不佔 Matching Slot | OCR／Registry／SQLite |
| **T12** | 8 slots 滿，第 9 個是必要安全硬條件 | BLOCKED_BY_BUDGET；阻擋 publish／propose，不刪既有硬條件 | Rust＋Android＋Server |
| **T13** | 重複主動提問、不同 Pair、過期答案 | 啟用／嘗試 ≤3（含 Goal）、Pair-only ≤2、每日未請求 ≤3；Pair 答案不升 Profile | Rust＋Session State |

這些是 v1.7 原始 Test ID，不重新編號，也不因先完成畫面就省略直接 SQL、Server API 或並發攻擊測試。

## 3. Phase 1 NO CLOUD AI：另有三項實測

| Test | 測試方式 | PASS 條件 |
|---|---|---|
| **AI-LOCAL-01** | 手機已配置好本地模型後，飛航模式 Drop 合成 N2 圖片並執行 OCR／E2B | 真正本地模型有結果；記錄 model hash／版本、runtime、延遲／記憶體；**不是假資料** |
| **AI-LOCAL-02** | 模型檔壞掉、加載超時／推論失敗；檢查 Client 網路與 Server 紀錄 | 明示模型錯誤、可人工修正；**完全沒有 fallback Cloud AI OCR／LLM／Embedding** |
| **AI-LOCAL-03** | 檢查 Supabase／RPC／Edge Function、Mobile Upload Payload | 只傳使用者授權的必要資料；伺服器只做非 AI 的 DB／比對／聊天；若傳向量必須先取得 Consent |

Harness 靜態 Cloud AI 關鍵字檢查只是快速防呆；不能代替三項實測。

## 4. 端到端使用者劇本（全部 P0）

| E2E-ID | 手機上怎麼做 | 必須看到什麼 |
|---|---|---|
| **E2E-01 正常** | Drop N2 圖 → 本地辨識 → 本人確認 Goal → 授權 → 真 A/B 配對 → 邀請 → Chat → 15 分鐘 → 雙方確認 | Server 僅有合法已完成 Session；沒有私有資料洩漏 |
| **E2E-02 只存單字** | Drop「パクる」，選擇只收藏、不找學伴 | 私人內容保存，無發佈／公開 Intent，不因單字增加大量 Matching Slots |
| **E2E-03 無候選** | 設定沒有任何合法相容候選的 Goal／Need／Offer | S04a 真的顯示無候選；不虛構真人或成功機率 |
| **E2E-04 拒絕／封鎖／撤銷** | B 拒絕／封鎖 A，A 直接呼叫 Chat API；A 離線撤銷再上網 | 未獲雙方同意不得聊天；本地立即排除、雲端待 ACK，重連後下架 |
| **E2E-05 Completion 負向** | 未滿 15 分鐘、只有單方確認、重複提交、取消、No-show、越權提交 | 不計入 COMPLETED；重試／連點不重複計數 |

**失敗時不能拿假成功資料補滿測試。** 存在產品行為不確定的項目（尤其年齡、分享欄位、邀請 TTL、斷線計時），只阻擋相關測試判定，請使用者針對該點決策；不鎖住其他開發工作。

## 5. Android 測試發布清單（B21）

- **可安裝**：APK 有版本號、checksum／hash，可在記錄的 Android 真機安裝與開啟；畫面、Rust Bridge、文字／圖像入口及基本返回操作正常。
- **可離線運算**：OCR／E2B 真正可離線推論；保留 Runtime／ABI／模型檔依賴與實測速度、RAM、電池影響。若模型過大或不相容，報 BLOCKED，不能換 Cloud AI。
- **安全**：Supabase RLS／配對的硬條件／Consent、公開欄位、撤銷、封鎖／檢舉與雙方完成 P0 PASS；沒有密鑰或真人私密資料進入 Repo／APK Client。
- **手機體驗**：S01–S09＋S04a 可操作；權限拒絕、空白、錯誤、載入／離線可理解；不要求 High-fi。
- **發布說明**：Android 支援最低版本、硬體限制／未驗證機種、安裝步驟、已知問題、如何撤銷測試環境資料與停止分發；不把 Pilot 宣稱正式商店上架。
- **回滾**：如測試版有事故，能停止分發／停用相關後端配對功能並保全資料；Database Migration 的回滾或補救策略需實際審視，避免不可逆修改。

## 6. CI、Debug 與最後判定

**Coding CI**：運行現有 Harness，依功能逐步加入可重現的 TypeScript／Rust／Unit／Contract／API Tests。缺形式性治理文件不得構成死鎖；但**真實 P0 功能／安全失敗必須標 FAIL 並修正**。

**Debug 流程**：失敗 Evidence → `B-ID / AC-ID / T-ID` → 重現 → 根因 → 修復 → 回歸 → 更新 PR。若一個修法已重複失敗，不要無限重試，換策略並留下風險。修復不需要新 Sprint Activation。

**最後成果判定**：
- `B19=PASS`：所有 **P0 T01–T13、AI-LOCAL-01–03、E2E-01–05、相關安全／真機案例**各有真實 PASS。
- `B21=PASS`：在 B19 完成後，Android Pilot APK 有可驗證安裝／操作／效能／發行與回滾證據。
- `B20` 可因 P1 為 NOT_RUN／IN_PROGRESS，不阻礙 B21，但應誠實揭露尚未量測的商業效果。
- **使用者仍擁有最終發布批准權**。CI PASS 不等於用戶驗收 PASS，更不等於 iOS 已跑通。

前段：[Sprint 1](SPRINT-01_LOCAL_AI.md)｜[Sprint 2](SPRINT-02_MATCHING.md)｜[三 Sprint 總覽](../SPRINT_PLAN.md)。
