//! B04/S1-AC04：Content != Goal != published Intent。
//!
//! AI/OCR 只寫 local_ocr / local_model_suggestions；只有「使用者明確確認 Goal」
//! 的 Tauri Command 會進到這張分離的 SQLite 私人草稿表。
//! B05–B08 才實作 Registry、lifecycle、分享授權與發佈；B04 永遠不 publish。
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const MAX_GOAL_BYTES: usize = 2048;
const MAX_GOAL_CHARS: usize = 500;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmGoalInput {
    pub asset_id: String,
    pub goal_text: String,
    /// 由「確認我的 Goal」UI 操作明確傳入；模型／OCR 無此 Command 的輸出通路。
    pub confirmation_action: String,
    /// 新建必須為 0；修改必須等於目前的 revision，避免過期 UI 靜默覆寫。
    pub expected_revision: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrivateGoalDraft {
    pub id: String,
    pub asset_id: String,
    pub goal_text: String,
    /// 人類確認的是 Goal 文字，**不是**配對條件、分享、匹配或完成。
    pub goal_state: String,
    /// B07 Intent lifecycle 尚未開始；目前只可留在本機的草稿。
    pub intent_state: String,
    pub visibility: String,
    pub share_consent: bool,
    pub publishable: bool,
    pub confirmed_event_id: String,
    pub source_sha256: String,
    pub revision: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

pub struct GoalStore {
    db_path: PathBuf,
}

fn err(code: &'static str) -> String {
    code.into()
}

fn now_ms() -> Result<i64, String> {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|_| err("CLOCK_UNAVAILABLE"))?.as_millis())
        .map_err(|_| err("CLOCK_OVERFLOW"))
}

fn valid_id(id: &str) -> Result<String, String> {
    Uuid::parse_str(id)
        .map(|uuid| uuid.to_string())
        .map_err(|_| err("INVALID_ASSET_ID"))
}

impl GoalStore {
    pub fn open(root: &Path) -> Result<Self, String> {
        // 始終由現有 B02 initializer 擁有 local_assets schema。
        // B04 只能附加自己的私有 table，不另建一套假的 Content。
        super::local_store::LocalStore::open(root)?;
        fs::create_dir_all(root).map_err(|_| err("STORAGE_UNAVAILABLE"))?;
        let store = Self { db_path: root.join("cakami_private.sqlite") };
        let conn = store.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS local_goal_drafts (
                id TEXT PRIMARY KEY NOT NULL,
                asset_id TEXT NOT NULL UNIQUE REFERENCES local_assets(id) ON DELETE CASCADE,
                goal_text TEXT NOT NULL CHECK(length(trim(goal_text)) > 0
                                                AND length(goal_text) <= 2048),
                goal_state TEXT NOT NULL DEFAULT 'confirmed_by_user'
                    CHECK(goal_state='confirmed_by_user'),
                intent_state TEXT NOT NULL DEFAULT 'draft_private'
                    CHECK(intent_state='draft_private'),
                visibility TEXT NOT NULL DEFAULT 'private'
                    CHECK(visibility='private'),
                share_consent INTEGER NOT NULL DEFAULT 0 CHECK(share_consent=0),
                confirmed_event_id TEXT NOT NULL REFERENCES local_goal_confirmation_events(id)
                    DEFERRABLE INITIALLY DEFERRED,
                source_sha256 TEXT NOT NULL CHECK(length(source_sha256)=64),
                revision INTEGER NOT NULL CHECK(revision >= 1),
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS local_goal_confirmation_events (
                id TEXT PRIMARY KEY NOT NULL,
                draft_id TEXT NOT NULL REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
                revision INTEGER NOT NULL CHECK(revision >= 1),
                action TEXT NOT NULL CHECK(action='explicit_user_confirmation'),
                confirmed_goal_text TEXT NOT NULL,
                at_ms INTEGER NOT NULL,
                UNIQUE(draft_id, revision)
             );
             CREATE INDEX IF NOT EXISTS idx_goal_drafts_recent
                ON local_goal_drafts(updated_at_ms DESC);",
        ).map_err(|_| err("DATABASE_MIGRATION_FAILED"))?;
        Ok(store)
    }

    fn connect(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|_| err("DATABASE_UNAVAILABLE"))?;
        // SQLite foreign_keys is per connection; cascade deletion must also work from B02.
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .map_err(|_| err("DATABASE_UNAVAILABLE"))?;
        Ok(conn)
    }

    /// 讀取不會新建草稿。純 Drop/OCR/Gemma 之後必須回傳 None。
    pub fn load(&self, asset_id: &str) -> Result<Option<PrivateGoalDraft>, String> {
        let id = valid_id(asset_id)?;
        let conn = self.connect()?;
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM local_assets WHERE id=?1)",
            params![id], |row| row.get(0),
        ).map_err(|_| err("DATABASE_READ_FAILED"))?;
        if !exists {
            return Err(err("ASSET_NOT_FOUND"));
        }
        conn.query_row(
            "SELECT id,asset_id,goal_text,goal_state,intent_state,visibility,
                share_consent,confirmed_event_id,source_sha256,revision,created_at_ms,updated_at_ms
             FROM local_goal_drafts WHERE asset_id=?1",
            params![id],
            |row| Ok(PrivateGoalDraft {
                id: row.get(0)?,
                asset_id: row.get(1)?,
                goal_text: row.get(2)?,
                goal_state: row.get(3)?,
                intent_state: row.get(4)?,
                visibility: row.get(5)?,
                share_consent: row.get::<_, i64>(6)? != 0,
                publishable: false,
                confirmed_event_id: row.get(7)?,
                source_sha256: row.get(8)?,
                revision: row.get(9)?,
                created_at_ms: row.get(10)?,
                updated_at_ms: row.get(11)?,
            }),
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))
    }

    /// Rust-owned confirmation event + private Intent draft in *one* SQLite transaction.
    /// This command alone cannot share/publish/activate an Intent. No model can set event IDs.
    pub fn confirm(&self, input: ConfirmGoalInput) -> Result<PrivateGoalDraft, String> {
        if input.confirmation_action != "confirm_goal" || input.expected_revision < 0 {
            return Err(err("GOAL_CONFIRMATION_REQUIRED"));
        }
        let asset_id = valid_id(&input.asset_id)?;
        let goal_text = input.goal_text.trim();
        if goal_text.is_empty() || goal_text.contains('\0') {
            return Err(err("GOAL_EMPTY"));
        }
        if goal_text.len() > MAX_GOAL_BYTES || goal_text.chars().count() > MAX_GOAL_CHARS {
            return Err(err("GOAL_TOO_LARGE"));
        }

        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| err("DATABASE_BUSY"))?;
        let source_sha256: Option<String> = tx.query_row(
            "SELECT sha256 FROM local_assets WHERE id=?1",
            params![asset_id], |row| row.get(0),
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?;
        let source_sha256 = source_sha256.ok_or_else(|| err("ASSET_NOT_FOUND"))?;
        let old: Option<(String, i64, i64)> = tx.query_row(
            "SELECT id,revision,created_at_ms FROM local_goal_drafts WHERE asset_id=?1",
            params![asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?;

        let (draft_id, revision) = match old {
            Some((id, current_revision, _created_at)) => {
                if input.expected_revision != current_revision {
                    return Err(err("GOAL_REVISION_CONFLICT"));
                }
                (id, current_revision.checked_add(1).ok_or_else(|| err("GOAL_REVISION_CONFLICT"))?)
            }
            None => {
                if input.expected_revision != 0 {
                    return Err(err("GOAL_REVISION_CONFLICT"));
                }
                (Uuid::new_v4().to_string(), 1)
            }
        };
        let event_id = Uuid::new_v4().to_string();
        let time = now_ms()?;
        if revision == 1 {
            tx.execute(
                "INSERT INTO local_goal_drafts
                  (id,asset_id,goal_text,goal_state,intent_state,visibility,share_consent,
                   confirmed_event_id,source_sha256,revision,created_at_ms,updated_at_ms)
                 VALUES (?1,?2,?3,'confirmed_by_user','draft_private','private',0,
                         ?4,?5,1,?6,?6)",
                params![draft_id,asset_id,goal_text,event_id,source_sha256,time],
            ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        } else {
            let changed = tx.execute(
                "UPDATE local_goal_drafts SET goal_text=?1, confirmed_event_id=?2,
                    source_sha256=?3, revision=?4, updated_at_ms=?5
                 WHERE id=?6 AND revision=?7",
                params![goal_text,event_id,source_sha256,revision,time,draft_id,revision-1],
            ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
            if changed != 1 {
                return Err(err("GOAL_REVISION_CONFLICT"));
            }
        }
        tx.execute(
            "INSERT INTO local_goal_confirmation_events
              (id,draft_id,revision,action,confirmed_goal_text,at_ms)
             VALUES (?1,?2,?3,'explicit_user_confirmation',?4,?5)",
            params![event_id,draft_id,revision,goal_text,time],
        ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        tx.commit().map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        self.load(&asset_id)?.ok_or_else(|| err("DATABASE_READ_FAILED"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_store::{IncomingDrop, LocalStore};

    fn text(content: &str) -> IncomingDrop {
        IncomingDrop { kind: "text".into(), source: "text".into(),
            text: Some(content.into()), base64: None, file_name: None, mime_type: None }
    }

    fn input(asset_id: &str, goal_text: &str, expected_revision: i64) -> ConfirmGoalInput {
        ConfirmGoalInput {
            asset_id: asset_id.into(), goal_text: goal_text.into(),
            confirmation_action: "confirm_goal".into(), expected_revision,
        }
    }

    #[test]
    fn b04_drop_word_and_restaurant_ad_never_create_goal_or_intent() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let goal = GoalStore::open(dir.path()).unwrap();
        for raw in ["パクる", "本日限定！居酒屋 全品半額"] {
            let a = assets.save(text(raw)).unwrap();
            assert!(goal.load(&a.id).unwrap().is_none());
        }
        let conn = goal.connect().unwrap();
        let drafts: i64 = conn.query_row("SELECT COUNT(*) FROM local_goal_drafts", [],
            |row| row.get(0)).unwrap();
        let events: i64 = conn.query_row("SELECT COUNT(*) FROM local_goal_confirmation_events", [],
            |row| row.get(0)).unwrap();
        assert_eq!((drafts, events), (0, 0));
    }

    #[test]
    fn b04_explicit_confirmation_creates_private_draft_and_real_event() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("パクる")).unwrap();
        let goal = GoalStore::open(dir.path()).unwrap();
        let confirmed = goal.confirm(input(&a.id, "和真人學伴練習 JLPT N2 日文", 0)).unwrap();
        assert_eq!(confirmed.goal_state, "confirmed_by_user");
        assert_eq!(confirmed.intent_state, "draft_private");
        assert_eq!(confirmed.visibility, "private");
        assert!(!confirmed.share_consent);
        assert!(!confirmed.publishable);
        assert_eq!(confirmed.revision, 1);
        assert_eq!(confirmed.asset_id, a.id);
        assert_ne!(confirmed.id, confirmed.confirmed_event_id);
        let conn = goal.connect().unwrap();
        let stored_event: (String, i64, String) = conn.query_row(
            "SELECT action, revision, confirmed_goal_text
             FROM local_goal_confirmation_events WHERE id=?1",
            params![confirmed.confirmed_event_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        assert_eq!(stored_event, ("explicit_user_confirmation".to_string(), 1,
            "和真人學伴練習 JLPT N2 日文".to_string()));
        drop(goal);
        let restarted = GoalStore::open(dir.path()).unwrap();
        assert_eq!(restarted.load(&a.id).unwrap(), Some(confirmed));
    }

    #[test]
    fn b04_manual_revision_creates_new_event_without_overwriting_history() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("日文文法")).unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        let original = store.confirm(input(&a.id, "找學伴練 N2", 0)).unwrap();
        let edited = store.confirm(input(&a.id, "找學伴練 N2 聽力", 1)).unwrap();
        assert_eq!(edited.id, original.id);
        assert_ne!(edited.confirmed_event_id, original.confirmed_event_id);
        assert_eq!(edited.revision, 2);
        assert_eq!(store.confirm(input(&a.id, "用舊畫面覆寫", 1)).unwrap_err(),
            "GOAL_REVISION_CONFLICT");
        assert_eq!(store.confirm(input(&a.id, "重建成第二張草稿", 0)).unwrap_err(),
            "GOAL_REVISION_CONFLICT");
        let conn = store.connect().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM local_goal_confirmation_events
                                    WHERE draft_id=?1", params![original.id], |r| r.get(0)).unwrap();
        assert_eq!(n, 2);
        assert_eq!(store.load(&a.id).unwrap().unwrap().goal_text, "找學伴練 N2 聽力");
    }

    #[test]
    fn b04_no_confirmation_or_invalid_goal_never_creates_draft() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("パクる")).unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        let mut fake = input(&a.id, "找學伴", 0);
        fake.confirmation_action = "model_proposed".into();
        assert_eq!(store.confirm(fake).unwrap_err(), "GOAL_CONFIRMATION_REQUIRED");
        assert_eq!(store.confirm(input(&a.id, "   ", 0)).unwrap_err(), "GOAL_EMPTY");
        assert_eq!(store.confirm(input(&a.id, &"文".repeat(501), 0)).unwrap_err(),
            "GOAL_TOO_LARGE");
        assert!(store.load(&a.id).unwrap().is_none());
    }

    #[test]
    fn b04_model_cannot_smuggle_confirmation_event_or_consent_fields() {
        for raw in [
            r#"{"assetId":"a","goalText":"study","confirmationAction":"confirm_goal","expectedRevision":0,"consent":true}"#,
            r#"{"assetId":"a","goalText":"study","confirmationAction":"confirm_goal","expectedRevision":0,"confirmedEventId":"forged"}"#,
            r#"{"assetId":"a","goalText":"study","confirmationAction":"confirm_goal","expectedRevision":0,"visibility":"published"}"#,
        ] {
            assert!(serde_json::from_str::<ConfirmGoalInput>(raw).is_err());
        }
    }

    #[test]
    fn b04_sqlite_constraints_reject_publish_or_consent() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("パクる")).unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        let confirmed = store.confirm(input(&a.id, "我想找學伴", 0)).unwrap();
        let conn = store.connect().unwrap();
        assert!(conn.execute("UPDATE local_goal_drafts SET visibility='published' WHERE id=?1",
            params![confirmed.id]).is_err());
        assert!(conn.execute("UPDATE local_goal_drafts SET share_consent=1 WHERE id=?1",
            params![confirmed.id]).is_err());
        assert!(conn.execute("UPDATE local_goal_drafts SET intent_state='active' WHERE id=?1",
            params![confirmed.id]).is_err());
        assert_eq!(store.load(&a.id).unwrap().unwrap().intent_state, "draft_private");
    }

    #[test]
    fn b04_delete_source_cascades_draft_and_user_confirmation_history() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("パクる")).unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        store.confirm(input(&a.id, "我想找真人學伴", 0)).unwrap();
        assets.delete(&a.id).unwrap();
        let conn = store.connect().unwrap();
        let drafts: i64 = conn.query_row("SELECT COUNT(*) FROM local_goal_drafts", [],
            |r| r.get(0)).unwrap();
        let events: i64 = conn.query_row("SELECT COUNT(*) FROM local_goal_confirmation_events", [],
            |r| r.get(0)).unwrap();
        assert_eq!((drafts, events), (0, 0));
    }

    #[test]
    fn b04_model_suggestion_does_not_create_confirmed_goal() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("パクる JLPT N2")).unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        let model_output = crate::local_store::ModelSuggestion {
            asset_id: a.id.clone(), status: "suggested_private".into(),
            evidence_quote: "パクる".into(), content_hint: Some("日語單字".into()),
            uncertainty: None, model_id: "gemma-4-E2B-it".into(),
            runtime: "litert-lm-android-0.16.1".into(), backend: "cpu".into(),
            elapsed_ms: 12, updated_at_ms: 0,
        };
        assert_eq!(model_output.status, "suggested_private");
        assert!(store.load(&a.id).unwrap().is_none());
        // 即使另有有效 AI 建議，也無法繞過本人明確確認。
        let native = tauri_plugin_cakami_model::NativeReply {
            raw_text: r#"{"evidenceQuote":"パクる","contentHint":"日語單字","uncertainty":null}"#.into(),
            model_id: "gemma-4-E2B-it".into(), runtime: "litert-lm-android-0.16.1".into(),
            backend: "cpu".into(), elapsed_ms: 12,
        };
        assets.save_model_suggestion(&a.id, "パクる JLPT N2", native).unwrap();
        assert!(store.load(&a.id).unwrap().is_none());
        let conn = store.connect().unwrap();
        let cnt: i64 = conn.query_row("SELECT COUNT(*) FROM local_goal_confirmation_events",
            [], |row| row.get(0)).unwrap();
        assert_eq!(cnt, 0);
    }

    #[test]
    fn b04_sqlite_requires_real_reference_to_confirmation_event() {
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let a = assets.save(text("パクる")).unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        let conn = store.connect().unwrap();
        // SQLite 無法提交沒有對應 Rust-owned event 的 fake 私人草稿。
        let result = conn.execute(
            "INSERT INTO local_goal_drafts
               (id,asset_id,goal_text,goal_state,intent_state,visibility,share_consent,
                confirmed_event_id,source_sha256,revision,created_at_ms,updated_at_ms)
             VALUES (?1,?2,'test','confirmed_by_user','draft_private','private',0,
                     ?3,?4,1,1,1)",
            params![Uuid::new_v4().to_string(),a.id,Uuid::new_v4().to_string(),
                    "a".repeat(64)],
        );
        assert!(result.is_err());
        assert!(store.load(&a.id).unwrap().is_none());
    }

    #[test]
    fn b04_unavailable_asset_or_invalid_id_cannot_generate_goal() {
        let dir = tempfile::tempdir().unwrap();
        let store = GoalStore::open(dir.path()).unwrap();
        assert_eq!(store.confirm(input("../../wrong", "學日文", 0)).unwrap_err(),
            "INVALID_ASSET_ID");
        let missing = Uuid::new_v4().to_string();
        assert_eq!(store.confirm(input(&missing, "學日文", 0)).unwrap_err(),
            "ASSET_NOT_FOUND");
        assert_eq!(store.load(&missing).unwrap_err(), "ASSET_NOT_FOUND");
    }
}
