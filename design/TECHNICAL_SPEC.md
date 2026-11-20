# CAKAMI｜技術介面與資料契約（Coding 參考規格）

> **狀態：可供 Coding 參考；技術簽章／DDL 屬 PROPOSED，未部署且未經真機驗證。既有 SOURCE／已核准原則不可因為提案而被改寫；不要求全部介面先 Freeze 才能 Coding。**  
> 日期：2026-11-17；範圍：Phase 1 Android First；B01–B18 的工程契約及 B19–B21 驗收證據入口。  
> 正式來源：../build/BACKLOG.md；doc/CAKAMI_System_Design_v1.7_MVP_8_Parameters.docx（§2、§3、§4–5、§7–8、§15–16）；doc/CAKAMI_MVP_Functional_Design_v1.8_S01-S09_Sequences.docx。  
> 本文件不變更技術骨架。**所有尚未由現有設計明確定義的具體名稱、欄位、路徑、函式簽章均為可審核提案，不能以文件存在當作已批准。**

## 已核准的 Phase 1 架構紅線：NO CLOUD AI

**User 決策日期：2026-11-17。** Phase 1 MVP 的 AI 推論必須全部在裝置本地處理。以下為 Coding 與驗收的硬限制：

- **禁止**：外部 LLM／VLM／OCR／Embedding API（例如 OpenAI、Gemini、Anthropic 或託管的 AI 服務）、雲端 Agent／模型推論、E2B 失敗自動 fallback 至 Cloud AI、雲端生成配對解釋或個人特徵。
- **允許**：以 Supabase Auth、PostgreSQL／pgvector、RLS、受控 RPC／Edge Function（**僅限一般規則與資料操作，不能呼叫 AI 模型**）、Realtime／Chat 實現多人合作；SQL／pgvector 的向量距離與硬條件運算屬資料庫計算，不是 AI 模型推論。
- **Embedding 規則**：若 B13 需要向量，必須在手機本地用已核准的本地模型產生；**禁止由雲端 API 產生向量**。只有使用者明確授權時才傳必要特徵；Embedding 也不是天然匿名。
- **模型不能跑時**：明確回報 MODEL_UNAVAILABLE／相關錯誤並提供人工修正，禁止暗中上傳圖片或文字給雲端 AI；不能把手動流程假算為 B03 本地 AI PASS。
- **仍待驗證**：本地 E2B 模型版本與運行環境、是否需要獨立本地 Embedding 模型、Android 記憶體與效能。不可把「本地 AI 原則已定」等同「本地 AI 技術已驗證」。

## 1. 層級責任與信任邊界（SOURCE）

~~~text
Android UI (Tauri v2 + TypeScript)
  └─ Tauri Command → Rust Controller
       ├─ Android Native Bridge → On-device OCR／E2B（候選／人工修正）
       ├─ SQLite + App-private files（原圖／OCR／私人 Intent／8 slots／events）
       └─ 授權後才同步最少必要公開摘要
            → Cloud API／Supabase Auth
            → PostgreSQL + pgvector + RLS + server validation
            → Candidates／Invitations／Chat／Session／Outcome
~~~

1. Android UI 與 E2B 不是可信確認事件來源；不可直接寫 Parameter Table。
2. Rust 認可的使用者確認事件必須由明確 UI 操作觸發並建立於受控 Controller；Server **不能只相信** client 發來的 event ID 或 is_confirmed 欄位。
3. Cloud 驗證登入帳號、持有權、Consent、狀態、版本、8-slot、期限與封鎖；強制必要狀態轉移與關係權限。RLS 不能取代 Server business validation。
4. 原始圖片、原始 OCR、私人筆記、完整 Intent／歷史不可作為普通公開同步 Payload。Embedding 亦需明確 Consent。
5. 不做 Phase 2 Map／GPS／Dating／Creature／通用 App Builder。

## 2. B01–B03：Android Native AI 介面（PROPOSED）

### Android Bridge 責任

| 方法類型（暫名） | 輸入 | 輸出 | 必要失敗型態 |
|---|---|---|---|
| local_asset_import | camera／picker content URI 或文字；呼叫方 Intent | local_asset_id、持久化路徑／來源、安全 metadata | permission_denied、unsupported_format、storage_error |
| local_ocr_recognize | local_asset_id | OCR segments：text、範圍、confidence（如有）、source_ref | model_unavailable、timeout、recognition_failed |
| local_model_propose | asset_ref／OCR、指定候選 schema | Proposal JSON、模型版本、輸入 evidence_ref、processing info | model_load_error、out_of_memory、timeout、invalid_output |
| local_asset_open | local_asset_id | 僅受本地控制器允許的可讀資源 | not_found、permission_denied |

- 上述是責任與資料方向，不是已定稿的 Rust／Kotlin function signatures。
- Android OCR 候選：on-device Japanese OCR；E2B 候選：Gemma E2B。**版本、量化、模型檔與實際 runtime 仍 OPEN**；先做真機 Spike，不能假設 Tauri + Rust 一定能直接部署模型。
- 不得因模型載入失敗改走 Cloud AI 而未經產品決策；可保留手動建立私人草稿，但 B03 不因此被視為完成。
- 真機 Evidence 必須註明機型、Android/API 版本、CPU/GPU/NPU 後端、冷／熱載入、延遲、峰值記憶體、耗電、OCR／意圖正確率及是否斷網。

### B03／B21 Gemma 模型獨立更新與安全回退（User 決策：2026-11-19；新增設計，尚未實作）

**SOURCE／已核准產品行為：APK 更新不等於模型更新。** Gemma 是 App-private 的獨立版本化檔案；已下載且驗證成功的同一模型版本，不應因 APK 的 UI、Rust、SQLite、Prompt 或一般引擎修正而重下載。相同 Android applicationId、相同簽章的**覆蓋更新**應保留模型及使用者私人資料；Android **解除安裝、清除資料、裝置更換**則通常會失去 App-private 模型，不能承諾保留。若 modelId／版本／SHA-256／runtime 相容性均未改變，啟動時使用現有模型。

**SOURCE／已核准的升級安全順序：** 新模型確實值得升級（例如正式修復、顯著更佳日文理解、必要相容性或新能力）才推送升級提示；**先下載新模型 → 完整校驗 → 切換至新版本 → 確認可載入／可推論 → 才清理舊版**。下載、驗證或啟用失敗一律繼續使用舊版；離線仍使用舊版，絕不因更新檢查失敗封鎖既有 AI，也不切換到 Cloud AI。

**實作介面／資料結構（PROPOSED，待 B03／B21 Coding）**：

1. 以版本化 Model Manifest 描述 `modelId`、`modelVersion`、`sha256`、`bytes`、`format`、`runtimeCompatibility`／`minAppVersion`、來源 URL、版本說明及驗證簽章。只有 App 信任的簽章／固定核准清單能授權新模型；網路 URL 或遠端 Manifest 不能任意決定任務 Prompt、私人資料上傳或 AI 推論端點。模型升級不得默默降級。Manifest 格式、簽章密鑰管理與端點尚 **OPEN**。
2. Rust 控制狀態：`ready(current)`、`update_available`、`downloading(candidate)`、`verifying`、`staged`、`activating`、`rollback`、`failed_keep_current`。使用 Android 原生下載 Adapter；Wi-Fi 預設、行動網路須使用者同意。可暫停／續傳，失敗不刪已可用的舊模型，重開機／重啟 App 須可恢復下載。
3. 使用 App-private、**不同版本**的暫存／模型路徑；新檔驗證大小、SHA-256 和相容性後，才原子更新「目前啟用模型」紀錄（含回退指標）。不可覆寫正在推論的舊檔；切換時需處理 LiteRT-LM 已載入模型的生命週期／鎖定。更換啟用版本需避免 crash 中途造成無模型可用。
4. 新模型切換後的載入與最小本地推論 Smoke Test 必須成功，才能將它標成 `active_verified`。若載入、推論或完整性檢查失敗，回退到舊模型並回報錯誤，不可把「SHA 通過」當成「推論已通過」。
5. 儲存空間不足以**同時保留舊模型與新模型＋暫存**時，不強制升級、不刪舊檔騰空間；UI 告知所需空間並允許以舊版繼續使用。只有新版啟用／驗收成功後才依清理政策刪除不再需要的舊版。完整多版本磁碟預算與保留期限尚 **OPEN**。
6. 新版檢查在網路可用時非阻斷式執行，失敗可以下次再檢查；**不因每次 App 啟動、APK 升級、或每次呼叫 Gemma 就無條件下載 2.59 GB**。可用者先收到版本說明／預估大小與「稍後／更新」選擇；正式檢查頻率、是否允許緊急強制升級為 **OPEN**，不得在 MVP 擅自實作。
7. 模型權重的下載源可以是 Hugging Face 或未來的 CAKAMI 自有儲存／備援 CDN，但必須使用經核准的版本與固定檔案雜湊；**下載公開模型檔不代表把私人 OCR、圖片、筆記、Prompt 發往遠端**。需遵守模型再散布授權及來源完整性要求。

**實作狀態：NOT_IMPLEMENTED**。目前 B03 的單一固定模型自動下載／Range 續傳／SHA-256 驗證，**不等於**版本化 Manifest、雙槽保留與原子切換、回退或遠端升級提示已完成。此節是後續 B03 維護與 B21 Release Gate 的正式設計需求。

### E2B → Rust 受控候選資料（PROPOSED JSON schema shape）

~~~json
{
  "asset_id": "local-asset-id",
  "model": {"id": "pending-validated-model", "version": "pending"},
  "content": {"kind": "vocabulary", "text": "パクる"},
  "proposals": [
    {
      "concept_alias": "study.jlpt_level",
      "value": "N2",
      "value_type": "enum",
      "source": "ocr",
      "evidence_ref": "local-evidence-id",
      "scope": "intent",
      "confidence": null
    }
  ],
  "unresolved": ["goal"]
}
~~~

**嚴禁**：候選 JSON 出現 confirmed_event_id、consent=true、visibility=published 或要求直接成為 CONFIRMED。若模型輸出類似欄位，Rust 必須拒絕／忽略非法指示並記錄錯誤；若證據缺失，保持 UNKNOWN／私人候選。上方 example 僅為 Contract Proposal，非 Gemma 已驗證輸出格式。

## 3. B04–B09：Parameter Contract（SOURCE + PROPOSED）

### 3.1 固定 Concept Registry，動態值

- 依 System v1.7：受控 Registry、同義概念標準化、typed values、scope = profile／intent／pair；未知概念只能短期 private draft。
- **SOURCE**：「N2」「JLPT N2」「日語二級」歸成同一 concept；單字圖片即使 100 張也不各占 Matching Slot。
- **PROPOSED**：Registry 記錄 concept_id、aliases、value_type、allowed_scopes、normalization_rule、expiry_default_policy、match_usage、version；初期採版本化靜態設定，不允許 AI 自行新增正式 concept。
- **OPEN T-01**：P1 完整白名單／別名／枚舉與高優先級硬條件清單；配對時哪些 UNKNOWN 會占 slot。未核准前不得自創全集。

### 3.2 Parameter 必填資料（SOURCE）

| 欄位 | 必要規則 |
|---|---|
| concept_id／value_type／value_json | Registry 已知、型別正確；UNKNOWN 的值可為 NULL |
| source／evidence_ref | source 在 ocr／model_inferred／user_explicit／system_derived；證據可追溯 |
| scope／owner_id | profile／intent／pair；pair 不能升為永久 Profile |
| state／confirmed_event_id | OBSERVED／INFERRED／UNKNOWN／CONFIRMED／REVOKED／EXPIRED；CONFIRMED 需真實控制器使用者事件 |
| expiry_policy／expires_at／review_due_at | fixed／intent_lifecycle／pair_lifecycle／persistent_review；fixed 有 expires_at；長期需 review_due_at |
| visibility／consent | 預設 private；公開需明示授權；撤銷使索引失效 |
| version／timestamps | version 正整數，變更 +1；UTC 保存時間並保留原時區／日期語意 |

### 3.3 SQLite 必須實作的雙層限制（SOURCE）

- 本地建議最小實體表：local_assets、local_intents、intent_parameters、intent_match_slots、parameter_evidence、confirmation_events、consent_events、draft_parameter_proposals、pair_answers、sync_outbox。
- **SOURCE 必要 Constraint**：foreign keys ON；slot_index BETWEEN 0 AND 7；UNIQUE(intent_id, slot_index)；UNIQUE(intent_id, concept_id)；來源、scope、state、visibility、expiry、版本的 NOT NULL／CHECK；CONFIRMED 需要 confirmed_event_id；跨表事件真實性與授權由 Rust／Server 驗證。
- 「第 9 個」在 8 個已佔用時回傳 BUDGET_FULL；如為必要安全硬條件，標記 BLOCKED_BY_BUDGET、停止 publish/propose；只在使用者明確替換／合併並留下歷史後才允許再試。
- Slot 失效／撤銷釋放必須同交易封存歷史，不可 DELETE 後抹除稽核證據；並發或版本衝突不得把第 9 個塞入。
- **PROPOSED implementation detail**：sync_outbox 維持 idempotency_key、intent_version、action、retry_count、last_error、acked_at，實現離線撤銷重試；精確欄位待核准。
- **OPEN T-02**：SQLite 資料型別／索引／遷移版本、內容實體文件或 BLOB、加密儲存策略、同步 outbox 失敗與過期清理精確順序。

### 3.4 Rust Command（PROPOSED signature list）

| Command（暫名） | 主要輸入 → 結果 | 信任檢查 |
|---|---|---|
| import_content | local asset → saved_asset | 本地儲存可用、保留來源 |
| propose_content | saved_asset → read-only candidate | AI 不允許寫 state／consent |
| save_intent_draft | candidate + explicit edits → draft | Content 不直接變成 Goal |
| confirm_goal | intent_id + UI action → confirmed Goal | Rust 建事件、檢查 provenance |
| validate_parameter | proposal → normalized result / errors | Registry、type、scope、expiry |
| confirm_parameter | intent/version + UI action → new version | event、bounded slots、transaction |
| list_intents／revoke_intent | owner context → result | owner／version／lifecycle |
| preview_share | intent/version → least-necessary fields | 只預覽合法欄位 |
| publish_intent | intent/version + explicit UI consent → sync result | Rust double-check；Server 後續再驗證 |
| answer_pair_question | pair/question + explicit answer → scoped result | Pair-only、TTL、question budget |

以上為契約範圍，**未決定實際 Tauri invoke 方法名稱與序列化型別**；不得直接把暫名當作已正式 API。

## 4. B10–B12：Auth／Cloud Payload／撤銷

### 最少同步內容

**SOURCE**：只有經真人確認與授權的最少必要 Intent 摘要、匹配條件／必要向量，才能上傳；原始圖片、OCR、私人筆記、私人全文不進 Cloud。

**PROPOSED 公開資料 envelope**：

~~~json
{
  "intent_id": "opaque-intent-id",
  "intent_version": 1,
  "consent_version": 1,
  "share_scope": ["goal", "study_level", "availability"],
  "match_concepts": [{"concept_id": "study.jlpt_level", "value": "N2"}],
  "expires_at": "2026-12-01T00:00:00Z",
  "idempotency_key": "opaque-request-id"
}
~~~

- 以上並**非最終核准的欄位白名單**；share_scope、study_level、availability 是否真的必要，須由 User 在 T-03 核定。任何 Server 可獲得的公開摘要都必須展示給使用者並留下 Consent 事件。
- Auth user ID 不採信 client 任意輸入的 owner_id；Server 從驗證過的 Token 判斷持有者。
- 嚴禁在 Android binary／前端放 Supabase service-role 或 secret key；只允許 publishable client key、合理權限及 Server-controlled 業務操作。
- 18+ 條件、封鎖與檢舉資料不可被任意使用者改寫；不能把自行可編輯的 user_metadata 當成可信權限來源。
- Server 以獨立授權事件及版本檢查，拒絕客戶端自填 confirmed=true；針對越權、過期、撤銷、8-slot 失敗回傳固定安全錯誤。
- 當裝置離線撤銷：本地立即不匹配，outbox 重試；**不得宣稱 Cloud 已撤銷**。Cloud 查詢必須在 Server 知情後排除；是否需額外短 TTL／租約為 OPEN T-04（可能影響公開時效）。
- **OPEN T-03**：正式公開／匹配欄位白名單、consent granularity、匿名對外稱呼與使用者資訊可見性。
- **OPEN T-04**：同意版本、revoke 可靠性／重試與過期政策，離線至雲端更新期間 UI 風險文案。
- **OPEN T-05**：成年用戶的驗證與帳號恢復策略、檢舉／封鎖資料保留政策。

## 5. B11–B18：Server Schema、Matching 與 Session

### Cloud 實體（SOURCE 類別；以下名稱待 DDL 審核）

- public_intents、embeddings、matches、questions、invitation_events、sessions、messages、blocks、reports、outcome_events。
- 所有對外可用表必須 RLS + 權限；即使擁有 authenticated role，也不能直接讀其他使用者的私人行。
- RLS 主要約束資料行存取；**參數版本、雙方同意、Chat 入場與完成條件必須由 Server transaction／受控 RPC 或 API 重新驗證**。

### Matching Pipeline（SOURCE）

~~~text
Server-revalidated ACTIVE Intent + Consent + expiry + current version
    → Same Goal / Need↔Offer recall（可使用 pgvector）
    → two-way feasibility + explicit hard-constraint gate
    → uncertainty penalty / high-value missing question
    → explainable rank（不宣稱成功率）
    → S05 real human candidates 或 S04a
~~~

- 基線排序遵守 v1.7：同目標 S_same、雙向 Need↔Offer S_recip、能力加成 S_add、共同執行 S_joint，權重與 U unknown penalty；具體權重不得自行宣稱學習完成率。
- Pair-only answer 限 Pair scope，最多 2 題；必要條件缺少則繼續 UNKNOWN，不能以向量加分覆蓋。
- **OPEN T-06（高風險）**：Embedding 由哪個模型在何端生成、模型版本／向量維度／正規化、pgvector index type、Top-K、baseline 權重、cold start 策略。不能在沒有可用 embeddings 的情況下宣稱 B13 完成。

### Invitation／Session 狀態（SOURCE + PROPOSED）

| 狀態 | 必要 Server Gate |
|---|---|
| PROPOSED | 兩方及兩個 Intent 仍有效、無封鎖、有可比對邀請版本 |
| MUTUAL_ACCEPTED | 雙方分別以有效登入身分明確接受、未拒絕／取消／超時 |
| SESSION | 僅該有效 Pair 建立 Chat／Session；任何其他帳號拒絕存取 |
| COMPLETED | Server-side 有效 duration ≥15 分鐘，且 A／B 都有確認事件 |
| REJECTED／EXPIRED／CANCELLED | 不可再次開放 Chat，結果不可計完成；重新建立需另有合法操作 |

- **PROPOSED**：所有 mutate 操作帶 idempotency_key + expected_version，Server transaction 再驗證目前狀態，避免同時接受、重複送出與網路重試引發雙重 Session／Outcome。
- **OPEN T-07**：邀請 TTL、Session 起算與暫停、離線是否計時、消息保存期限與 Rate Limits、No-show 的實際判準。
- **OPEN T-08**：文字 Chat 實作的表名、Realtime 權限、推播／背景通知範圍；不能承諾尚未確定的通知服務。

### Cloud API 責任（PROPOSED，非已鎖定路由）

| 操作 | 規則 | 主要 B-ID |
|---|---|---|
| publishIntent / revokeIntent | owner + consent + version + 8-slot + expiry check | B11–B12 |
| findCandidates | 只查已授權有效 Intent + hard-filter | B13 |
| askClarification / answerPair | 只問真缺口，查 Pair scope / TTL / budget | B15 |
| proposePair / respondToProposal | 雙方有效、邀請與回應不能越權／重放 | B16 |
| createSession / sendMessage | 必須 MUTUAL_ACCEPTED；僅 Pair 成員 | B17 |
| completeSession / recordOutcome | duration + A/B 分別確認；不可重複計數 | B18 |

## 6. Error / Observable State Contract（PROPOSED）

| 錯誤碼族 | 應用到畫面 | 必須顯示的真實狀態 |
|---|---|---|
| LOCAL_PERMISSION_DENIED／LOCAL_STORAGE_ERROR | S01 | 未保存；可重試或更換輸入 |
| OCR_UNKNOWN／MODEL_UNAVAILABLE／MODEL_TIMEOUT | S02 | 辨識未知或失敗；原始內容仍可人工修正 |
| INVALID_SOURCE／INVALID_SCOPE／INVALID_TYPE／CONFIRMATION_REQUIRED | S02–S03 | 拒絕不可信候選，列出可修正內容 |
| BUDGET_FULL／BLOCKED_BY_BUDGET／VERSION_CONFLICT | S03／S06 | 暫停發佈、不得偷偷刪條件 |
| CONSENT_REQUIRED／SYNC_PENDING／REVOCATION_PENDING | S03／S04 | 明確告知未公開、待同步、雲端可能尚未撤銷 |
| AUTH_REQUIRED／FORBIDDEN／CANDIDATE_EXPIRED | S04–S07 | 不展示越權資料、不允許失效邀請 |
| MUTUAL_CONSENT_REQUIRED／BLOCKED／RATE_LIMITED | S07–S08 | 禁止 Chat，提供合理下一步 |
| SESSION_TOO_SHORT／SECOND_CONFIRMATION_PENDING | S09 | 不計 COMPLETED |

名稱為提案，Rust/TS/Server 最終需要同一份版本化 error registry，不能產生多份不同字串。錯誤不可洩漏另一個用戶私人資料。

## 7. 驗證、CI 與 Release Gate（SOURCE）

- **T01–T13**：採 System v1.7 §15.5／§16.5 原意，全部自動化並保留 Evidence；詳見 ../test/ACCEPTANCE_SPEC.md。
- SQLite：CHECK／UNIQUE／Foreign key／交易競爭與故障注入；Rust 不能偽造確認／繞過 8-slot。
- Cloud：以 A、B、C 多帳號測試 RLS／版本／Consent／封鎖／撤銷／Chat；Client 偽造 API 不得成功。
- Android：真機 S01–S09 + S04a、離線／中斷／回歸；記錄 E2B 速度、RAM、電池。
- Release：所有 P0 Gate PASS 才允許 B21 Android Pilot；B20 是 P1 且需可重現 Benchmark，不把真實用戶成效未取得的資料偽造成已通過。

## 8. 只在影響相應功能時才需確認的 Engineering Decisions

| 決策 | 必須拿到的答案／Evidence | 影響 |
|---|---|---|
| T-01 Registry | 受控 Concept 清單、型別、別名、硬條件 | B05–B06 |
| T-02 SQLite | migration、events、slot lifecycle、outbox Schema | B06–B07／B12 |
| T-03 Consent | 公開欄位白名單、同意版本／範圍 | B08／B11–B12 |
| T-04 Revoke | 離線、TTL、失敗可見性與雲端刪除 SLA | B07／B12 |
| T-05 Auth | 18+ Gate、驗證／封鎖／檢舉最低要求 | B10／B16 |
| T-06 Matching | Embedding 來源／維度／模型版本與排序基線 | B13–B15 |
| T-07 Collaboration | 邀請 TTL、Session 計時、No-show、Chat 限流 | B16–B18 |
| T-08 Android AI | E2B 模型版本、格式、量化、runtime、設備最低規格 | B01–B03／B21 |
| T-09 Interface | 正式命名的 typed command／Cloud route／error enum／idempotency | B01–B18 |

**決策處理**：先核對 v1.7／v1.8 是否已定義；只有真的牽涉產品行為、隱私或安全之未決事項才交 Human 決策，技術內部細節由 Coding Agent 提案並測試。決策後更新本文件與受影響來源／測試，留下 Commit。**不得以此流程對其他不受影響 Coding 設全局 HOLD。** 沒有真機 Spike 不可把 T-08 標為已驗證。

## 9. 文件責任

- GUI 與文案呈現：GUI_UX_SPEC.md。
- 行為與生命週期的產品意義：FUNCTIONAL_SPEC.md。
- 本文件：資料、介面、驗證責任與技術不變條件。
- 測試證據：../test/ACCEPTANCE_SPEC.md。
- 工作與真實進度：../build/BACKLOG.md（Bible）。
