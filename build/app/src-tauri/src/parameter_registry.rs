//! B05 / S1-AC05 — versioned private Parameter Concept Registry and Rust validation.
//!
//! This is NOT B06's 8-slot matching schema. T-01's full whitelist is OPEN.
//! Only the already-specified study.jlpt_level / N2 aliases are registered.
//! All other concepts remain private UNKNOWN proposals and can never be confirmed,
//! shared or used in matching. No model path can call the user confirmation command.
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use uuid::Uuid;

const REGISTRY_VERSION: i64 = 1;
const MAX_ALIAS_BYTES: usize = 160;
const MAX_VALUE_BYTES: usize = 512;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryConcept {
    pub concept_id: &'static str,
    pub aliases: Vec<&'static str>,
    pub value_type: &'static str,
    pub allowed_scopes: Vec<&'static str>,
    pub expiry_policy: &'static str,
    pub match_usage: &'static str,
    pub registry_version: i64,
}

pub fn registry() -> Vec<RegistryConcept> {
    vec![RegistryConcept {
        concept_id: "study.jlpt_level",
        aliases: vec!["N2", "JLPT N2", "日語二級"],
        value_type: "enum",
        allowed_scopes: vec!["intent"],
        expiry_policy: "intent_lifecycle",
        match_usage: "pending_T01_review",
        registry_version: REGISTRY_VERSION,
    }]
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveProposalInput {
    pub draft_id: String,
    pub expected_draft_revision: i64,
    pub concept_alias: String,
    #[serde(default)]
    pub value: Option<String>,
    pub scope: String,
    pub expiry_policy: String,
    #[serde(default)]
    pub expires_at_ms: Option<i64>,
    pub expected_version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmParameterInput {
    pub draft_id: String,
    pub proposal_id: String,
    pub expected_version: i64,
    pub confirmation_action: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrivateParameter {
    pub id: String,
    pub draft_id: String,
    pub concept_id: Option<String>,
    pub requested_alias: String,
    pub value_type: String,
    pub value_json: Option<String>,
    pub raw_value: Option<String>,
    pub source: String,
    pub evidence_ref: String,
    pub scope: String,
    pub state: String,
    pub expiry_policy: String,
    pub expires_at_ms: Option<i64>,
    pub review_due_at_ms: Option<i64>,
    pub visibility: String,
    pub share_consent: bool,
    pub confirmed_event_id: Option<String>,
    pub registry_version: i64,
    pub goal_revision: i64,
    pub version: i64,
    pub stale: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug)]
struct Normalized {
    concept_id: Option<&'static str>,
    value_type: &'static str,
    value_json: Option<String>,
    raw_value: Option<String>,
    state: &'static str,
    registry_version: i64,
}

fn err(s: &'static str) -> String { s.into() }

fn clock_ms() -> Result<i64, String> {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|_| err("CLOCK_UNAVAILABLE"))?.as_millis())
        .map_err(|_| err("CLOCK_OVERFLOW"))
}

fn uuid(raw: &str) -> Result<String, String> {
    Uuid::parse_str(raw).map(|v| v.to_string()).map_err(|_| err("INVALID_DRAFT_ID"))
}

fn alias_key(raw: &str) -> String {
    raw.chars().filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase()).collect()
}

fn n2_alias(raw: &str) -> bool {
    matches!(alias_key(raw).as_str(),
        "n2" | "jlptn2" | "日語二級" | "日文二級" |
        "日本語能力試験n2" | "日本語能力試驗n2")
}

fn clean(raw: &str, max: usize) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() || value.len() > max ||
        value.chars().any(|c| c.is_control() && c != '\n')
    {
        return Err(err("INVALID_PARAMETER_INPUT"));
    }
    Ok(value.into())
}

/// Pure deterministic normalization, no model and no dynamic registry extension.
fn normalize(alias: &str, value: Option<&str>) -> Result<Normalized, String> {
    let alias = clean(alias, MAX_ALIAS_BYTES)?;
    let explicit = value.map(|v| clean(v, MAX_VALUE_BYTES)).transpose()?;
    if alias == "study.jlpt_level" || n2_alias(&alias) {
        if let Some(v) = explicit.as_deref() {
            if !n2_alias(v) { return Err(err("INVALID_TYPE")); }
        }
        if explicit.is_none() && alias == "study.jlpt_level" {
            return Ok(Normalized {
                concept_id: Some("study.jlpt_level"), value_type: "enum",
                value_json: None, raw_value: None, state: "UNKNOWN",
                registry_version: REGISTRY_VERSION,
            });
        }
        return Ok(Normalized {
            concept_id: Some("study.jlpt_level"), value_type: "enum",
            value_json: Some(r#""N2""#.into()),
            raw_value: Some("N2".into()), state: "OBSERVED",
            registry_version: REGISTRY_VERSION,
        });
    }
    Ok(Normalized {
        concept_id: None, value_type: "unknown", value_json: None,
        raw_value: explicit.or_else(|| Some(alias)),
        state: "UNKNOWN", registry_version: 0,
    })
}

fn validate_expiry(policy: &str, expires: Option<i64>, now: i64) -> Result<(), String> {
    match (policy, expires) {
        ("intent_lifecycle", None) => Ok(()),
        ("fixed", Some(ms)) if ms > now => Ok(()),
        // Never invent a date for "Saturday" or silently make it recurring.
        _ => Err(err("INVALID_EXPIRY")),
    }
}

pub struct ParameterStore { path: PathBuf }

impl ParameterStore {
    pub fn open(root: &Path) -> Result<Self, String> {
        // The B04 user-confirmed Goal must own this private draft.
        super::goal_draft::GoalStore::open(root)?;
        let store = Self { path: root.join("cakami_private.sqlite") };
        let conn = store.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS local_parameter_evidence (
                id TEXT PRIMARY KEY NOT NULL,
                draft_id TEXT NOT NULL REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
                source TEXT NOT NULL CHECK(source='user_explicit'),
                raw_input TEXT NOT NULL CHECK(length(trim(raw_input)) > 0),
                created_at_ms INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS local_parameter_drafts (
                id TEXT PRIMARY KEY NOT NULL,
                draft_id TEXT NOT NULL REFERENCES local_goal_drafts(id) ON DELETE CASCADE,
                concept_id TEXT,
                requested_alias TEXT NOT NULL CHECK(length(trim(requested_alias)) > 0),
                value_type TEXT NOT NULL CHECK(value_type IN ('enum','unknown')),
                value_json TEXT,
                raw_value TEXT,
                source TEXT NOT NULL CHECK(source='user_explicit'),
                evidence_ref TEXT NOT NULL REFERENCES local_parameter_evidence(id)
                    DEFERRABLE INITIALLY DEFERRED,
                scope TEXT NOT NULL CHECK(scope='intent'),
                state TEXT NOT NULL CHECK(state IN ('OBSERVED','UNKNOWN','CONFIRMED')),
                expiry_policy TEXT NOT NULL
                    CHECK(expiry_policy IN ('fixed','intent_lifecycle')),
                expires_at_ms INTEGER,
                review_due_at_ms INTEGER CHECK(review_due_at_ms IS NULL),
                visibility TEXT NOT NULL CHECK(visibility='private'),
                share_consent INTEGER NOT NULL CHECK(share_consent=0),
                confirmed_event_id TEXT
                    REFERENCES local_parameter_confirmation_events(id)
                    DEFERRABLE INITIALLY DEFERRED,
                registry_version INTEGER NOT NULL CHECK(registry_version IN (0,1)),
                goal_revision INTEGER NOT NULL CHECK(goal_revision >= 1),
                version INTEGER NOT NULL CHECK(version >= 1),
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL,
                CHECK((expiry_policy='fixed' AND expires_at_ms > 0)
                    OR (expiry_policy='intent_lifecycle' AND expires_at_ms IS NULL)),
                CHECK((state='CONFIRMED' AND confirmed_event_id IS NOT NULL)
                    OR (state!='CONFIRMED' AND confirmed_event_id IS NULL)),
                CHECK((concept_id IS NULL AND state='UNKNOWN' AND value_type='unknown'
                       AND value_json IS NULL AND registry_version=0)
                    OR (concept_id='study.jlpt_level' AND value_type='enum'
                        AND registry_version=1
                        AND ((value_json='\"N2\"' AND state IN ('OBSERVED','CONFIRMED'))
                          OR (value_json IS NULL AND state='UNKNOWN'))))
             );
             CREATE TABLE IF NOT EXISTS local_parameter_confirmation_events (
                id TEXT PRIMARY KEY NOT NULL,
                parameter_id TEXT NOT NULL
                    REFERENCES local_parameter_drafts(id) ON DELETE CASCADE,
                version INTEGER NOT NULL CHECK(version >= 2),
                action TEXT NOT NULL CHECK(action='explicit_parameter_confirmation'),
                source TEXT NOT NULL CHECK(source='user_explicit'),
                evidence_ref TEXT NOT NULL REFERENCES local_parameter_evidence(id),
                at_ms INTEGER NOT NULL,
                UNIQUE(parameter_id,version)
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_parameter_known_concept
               ON local_parameter_drafts(draft_id,concept_id)
               WHERE concept_id IS NOT NULL;
             CREATE INDEX IF NOT EXISTS idx_parameter_draft
               ON local_parameter_drafts(draft_id,updated_at_ms DESC);"
        ).map_err(|_| err("DATABASE_MIGRATION_FAILED"))?;
        Ok(store)
    }

    fn connect(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.path)
            .map_err(|_| err("DATABASE_UNAVAILABLE"))?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
            .map_err(|_| err("DATABASE_UNAVAILABLE"))?;
        Ok(conn)
    }

    fn goal_revision(conn: &Connection, draft_id: &str) -> Result<i64, String> {
        conn.query_row(
            "SELECT revision FROM local_goal_drafts
             WHERE id=?1 AND intent_state='draft_private' AND visibility='private'
               AND share_consent=0",
            params![draft_id], |row| row.get(0),
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?
            .ok_or_else(|| err("GOAL_CONFIRMATION_REQUIRED"))
    }

    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PrivateParameter> {
        let version: i64 = row.get(19)?;
        let current_goal_revision: i64 = row.get(22)?;
        let goal_revision: i64 = row.get(18)?;
        Ok(PrivateParameter {
            id: row.get(0)?, draft_id: row.get(1)?, concept_id: row.get(2)?,
            requested_alias: row.get(3)?, value_type: row.get(4)?,
            value_json: row.get(5)?, raw_value: row.get(6)?,
            source: row.get(7)?, evidence_ref: row.get(8)?, scope: row.get(9)?,
            state: row.get(10)?, expiry_policy: row.get(11)?, expires_at_ms: row.get(12)?,
            review_due_at_ms: row.get(13)?, visibility: row.get(14)?,
            share_consent: row.get::<_, i64>(15)? != 0,
            confirmed_event_id: row.get(16)?,
            registry_version: row.get(17)?, goal_revision,
            version, stale: goal_revision != current_goal_revision,
            created_at_ms: row.get(20)?, updated_at_ms: row.get(21)?,
        })
    }

    pub fn list(&self, draft_id: &str) -> Result<Vec<PrivateParameter>, String> {
        let draft_id = uuid(draft_id)?;
        let conn = self.connect()?;
        Self::goal_revision(&conn, &draft_id)?;
        let mut stmt = conn.prepare(
            "SELECT p.id,p.draft_id,p.concept_id,p.requested_alias,p.value_type,
                    p.value_json,p.raw_value,p.source,p.evidence_ref,p.scope,p.state,
                    p.expiry_policy,p.expires_at_ms,p.review_due_at_ms,p.visibility,
                    p.share_consent,p.confirmed_event_id,p.registry_version,
                    p.goal_revision,p.version,p.created_at_ms,p.updated_at_ms,
                    g.revision
             FROM local_parameter_drafts p
             JOIN local_goal_drafts g ON g.id=p.draft_id
             WHERE p.draft_id=?1 ORDER BY p.updated_at_ms DESC,p.id DESC"
        ).map_err(|_| err("DATABASE_READ_FAILED"))?;
        stmt.query_map(params![draft_id], Self::from_row)
            .map_err(|_| err("DATABASE_READ_FAILED"))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| err("DATABASE_READ_FAILED"))
    }

    /// User-entered private proposal. No input fields for source, state, event,
    /// owner, visibility, consent or match slot; serde deny_unknown_fields.
    pub fn save(&self, input: SaveProposalInput) -> Result<PrivateParameter, String> {
        let draft_id = uuid(&input.draft_id)?;
        let alias = clean(&input.concept_alias, MAX_ALIAS_BYTES)?;
        let normalized = normalize(&alias, input.value.as_deref())?;
        if input.scope != "intent" { return Err(err("INVALID_SCOPE")); }
        if input.expected_version < 0 || input.expected_draft_revision < 1 {
            return Err(err("INVALID_VERSION"));
        }
        let now = clock_ms()?;
        validate_expiry(&input.expiry_policy, input.expires_at_ms, now)?;
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| err("DATABASE_BUSY"))?;
        let goal_revision = Self::goal_revision(&tx, &draft_id)?;
        if goal_revision != input.expected_draft_revision {
            return Err(err("GOAL_REVISION_CONFLICT"));
        }
        let key = normalized.concept_id.unwrap_or(&alias);
        let prior: Option<(String, i64, i64)> = tx.query_row(
            "SELECT id,version,created_at_ms
             FROM local_parameter_drafts
             WHERE draft_id=?1 AND
               ((concept_id=?2 AND concept_id IS NOT NULL)
                OR (concept_id IS NULL AND requested_alias=?2))",
            params![draft_id,key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?;
        let (id,version,created_at) = if let Some((id,current,created)) = prior {
            if input.expected_version != current { return Err(err("PARAMETER_VERSION_CONFLICT")); }
            (id,current.checked_add(1).ok_or_else(|| err("INVALID_VERSION"))?,created)
        } else {
            if input.expected_version != 0 { return Err(err("PARAMETER_VERSION_CONFLICT")); }
            (Uuid::new_v4().to_string(),1,now)
        };
        let evidence_id = Uuid::new_v4().to_string();
        let raw_input = format!("{}{}",alias,input.value.as_ref()
            .map(|v| format!(": {}",v)).unwrap_or_default());
        tx.execute(
            "INSERT INTO local_parameter_evidence (id,draft_id,source,raw_input,created_at_ms)
             VALUES (?1,?2,'user_explicit',?3,?4)",
            params![evidence_id,draft_id,raw_input,now],
        ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        let concept_id = normalized.concept_id;
        if version == 1 {
            tx.execute(
                "INSERT INTO local_parameter_drafts
                  (id,draft_id,concept_id,requested_alias,value_type,value_json,
                   raw_value,source,evidence_ref,scope,state,expiry_policy,expires_at_ms,
                   review_due_at_ms,visibility,share_consent,confirmed_event_id,
                   registry_version,goal_revision,version,created_at_ms,updated_at_ms)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,'user_explicit',?8,'intent',?9,?10,?11,
                         NULL,'private',0,NULL,?12,?13,1,?14,?14)",
                params![id,draft_id,concept_id,alias,normalized.value_type,
                    normalized.value_json,normalized.raw_value,evidence_id,
                    normalized.state,input.expiry_policy,input.expires_at_ms,
                    normalized.registry_version,goal_revision,created_at],
            ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        } else {
            tx.execute(
                "UPDATE local_parameter_drafts
                 SET requested_alias=?1,value_type=?2,value_json=?3,raw_value=?4,
                     evidence_ref=?5,state=?6,expiry_policy=?7,expires_at_ms=?8,
                     confirmed_event_id=NULL,registry_version=?9,goal_revision=?10,
                     version=?11,updated_at_ms=?12
                 WHERE id=?13 AND version=?14",
                params![alias,normalized.value_type,normalized.value_json,
                    normalized.raw_value,evidence_id,normalized.state,
                    input.expiry_policy,input.expires_at_ms,normalized.registry_version,
                    goal_revision,version,now,id,version-1],
            ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        }
        tx.commit().map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        self.list(&draft_id)?.into_iter().find(|x| x.id==id)
            .ok_or_else(|| err("DATABASE_READ_FAILED"))
    }

    /// Only this explicit user action can change known OBSERVED into CONFIRMED.
    /// Rust creates the event, in the same transaction as the state transition.
    pub fn confirm(&self, input: ConfirmParameterInput) -> Result<PrivateParameter,String> {
        if input.confirmation_action != "confirm_parameter" {
            return Err(err("PARAMETER_CONFIRMATION_REQUIRED"));
        }
        let draft_id = uuid(&input.draft_id)?;
        let id = uuid(&input.proposal_id)?;
        if input.expected_version < 1 { return Err(err("INVALID_VERSION")); }
        let now=clock_ms()?;
        let mut conn=self.connect()?;
        let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| err("DATABASE_BUSY"))?;
        let goal_revision=Self::goal_revision(&tx,&draft_id)?;
        let row: Option<(String,Option<String>,Option<String>,String,i64,i64,
            Option<i64>,String)> = tx.query_row(
            "SELECT state,concept_id,value_json,evidence_ref,version,goal_revision,
                    expires_at_ms,expiry_policy
             FROM local_parameter_drafts WHERE id=?1 AND draft_id=?2",
            params![id,draft_id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,
                    r.get(5)?,r.get(6)?,r.get(7)?))
        ).optional().map_err(|_| err("DATABASE_READ_FAILED"))?;
        let (state,concept,value,evidence,version,source_goal_revision,expires,policy)=
            row.ok_or_else(|| err("PARAMETER_NOT_FOUND"))?;
        if version != input.expected_version { return Err(err("PARAMETER_VERSION_CONFLICT")); }
        if source_goal_revision != goal_revision { return Err(err("GOAL_REVISION_CONFLICT")); }
        if concept.is_none() || value.is_none() || state == "UNKNOWN" {
            return Err(err("PARAMETER_UNAPPROVED_CONCEPT"));
        }
        validate_expiry(&policy,expires,now)?;
        let new_version=version.checked_add(1).ok_or_else(|| err("INVALID_VERSION"))?;
        let event_id=Uuid::new_v4().to_string();
        let updated=tx.execute(
            "UPDATE local_parameter_drafts SET state='CONFIRMED',
                    confirmed_event_id=?1,version=?2,updated_at_ms=?3
             WHERE id=?4 AND draft_id=?5 AND version=?6",
            params![event_id,new_version,now,id,draft_id,version],
        ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        if updated!=1 { return Err(err("PARAMETER_VERSION_CONFLICT")); }
        tx.execute(
            "INSERT INTO local_parameter_confirmation_events
              (id,parameter_id,version,action,source,evidence_ref,at_ms)
             VALUES (?1,?2,?3,'explicit_parameter_confirmation','user_explicit',?4,?5)",
            params![event_id,id,new_version,evidence,now],
        ).map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        tx.commit().map_err(|_| err("DATABASE_WRITE_FAILED"))?;
        self.list(&draft_id)?.into_iter().find(|x| x.id==id)
            .ok_or_else(|| err("DATABASE_READ_FAILED"))
    }
}
