# CKM｜B01–B21：產品規格 → Sprint → Acceptance 追溯表

> **用途：Coding Agent 開始工作前先查這張表**，不要只讀 Sprint 計畫，更不能由 Sprint 表格憑空生成產品規則。  
> **狀態**：設計／測試的追溯目錄，**不是**額外審批或治理 Gate；Coding 已開啟。  
> 來源：正式產品規格 [Business v1.2](../design/doc/CAKAMI_Business_Proposal_v1.2_MVP_Aligned.md)／[System v1.7](../design/doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx)／[Functional v1.8](../design/doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx) 及 [Bible B01–B21](BACKLOG.md)。

## 1. 必須一起閱讀的四份實作規格

- **[GUI / UX](../design/GUI_UX_SPEC.md)**：S01–S09／S04a 操作、共用元件與空白／錯誤／離線／權限狀態。已決定 Code-first，**無需 High-fi**。
- **[Functional](../design/FUNCTIONAL_SPEC.md)**：逐畫面行為、Intent／Consent／Chat／Session Gate、未決產品行為。
- **[Technical](../design/TECHNICAL_SPEC.md)**：本地 Rust／SQLite／E2B 和 Supabase 介面、資料權限與信任邊界；具體命名屬 PROPOSED。
- **[Acceptance](../test/ACCEPTANCE_SPEC.md)**：B01–B21 驗收矩陣、T01–T13、GUI-01–GUI-10、E2E-01–05、AI-LOCAL-01–03 與發布條件。

### 優先順序（避免再出現自相矛盾）

1. **使用者已明確確認的決策**：Phase 1 無 Cloud AI、Code-first GUI、Day 1 Android/iOS 共用可移植架構、單一 ChatGPT Agent／CI 治理不阻擋 Coding。
2. **正式設計＋Bible**：Business v1.2、System v1.7、Functional v1.8 與 `build/BACKLOG.md`。
3. **四份實作細節規格**：其 SOURCE 為對上層的解釋，PROPOSED／OPEN 是尚未核准的提案。
4. **Sprint 與本追溯表**：只分工作與定義可重現驗收，不得修改產品真相。若矛盾，應修正 Sprint／追溯表，不得偷偷挑較容易做的版本。

## 2. 每個 B-ID 對哪個 Spec／哪個 AC

下表各欄所稱 Functional、Technical、GUI、Acceptance 指本文件第 1 節連結的**同一份 main 規格**；括號內標明章節／畫面。**每一列都必須能返回對應 Bible 的 B-ID。**

| Bible | Sprint AC | Functional Spec | Technical Spec | GUI Spec | Acceptance Spec／必測 |
|---|---|---|---|---|---|
| **B01** | [S1-AC01](sprints/SPRINT-01_LOCAL_AI.md) | S01／基本操作 | §1–2 Rust Bridge、平台邊界 | 共用規則／S01 | B01、GUI-01／02 |
| **B02** | [S1-AC02](sprints/SPRINT-01_LOCAL_AI.md) | S01 Drop | §2、§3 私有資料 | S01／失敗狀態 | B02、GUI-03、T09 |
| **B03** | [S1-AC03](sprints/SPRINT-01_LOCAL_AI.md) | S02 OCR／E2B | §2 本地 OCR／E2B JSON | S02／人工更正 | B03、T01、T11、AI-LOCAL-01／02 |
| **B04** | [S1-AC04](sprints/SPRINT-01_LOCAL_AI.md) | S02 Goal Gate | §3 Rust／Intent | S02 只存私人內容 | B04、T05、E2E-02 |
| **B05** | [S1-AC05](sprints/SPRINT-01_LOCAL_AI.md) | S03 Registry | §3 Registry／Parameter | S03／S06 | B05、T01–T04、T10–T12 |
| **B06** | [S1-AC06](sprints/SPRINT-01_LOCAL_AI.md) | S03 8-slot | §3 SQLite Transaction／Schema | S03 滿額 | B06、T02、T09–T12 |
| **B07** | [S1-AC07](sprints/SPRINT-01_LOCAL_AI.md) | S03 Intent Lifecycle | §3～4 Revoke／Expiry | S03 撤銷／離線 | B07、T03、T06、T07 |
| **B08** | [S1-AC08](sprints/SPRINT-01_LOCAL_AI.md) | S03 Consent Gate | §4 同意與同步 Payload | S03 Preview／Consent | B08、T07、GUI-04 |
| **B09** | [S1-AC09](sprints/SPRINT-01_LOCAL_AI.md) | S02／S06 Questions | §3 Budget／Scope | S02／S06 | B09、T13 |
| **B10** | [S2-AC10](sprints/SPRINT-02_MATCHING.md) | 共用 Account／Safety | §4 Auth／Age | 登入／Safety 狀態 | B10、GUI-01／10 |
| **B11** | [S2-AC11](sprints/SPRINT-02_MATCHING.md) | S03～S09 Server Gate | §4～5 Schema／RLS／API | S03／S07～S09 | B11、T08、T10、AI-LOCAL-03 |
| **B12** | [S2-AC12](sprints/SPRINT-02_MATCHING.md) | S03／S04 Sync | §4 Consent／Outbox | S03／S04 | B12、T07、E2E-04 |
| **B13** | [S2-AC13](sprints/SPRINT-02_MATCHING.md) | S04 Matching | §5 Hard Gates | S04 | B13、T06、AI-LOCAL-03 |
| **B14** | [S2-AC14](sprints/SPRINT-02_MATCHING.md) | S04a／S05 | §5 Rank／Reason | S04a／S05 | B14、GUI-05／06、E2E-03 |
| **B15** | [S2-AC15](sprints/SPRINT-02_MATCHING.md) | S06 Pair-only | §3、§5 Pair Questions | S06 | B15、T12／T13 |
| **B16** | [S2-AC16](sprints/SPRINT-02_MATCHING.md) | S07 Invitation Gate | §5 Invitations／State | S07 | B16、GUI-07、E2E-04 |
| **B17** | [S2-AC17](sprints/SPRINT-02_MATCHING.md) | S08 Session／Chat | §5 Chat／Session | S08 | B17、GUI-08、E2E-01／05 |
| **B18** | [S2-AC18](sprints/SPRINT-02_MATCHING.md) | S09 Completion | §5 Outcome／State | S09 | B18、GUI-08、E2E-01／05 |
| **B19** | [S3-AC19](sprints/SPRINT-03_RELEASE.md) | S01–S09 全部 | §7 Verification | 全部／GUI-01～10 | B19、T01–T13、AI-LOCAL-01～03、E2E-01～05 |
| **B20** | [S3-AC20](sprints/SPRINT-03_RELEASE.md) | S04–S09 配對效果 | §5 Matching Algorithm | 無新增畫面 | B20：四種 Baseline 公平比較（P1） |
| **B21** | [S3-AC21](sprints/SPRINT-03_RELEASE.md) | S01–S09＋S04a | §7 Release | 全部真機狀態 | B21：B19 P0 PASS 才可發布；B20 是參考 |

## 3. 單一正式 Test ID 定義（修正 Sprint 寫法不一致）

**不要重新定義 E2E-01～05！** 以下全部以 [Acceptance Spec 第 5 節](../test/ACCEPTANCE_SPEC.md) 為準：

| Test ID | 唯一解釋 | 主要 B-ID |
|---|---|---|
| **E2E-01** | N2 Drop → 真配對 → 雙方練習並完成 | B01–B18 |
| **E2E-02** | 只收藏「パクる」，不找學伴、不發布 | B02–B04 |
| **E2E-03** | S04a 無候選人（不虛構人） | B13–B14 |
| **E2E-04** | 邀請拒絕／封鎖／撤銷、直接 Chat API 拒絕／離線下架 | B07／B12／B16 |
| **E2E-05** | 未達 15 分鐘／單方確認／No-show／重放／越權不算完成 | B17–B18 |

**T01–T13** 保留 System v1.7 原始 ID；**AI-LOCAL-01～03** 來自已合併 [Acceptance Spec](../test/ACCEPTANCE_SPEC.md) 的已確認 No Cloud AI 原則。不能把新的 Sprint Demo 自己叫作 E2E-02 等現有 ID；要另用 `S2-FLOW-*` 作開發劇本的局部代號。

## 4. 如何用這張表 Coding／Debug

```text
選 B-ID
 → 查 Bible B-ID 原始需求
 → 讀本表所列 GUI／Functional／Technical Spec（SOURCE／PROPOSED／OPEN）
 → 依 Sprint AC Coding
 → 依 Acceptance Spec 的正式 Test IDs 寫測試
 → Debug／重測
 → 記 GitHub PR + Commit + 實際 Evidence
```

發現差異：先核對既有已批准產品決策；工程命名／實作內部細節可自行設計並測試。**只有會改變公開資料、安全、對人行為的產品選項，才需 Human 決策**。記錄受影響 B-ID，不全局停止 Coding。

**驗證狀態**：本表是文件之間的完整對應，**不是已執行的產品測試報告**。所有沒有真實 Evidence 的 B-ID 都仍未 PASS。
