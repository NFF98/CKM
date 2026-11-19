//! B08 / Sprint 1: explicit, revocable, per-field LOCAL share-review record.
//! This is NOT Cloud consent or publication. B12 must independently authorize
//! a versioned server payload after T-01 and Cloud schema are approved.
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;

fn err(value: &'static str) -> String { value.into() }
fn clock_ms() -> Result<i64, String> {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|_| err("CLOCK_UNAVAILABLE"))?.as_millis())
        .map_err(|_| err("CLOCK_OVERFLOW"))
}
fn uuid(raw: &str) -> Result<String, String> {
    Uuid::parse_str(raw).map(|v| v.to_string()).map_err(|_|err("INVALID_ASSET_ID"))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct LocalShareReview {
    pub draft_id: String,
    pub goal_revision: i64,
    pub lifecycle_version: i64,
    pub reviewable_field_ids: Vec<String>,
    pub selected_field_ids: Vec<String>,
    pub consent_recorded: bool,
    pub consent_version: i64,
    pub can_upload: bool,
    pub visibility: &'static str,
    pub scope: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReviewInput {
    pub asset_id: String,
    pub expected_goal_revision: i64,
    pub expected_lifecycle_version: i64,
    pub expected_consent_version: i64,
    pub selected_field_ids: Vec<String>,
    pub confirmation_action: String,
}

#[derive(Debug)]
struct Snapshot {
    draft_id: String,
    goal_revision: i64,
    lifecycle_version: i64,
    active: bool,
    // Only T-01 approved, still-private fields can be reviewed.
    // Value includes source/event/version so edits invalidate old consent.
    fields: Vec<(String,String)>,
    source_hash: String,
}

impl Snapshot {
    fn selected_digest(&self, selected: &[String]) -> Result<String, String> {
        let values: Vec<(&str,&str)> = selected.iter().map(|field| {
            self.fields.iter().find(|(id,_)|id==field)
                .map(|(id,value)|(id.as_str(),value.as_str()))
                .ok_or_else(||err("B08_FIELD_NOT_APPROVED"))
        }).collect::<Result<_,_>>()?;
        let bytes=serde_json::to_vec(&(&self.draft_id,self.goal_revision,
            self.lifecycle_version,&self.source_hash,values))
            .map_err(|_|err("B08_INVALID_CONSENT"))?;
        Ok(format!("{:x}",Sha256::digest(bytes)))
    }
    fn allowed(&self) -> Vec<String> {
        if !self.active {return vec![];}
        self.fields.iter().map(|(id,_)|id.clone()).collect()
    }
}

pub struct LocalConsentStore { db_path: PathBuf }

impl LocalConsentStore {
    pub fn open(root: &Path) -> Result<Self,String> {
        super::parameter_registry::ParameterStore::open(root)?;
        let store=Self {db_path:root.join("cakami_private.sqlite")};
        let conn=store.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS local_share_preview_approvals (
               draft_id TEXT PRIMARY KEY REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
               version INTEGER NOT NULL CHECK(version>=1),
               goal_revision INTEGER NOT NULL CHECK(goal_revision>=1),
               lifecycle_version INTEGER NOT NULL CHECK(lifecycle_version>=0),
               state TEXT NOT NULL CHECK(state IN ('locally_reviewed','revoked')),
               field_ids_json TEXT NOT NULL,
               snapshot_digest TEXT NOT NULL CHECK(length(snapshot_digest)=64),
               recorded_at_ms INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS local_share_preview_events (
               id TEXT PRIMARY KEY NOT NULL,
               draft_id TEXT NOT NULL REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
               version INTEGER NOT NULL CHECK(version>=1),
               action TEXT NOT NULL CHECK(action IN ('approve_local_preview','revoke_local_preview')),
               origin TEXT NOT NULL CHECK(origin='explicit_user_action'),
               field_ids_json TEXT NOT NULL,
               at_ms INTEGER NOT NULL,
               UNIQUE(draft_id,version)
             );"
        ).map_err(|_|err("DATABASE_MIGRATION_FAILED"))?;
        Ok(store)
    }
    fn connect(&self)->Result<Connection,String>{
        let conn=Connection::open(&self.db_path).map_err(|_|err("DATABASE_UNAVAILABLE"))?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .map_err(|_|err("DATABASE_UNAVAILABLE"))?;
        Ok(conn)
    }
    fn snapshot(conn: &Connection, asset_id: &str)->Result<Snapshot,String>{
        let goal: Option<(String,i64,String,String,String,Option<i64>,i64)> =
            conn.query_row(
            "SELECT g.id,g.revision,g.goal_text,g.source_sha256,
                    coalesce(l.state,'active_private'),l.expires_at_ms,
                    coalesce(l.version,0)
             FROM local_goal_drafts g
             LEFT JOIN local_goal_lifecycle l ON l.draft_id=g.id
             WHERE g.asset_id=?1 AND g.visibility='private'
                   AND g.share_consent=0 AND g.intent_state='draft_private'",
            params![asset_id],
            |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,
                    r.get(4)?,r.get(5)?,r.get(6)?))
        ).optional().map_err(|_|err("DATABASE_READ_FAILED"))?;
        let (draft_id,goal_revision,goal_text,source_hash,lifecycle_state,expiry,
             lifecycle_version)=goal.ok_or_else(||err("GOAL_CONFIRMATION_REQUIRED"))?;
        let now=clock_ms()?;
        let active=lifecycle_state=="active_private"
            && expiry.is_none_or(|value|value>now);
        let mut fields=Vec::new();
        if active {
            fields.push(("goal_text".into(),goal_text));
            // Match only the ONE currently approved P1 concept. Never expose
            // OCR, images, clarifications, unknown values or local evidence.
            let parameter: Option<(String,i64,String,Option<i64>)>=conn.query_row(
                "SELECT p.value_json,p.version,p.confirmed_event_id,p.expires_at_ms
                 FROM local_matching_slots s
                 JOIN local_parameter_drafts p ON p.id=s.parameter_id
                 JOIN local_parameter_confirmation_events e ON e.id=p.confirmed_event_id
                 WHERE s.draft_id=?1 AND s.concept_id='study.jlpt_level'
                   AND s.goal_revision=?2 AND p.goal_revision=?2
                   AND p.concept_id='study.jlpt_level'
                   AND p.state='CONFIRMED' AND p.value_json='\"N2\"'
                   AND p.visibility='private' AND p.share_consent=0
                   AND p.source='user_explicit'
                   AND e.parameter_id=p.id AND e.version=p.version
                   AND (p.expires_at_ms IS NULL OR p.expires_at_ms>?3)",
                params![draft_id,goal_revision,now],
                |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)),
            ).optional().map_err(|_|err("DATABASE_READ_FAILED"))?;
            if let Some((value,version,event_id,deadline))=parameter {
                fields.push(("study.jlpt_level".into(),
                    format!("{value}:{version}:{event_id}:{deadline:?}")));
            }
        }
        Ok(Snapshot {draft_id,goal_revision,lifecycle_version,active,fields,source_hash})
    }
    fn stored(conn:&Connection,draft_id:&str)
        -> Result<Option<(i64,String,String,String)>,String>
    {
        conn.query_row(
            "SELECT version,state,field_ids_json,snapshot_digest
             FROM local_share_preview_approvals WHERE draft_id=?1",
            params![draft_id],
            |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
        ).optional().map_err(|_|err("DATABASE_READ_FAILED"))
    }
    fn view(conn:&Connection,snapshot:Snapshot)->Result<LocalShareReview,String>{
        let existing=Self::stored(conn,&snapshot.draft_id)?;
        let mut selected=Vec::new();
        let mut recorded=false;
        let version=existing.as_ref().map(|(v,_,_,_)|*v).unwrap_or(0);
        if snapshot.active {
            if let Some((_,state,json,digest))=existing {
                if state=="locally_reviewed" {
                    let claimed:Vec<String>=serde_json::from_str(&json)
                        .map_err(|_|err("DATABASE_READ_FAILED"))?;
                    if !claimed.is_empty()
                        && claimed.iter().all(|id|snapshot.fields.iter().any(|(key,_)|key==id))
                        && snapshot.selected_digest(&claimed)?==digest
                    {
                        recorded=true;
                        selected=claimed;
                    }
                }
            }
        }
        Ok(LocalShareReview {
            draft_id:snapshot.draft_id,
            goal_revision:snapshot.goal_revision,
            lifecycle_version:snapshot.lifecycle_version,
            reviewable_field_ids:snapshot.allowed(),
            selected_field_ids:selected,
            consent_recorded:recorded,
            consent_version:version,
            can_upload:false,
            visibility:"private",
            scope:"local_preview_only",
        })
    }
    pub fn load(&self,asset_id:&str)->Result<LocalShareReview,String>{
        let id=uuid(asset_id)?;
        let conn=self.connect()?;
        Self::view(&conn,Self::snapshot(&conn,&id)?)
    }
    pub fn review(&self,input:ReviewInput)->Result<LocalShareReview,String>{
        let id=uuid(&input.asset_id)?;
        if input.expected_goal_revision<1 || input.expected_lifecycle_version<0
            || input.expected_consent_version<0 {return Err(err("B08_STALE_REVIEW"));}
        let approve=input.confirmation_action=="approve_local_preview";
        let revoke=input.confirmation_action=="revoke_local_preview";
        if !approve && !revoke {return Err(err("B08_EXPLICIT_ACTION_REQUIRED"));}
        if (approve && (input.selected_field_ids.is_empty()
                || input.selected_field_ids.len()>2))
           || (revoke && !input.selected_field_ids.is_empty()) {
            return Err(err("B08_INVALID_FIELDS"));
        }
        let mut sorted=input.selected_field_ids.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len()!=input.selected_field_ids.len() {
            return Err(err("B08_INVALID_FIELDS"));
        }
        let mut conn=self.connect()?;
        let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_|err("DATABASE_BUSY"))?;
        let snapshot=Self::snapshot(&tx,&id)?;
        if snapshot.goal_revision!=input.expected_goal_revision
            || snapshot.lifecycle_version!=input.expected_lifecycle_version {
            return Err(err("B08_STALE_REVIEW"));
        }
        if approve && !snapshot.active {return Err(err("LIFECYCLE_INACTIVE"));}
        if approve && !sorted.iter().all(|id|
            snapshot.fields.iter().any(|(known,_)|known==id)) {
            return Err(err("B08_FIELD_NOT_APPROVED"));
        }
        let previous=Self::stored(&tx,&snapshot.draft_id)?;
        let current=previous.as_ref().map(|(v,_,_,_)|*v).unwrap_or(0);
        if current!=input.expected_consent_version {
            return Err(err("B08_STALE_REVIEW"));
        }
        if revoke && current==0 {return Err(err("B08_NOT_RECORDED"));}
        let version=current.checked_add(1).ok_or_else(||err("B08_STALE_REVIEW"))?;
        let state=if approve {"locally_reviewed"} else {"revoked"};
        let field_json=serde_json::to_string(&sorted)
            .map_err(|_|err("B08_INVALID_FIELDS"))?;
        let digest=if approve {snapshot.selected_digest(&sorted)?}
            else {"0".repeat(64)};
        let at=clock_ms()?;
        tx.execute(
            "INSERT INTO local_share_preview_approvals
               (draft_id,version,goal_revision,lifecycle_version,state,
                field_ids_json,snapshot_digest,recorded_at_ms)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(draft_id) DO UPDATE SET
                version=excluded.version,goal_revision=excluded.goal_revision,
                lifecycle_version=excluded.lifecycle_version,state=excluded.state,
                field_ids_json=excluded.field_ids_json,
                snapshot_digest=excluded.snapshot_digest,
                recorded_at_ms=excluded.recorded_at_ms",
            params![snapshot.draft_id,version,snapshot.goal_revision,
                snapshot.lifecycle_version,state,field_json,digest,at]
        ).map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        tx.execute(
            "INSERT INTO local_share_preview_events
               (id,draft_id,version,action,origin,field_ids_json,at_ms)
             VALUES (?1,?2,?3,?4,'explicit_user_action',?5,?6)",
            params![Uuid::new_v4().to_string(),snapshot.draft_id,version,
                input.confirmation_action,field_json,at]
        ).map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        tx.commit().map_err(|_|err("DATABASE_WRITE_FAILED"))?;
        self.load(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{goal_draft::{ConfirmGoalInput,GoalStore,LifecycleUpdateInput},
        local_store::{IncomingDrop,LocalStore},
        parameter_registry::{ConfirmParameterInput,ParameterStore,SaveProposalInput}};

    fn setup()->(tempfile::TempDir,String,String){
        let dir=tempfile::tempdir().unwrap();
        let asset=LocalStore::open(dir.path()).unwrap().save(IncomingDrop{
            kind:"text".into(),source:"text".into(),text:Some("我想找人練 JLPT N2".into()),
            base64:None,file_name:None,mime_type:None,
        }).unwrap();
        let goal=GoalStore::open(dir.path()).unwrap().confirm(ConfirmGoalInput{
            asset_id:asset.id.clone(),goal_text:"一起練 N2".into(),
            confirmation_action:"confirm_goal".into(),expected_revision:0,
        }).unwrap();
        (dir,asset.id,goal.id)
    }
    fn action(asset:&str,version:i64,selected:&[&str],verb:&str)->ReviewInput{
        ReviewInput {
            asset_id:asset.into(),expected_goal_revision:1,
            expected_lifecycle_version:0,expected_consent_version:version,
            selected_field_ids:selected.iter().map(|s|s.to_string()).collect(),
            confirmation_action:verb.into()
        }
    }
    #[test]
    fn b08_defaults_private_and_no_local_or_cloud_consent(){
        let (dir,asset,_)=setup();
        let store=LocalConsentStore::open(dir.path()).unwrap();
        let view=store.load(&asset).unwrap();
        assert_eq!(view.reviewable_field_ids,vec!["goal_text"]);
        assert!(!view.consent_recorded);
        assert_eq!(view.consent_version,0);
        assert!(!view.can_upload);
        assert_eq!(view.visibility,"private");
    }
    #[test]
    fn b08_per_field_user_review_is_local_versioned_and_revocable(){
        let (dir,asset,_)=setup();
        let store=LocalConsentStore::open(dir.path()).unwrap();
        assert_eq!(store.review(action(&asset,0,&["study.jlpt_level"],
            "approve_local_preview")).unwrap_err(),"B08_FIELD_NOT_APPROVED");
        let granted=store.review(action(&asset,0,&["goal_text"],
            "approve_local_preview")).unwrap();
        assert!(granted.consent_recorded);
        assert_eq!(granted.selected_field_ids,vec!["goal_text"]);
        assert_eq!(granted.consent_version,1);
        assert!(!granted.can_upload);
        assert_eq!(store.review(action(&asset,0,&["goal_text"],
            "approve_local_preview")).unwrap_err(),"B08_STALE_REVIEW");
        drop(store);
        let reopened=LocalConsentStore::open(dir.path()).unwrap();
        assert!(reopened.load(&asset).unwrap().consent_recorded);
        let revoked=reopened.review(action(&asset,1,&[],
            "revoke_local_preview")).unwrap();
        assert_eq!(revoked.consent_version,2);
        assert!(!revoked.consent_recorded);
        let conn=reopened.connect().unwrap();
        let count:i64=conn.query_row("SELECT count(*) FROM local_share_preview_events",[],
            |r|r.get(0)).unwrap();
        assert_eq!(count,2);
    }
    #[test]
    fn b08_approved_n2_field_only_after_actual_parameter_confirmation(){
        let (dir,asset,draft)=setup();
        let store=LocalConsentStore::open(dir.path()).unwrap();
        let p=ParameterStore::open(dir.path()).unwrap();
        let observed=p.save(SaveProposalInput {
            draft_id:draft.clone(),expected_draft_revision:1,
            concept_alias:"N2".into(),value:None,scope:"intent".into(),
            expiry_policy:"intent_lifecycle".into(),expires_at_ms:None,
            expected_version:0,
        }).unwrap();
        assert_eq!(store.load(&asset).unwrap().reviewable_field_ids,vec!["goal_text"]);
        p.confirm(ConfirmParameterInput {
            draft_id:draft,proposal_id:observed.id.clone(),expected_version:1,
            confirmation_action:"confirm_parameter".into()
        }).unwrap();
        assert_eq!(store.load(&asset).unwrap().reviewable_field_ids,
            vec!["goal_text","study.jlpt_level"]);
        let consent=store.review(action(&asset,0,&["study.jlpt_level"],
            "approve_local_preview")).unwrap();
        assert!(consent.consent_recorded);
        assert!(!consent.can_upload);
        // Editing the parameter removes its B06 slot and invalidates the old review.
        p.save(SaveProposalInput {
            draft_id:consent.draft_id,expected_draft_revision:1,
            concept_alias:"JLPT N2".into(),value:None,scope:"intent".into(),
            expiry_policy:"intent_lifecycle".into(),expires_at_ms:None,
            expected_version:2,
        }).unwrap();
        assert!(!store.load(&asset).unwrap().consent_recorded);
    }
    #[test]
    fn b08_revision_cancel_expiry_and_private_source_delete_invalidate_review(){
        let (dir,asset,draft)=setup();
        let store=LocalConsentStore::open(dir.path()).unwrap();
        store.review(action(&asset,0,&["goal_text"],"approve_local_preview")).unwrap();
        let goals=GoalStore::open(dir.path()).unwrap();
        goals.confirm(ConfirmGoalInput {
            asset_id:asset.clone(),goal_text:"改為只練日文聽力".into(),
            confirmation_action:"confirm_goal".into(),expected_revision:1
        }).unwrap();
        let invalid=store.load(&asset).unwrap();
        assert!(!invalid.consent_recorded);
        let mut stale=action(&asset,1,&["goal_text"],"approve_local_preview");
        assert_eq!(store.review(stale).unwrap_err(),"B08_STALE_REVIEW");
        stale.expected_goal_revision=2;
        store.review(stale).unwrap();
        assert!(store.load(&asset).unwrap().consent_recorded);
        goals.update_lifecycle(LifecycleUpdateInput {
            asset_id:asset.clone(),expected_revision:2,
            expected_lifecycle_version:0,action:"cancel_goal".into(),expires_at_ms:None
        }).unwrap();
        assert!(!store.load(&asset).unwrap().consent_recorded);
        assert!(store.load(&asset).unwrap().reviewable_field_ids.is_empty());
        let mut revoke=action(&asset,2,&[],"revoke_local_preview");
        revoke.expected_goal_revision=2;revoke.expected_lifecycle_version=1;
        store.review(revoke).unwrap();
        LocalStore::open(dir.path()).unwrap().delete(&asset).unwrap();
        let conn=store.connect().unwrap();
        for table in ["local_share_preview_approvals","local_share_preview_events"] {
            let sql=format!("SELECT count(*) FROM {table}");
            let n:i64=conn.query_row(&sql,[],|r|r.get(0)).unwrap();
            assert_eq!(n,0,"{table} must cascade with private source");
        }
        assert!(!draft.is_empty());
    }
    #[test]
    fn b08_rejects_forged_actions_unknown_fields_and_overposting(){
        let (dir,asset,_)=setup();
        let store=LocalConsentStore::open(dir.path()).unwrap();
        for (fields,action) in [
            (vec!["raw_ocr"],"approve_local_preview"),
            (vec!["private_notes"],"approve_local_preview"),
            (vec!["goal_text","goal_text"],"approve_local_preview"),
            (vec!["goal_text"],"publish"),
            (vec!["goal_text"],"revoke_local_preview"),
        ] {
            assert!(store.review(ReviewInput {
                asset_id:asset.clone(),expected_goal_revision:1,
                expected_lifecycle_version:0,expected_consent_version:0,
                selected_field_ids:fields.iter().map(|s|s.to_string()).collect(),
                confirmation_action:action.into()
            }).is_err());
        }
        assert!(serde_json::from_value::<ReviewInput>(serde_json::json!({
            "assetId":asset,"expectedGoalRevision":1,"expectedLifecycleVersion":0,
            "expectedConsentVersion":0,"selectedFieldIds":["goal_text"],
            "confirmationAction":"approve_local_preview","canUpload":true
        })).is_err());
        assert_eq!(store.load(&asset).unwrap().consent_version,0);
    }
}
