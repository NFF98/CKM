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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LifecycleUpdateInput {
    pub asset_id: String,
    pub expected_revision: i64,
    pub expected_lifecycle_version: i64,
    pub action: String,
    pub expires_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateSharePreview {
    pub draft_id: String,
    pub goal_text: String,
    pub lifecycle_state: String,
    pub possible_concept_ids: Vec<String>,
    pub visibility: String,
    pub consent_recorded: bool,
    pub can_upload: bool,
    pub approval_status: String,
    pub excluded_private_sources: Vec<&'static str>,
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
    pub lifecycle_state: String,
    pub expires_at_ms: Option<i64>,
    pub lifecycle_version: i64,
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
                ON local_goal_drafts(updated_at_ms DESC);
             CREATE TABLE IF NOT EXISTS local_goal_lifecycle (
                draft_id TEXT PRIMARY KEY NOT NULL
                    REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
                state TEXT NOT NULL CHECK(state IN ('active_private','cancelled_private')),
                expires_at_ms INTEGER CHECK(expires_at_ms IS NULL OR expires_at_ms>0),
                version INTEGER NOT NULL CHECK(version>=1),
                updated_at_ms INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS local_goal_lifecycle_events (
                id TEXT PRIMARY KEY NOT NULL,
                draft_id TEXT NOT NULL REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
                version INTEGER NOT NULL CHECK(version>=1),
                action TEXT NOT NULL CHECK(action IN ('set_expiry','cancel_goal')),
                source TEXT NOT NULL CHECK(source='user_explicit'),
                at_ms INTEGER NOT NULL,
                UNIQUE(draft_id,version)
             );",
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
            "SELECT g.id,g.asset_id,g.goal_text,g.goal_state,g.intent_state,g.visibility,
                g.share_consent,g.confirmed_event_id,g.source_sha256,g.revision,g.created_at_ms,
                g.updated_at_ms,coalesce(l.state,'active_private'),l.expires_at_ms,
                coalesce(l.version,0)
             FROM local_goal_drafts g
             LEFT JOIN local_goal_lifecycle l ON l.draft_id=g.id
             WHERE g.asset_id=?1",
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
                lifecycle_state: {
                    let state: String = row.get(12)?;
                    let expiry: Option<i64> = row.get(13)?;
                    if state == "cancelled_private" {
                        "cancelled_private".to_owned()
                    } else if expiry.is_some_and(|value| value <= now_ms().unwrap_or(i64::MAX)) {
                        "expired_private".to_owned()
                    } else {
                        "active_private".to_owned()
                    }
                },
                expires_at_ms: row.get(13)?,
                lifecycle_version: row.get(14)?,
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
                let lifecycle: Option<(String,Option<i64>)> = tx.query_row(
                    "SELECT state,expires_at_ms FROM local_goal_lifecycle WHERE draft_id=?1",
                    params![id],|r|Ok((r.get(0)?,r.get(1)?)),
                ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?;
                if lifecycle.as_ref().is_some_and(|(state,expiry)|
                    state=="cancelled_private" || expiry.is_some_and(|ms|ms<=now_ms().unwrap_or(i64::MAX)))
                { return Err(err("LIFECYCLE_INACTIVE")); }
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
    /// B07: only the person can set an explicit future deadline or cancel.
    /// Cancellation never grants Cloud consent and cannot be undone by editing Goal.
    pub fn update_lifecycle(&self, input: LifecycleUpdateInput)
        -> Result<PrivateGoalDraft, String>
    {
        let asset_id=valid_id(&input.asset_id)?;
        if input.expected_revision<1 || input.expected_lifecycle_version<0 {
            return Err(err("GOAL_REVISION_CONFLICT"));
        }
        let now=now_ms()?;
        match input.action.as_str() {
            "set_expiry" if input.expires_at_ms.is_some_and(|ms|ms>now) => {},
            "cancel_goal" if input.expires_at_ms.is_none() => {},
            _ => return Err(err("LIFECYCLE_INVALID_ACTION")),
        }
        let mut conn=self.connect()?;
        let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| err("DATABASE_BUSY"))?;
        let draft: Option<(String,i64)> = tx.query_row(
            "SELECT id,revision FROM local_goal_drafts WHERE asset_id=?1 AND
             intent_state='draft_private' AND visibility='private' AND share_consent=0",
            params![asset_id],|r|Ok((r.get(0)?,r.get(1)?))
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?;
        let (draft_id,revision)=draft.ok_or_else(||err("GOAL_CONFIRMATION_REQUIRED"))?;
        if revision!=input.expected_revision {return Err(err("GOAL_REVISION_CONFLICT"));}
        let prior: Option<(String,Option<i64>,i64)> = tx.query_row(
            "SELECT state,expires_at_ms,version FROM local_goal_lifecycle WHERE draft_id=?1",
            params![draft_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
        ).optional().map_err(|_|err("DATABASE_READ_FAILED"))?;
        let version=prior.as_ref().map(|(_,_,v)|*v).unwrap_or(0);
        if version!=input.expected_lifecycle_version {
            return Err(err("LIFECYCLE_VERSION_CONFLICT"));
        }
        if prior.as_ref().is_some_and(|(state,expiry,_)|
            state=="cancelled_private" || expiry.is_some_and(|ms|ms<=now)) {
            return Err(err("LIFECYCLE_INACTIVE"));
        }
        let next_version=version+1;
        let state=if input.action=="cancel_goal" {"cancelled_private"} else {"active_private"};
        let expiry=if input.action=="cancel_goal" {None} else {input.expires_at_ms};
        if version==0 {
            tx.execute(
                "INSERT INTO local_goal_lifecycle
                    (draft_id,state,expires_at_ms,version,updated_at_ms)
                 VALUES (?1,?2,?3,1,?4)",
                params![draft_id,state,expiry,now],
            ).map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        } else {
            tx.execute(
                "UPDATE local_goal_lifecycle SET state=?1,expires_at_ms=?2,
                   version=?3,updated_at_ms=?4 WHERE draft_id=?5 AND version=?6",
                params![state,expiry,next_version,now,draft_id,version],
            ).map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        }
        tx.execute(
            "INSERT INTO local_goal_lifecycle_events
              (id,draft_id,version,action,source,at_ms)
             VALUES (?1,?2,?3,?4,'user_explicit',?5)",
            params![Uuid::new_v4().to_string(),draft_id,next_version,input.action,now],
        ).map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        tx.commit().map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        self.load(&asset_id)?.ok_or_else(||err("DATABASE_READ_FAILED"))
    }

    /// B08: a LOCAL-ONLY preview; no approved cloud payload or Consent handler.
    /// No image, raw OCR, notes or user answer is ever exported by this method.
    pub fn share_preview(&self, asset_id: &str) -> Result<PrivateSharePreview,String> {
        let goal=self.load(asset_id)?.ok_or_else(||err("GOAL_CONFIRMATION_REQUIRED"))?;
        // Existing B05/B06 schema owns the preview's valid Concept projection.
        // Never create a separate, unvalidated list for sharing.
        super::parameter_registry::ParameterStore::open(
            self.db_path.parent().ok_or_else(||err("STORAGE_UNAVAILABLE"))?
        )?;
        let conn=self.connect()?;
        let possible=if goal.lifecycle_state=="active_private" {
            let mut stmt=conn.prepare(
                "SELECT s.concept_id FROM local_matching_slots s
                 JOIN local_parameter_drafts p ON p.id=s.parameter_id
                 WHERE s.draft_id=?1 AND s.goal_revision=?2
                   AND p.state='CONFIRMED' AND p.visibility='private'
                   AND p.share_consent=0 AND p.goal_revision=?2 AND
                   (p.expires_at_ms IS NULL OR p.expires_at_ms>?3)
                 ORDER BY s.slot_index",
            ).map_err(|_|err("DATABASE_READ_FAILED"))?;
            let iter=stmt.query_map(params![goal.id,goal.revision,now_ms()?],
                |row|row.get::<_,String>(0)).map_err(|_|err("DATABASE_READ_FAILED"))?;
            iter.collect::<rusqlite::Result<Vec<String>>>()
                .map_err(|_|err("DATABASE_READ_FAILED"))?
        } else { Vec::new() };
        Ok(PrivateSharePreview {
            draft_id:goal.id,goal_text:goal.goal_text,
            lifecycle_state:goal.lifecycle_state,
            possible_concept_ids:possible,visibility:"private".into(),
            consent_recorded:false,can_upload:false,
            approval_status:"T01_AND_CONSENT_PENDING".into(),
            excluded_private_sources:vec!["original_image","raw_ocr",
                "corrected_ocr","private_notes","clarification_answers"],
        })
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
    fn us01a_end_to_end_text_model_candidate_human_confirmation_sqlite_reopen() {
        // 這個測試串起 B02 → B03 的候選 → B04 本人確認 → 真 SQLite，
        // 不能由「各自模組 CI PASS」冒充 User Story PASS。
        let dir = tempfile::tempdir().unwrap();
        let assets = LocalStore::open(dir.path()).unwrap();
        let source = "我想找人一起練 JLPT N2 口說";
        let asset = assets.save(text(source)).unwrap();
        let reply = tauri_plugin_cakami_model::NativeReply {
            raw_text: r#"{"evidenceQuote":"我想找人","contentHint":"想共同練日語口說",
                "intentCandidate":{"domain":"study",
                "goalText":"我想找真人學伴一起練 JLPT N2 口說",
                "evidenceQuote":"我想找人一起練"}}"#.into(),
            model_id: "gemma-4-E2B-it".into(), runtime: "litert-lm-android-0.16.1".into(),
            backend: "cpu".into(), elapsed_ms: 88,
        };
        let suggested = assets.save_model_suggestion(&asset.id, source, reply).unwrap();
        let candidate = suggested.intent_candidate.expect("private candidate");
        let goal = GoalStore::open(dir.path()).unwrap();
        assert!(goal.load(&asset.id).unwrap().is_none(),
            "Gemma cannot confirm before user action");
        let confirmed = goal.confirm(input(&asset.id, &candidate.goal_text, 0)).unwrap();
        assert_eq!(confirmed.goal_text, candidate.goal_text);
        assert_eq!(confirmed.goal_state, "confirmed_by_user");
        assert_eq!(confirmed.intent_state, "draft_private");
        assert_eq!(confirmed.visibility, "private");
        assert!(!confirmed.share_consent);
        assert!(!confirmed.publishable);
        let conn = goal.connect().unwrap();
        let events: i64 = conn.query_row(
            "SELECT count(*) FROM local_goal_confirmation_events WHERE draft_id=?1",
            params![confirmed.id], |r| r.get(0)).unwrap();
        assert_eq!(events, 1);
        drop(goal);
        drop(assets);
        let reopened = GoalStore::open(dir.path()).unwrap();
        assert_eq!(reopened.load(&asset.id).unwrap().unwrap().goal_text,
            candidate.goal_text);
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
            uncertainty: None, intent_candidate: None, pending_question: None,
            model_id: "gemma-4-E2B-it".into(),
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
    #[test]
    fn us01d_local_expiry_preview_and_cancel_cascade_to_b06_slots_offline() {
        use crate::parameter_registry::{
            ConfirmParameterInput,ParameterStore,SaveProposalInput
        };
        let dir=tempfile::tempdir().unwrap();
        let assets=LocalStore::open(dir.path()).unwrap();
        let asset=assets.save(text("我想找人一起練 JLPT N2 口說")).unwrap();
        let goals=GoalStore::open(dir.path()).unwrap();
        let goal=goals.confirm(input(&asset.id,"找 N2 口說學伴",0)).unwrap();
        let parameters=ParameterStore::open(dir.path()).unwrap();
        let p=parameters.save(SaveProposalInput {
            draft_id:goal.id.clone(),expected_draft_revision:1,
            concept_alias:"JLPT N2".into(),value:None,scope:"intent".into(),
            expiry_policy:"intent_lifecycle".into(),expires_at_ms:None,expected_version:0,
        }).unwrap();
        parameters.confirm(ConfirmParameterInput {
            draft_id:goal.id.clone(),proposal_id:p.id,expected_version:1,
            confirmation_action:"confirm_parameter".into(),
        }).unwrap();
        let preview=goals.share_preview(&asset.id).unwrap();
        assert_eq!(preview.possible_concept_ids,vec!["study.jlpt_level"]);
        assert!(!preview.can_upload);
        assert!(!preview.consent_recorded);
        assert_eq!(preview.visibility,"private");
        assert!(!preview.excluded_private_sources.contains(&"local_matching_slots"));
        let expiry=now_ms().unwrap()+86_400_000;
        let dated=goals.update_lifecycle(LifecycleUpdateInput {
            asset_id:asset.id.clone(),expected_revision:1,expected_lifecycle_version:0,
            action:"set_expiry".into(),expires_at_ms:Some(expiry),
        }).unwrap();
        assert_eq!(dated.expires_at_ms,Some(expiry));
        assert_eq!(dated.lifecycle_state,"active_private");
        assert_eq!(dated.lifecycle_version,1);
        let withdrawn=goals.update_lifecycle(LifecycleUpdateInput {
            asset_id:asset.id.clone(),expected_revision:1,expected_lifecycle_version:1,
            action:"cancel_goal".into(),expires_at_ms:None,
        }).unwrap();
        assert_eq!(withdrawn.lifecycle_state,"cancelled_private");
        assert!(!withdrawn.publishable);
        assert!(!withdrawn.share_consent);
        let no_share=goals.share_preview(&asset.id).unwrap();
        assert!(no_share.possible_concept_ids.is_empty());
        assert!(!no_share.can_upload);
        assert_eq!(goals.confirm(input(&asset.id,"復活",1)).unwrap_err(),
            "LIFECYCLE_INACTIVE");
        assert_eq!(goals.update_lifecycle(LifecycleUpdateInput {
            asset_id:asset.id.clone(),expected_revision:1,expected_lifecycle_version:2,
            action:"set_expiry".into(),expires_at_ms:Some(now_ms().unwrap()+86_400_000),
        }).unwrap_err(),"LIFECYCLE_INACTIVE");
        let conn=goals.connect().unwrap();
        let slots:i64=conn.query_row("SELECT count(*) FROM local_matching_slots",[],
            |r|r.get(0)).unwrap();
        assert_eq!(slots,0,"Cancellation must immediately revoke B06 private slots");
        drop(goals);drop(parameters);drop(assets);
        let reopened=GoalStore::open(dir.path()).unwrap();
        assert_eq!(reopened.load(&asset.id).unwrap().unwrap().lifecycle_state,
            "cancelled_private");
        assert!(!reopened.share_preview(&asset.id).unwrap().can_upload);
    }

    #[test]
    fn us01d_expired_goal_blocks_parameter_admission_without_cloud() {
        let dir=tempfile::tempdir().unwrap();
        let assets=LocalStore::open(dir.path()).unwrap();
        let asset=assets.save(text("N2")).unwrap();
        let goals=GoalStore::open(dir.path()).unwrap();
        let goal=goals.confirm(input(&asset.id,"找 N2 學伴",0)).unwrap();
        goals.update_lifecycle(LifecycleUpdateInput {
            asset_id:asset.id.clone(),expected_revision:1,expected_lifecycle_version:0,
            action:"set_expiry".into(),expires_at_ms:Some(now_ms().unwrap()+86_400_000),
        }).unwrap();
        // Simulate the wall clock after expiry without sleeping or networking.
        let conn=goals.connect().unwrap();
        conn.execute("UPDATE local_goal_lifecycle SET expires_at_ms=1 WHERE draft_id=?1",
            params![goal.id]).unwrap();
        let expired=goals.load(&asset.id).unwrap().unwrap();
        assert_eq!(expired.lifecycle_state,"expired_private");
        assert!(!expired.publishable);
        assert!(goals.share_preview(&asset.id).unwrap().possible_concept_ids.is_empty());
        let p=crate::parameter_registry::ParameterStore::open(dir.path()).unwrap();
        assert_eq!(p.save(crate::parameter_registry::SaveProposalInput {
            draft_id:goal.id,expected_draft_revision:1,concept_alias:"N2".into(),
            value:None,scope:"intent".into(),expiry_policy:"intent_lifecycle".into(),
            expires_at_ms:None,expected_version:0,
        }).unwrap_err(),"GOAL_CONFIRMATION_REQUIRED");
    }

    #[test]
    fn us01d_rejects_forged_consent_deadline_and_stale_user_revision() {
        let dir=tempfile::tempdir().unwrap();
        let asset=LocalStore::open(dir.path()).unwrap()
            .save(text("N2")).unwrap();
        let goals=GoalStore::open(dir.path()).unwrap();
        let goal=goals.confirm(input(&asset.id,"我要練 N2",0)).unwrap();
        assert!(serde_json::from_value::<LifecycleUpdateInput>(serde_json::json!({
            "assetId":asset.id,"expectedRevision":1,"expectedLifecycleVersion":0,
            "action":"set_expiry","expiresAtMs":now_ms().unwrap()+100_000,
            "consent":true
        })).is_err());
        for (action,expiry) in [
            ("set_expiry",None),("cancel_goal",Some(now_ms().unwrap()+50_000)),
            ("publish",None),("set_expiry",Some(1)),
        ] {
            assert_eq!(goals.update_lifecycle(LifecycleUpdateInput {
                asset_id:asset.id.clone(),expected_revision:1,
                expected_lifecycle_version:0,action:action.into(),
                expires_at_ms:expiry,
            }).unwrap_err(),"LIFECYCLE_INVALID_ACTION");
        }
        assert_eq!(goals.update_lifecycle(LifecycleUpdateInput {
            asset_id:asset.id.clone(),expected_revision:2,expected_lifecycle_version:0,
            action:"cancel_goal".into(),expires_at_ms:None,
        }).unwrap_err(),"GOAL_REVISION_CONFLICT");
        assert_eq!(goals.load(&asset.id).unwrap().unwrap().id,goal.id);
        assert_eq!(goals.load(&asset.id).unwrap().unwrap().lifecycle_version,0);
    }

}
