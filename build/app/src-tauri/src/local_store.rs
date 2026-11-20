//! B02 — Android/iOS 共用的 App-private SQLite + 原圖檔案儲存。
//! 不包含網路服務、雲端 OCR、Intent 建立或授權發布。
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;
use tauri_plugin_cakami_ocr::{NativeOcrResult, OcrSegment};
use tauri_plugin_cakami_model::NativeReply;

// 暫時的工程安全上限；正式批次、格式和容量策略屬 F-01 待決，不是已核准產品規則。
pub const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
const LIST_LIMIT: i64 = 100;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingDrop {
    pub kind: String,
    pub source: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub base64: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DropSummary {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub file_name: Option<String>,
    pub mime_type: String,
    pub byte_length: usize,
    pub created_at_ms: i64,
    pub text_excerpt: Option<String>,
    pub is_duplicate: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropDetail {
    pub summary: DropSummary,
    pub text: Option<String>,
    pub data_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRecord {
    pub asset_id: String,
    /// "not_run"／"recognized"／"unknown"；沒有任何假 OCR 文本。
    pub status: String,
    pub raw_text: Option<String>,
    pub corrected_text: Option<String>,
    pub engine: Option<String>,
    pub engine_version: Option<String>,
    pub segments: Vec<OcrSegment>,
    pub elapsed_ms: Option<u64>,
    pub updated_at_ms: Option<i64>,
}

/// B09: only a private, evidence-versioned question, not a Goal or matching Concept.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PendingIntentQuestion {
    pub ordinal: i64,
    pub text: String,
}

/// US-01A: an unconfirmed, private suggestion. It is neither a Goal event nor Consent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PrivateIntentCandidate {
    pub domain: String,
    pub goal_text: String,
    pub evidence_quote: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelSuggestion {
    pub asset_id: String,
    pub status: String,
    pub evidence_quote: String,
    pub content_hint: Option<String>,
    pub uncertainty: Option<String>,
    pub intent_candidate: Option<PrivateIntentCandidate>,
    pub intent_candidates: Vec<PrivateIntentCandidate>,
    pub pending_question: Option<PendingIntentQuestion>,
    pub model_id: String,
    pub runtime: String,
    pub backend: String,
    pub elapsed_ms: u64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawSuggestion {
    // Missing quote? Rust may quote the saved original, never an AI guess.
    evidence_quote: Option<String>,
    content_hint: Option<String>,
    uncertainty: Option<String>,
    intent_candidate: Option<PrivateIntentCandidate>,
    #[serde(default)]
    intent_candidates: Option<Vec<PrivateIntentCandidate>>,
    #[serde(default)]
    clarifying_question: Option<String>,
}

/// Rust deliberately requires first-person collaborative intent evidence before offering
/// a matching Goal. Topic-only content (including "パクる") must remain UNKNOWN.
/// This conservative gate can abstain; it never creates or confirms a Goal.
fn explicit_collaboration_intent(source: &str) -> bool {
    let s = source.to_lowercase();
    let wants = ["我想", "我要", "我希望", "想找", "希望找", "想和", "想跟",
        "i want", "i need", "i'd like", "looking for", "i am looking",
        "探してい", "探したい", "一緒にしたい", "一緒に勉強したい"]
        .iter().any(|needle| s.contains(needle));
    let together = ["找人", "找個人", "學伴", "伙伴", "夥伴", "一起", "合作",
        "搭子", "找真人", "和人", "跟人", "partner", "someone",
        "with others", "with someone", "together", "一緒", "仲間"]
        .iter().any(|needle| s.contains(needle));
    wants && together
}

fn source_quote(source: &str) -> String {
    source.chars().take(80).collect()
}

/// On-device small models may return JSON, fenced JSON or plain text.
/// Only private hints are stored. Source evidence can only come from the saved
/// private original. No model output can create a Goal / Consent / match.
fn parse_private_model_reply(
    original: &str,
    reply: &str,
) -> Result<(String, Option<String>, Option<String>, Option<PrivateIntentCandidate>, Option<String>, Vec<PrivateIntentCandidate>), String> {
    let text = reply.trim().trim_start_matches('\u{feff}').trim();
    if text.is_empty() || text.len() > 8192 {
        return Err(store_error("MODEL_OUTPUT_FORMAT"));
    }

    let body = if let Some(after) = text.strip_prefix("```json\n")
        .or_else(|| text.strip_prefix("```JSON\n"))
        .or_else(|| text.strip_prefix("```\n"))
    {
        after.strip_suffix("\n```").ok_or("MODEL_OUTPUT_FORMAT")?.trim()
    } else {
        text
    };

    let raw: RawSuggestion = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => {
            // Malformed JSON or structured instructions must never be accepted.
            // Only short unstructured explanations become private-only hints.
            if body.starts_with('{') || body.starts_with('[')
                || body.starts_with('"') || body.contains('`')
                || body.contains('{') || body.contains('}')
                || body.chars().count() > 200
            {
                return Err(store_error("MODEL_OUTPUT_FORMAT"));
            }
            let lowercase = body.to_ascii_lowercase();
            if ["consent", "confirmed", "visibility", "publish",
                "goal:", "offer:", "need:", "source=", "owner="]
                .iter().any(|unsafe_word| lowercase.contains(unsafe_word))
            {
                return Err(store_error("MODEL_OUTPUT_UNAPPROVED_FIELDS"));
            }
            RawSuggestion {
                evidence_quote: None,
                content_hint: Some(body.to_owned()),
                uncertainty: Some("模型回覆為一般文字，尚未結構化；請自行核對。".into()),
                intent_candidate: None,
                intent_candidates: None,
                clarifying_question: None,
            }
        }
    };

    if raw.content_hint.as_ref().is_some_and(|v| v.chars().count() > 200)
        || raw.uncertainty.as_ref().is_some_and(|v| v.chars().count() > 160)
    {
        return Err(store_error("MODEL_OUTPUT_TOO_LONG"));
    }

    let quote = match raw.evidence_quote.as_deref() {
        Some(proposed) if !proposed.trim().is_empty() => {
            let candidate = proposed.trim().trim_matches(&[
                '「', '」', '『', '』', '“', '”', '"', '\''
            ][..]).trim();
            if candidate.is_empty() || candidate.chars().count() > 80
                || !original.contains(candidate)
            {
                return Err(store_error("MODEL_EVIDENCE_MISMATCH"));
            }
            candidate.to_owned()
        }
        // Only bytes from the local asset or corrected OCR are copied.
        _ => original.chars().take(80).collect(),
    };
    if quote.trim().is_empty() || !original.contains(&quote) {
        return Err(store_error("MODEL_EVIDENCE_MISMATCH"));
    }
    // Content -> hypotheses -> human confirmation are distinct.
    // Any topic, even "JLPT", may yield visible PRIVATE hypotheses. These
    // cannot create a confirmed Goal, share consent, or a public match.
    // Still reject fabricated evidence, unsupported fields and stale sources.
    let can_suggest = !original.trim().is_empty();
    // US-01: up to three PRIVATE hypotheses, never guaranteed user Intent.
    // Reject evidence tampering and fabricated/unapproved fields. Maintain the
    // legacy single-candidate model contract for existing installed Android versions.
    let mut candidates: Vec<PrivateIntentCandidate> = Vec::new();
    let supplied=raw.intent_candidates.unwrap_or_default();
    if supplied.len()>3 {
        return Err(store_error("MODEL_OUTPUT_UNAPPROVED_FIELDS"));
    }
    let mut proposed=Vec::new();
    if let Some(single)=raw.intent_candidate {proposed.push(single);}
    proposed.extend(supplied);
    if can_suggest {
        for item in proposed {
            let evidence=item.evidence_quote.trim();
            if item.goal_text.trim().is_empty() ||
                item.goal_text.chars().count()>500 || item.goal_text.len()>2048 ||
                evidence.is_empty() || evidence.chars().count()>80 ||
                !original.contains(evidence) ||
                !matches!(item.domain.as_str(),"study"|"fashion"|"car"|"other"|"unknown")
            {
                continue; // Invalid AI hypothesis never becomes confirmed or an extra slot.
            }
            let normalized=PrivateIntentCandidate {
                domain:item.domain,
                goal_text:item.goal_text.trim().to_owned(),
                evidence_quote:evidence.to_owned(),
            };
            if !candidates.iter().any(|x| x.goal_text==normalized.goal_text &&
                x.domain==normalized.domain) {
                candidates.push(normalized);
            }
            if candidates.len()==3 {break;}
        }
        if candidates.is_empty() && explicit_collaboration_intent(original)
            && original.chars().count() <= 500 && original.len() <= 2048 {
            // For an explicit personal collaboration request, keep the old
            // source-only fallback; never invent it for an isolated topic.
            candidates.push(PrivateIntentCandidate {
                domain:"unknown".into(),
                goal_text:original.trim().to_owned(),
                evidence_quote:source_quote(original),
            });
        }
    }
    let intent_candidate = candidates.first().cloned();
    let question = match raw.clarifying_question {
        Some(value) if intent_candidate.is_none() => {
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 160 ||
               trimmed.len() > 512 || trimmed.chars().any(|ch| ch.is_control())
            { return Err(store_error("B09_INVALID_QUESTION")); }
            Some(trimmed.to_owned())
        }
        _ => None,
    };
    Ok((quote, raw.content_hint, raw.uncertainty, intent_candidate, question, candidates))
}

pub struct LocalStore {
    root: PathBuf,
}

fn store_error(code: &'static str) -> String {
    code.to_string()
}

fn clock_ms() -> Result<i64, String> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| store_error("CLOCK_UNAVAILABLE"))?
        .as_millis();
    i64::try_from(ms).map_err(|_| store_error("CLOCK_OVERFLOW"))
}

fn image_type(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(("image/png", "png"))
    } else if bytes.len() >= 3 && bytes[..3] == [0xff, 0xd8, 0xff] {
        Some(("image/jpeg", "jpg"))
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(("image/webp", "webp"))
    } else {
        None
    }
}


fn extension_for_mime(mime_type: &str) -> Result<&'static str, String> {
    match mime_type {
        "image/png" => Ok("png"),
        "image/jpeg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        _ => Err(store_error("UNSUPPORTED_OR_CORRUPT_IMAGE")),
    }
}

fn summary_from_row(row: &Row<'_>) -> rusqlite::Result<DropSummary> {
    let byte_length: i64 = row.get(5)?;
    let text: Option<String> = row.get(7)?;
    let is_duplicate: i64 = row.get(8)?;
    Ok(DropSummary {
        id: row.get(0)?,
        kind: row.get(1)?,
        source: row.get(2)?,
        file_name: row.get(3)?,
        mime_type: row.get(4)?,
        byte_length: byte_length.max(0) as usize,
        created_at_ms: row.get(6)?,
        text_excerpt: text.map(|t| t.chars().take(80).collect()),
        is_duplicate: is_duplicate != 0,
    })
}

const SELECT_SUMMARY: &str =
    "SELECT id, kind, source, file_name, mime_type, byte_length, created_at_ms, text_content, is_duplicate FROM local_assets";

impl LocalStore {
    pub fn open(root: &Path) -> Result<Self, String> {
        fs::create_dir_all(root).map_err(|_| store_error("STORAGE_UNAVAILABLE"))?;
        let conn = Connection::open(root.join("cakami_private.sqlite"))
            .map_err(|_| store_error("DATABASE_UNAVAILABLE"))?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS local_assets (
               id TEXT PRIMARY KEY NOT NULL,
               kind TEXT NOT NULL CHECK(kind IN ('text','image')),
               source TEXT NOT NULL CHECK(source IN ('text','picker','camera','drop')),
               file_name TEXT,
               mime_type TEXT NOT NULL,
               sha256 TEXT NOT NULL,
               byte_length INTEGER NOT NULL CHECK(byte_length > 0),
               created_at_ms INTEGER NOT NULL,
               text_content TEXT,
               is_duplicate INTEGER NOT NULL CHECK(is_duplicate IN (0,1)),
               CHECK((kind='text' AND text_content IS NOT NULL AND file_name IS NULL)
                  OR (kind='image' AND text_content IS NULL))
             );
             CREATE INDEX IF NOT EXISTS idx_assets_recent ON local_assets(created_at_ms DESC);
             CREATE INDEX IF NOT EXISTS idx_assets_fingerprint ON local_assets(sha256);
             CREATE TABLE IF NOT EXISTS local_ocr (
               asset_id TEXT PRIMARY KEY NOT NULL REFERENCES local_assets(id) ON DELETE CASCADE,
               status TEXT NOT NULL CHECK(status IN ('not_run','recognized','unknown')),
               raw_text TEXT,
               corrected_text TEXT,
               engine TEXT,
               engine_version TEXT,
               segments_json TEXT NOT NULL DEFAULT '[]',
               elapsed_ms INTEGER,
               updated_at_ms INTEGER NOT NULL,
               CHECK(raw_text IS NULL OR length(raw_text) <= 65536),
               CHECK(corrected_text IS NULL OR length(corrected_text) <= 65536)
             );
             CREATE TABLE IF NOT EXISTS local_intent_candidates (
               asset_id TEXT PRIMARY KEY NOT NULL REFERENCES local_assets(id) ON DELETE CASCADE,
               source_sha256 TEXT NOT NULL CHECK(length(source_sha256)=64),
               candidate_json TEXT NOT NULL CHECK(length(candidate_json) <= 4096),
               state TEXT NOT NULL DEFAULT 'suggested_private'
                 CHECK(state='suggested_private')
             );
             CREATE TABLE IF NOT EXISTS local_intent_questions (
                asset_id TEXT NOT NULL REFERENCES local_assets(id) ON DELETE CASCADE,
                base_source_sha256 TEXT NOT NULL CHECK(length(base_source_sha256)=64),
                ordinal INTEGER NOT NULL CHECK(ordinal BETWEEN 1 AND 2),
                question TEXT NOT NULL CHECK(length(trim(question)) > 0
                                                AND length(question) <= 512),
                answer TEXT CHECK(answer IS NULL OR
                                 (length(trim(answer)) > 0 AND length(answer) <= 2048)),
                issued_at_ms INTEGER NOT NULL,
                answered_at_ms INTEGER,
                PRIMARY KEY(asset_id,base_source_sha256,ordinal),
                CHECK((answer IS NULL AND answered_at_ms IS NULL) OR
                      (answer IS NOT NULL AND answered_at_ms IS NOT NULL))
             );
             CREATE UNIQUE INDEX IF NOT EXISTS b09_one_pending_question
               ON local_intent_questions(asset_id,base_source_sha256)
               WHERE answer IS NULL;
             CREATE TABLE IF NOT EXISTS local_model_suggestions (
               asset_id TEXT PRIMARY KEY NOT NULL REFERENCES local_assets(id) ON DELETE CASCADE,
               status TEXT NOT NULL CHECK(status='suggested_private'),
               source_sha256 TEXT NOT NULL CHECK(length(source_sha256)=64),
               evidence_quote TEXT NOT NULL CHECK(length(evidence_quote)>0),
               content_hint TEXT,
               uncertainty TEXT,
               model_id TEXT NOT NULL,
               runtime TEXT NOT NULL,
               backend TEXT NOT NULL,
               elapsed_ms INTEGER NOT NULL,
               updated_at_ms INTEGER NOT NULL
             );",
        )
        .map_err(|_| store_error("DATABASE_MIGRATION_FAILED"))?;
        let store = Self { root: root.to_path_buf() };
        // 每次重新開啟儲存時修復中斷的刪除操作；以 SQLite 寫入鎖序列化恢復。
        store.recover_pending_deletes()?;
        Ok(store)
    }

    fn connect(&self) -> Result<Connection, String> {
        let conn = Connection::open(self.root.join("cakami_private.sqlite"))
            .map_err(|_| store_error("DATABASE_UNAVAILABLE"))?;
        // SQLite FK 設定是逐 Connection 生效，必須在每次開啟時啟用。
        // 刪除 Asset 時才能一起刪掉只留在手機的 OCR 與人工修正。
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|_| store_error("DATABASE_UNAVAILABLE"))?;
        Ok(conn)
    }

    pub fn save(&self, input: IncomingDrop) -> Result<DropSummary, String> {
        let (raw, mime_type, extension, text_content, file_name) = match input.kind.as_str() {
            "text" => {
                if input.source != "text" {
                    return Err(store_error("INVALID_SOURCE"));
                }
                let content = input.text.ok_or_else(|| store_error("EMPTY_TEXT"))?;
                if content.trim().is_empty() {
                    return Err(store_error("EMPTY_TEXT"));
                }
                if content.len() > MAX_TEXT_BYTES {
                    return Err(store_error("TEXT_TOO_LARGE"));
                }
                (content.as_bytes().to_vec(), "text/plain".to_string(),
                 None, Some(content), None)
            }
            "image" => {
                if !matches!(input.source.as_str(), "picker" | "camera" | "drop") {
                    return Err(store_error("INVALID_SOURCE"));
                }
                let encoded = input.base64.ok_or_else(|| store_error("EMPTY_IMAGE"))?;
                if encoded.len() > (MAX_IMAGE_BYTES + 2) / 3 * 4 + 8 {
                    return Err(store_error("IMAGE_TOO_LARGE"));
                }
                let raw = STANDARD.decode(&encoded).map_err(|_| store_error("CORRUPT_IMAGE"))?;
                if raw.is_empty() {
                    return Err(store_error("EMPTY_IMAGE"));
                }
                if raw.len() > MAX_IMAGE_BYTES {
                    return Err(store_error("IMAGE_TOO_LARGE"));
                }
                let (actual_mime, ext) = image_type(&raw)
                    .ok_or_else(|| store_error("UNSUPPORTED_OR_CORRUPT_IMAGE"))?;
                if let Some(declared_mime) = input.mime_type {
                    if !declared_mime.is_empty() && declared_mime != actual_mime {
                        return Err(store_error("IMAGE_TYPE_MISMATCH"));
                    }
                }
                let name = input.file_name
                    .map(|s| s.chars().take(120).collect::<String>())
                    .filter(|s| !s.is_empty());
                (raw, actual_mime.to_string(), Some(ext), None, name)
            }
            _ => return Err(store_error("UNSUPPORTED_DROP_KIND")),
        };

        let sha256 = format!("{:x}", Sha256::digest(&raw));
        let id = Uuid::new_v4().to_string();
        let created_at_ms = clock_ms()?;

        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| store_error("DATABASE_BUSY"))?;
        let duplicate_count: i64 = tx.query_row(
            "SELECT count(*) FROM local_assets WHERE sha256=?1 AND kind=?2",
            params![sha256, input.kind], |row| row.get(0),
        ).map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let is_duplicate = duplicate_count > 0;

        // UUID 由 Rust 產生，目錄及副檔名不可由前端控制。
        let saved_path = if let Some(ext) = extension {
            let private_dir = self.root.join("private_assets");
            fs::create_dir_all(&private_dir)
                .map_err(|_| store_error("STORAGE_UNAVAILABLE"))?;
            let path = private_dir.join(format!("{id}.{ext}"));
            let mut file = OpenOptions::new().write(true).create_new(true).open(&path)
                .map_err(|_| store_error("STORAGE_WRITE_FAILED"))?;
            if file.write_all(&raw).and_then(|_| file.sync_all()).is_err() {
                let _ = fs::remove_file(&path);
                return Err(store_error("STORAGE_WRITE_FAILED"));
            }
            Some(path)
        } else {
            None
        };

        let result = tx.execute(
            "INSERT INTO local_assets
             (id, kind, source, file_name, mime_type, sha256, byte_length,
              created_at_ms, text_content, is_duplicate)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![id, input.kind, input.source, file_name, mime_type,
                sha256, raw.len() as i64, created_at_ms, text_content, is_duplicate],
        ).and_then(|_| tx.commit());
        if result.is_err() {
            if let Some(path) = &saved_path {
                let _ = fs::remove_file(path);
            }
            return Err(store_error("DATABASE_WRITE_FAILED"));
        }

        Ok(DropSummary {
            id,
            kind: input.kind,
            source: input.source,
            file_name,
            mime_type,
            byte_length: raw.len(),
            created_at_ms,
            text_excerpt: text_content.map(|t| t.chars().take(80).collect()),
            is_duplicate,
        })
    }


    /// SQLite 與檔案系統沒有跨資源交易：檔案先更名為暫存檔；
    /// 若 DB 尚有紀錄則恢復，若紀錄已刪除則清理暫存檔。
    /// 只處理 Rust UUID 命名的本地圖片，忽略未知檔案，避免碰觸其他資料。
    fn recover_pending_deletes(&self) -> Result<(), String> {
        let private_dir = self.root.join("private_assets");
        let pending_dir = private_dir.join(".pending_deletes");
        if !pending_dir.exists() {
            return Ok(());
        }
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| store_error("DATABASE_BUSY"))?;
        for entry in fs::read_dir(&pending_dir)
            .map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?
        {
            let entry = entry.map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?;
            let name = entry.file_name().into_string()
                .map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?;
            let Some((id, ext)) = name.rsplit_once('.') else { continue };
            if !matches!(ext, "png" | "jpg" | "webp")
                || Uuid::parse_str(id).map(|uuid| uuid.to_string() != id).unwrap_or(true)
            {
                continue;
            }
            if !entry.file_type().map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?.is_file() {
                return Err(store_error("STORAGE_RECOVERY_FAILED"));
            }
            let mime: Option<String> = tx.query_row(
                "SELECT mime_type FROM local_assets WHERE id=?1 AND kind='image'",
                params![id], |row| row.get(0),
            ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?;
            match mime {
                Some(mime_type) => {
                    if extension_for_mime(&mime_type)? != ext {
                        return Err(store_error("STORAGE_RECOVERY_FAILED"));
                    }
                    let original = private_dir.join(&name);
                    if original.exists() {
                        return Err(store_error("STORAGE_RECOVERY_FAILED"));
                    }
                    fs::rename(entry.path(), original)
                        .map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?;
                }
                None => {
                    fs::remove_file(entry.path())
                        .map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?;
                }
            }
        }
        tx.commit().map_err(|_| store_error("DATABASE_WRITE_FAILED"))
    }

    /// B02：按 UUID 刪除單一私人收藏。不可由 WebView 指定任何磁碟路徑。
    /// 成功回傳 true；失敗不會對外宣稱刪除成功。
    pub fn delete(&self, id: &str) -> Result<bool, String> {
        let id = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?
            .to_string();
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| store_error("DATABASE_BUSY"))?;
        let asset: Option<(String, String)> = tx.query_row(
            "SELECT kind, mime_type FROM local_assets WHERE id=?1",
            params![id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let (kind, mime_type) = asset.ok_or_else(|| store_error("ASSET_NOT_FOUND"))?;

        // 同一筆圖片有自己的 UUID 檔名，重複圖片也不共用檔案。
        let staged = if kind == "image" {
            let ext = extension_for_mime(&mime_type)?;
            let private_dir = self.root.join("private_assets");
            let pending_dir = private_dir.join(".pending_deletes");
            fs::create_dir_all(&pending_dir)
                .map_err(|_| store_error("STORAGE_DELETE_FAILED"))?;
            let name = format!("{id}.{ext}");
            let original = private_dir.join(&name);
            let pending = pending_dir.join(&name);
            if pending.exists() {
                return Err(store_error("STORAGE_RECOVERY_FAILED"));
            }
            fs::rename(&original, &pending).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    store_error("ASSET_FILE_MISSING")
                } else {
                    store_error("STORAGE_DELETE_FAILED")
                }
            })?;
            Some((original, pending))
        } else {
            None
        };

        // BEGIN IMMEDIATE 讓兩個同時的刪除請求不會互相踩資料。
        let write = tx.execute("DELETE FROM local_assets WHERE id=?1", params![id]);
        if !matches!(write, Ok(1)) {
            drop(tx); // rollback before restoring the file
            if let Some((original, pending)) = &staged {
                fs::rename(pending, original)
                    .map_err(|_| store_error("STORAGE_RECOVERY_FAILED"))?;
            }
            return Err(store_error("DATABASE_WRITE_FAILED"));
        }
        if tx.commit().is_err() {
            // commit 不確定時，交由下次 open 對照 SQLite 決定還原／清理。
            return Err(store_error("STORAGE_RECOVERY_REQUIRED"));
        }

        if let Some((_, pending)) = staged {
            // DB 已提交，但圖片暫存檔也必須真的清除才可回傳成功。
            // 失敗時保留暫存檔供下次 open 重試，並清楚回報未完全清理。
            fs::remove_file(pending)
                .map_err(|_| store_error("STORAGE_CLEANUP_PENDING"))?;
        }
        Ok(true)
    }


    fn ensure_image(&self, conn: &Connection, id: &str) -> Result<(), String> {
        let kind: Option<String> = conn.query_row(
            "SELECT kind FROM local_assets WHERE id=?1",
            params![id], |row| row.get(0),
        ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        match kind.as_deref() {
            Some("image") => Ok(()),
            Some("text") => Err(store_error("OCR_IMAGE_REQUIRED")),
            _ => Err(store_error("ASSET_NOT_FOUND")),
        }
    }

    /// 只接受 Rust 從 native plugin 收到的辨識結果，不暴露任意「偽 OCR」寫入 Command。
    pub fn save_recognition(&self, id: &str, native: NativeOcrResult) -> Result<OcrRecord, String> {
        let id = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?.to_string();
        let trusted_engine =
            (native.engine == "mlkit_japanese_bundled" && native.engine_version == "16.0.1") ||
            (native.engine == "paddle_ocrv6_small_onnx" && native.engine_version == "6.0.0");
        if !trusted_engine {
            return Err(store_error("OCR_INVALID_RESULT"));
        }
        if native.text.len() > MAX_TEXT_BYTES || native.segments.len() > 500 ||
            native.segments.iter().any(|segment| segment.text.len() > MAX_TEXT_BYTES)
        {
            return Err(store_error("OCR_INVALID_RESULT"));
        }
        let segments_json = serde_json::to_string(&native.segments)
            .map_err(|_| store_error("OCR_INVALID_RESULT"))?;
        if segments_json.len() > 256 * 1024 {
            return Err(store_error("OCR_INVALID_RESULT"));
        }
        let conn = self.connect()?;
        self.ensure_image(&conn, &id)?;
        let now = clock_ms()?;
        let status = if native.text.trim().is_empty() { "unknown" } else { "recognized" };
        conn.execute(
            "INSERT INTO local_ocr (asset_id, status, raw_text, engine, engine_version,
                segments_json, elapsed_ms, updated_at_ms)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(asset_id) DO UPDATE SET
                 status=excluded.status,
                 raw_text=excluded.raw_text,
                 engine=excluded.engine,
                 engine_version=excluded.engine_version,
                 segments_json=excluded.segments_json,
                 elapsed_ms=excluded.elapsed_ms,
                 updated_at_ms=excluded.updated_at_ms",
            params![id, status, native.text, native.engine, native.engine_version,
                segments_json, native.elapsed_ms as i64, now],
        ).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        self.load_ocr(&id)
    }

    /// 本人人工更正：與原始辨識分欄保存，並保留引擎與來源；不等於 Goal 確認。
    pub fn correct_ocr(&self, id: &str, corrected: &str) -> Result<OcrRecord, String> {
        let id = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?.to_string();
        if corrected.trim().is_empty() {
            return Err(store_error("EMPTY_TEXT"));
        }
        if corrected.len() > MAX_TEXT_BYTES {
            return Err(store_error("TEXT_TOO_LARGE"));
        }
        let conn = self.connect()?;
        self.ensure_image(&conn, &id)?;
        conn.execute(
            "INSERT INTO local_ocr (asset_id, status, corrected_text, updated_at_ms)
             VALUES (?1,'not_run',?2,?3)
             ON CONFLICT(asset_id) DO UPDATE SET
               corrected_text=excluded.corrected_text,
               updated_at_ms=excluded.updated_at_ms",
            params![id, corrected, clock_ms()?],
        ).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        self.load_ocr(&id)
    }

    pub fn load_ocr(&self, id: &str) -> Result<OcrRecord, String> {
        let id = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?.to_string();
        let conn = self.connect()?;
        self.ensure_image(&conn, &id)?;
        let found: Option<(String, Option<String>, Option<String>, Option<String>,
            Option<String>, String, Option<i64>, i64)> = conn.query_row(
            "SELECT status, raw_text, corrected_text, engine, engine_version,
                segments_json, elapsed_ms, updated_at_ms
             FROM local_ocr WHERE asset_id=?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?,
                row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)),
        ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        if let Some((status, raw_text, corrected_text, engine, engine_version,
            segments_json, elapsed_ms, updated_at_ms)) = found {
            let segments = serde_json::from_str(&segments_json)
                .map_err(|_| store_error("DATABASE_READ_FAILED"))?;
            Ok(OcrRecord {
                asset_id: id, status, raw_text, corrected_text, engine, engine_version,
                segments, elapsed_ms: elapsed_ms.map(|v| v.max(0) as u64),
                updated_at_ms: Some(updated_at_ms),
            })
        } else {
            Ok(OcrRecord {
                asset_id: id, status: "not_run".to_string(),
                raw_text: None, corrected_text: None, engine: None,
                engine_version: None, segments: Vec::new(),
                elapsed_ms: None, updated_at_ms: None,
            })
        }
    }


    /// A single source of truth for text and image/OCR input, always read from
    /// the same SQLite connection (also under the inference commit transaction).
    fn base_source(conn: &Connection, id: &str) -> Result<String, String> {
        let row: Option<(String, Option<String>)> = conn.query_row(
            "SELECT kind,text_content FROM local_assets WHERE id=?1",
            params![id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let (kind, text) = row.ok_or_else(|| store_error("ASSET_NOT_FOUND"))?;
        let raw = match kind.as_str() {
            "text" => text.unwrap_or_default(),
            "image" => conn.query_row(
                "SELECT coalesce(nullif(trim(corrected_text),''),nullif(trim(raw_text),''),'')
                 FROM local_ocr WHERE asset_id=?1",
                params![id], |row| row.get::<_, String>(0),
            ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?
                .unwrap_or_default(),
            _ => return Err(store_error("MODEL_INPUT_UNKNOWN")),
        };
        let trimmed=raw.trim();
        if trimmed.is_empty() { return Err(store_error("MODEL_INPUT_UNKNOWN")); }
        Ok(trimmed.to_owned())
    }

    /// Only the PERSON'S answers (not AI-written questions) are added to model
    /// evidence; no model question is allowed to manufacture user collaboration.
    fn with_user_answers(conn: &Connection, id: &str, base: &str) -> Result<String, String> {
        let base_sha=format!("{:x}",Sha256::digest(base.as_bytes()));
        let mut stmt=conn.prepare(
            "SELECT answer FROM local_intent_questions
             WHERE asset_id=?1 AND base_source_sha256=?2 AND answer IS NOT NULL
             ORDER BY ordinal"
        ).map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let responses=stmt.query_map(params![id,base_sha],|row|row.get::<_,String>(0))
            .map_err(|_| store_error("DATABASE_READ_FAILED"))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let mut input=base.to_owned();
        for answer in responses {
            input.push_str("\n\n【本人補充（非模型推測）】\n");
            input.push_str(&answer);
        }
        if input.len()>8192 { return Err(store_error("MODEL_INPUT_TOO_LARGE")); }
        Ok(input)
    }

    /// B02 text and B03 OCR + any answer converge on this ONE Gemma input path.
    pub fn model_input_for_asset(&self, id: &str) -> Result<String, String> {
        let uuid=Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?.to_string();
        let conn=self.connect()?;
        let base=Self::base_source(&conn,&uuid)?;
        Self::with_user_answers(&conn,&uuid,&base)
    }

    /// Person must answer the current source-backed question. One question at a
    /// time, max TWO clarifications; the separate Goal confirmation is question 3.
    pub fn answer_intent_question(&self, id: &str, ordinal: i64, answer: &str)
        -> Result<(), String>
    {
        let uuid=Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?.to_string();
        let value=answer.trim();
        if !(1..=2).contains(&ordinal) || value.is_empty() ||
            value.len()>2048 || value.chars().count()>500 ||
            value.chars().any(|ch| ch.is_control() && ch!='\n')
        { return Err(store_error("B09_INVALID_ANSWER")); }
        let mut conn=self.connect()?;
        let tx=conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| store_error("DATABASE_BUSY"))?;
        let base=Self::base_source(&tx,&uuid)?;
        let source_sha=format!("{:x}",Sha256::digest(base.as_bytes()));
        let affected=tx.execute(
            "UPDATE local_intent_questions SET answer=?1, answered_at_ms=?2
             WHERE asset_id=?3 AND base_source_sha256=?4 AND ordinal=?5
               AND answer IS NULL",
            params![value,clock_ms()?,uuid,source_sha,ordinal],
        ).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        if affected!=1 { return Err(store_error("B09_QUESTION_STALE")); }
        tx.commit().map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        Ok(())
    }

    /// Model output never confirms a Goal/Need/Offer. Reject unknown JSON keys and
    /// any quote that is not a literal substring of the private source.
    pub fn save_model_suggestion(
        &self, id: &str, original_input: &str, native: NativeReply,
    ) -> Result<ModelSuggestion, String> {
        let uuid = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?
            .to_string();
        if native.model_id != "gemma-4-E2B-it"
            || native.runtime != "litert-lm-android-0.16.1"
            || native.backend != "cpu"
            || native.raw_text.len() > 8192
        {
            return Err(store_error("MODEL_INVALID_OUTPUT"));
        }
        let (quote, content_hint, uncertainty, intent_candidate, clarifying_question, candidates) =
            parse_private_model_reply(original_input, &native.raw_text)?;
        let sha = format!("{:x}", Sha256::digest(original_input.as_bytes()));
        let now = clock_ms()?;
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| store_error("DATABASE_BUSY"))?;
        // Recompute base+answered context in the SAME write transaction.
        // OCR correction/source deletion invalidates old questions and model results.
        let current_base=Self::base_source(&tx,&uuid)?;
        let current=Self::with_user_answers(&tx,&uuid,&current_base)?;
        if current != original_input {
            return Err(store_error("MODEL_STALE_INPUT"));
        }
        tx.execute(
            "INSERT INTO local_model_suggestions
             (asset_id,status,source_sha256,evidence_quote,content_hint,uncertainty,
              model_id,runtime,backend,elapsed_ms,updated_at_ms)
             VALUES (?1,'suggested_private',?2,?3,?4,?5,?6,?7,?8,?9,?10)
             ON CONFLICT(asset_id) DO UPDATE SET
               source_sha256=excluded.source_sha256,
               evidence_quote=excluded.evidence_quote,
               content_hint=excluded.content_hint,
               uncertainty=excluded.uncertainty,
               model_id=excluded.model_id,
               runtime=excluded.runtime,
               backend=excluded.backend,
               elapsed_ms=excluded.elapsed_ms,
               updated_at_ms=excluded.updated_at_ms",
            params![uuid, sha, quote, content_hint, uncertainty, native.model_id,
                    native.runtime, native.backend, native.elapsed_ms as i64, now],
        ).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        // Update content hint + optional private intent in one transaction.
        // A model may never insert into local_goal_drafts/confirmation_events.
        tx.execute("DELETE FROM local_intent_candidates WHERE asset_id=?1",
            params![uuid]).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        if !candidates.is_empty() {
            let json = serde_json::to_string(&candidates)
                .map_err(|_| store_error("MODEL_OUTPUT_FORMAT"))?;
            tx.execute(
                "INSERT INTO local_intent_candidates
                 (asset_id,source_sha256,candidate_json,state)
                 VALUES (?1,?2,?3,'suggested_private')",
                params![uuid, sha, json],
            ).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        }
        if let Some(ref question)=clarifying_question {
            let base_sha=format!("{:x}",Sha256::digest(current_base.as_bytes()));
            let (asked,pending,repeat):(i64,i64,i64)=tx.query_row(
                "SELECT count(*),coalesce(sum(CASE WHEN answer IS NULL THEN 1 ELSE 0 END),0),
                        coalesce(sum(CASE WHEN question=?3 THEN 1 ELSE 0 END),0)
                 FROM local_intent_questions
                 WHERE asset_id=?1 AND base_source_sha256=?2",
                params![uuid,base_sha,question],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)),
            ).map_err(|_| store_error("DATABASE_READ_FAILED"))?;
            if intent_candidate.is_none() && pending==0 && repeat==0 && asked<2 {
                tx.execute(
                    "INSERT INTO local_intent_questions
                     (asset_id,base_source_sha256,ordinal,question,answer,issued_at_ms)
                     VALUES (?1,?2,?3,?4,NULL,?5)",
                    params![uuid,base_sha,asked+1,question,now],
                ).map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
            }
        }
        tx.commit().map_err(|_| store_error("DATABASE_WRITE_FAILED"))?;
        self.load_model_suggestion(&uuid)?
            .ok_or_else(|| store_error("DATABASE_READ_FAILED"))
    }

    /// Old AI guesses become invisible once the user corrects OCR text.
    pub fn load_model_suggestion(&self, id: &str) -> Result<Option<ModelSuggestion>, String> {
        let uuid = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?.to_string();
        let input = self.model_input_for_asset(&uuid)?;
        let sha = format!("{:x}", Sha256::digest(input.as_bytes()));
        let conn = self.connect()?;
        let base=Self::base_source(&conn,&uuid)?;
        let base_sha=format!("{:x}",Sha256::digest(base.as_bytes()));
        conn.query_row(
            "SELECT s.asset_id,s.status,s.evidence_quote,s.content_hint,s.uncertainty,
                    s.model_id,s.runtime,s.backend,s.elapsed_ms,s.updated_at_ms,
                    i.candidate_json,
                    CASE WHEN i.candidate_json IS NULL THEN q.ordinal ELSE NULL END,
                    CASE WHEN i.candidate_json IS NULL THEN q.question ELSE NULL END
             FROM local_model_suggestions s
             LEFT JOIN local_intent_candidates i ON i.asset_id=s.asset_id
               AND i.source_sha256=s.source_sha256
             LEFT JOIN local_intent_questions q ON q.asset_id=s.asset_id
               AND q.base_source_sha256=?3 AND q.answer IS NULL
             WHERE s.asset_id=?1 AND s.source_sha256=?2",
            params![uuid, sha,base_sha],
            |row| {
                let json: Option<String> = row.get(10)?;
                let candidates: Vec<PrivateIntentCandidate> = match json {
                    None => Vec::new(),
                    Some(value) => {
                        let parsed: serde_json::Value=serde_json::from_str(&value)
                            .map_err(|_|rusqlite::Error::InvalidQuery)?;
                        if parsed.is_array() {
                            serde_json::from_value(parsed)
                                .map_err(|_|rusqlite::Error::InvalidQuery)?
                        } else {
                            // Migration-safe: PR #16 wrote a single JSON object.
                            vec![serde_json::from_value(parsed)
                                .map_err(|_|rusqlite::Error::InvalidQuery)?]
                        }
                    }
                };
                if candidates.len()>3 {return Err(rusqlite::Error::InvalidQuery);}
                let candidate=candidates.first().cloned();
                Ok(ModelSuggestion {
                    asset_id: row.get(0)?, status: row.get(1)?,
                    evidence_quote: row.get(2)?, content_hint: row.get(3)?,
                    uncertainty: row.get(4)?, intent_candidate: candidate,
                    intent_candidates:candidates,
                    pending_question: match row.get::<_,Option<i64>>(11)? {
                        Some(ordinal) => Some(PendingIntentQuestion {
                            ordinal,
                            text: row.get(12)?,
                        }),
                        None => None,
                    },
                    model_id: row.get(5)?, runtime: row.get(6)?, backend: row.get(7)?,
                    elapsed_ms: row.get::<_, i64>(8)?.max(0) as u64,
                    updated_at_ms: row.get(9)?,
                })
            },
        ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))
    }

    pub fn list(&self) -> Result<Vec<DropSummary>, String> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(&format!(
            "{SELECT_SUMMARY} ORDER BY created_at_ms DESC, rowid DESC LIMIT ?1"
        )).map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let rows = stmt.query_map(params![LIST_LIMIT], summary_from_row)
            .map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| store_error("DATABASE_READ_FAILED"))
    }

    pub fn load(&self, id: &str) -> Result<DropDetail, String> {
        let validated_id = Uuid::parse_str(id).map_err(|_| store_error("INVALID_ASSET_ID"))?;
        let conn = self.connect()?;
        let maybe = conn.query_row(
            &format!("{SELECT_SUMMARY} WHERE id=?1"),
            params![validated_id.to_string()],
            |row| {
                let summary = summary_from_row(row)?;
                let text: Option<String> = row.get(7)?;
                Ok((summary, text))
            },
        ).optional().map_err(|_| store_error("DATABASE_READ_FAILED"))?;
        let (summary, text) = maybe.ok_or_else(|| store_error("ASSET_NOT_FOUND"))?;

        let data_base64 = if summary.kind == "image" {
            let ext = extension_for_mime(&summary.mime_type)?;
            let path = self.root.join("private_assets")
                .join(format!("{}.{ext}", validated_id));
            let mut file = fs::File::open(path)
                .map_err(|_| store_error("ASSET_FILE_MISSING"))?;
            let mut buf = Vec::new();
            file.take(MAX_IMAGE_BYTES as u64 + 1).read_to_end(&mut buf)
                .map_err(|_| store_error("STORAGE_READ_FAILED"))?;
            if buf.len() > MAX_IMAGE_BYTES {
                return Err(store_error("IMAGE_TOO_LARGE"));
            }
            if image_type(&buf).map(|(mime, _)| mime) != Some(summary.mime_type.as_str()) {
                return Err(store_error("CORRUPT_IMAGE"));
            }
            Some(STANDARD.encode(buf))
        } else {
            None
        };

        Ok(DropDetail { summary, text, data_base64 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn text_drop(text: &str) -> IncomingDrop {
        IncomingDrop {
            kind: "text".into(), source: "text".into(), text: Some(text.into()),
            base64: None, file_name: None, mime_type: None,
        }
    }

    fn tiny_png() -> Vec<u8> {
        // 合成 PNG：測試本地儲存／魔術位元組／還原；無私人圖片。
        vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0, 0]
    }

    fn png_drop(bytes: &[u8]) -> IncomingDrop {
        IncomingDrop {
            kind: "image".into(), source: "picker".into(), text: None,
            base64: Some(STANDARD.encode(bytes)),
            file_name: Some("jlpt-n2.png".into()), mime_type: Some("image/png".into()),
        }
    }

    #[test]
    fn b02_text_persists_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる\nJLPT N2")).unwrap();
        assert_eq!(saved.kind, "text");
        assert_eq!(saved.text_excerpt.as_deref(), Some("パクる\nJLPT N2"));
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert_eq!(reopened.list().unwrap().len(), 1);
        let loaded = reopened.load(&saved.id).unwrap();
        assert_eq!(loaded.text.as_deref(), Some("パクる\nJLPT N2"));
        assert!(loaded.data_base64.is_none());
    }

    #[test]
    fn b02_image_persists_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let png = tiny_png();
        let saved = store.save(png_drop(&png)).unwrap();
        assert_eq!(saved.mime_type, "image/png");
        assert_eq!(saved.byte_length, png.len());
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        let opened = reopened.load(&saved.id).unwrap();
        assert_eq!(STANDARD.decode(opened.data_base64.unwrap()).unwrap(), png);
    }

    #[test]
    fn b02_empty_text_is_rejected_without_record() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        assert_eq!(store.save(text_drop("   ")).unwrap_err(), "EMPTY_TEXT");
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn b02_bad_image_bytes_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        assert_eq!(store.save(png_drop(b"not a png")).unwrap_err(),
                   "UNSUPPORTED_OR_CORRUPT_IMAGE");
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn b02_non_base64_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let mut input = png_drop(&tiny_png());
        input.base64 = Some("$%bad".into());
        assert_eq!(store.save(input).unwrap_err(), "CORRUPT_IMAGE");
    }

    #[test]
    fn b02_duplicate_is_visible_and_both_original_records_survive() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let first = store.save(text_drop("N2 讀書")).unwrap();
        let duplicate = store.save(text_drop("N2 讀書")).unwrap();
        assert!(!first.is_duplicate);
        assert!(duplicate.is_duplicate);
        assert_ne!(first.id, duplicate.id);
        assert_eq!(store.list().unwrap().len(), 2);
    }

    #[test]
    fn b02_filename_cannot_escape_private_storage() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let mut input = png_drop(&tiny_png());
        input.file_name = Some("../../secret.png".into());
        let summary = store.save(input).unwrap();
        assert!(dir.path().join("private_assets")
            .join(format!("{}.png", summary.id)).exists());
        assert!(!dir.path().join("secret.png").exists());
        assert_eq!(store.load("../../secret.png").unwrap_err(), "INVALID_ASSET_ID");
    }

    #[test]
    fn b02_rejects_oversized_payload() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let mut input = png_drop(&tiny_png());
        input.base64 = Some("A".repeat((MAX_IMAGE_BYTES + 2) / 3 * 4 + 9));
        assert_eq!(store.save(input).unwrap_err(), "IMAGE_TOO_LARGE");
    }

    #[test]
    fn b02_unknown_id_rejected_without_leaking_paths() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        assert_eq!(store.load(&Uuid::new_v4().to_string()).unwrap_err(), "ASSET_NOT_FOUND");
    }

    #[test]
    fn b02_rejects_invalid_sources_and_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let mut input = text_drop("內容");
        input.source = "cloud".into();
        assert_eq!(store.save(input).unwrap_err(), "INVALID_SOURCE");
        let mut input = text_drop("內容");
        input.kind = "consent".into();
        assert_eq!(store.save(input).unwrap_err(), "UNSUPPORTED_DROP_KIND");
    }

    #[test]
    fn b02_delete_text_and_restart_keeps_it_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる")).unwrap();
        assert!(store.delete(&saved.id).unwrap());
        assert_eq!(store.load(&saved.id).unwrap_err(), "ASSET_NOT_FOUND");
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert!(reopened.list().unwrap().is_empty());
        assert_eq!(reopened.delete(&saved.id).unwrap_err(), "ASSET_NOT_FOUND");
    }

    #[test]
    fn b02_delete_image_removes_file_and_row_after_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        let image = dir.path().join("private_assets").join(format!("{}.png", saved.id));
        assert!(image.exists());
        assert!(store.delete(&saved.id).unwrap());
        assert!(!image.exists());
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert!(reopened.list().unwrap().is_empty());
        assert_eq!(reopened.load(&saved.id).unwrap_err(), "ASSET_NOT_FOUND");
    }

    #[test]
    fn b02_delete_duplicate_does_not_remove_sibling() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let first = store.save(png_drop(&tiny_png())).unwrap();
        let second = store.save(png_drop(&tiny_png())).unwrap();
        assert!(second.is_duplicate);
        store.delete(&first.id).unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        assert_eq!(store.load(&second.id).unwrap().summary.id, second.id);
    }

    #[test]
    fn b02_delete_invalid_id_cannot_escape_private_storage() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("留著")).unwrap();
        assert_eq!(store.delete("../../secret").unwrap_err(), "INVALID_ASSET_ID");
        assert!(store.load(&saved.id).is_ok());
    }

    #[test]
    fn b02_missing_image_refuses_to_delete_database_record() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        fs::remove_file(dir.path().join("private_assets").join(format!("{}.png", saved.id))).unwrap();
        assert_eq!(store.delete(&saved.id).unwrap_err(), "ASSET_FILE_MISSING");
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn b02_failed_sql_delete_restores_file_and_row() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        let conn = store.connect().unwrap();
        conn.execute_batch("CREATE TRIGGER prevent_delete BEFORE DELETE ON local_assets
            BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
        assert_eq!(store.delete(&saved.id).unwrap_err(), "DATABASE_WRITE_FAILED");
        assert_eq!(store.load(&saved.id).unwrap().summary.id, saved.id);
        let path = dir.path().join("private_assets").join(format!("{}.png", saved.id));
        assert!(path.exists());
    }

    #[test]
    fn b02_recovery_restores_uncommitted_image_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        let private_dir = dir.path().join("private_assets");
        let pending_dir = private_dir.join(".pending_deletes");
        fs::create_dir_all(&pending_dir).unwrap();
        let name = format!("{}.png", saved.id);
        fs::rename(private_dir.join(&name), pending_dir.join(&name)).unwrap();
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert_eq!(reopened.load(&saved.id).unwrap().summary.id, saved.id);
        assert!(!pending_dir.join(&name).exists());
    }

    #[test]
    fn b02_recovery_purges_committed_image_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        let private_dir = dir.path().join("private_assets");
        let pending_dir = private_dir.join(".pending_deletes");
        fs::create_dir_all(&pending_dir).unwrap();
        let name = format!("{}.png", saved.id);
        fs::rename(private_dir.join(&name), pending_dir.join(&name)).unwrap();
        store.connect().unwrap().execute(
            "DELETE FROM local_assets WHERE id=?1", params![saved.id]).unwrap();
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert!(reopened.list().unwrap().is_empty());
        assert!(!pending_dir.join(&name).exists());
    }


    fn recognized_ocr(text: &str) -> NativeOcrResult {
        NativeOcrResult {
            text: text.into(), segments: vec![OcrSegment {
                text: text.into(), bounds: [0, 0, 100, 50],
            }],
            engine: "mlkit_japanese_bundled".into(), engine_version: "16.0.1".into(),
            elapsed_ms: 42,
        }
    }

    #[test]
    fn b03_ocr_persists_raw_text_and_manual_correction_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let asset = store.save(png_drop(&tiny_png())).unwrap();
        let initial = store.load_ocr(&asset.id).unwrap();
        assert_eq!(initial.status, "not_run");
        let result = store.save_recognition(&asset.id, recognized_ocr("パクる")).unwrap();
        assert_eq!(result.status, "recognized");
        assert_eq!(result.raw_text.as_deref(), Some("パクる"));
        assert!(result.corrected_text.is_none());
        store.correct_ocr(&asset.id, "パクる（模仿）").unwrap();
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        let saved = reopened.load_ocr(&asset.id).unwrap();
        assert_eq!(saved.raw_text.as_deref(), Some("パクる"));
        assert_eq!(saved.corrected_text.as_deref(), Some("パクる（模仿）"));
        assert_eq!(saved.engine.as_deref(), Some("mlkit_japanese_bundled"));
        assert_eq!(saved.segments[0].bounds, [0, 0, 100, 50]);
    }

    #[test]
    fn b03_paddle_v6_replaces_mlkit_without_migrating_private_photos() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        let mut updated = recognized_ocr("日本語能力試験 JLPT N2");
        updated.engine = "paddle_ocrv6_small_onnx".into();
        updated.engine_version = "6.0.0".into();
        let ocr = store.save_recognition(&saved.id, updated).unwrap();
        assert_eq!(ocr.raw_text.as_deref(), Some("日本語能力試験 JLPT N2"));
        assert_eq!(ocr.engine.as_deref(), Some("paddle_ocrv6_small_onnx"));
        assert_eq!(ocr.engine_version.as_deref(), Some("6.0.0"));
        assert_eq!(store.load(&saved.id).unwrap().summary.id, saved.id);
    }

    #[test]
    fn b03_unknown_is_not_faked_and_manual_correction_stays_independent() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let asset = store.save(png_drop(&tiny_png())).unwrap();
        let result = store.save_recognition(&asset.id, recognized_ocr("")).unwrap();
        assert_eq!(result.status, "unknown");
        assert_eq!(result.raw_text.as_deref(), Some(""));
        let edited = store.correct_ocr(&asset.id, "本人手動輸入").unwrap();
        assert_eq!(edited.corrected_text.as_deref(), Some("本人手動輸入"));
        let later = store.save_recognition(&asset.id, recognized_ocr("自動更新")).unwrap();
        assert_eq!(later.corrected_text.as_deref(), Some("本人手動輸入"));
        assert_eq!(later.raw_text.as_deref(), Some("自動更新"));
    }

    #[test]
    fn b03_correction_on_text_or_missing_id_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let text = store.save(text_drop("JLPT N2")).unwrap();
        assert_eq!(store.correct_ocr(&text.id, "other").unwrap_err(), "OCR_IMAGE_REQUIRED");
        assert_eq!(store.load_ocr(&text.id).unwrap_err(), "OCR_IMAGE_REQUIRED");
        assert_eq!(store.load_ocr("../../secret").unwrap_err(), "INVALID_ASSET_ID");
    }

    #[test]
    fn b03_empty_and_oversized_correction_does_not_write() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let asset = store.save(png_drop(&tiny_png())).unwrap();
        assert_eq!(store.correct_ocr(&asset.id, "  ").unwrap_err(), "EMPTY_TEXT");
        assert_eq!(store.correct_ocr(&asset.id, &"日".repeat(23000)).unwrap_err(),
            "TEXT_TOO_LARGE");
        assert_eq!(store.load_ocr(&asset.id).unwrap().status, "not_run");
    }

    #[test]
    fn b03_delete_image_cascades_local_ocr_and_does_not_touch_other_asset() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let first = store.save(png_drop(&tiny_png())).unwrap();
        let second = store.save(png_drop(&tiny_png())).unwrap();
        store.correct_ocr(&first.id, "秘密文字").unwrap();
        store.correct_ocr(&second.id, "保留文字").unwrap();
        store.delete(&first.id).unwrap();
        assert_eq!(store.load_ocr(&first.id).unwrap_err(), "ASSET_NOT_FOUND");
        assert_eq!(store.load_ocr(&second.id).unwrap().corrected_text.as_deref(),
            Some("保留文字"));
        let conn = store.connect().unwrap();
        let n: i64 = conn.query_row("SELECT count(*) FROM local_ocr", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn b03_untrusted_ocr_engine_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let asset = store.save(png_drop(&tiny_png())).unwrap();
        let mut result = recognized_ocr("偽造");
        result.engine = "cloud_ai".into();
        assert_eq!(store.save_recognition(&asset.id, result).unwrap_err(), "OCR_INVALID_RESULT");
        assert_eq!(store.load_ocr(&asset.id).unwrap().status, "not_run");
    }


    fn model_reply(json: &str) -> NativeReply {
        NativeReply {
            raw_text: json.into(), model_id: "gemma-4-E2B-it".into(),
            runtime: "litert-lm-android-0.16.1".into(),
            backend: "cpu".into(), elapsed_ms: 98,
        }
    }

    #[test]
    fn b03_model_saves_evidence_only_as_private_suggestion() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる JLPT N2")).unwrap();
        let input = store.model_input_for_asset(&saved.id).unwrap();
        let reply = model_reply(r#"{"evidenceQuote":"パクる","contentHint":"日語學習詞彙","uncertainty":null}"#);
        let suggestion = store.save_model_suggestion(&saved.id, &input, reply).unwrap();
        assert_eq!(suggestion.status, "suggested_private");
        assert_eq!(suggestion.evidence_quote, "パクる");
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert_eq!(reopened.load_model_suggestion(&saved.id).unwrap().unwrap().content_hint,
                   Some("日語學習詞彙".into()));
    }

    #[test]
    fn b03_model_rejects_fabricated_quote_and_consent_forgery() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる")).unwrap();
        for (json, code) in [
            (r#"{"evidenceQuote":"不存在","contentHint":null,"uncertainty":null}"#,
             "MODEL_EVIDENCE_MISMATCH"),
            (r#"{"evidenceQuote":"パクる","contentHint":null,"uncertainty":null,"consent":true}"#,
             "MODEL_OUTPUT_FORMAT"),
            (r#"{"evidenceQuote":"パクる","contentHint":null,"uncertainty":null,"goal":"Study Match"}"#,
             "MODEL_OUTPUT_FORMAT"),
            (r#"{"contentHint":"bad", "visibility":"public"}"#,
             "MODEL_OUTPUT_FORMAT"),
            ("{\"contentHint\":", "MODEL_OUTPUT_FORMAT"),
            ("Consent: true; Goal: match this person", "MODEL_OUTPUT_UNAPPROVED_FIELDS"),
        ] {
            assert_eq!(store.save_model_suggestion(&saved.id, "パクる", model_reply(json))
                       .unwrap_err(), code);
        }
        assert!(store.load_model_suggestion(&saved.id).unwrap().is_none());
    }

    #[test]
    fn us01a_text_goal_candidate_persists_without_creating_confirmed_goal() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let source = "我想找人一起練 JLPT N2 口說";
        let asset = store.save(text_drop(source)).unwrap();
        let reply = model_reply(r#"{"evidenceQuote":"我想找人","contentHint":"想與人練口說",
            "uncertainty":null,
            "intentCandidate":{"domain":"study",
            "goalText":"我想找真人學伴一起練 JLPT N2 口說",
            "evidenceQuote":"我想找人一起練"}}"#);
        let saved = store.save_model_suggestion(&asset.id, source, reply).unwrap();
        let candidate = saved.intent_candidate.unwrap();
        assert_eq!(candidate.domain, "study");
        assert_eq!(candidate.goal_text, "我想找真人學伴一起練 JLPT N2 口說");
        assert_eq!(candidate.evidence_quote, "我想找人一起練");
        let goal = crate::goal_draft::GoalStore::open(dir.path()).unwrap();
        assert!(goal.load(&asset.id).unwrap().is_none(),
            "AI must not create user confirmation event/Goal");
        drop(store);
        let store = LocalStore::open(dir.path()).unwrap();
        assert_eq!(store.load_model_suggestion(&asset.id).unwrap()
            .unwrap().intent_candidate.unwrap().domain, "study");
        assert!(goal.load(&asset.id).unwrap().is_none());
    }

    #[test]
    fn us01a_plain_model_reply_keeps_explicit_user_goal_editable() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let source = "我想找人一起練 JLPT N2 口說";
        let asset = store.save(text_drop(source)).unwrap();
        let saved = store.save_model_suggestion(&asset.id, source,
            model_reply("使用者想找人一起練習 JLPT N2 口說。")).unwrap();
        let candidate = saved.intent_candidate.expect("source-extracted private fallback");
        assert_eq!(candidate.domain, "unknown");
        assert_eq!(candidate.goal_text, source);
        assert_eq!(candidate.evidence_quote, source);
    }

    #[test]
    fn us01a_single_word_and_advertisement_must_not_forge_match_intent() {
        for source in ["パクる", "本日限定！居酒屋 全品半額", "BMW E46 改裝教學"] {
            let dir = tempfile::tempdir().unwrap();
            let store = LocalStore::open(dir.path()).unwrap();
            let asset = store.save(text_drop(source)).unwrap();
            let reply = format!(r#"{{"evidenceQuote":"{}","contentHint":"內容辨識",
                "intentCandidate":{{"domain":"study","goalText":"想找真人學伴",
                "evidenceQuote":"{}"}}}}"#, source, source);
            let saved = store.save_model_suggestion(&asset.id, source, model_reply(&reply)).unwrap();
            assert_eq!(saved.intent_candidate.as_ref().unwrap().evidence_quote, source,
                "AI hypotheses remain displayable for ambiguous topics: {source}");
            assert_eq!(saved.status, "suggested_private");
            assert!(crate::goal_draft::GoalStore::open(dir.path())
                .unwrap().load(&asset.id).unwrap().is_none());
        }
    }

    #[test]
    fn us01_e2b_jlpt_three_private_hypotheses_visible_without_explicit_wish() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let asset = store.save(text_drop("JLPT")).unwrap();
        let response = model_reply(r#"{"contentHint":"日語能力測驗",
            "intentCandidates":[
                {"domain":"study","goalText":"了解 JLPT 級別","evidenceQuote":"JLPT"},
                {"domain":"study","goalText":"準備 JLPT 考試","evidenceQuote":"JLPT"},
                {"domain":"study","goalText":"找人一起練日文","evidenceQuote":"JLPT"}]}"#);
        let result = store.save_model_suggestion(&asset.id, "JLPT", response).unwrap();
        assert_eq!(result.intent_candidates.len(), 3);
        assert_eq!(result.intent_candidates[0].goal_text, "了解 JLPT 級別");
        assert_eq!(result.intent_candidates[2].goal_text, "找人一起練日文");
        assert_eq!(result.status, "suggested_private");
        assert!(crate::goal_draft::GoalStore::open(dir.path())
            .unwrap().load(&asset.id).unwrap().is_none(),
            "AI hypotheses cannot forge Goal confirmation");
        drop(store);
        let reopened = LocalStore::open(dir.path()).unwrap();
        assert_eq!(reopened.load_model_suggestion(&asset.id).unwrap()
            .unwrap().intent_candidates.len(), 3);
    }

    #[test]
    fn us01a_fashion_candidate_private_but_never_auto_publish() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let source = "我想找人一起研究服裝設計";
        let asset = store.save(text_drop(source)).unwrap();
        let reply = model_reply(r#"{"contentHint":"時尚合作",
            "intentCandidate":{"domain":"fashion","goalText":"找人一起研究服裝設計",
            "evidenceQuote":"找人一起研究服裝設計"}}"#);
        let saved = store.save_model_suggestion(&asset.id, source, reply).unwrap();
        assert_eq!(saved.intent_candidate.unwrap().domain, "fashion");
        assert!(crate::goal_draft::GoalStore::open(dir.path())
            .unwrap().load(&asset.id).unwrap().is_none());
    }

    #[test]
    fn us01a_bad_intent_evidence_and_fake_consent_are_not_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let source = "我想找人一起練 JLPT N2 口說";
        let asset = store.save(text_drop(source)).unwrap();
        let invalid_evidence = model_reply(r#"{"contentHint":"學習",
           "intentCandidate":{"domain":"study","goalText":"練口說",
           "evidenceQuote":"來源沒有這句"}} "#);
        let result = store.save_model_suggestion(&asset.id, source, invalid_evidence).unwrap();
        let fallback = result.intent_candidate.expect("source-only fallback");
        assert_eq!(fallback.domain, "unknown");
        assert_eq!(fallback.goal_text, source);
        assert_eq!(fallback.evidence_quote, source);
        let fake_consent = model_reply(r#"{"evidenceQuote":"我想找人","contentHint":"學習",
           "intentCandidate":{"domain":"study","goalText":"練口說",
           "evidenceQuote":"我想找人","consent":true}}"#);
        assert_eq!(store.save_model_suggestion(&asset.id, source, fake_consent).unwrap_err(),
            "MODEL_OUTPUT_FORMAT");
    }

    #[test]
    fn b03_phone_regression_japanese_word_fenced_json_is_validated_and_private() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる")).unwrap();
        let reply = model_reply("```json\n{\"evidenceQuote\":\"「パクる」\",\"contentHint\":\"模仿或抄襲、偷竊（視語境）\",\"uncertainty\":null}\n```");
        let result = store.save_model_suggestion(&saved.id, "パクる", reply).unwrap();
        assert_eq!(result.status, "suggested_private");
        assert_eq!(result.evidence_quote, "パクる");
        assert_eq!(result.content_hint.as_deref(), Some("模仿或抄襲、偷竊（視語境）"));
        assert_eq!(store.load_model_suggestion(&saved.id).unwrap().unwrap().status,
                   "suggested_private");
    }

    #[test]
    fn b03_phone_regression_japanese_word_freeform_hint_can_be_saved_privately() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる")).unwrap();
        let result = store.save_model_suggestion(
            &saved.id, "パクる", model_reply("「パクる」は日常語，表示模仿、抄襲或偷竊。"),
        ).unwrap();
        assert_eq!(result.evidence_quote, "パクる");
        assert_eq!(result.status, "suggested_private");
        assert!(result.uncertainty.unwrap().contains("一般文字"));
    }

    #[test]
    fn b03_phone_regression_missing_model_quote_is_sourced_by_rust_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる")).unwrap();
        let reply = model_reply(r#"{"contentHint":"抄襲或偷竊","uncertainty":null}"#);
        let result = store.save_model_suggestion(&saved.id, "パクる", reply).unwrap();
        assert_eq!(result.evidence_quote, "パクる");
        assert_eq!(result.content_hint.as_deref(), Some("抄襲或偷竊"));
    }

    #[test]
    fn b03_phone_regression_no_untrusted_reply_is_allowed_to_invent_evidence() {
        assert_eq!(
            parse_private_model_reply("パクる", r#"{"evidenceQuote":"勉強します","contentHint":"foo","uncertainty":null}"#)
                .unwrap_err(), "MODEL_EVIDENCE_MISMATCH"
        );
        assert_eq!(
            parse_private_model_reply("パクる", "{\"contentHint\":\"not closed\"")
                .unwrap_err(), "MODEL_OUTPUT_FORMAT"
        );
        assert_eq!(
            parse_private_model_reply("パクる", "```json\n{\"contentHint\": \"x\"}\n```\nExtra")
                .unwrap_err(), "MODEL_OUTPUT_FORMAT"
        );
    }

    #[test]
    fn b03_model_rejects_stale_ocr_correction_and_deletion_cascades() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(png_drop(&tiny_png())).unwrap();
        assert_eq!(store.model_input_for_asset(&saved.id).unwrap_err(), "MODEL_INPUT_UNKNOWN");
        store.correct_ocr(&saved.id, "パクる").unwrap();
        let before = store.model_input_for_asset(&saved.id).unwrap();
        let reply = model_reply(r#"{"evidenceQuote":"パクる","contentHint":"學習","uncertainty":null}"#);
        store.save_model_suggestion(&saved.id, &before, reply).unwrap();
        store.correct_ocr(&saved.id, "あたらしい").unwrap();
        assert!(store.load_model_suggestion(&saved.id).unwrap().is_none());
        assert_eq!(store.save_model_suggestion(&saved.id, &before,
            model_reply(r#"{"evidenceQuote":"パクる","contentHint":null,"uncertainty":null}"#))
            .unwrap_err(), "MODEL_STALE_INPUT");
        store.delete(&saved.id).unwrap();
        let conn = store.connect().unwrap();
        let n: i64 = conn.query_row("SELECT count(*) FROM local_model_suggestions", [],
            |row| row.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn b03_model_rejects_untrusted_runtime_and_bad_ids() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::open(dir.path()).unwrap();
        let saved = store.save(text_drop("パクる")).unwrap();
        let mut reply = model_reply(r#"{"evidenceQuote":"パクる","contentHint":null,"uncertainty":null}"#);
        reply.runtime = "cloud_ai".into();
        assert_eq!(store.save_model_suggestion(&saved.id, "パクる", reply).unwrap_err(),
                   "MODEL_INVALID_OUTPUT");
        assert_eq!(store.model_input_for_asset("../../secret").unwrap_err(), "INVALID_ASSET_ID");
    }

    #[test]
    fn us01b_ocr_and_text_share_the_private_intent_goal_pipeline() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let image=store.save(png_drop(&tiny_png())).unwrap();
        let original="我想找人一起練 JLPT N2 口說";
        assert_eq!(store.model_input_for_asset(&image.id).unwrap_err(),"MODEL_INPUT_UNKNOWN");
        store.save_recognition(&image.id,recognized_ocr(original)).unwrap();
        assert_eq!(store.model_input_for_asset(&image.id).unwrap(),original);
        let result=store.save_model_suggestion(&image.id,original,
            model_reply(r#"{"contentHint":"日文口說",
                "intentCandidate":{"domain":"study","goalText":"找 N2 口說學伴",
                "evidenceQuote":"我想找人一起練"}}"#)).unwrap();
        assert_eq!(result.intent_candidate.unwrap().domain,"study");
        let goals=crate::goal_draft::GoalStore::open(dir.path()).unwrap();
        assert!(goals.load(&image.id).unwrap().is_none());
        let confirmed=goals.confirm(crate::goal_draft::ConfirmGoalInput {
            asset_id:image.id.clone(), goal_text:"本人確認 N2 口說".into(),
            confirmation_action:"confirm_goal".into(),expected_revision:0,
        }).unwrap();
        assert_eq!(confirmed.intent_state,"draft_private");
        assert!(!confirmed.share_consent);
        assert_eq!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&image.id).unwrap().unwrap().goal_text,"本人確認 N2 口說");
    }

    #[test]
    fn us01b_ocr_source_correction_invalidates_candidate_and_rejects_old_result() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let image=store.save(png_drop(&tiny_png())).unwrap();
        let first="我想找人一起練 JLPT N2 口說";
        store.save_recognition(&image.id,recognized_ocr(first)).unwrap();
        store.save_model_suggestion(&image.id,first,
            model_reply(r#"{"contentHint":"N2","intentCandidate":{"domain":"study",
                "goalText":"找學伴","evidenceQuote":"我想找人"}}"#)).unwrap();
        store.correct_ocr(&image.id,"時裝版型設計參考 パクる").unwrap();
        assert_eq!(store.model_input_for_asset(&image.id).unwrap(),"時裝版型設計參考 パクる");
        assert!(store.load_model_suggestion(&image.id).unwrap().is_none());
        assert_eq!(store.save_model_suggestion(&image.id,first,
            model_reply(r#"{"contentHint":"過期"}"#)).unwrap_err(),"MODEL_STALE_INPUT");
        let next=store.save_model_suggestion(&image.id,"時裝版型設計參考 パクる",
            model_reply(r#"{"contentHint":"服裝設計語境",
                "intentCandidate":{"domain":"study","goalText":"找學伴",
                "evidenceQuote":"パクる"}}"#)).unwrap();
        assert_eq!(next.intent_candidate.as_ref().unwrap().goal_text, "找學伴",
            "private hypothesis may display even when context is ambiguous");
        assert!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&image.id).unwrap().is_none());
    }

    #[test]
    fn us01b_image_without_ocr_remains_unknown_and_private() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let image=store.save(png_drop(&tiny_png())).unwrap();
        store.save_recognition(&image.id,recognized_ocr("")).unwrap();
        assert_eq!(store.model_input_for_asset(&image.id).unwrap_err(),"MODEL_INPUT_UNKNOWN");
        assert!(store.load_ocr(&image.id).unwrap().raw_text.unwrap().is_empty());
        assert!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&image.id).unwrap().is_none());
    }

    #[test]
    fn b09_one_question_then_person_answer_feeds_same_private_gemma_source() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let image=store.save(png_drop(&tiny_png())).unwrap();
        store.save_recognition(&image.id,recognized_ocr("パクる")).unwrap();
        let first=store.model_input_for_asset(&image.id).unwrap();
        let proposal=store.save_model_suggestion(&image.id,&first,model_reply(
            r#"{"evidenceQuote":"パクる","contentHint":"一個多義詞",
                "clarifyingQuestion":"你想只收藏，還是找真人練習？"}"#,
        )).unwrap();
        let question=proposal.pending_question.expect("model can ask one private question");
        assert_eq!(question.ordinal,1);
        assert_eq!(question.text,"你想只收藏，還是找真人練習？");
        assert!(proposal.intent_candidate.is_none());
        assert!(!first.contains(&question.text),
            "Model-written question must not be passed off as user evidence");
        assert_eq!(store.model_input_for_asset(&image.id).unwrap(),"パクる");
        store.answer_intent_question(&image.id,1,"我想找人一起練 JLPT N2 口說").unwrap();
        assert_eq!(store.answer_intent_question(&image.id,1,"再答一次").unwrap_err(),
            "B09_QUESTION_STALE");
        let answer_context=store.model_input_for_asset(&image.id).unwrap();
        assert!(answer_context.contains("【本人補充（非模型推測）】"));
        assert!(answer_context.contains("我想找人一起練 JLPT N2 口說"));
        assert!(!answer_context.contains(&question.text));
        assert!(store.load_model_suggestion(&image.id).unwrap().is_none(),
            "Source-versioned model output must be recomputed after human answer");
        let second=store.save_model_suggestion(&image.id,&answer_context,model_reply(
            r#"{"contentHint":"真人口說練習",
                "intentCandidate":{"domain":"study",
                "goalText":"想找真人學伴練 JLPT N2 口說",
                "evidenceQuote":"我想找人一起練 JLPT N2 口說"}}"#,
        )).unwrap();
        assert!(second.pending_question.is_none());
        assert_eq!(second.intent_candidate.unwrap().domain,"study");
        assert!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&image.id).unwrap().is_none(),"AI cannot confirm Goal");
    }

    #[test]
    fn b09_only_two_clarifications_then_human_goal_confirm_is_third_checkpoint() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let image=store.save(text_drop("パクる")).unwrap();
        let first=store.model_input_for_asset(&image.id).unwrap();
        let reply1=r#"{"contentHint":"多義詞","clarifyingQuestion":"你想了解詞義嗎？"}"#;
        let v1=store.save_model_suggestion(&image.id,&first,model_reply(reply1)).unwrap();
        assert_eq!(v1.pending_question.unwrap().ordinal,1);
        let repeated=store.save_model_suggestion(&image.id,&first,
            model_reply(r#"{"contentHint":"多義詞","clarifyingQuestion":"想找人聊聊嗎？"}"#)
        ).unwrap();
        assert_eq!(repeated.pending_question.unwrap().ordinal,1,
            "Retry cannot issue an extra question while one is outstanding");
        store.answer_intent_question(&image.id,1,"想了解詞義").unwrap();
        let next=store.model_input_for_asset(&image.id).unwrap();
        let v2=store.save_model_suggestion(&image.id,&next,
            model_reply(r#"{"contentHint":"多義詞",
                "clarifyingQuestion":"你要保存還是找人練習？"}"#)).unwrap();
        assert_eq!(v2.pending_question.unwrap().ordinal,2);
        store.answer_intent_question(&image.id,2,"先只收藏這個詞").unwrap();
        let last=store.model_input_for_asset(&image.id).unwrap();
        let v3=store.save_model_suggestion(&image.id,&last,model_reply(
            r#"{"contentHint":"詞義",
                "clarifyingQuestion":"第三個追問不應發出"}"#,
        )).unwrap();
        assert!(v3.pending_question.is_none(),
            "SQLite should permit at most two model clarifications; Goal confirm is separate");
        assert_eq!(store.answer_intent_question(&image.id,3,"不可以").unwrap_err(),
            "B09_INVALID_ANSWER");
        let conn=store.connect().unwrap();
        let count:i64=conn.query_row(
            "SELECT count(*) FROM local_intent_questions WHERE asset_id=?1",
            params![image.id],|r|r.get(0)).unwrap();
        assert_eq!(count,2);
        assert!(conn.execute(
            "INSERT INTO local_intent_questions
             (asset_id,base_source_sha256,ordinal,question,issued_at_ms)
             VALUES (?1,?2,3,'wrong',123)",
            params![image.id,format!("{:x}",Sha256::digest("パクる".as_bytes()))],
        ).is_err(),"direct SQL cannot insert a third question");
        assert!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&image.id).unwrap().is_none());
    }

    #[test]
    fn b09_source_revision_change_rejects_old_ocr_question_answer() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let image=store.save(png_drop(&tiny_png())).unwrap();
        store.correct_ocr(&image.id,"パクる").unwrap();
        let source=store.model_input_for_asset(&image.id).unwrap();
        store.save_model_suggestion(&image.id,&source,model_reply(
            r#"{"contentHint":"多義詞","clarifyingQuestion":"要找人討論嗎？"}"#,
        )).unwrap();
        store.correct_ocr(&image.id,"BMW E46 改裝參考").unwrap();
        assert!(store.load_model_suggestion(&image.id).unwrap().is_none());
        assert_eq!(store.answer_intent_question(&image.id,1,"找人").unwrap_err(),
            "B09_QUESTION_STALE");
        assert_eq!(store.save_model_suggestion(&image.id,&source,
            model_reply(r#"{"contentHint":"過期"}"#)).unwrap_err(),"MODEL_STALE_INPUT");
    }

    #[test]
    fn us01_multiple_private_hypotheses_evidence_checked_and_sqlite_legacy_compatible() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let input="我想找人一起練 JLPT N2 口說，也想找人交換文法題目";
        let asset=store.save(text_drop(input)).unwrap();
        let result=store.save_model_suggestion(&asset.id,input,model_reply(
            r#"{"evidenceQuote":"我想找人一起練",
                "contentHint":"日語合作",
                "intentCandidates":[
                  {"domain":"study","goalText":"找人一起練 N2 口說",
                   "evidenceQuote":"我想找人一起練"},
                  {"domain":"study","goalText":"找人交換 N2 文法題",
                   "evidenceQuote":"想找人交換文法題目"}
                ]}"#,
        )).unwrap();
        assert_eq!(result.intent_candidates.len(),2);
        assert_eq!(result.intent_candidate.as_ref().unwrap(),&result.intent_candidates[0]);
        assert!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&asset.id).unwrap().is_none(),"multiple AI hypotheses cannot confirm Goal");
        assert_eq!(store.load_model_suggestion(&asset.id).unwrap()
            .unwrap().intent_candidates.len(),2);
        // Previously installed PR#16 Android stored a single candidate JSON object,
        // not an array. A schema upgrade must still read it without data loss.
        let prior=serde_json::to_string(&result.intent_candidates[0]).unwrap();
        store.connect().unwrap().execute(
            "UPDATE local_intent_candidates SET candidate_json=?1 WHERE asset_id=?2",
            params![prior,asset.id],
        ).unwrap();
        let old=store.load_model_suggestion(&asset.id).unwrap().unwrap();
        assert_eq!(old.intent_candidates.len(),1);
        assert_eq!(old.intent_candidate.unwrap().goal_text,"找人一起練 N2 口說");
    }

    #[test]
    fn us01_no_goal_provenance_cannot_generate_any_private_hypotheses() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let input="時裝設計をパクる　BMW E46 改裝參考";
        let asset=store.save(text_drop(input)).unwrap();
        let response=store.save_model_suggestion(&asset.id,input,model_reply(
            r#"{"contentHint":"時裝／汽車不同主題",
                "intentCandidates":[
                  {"domain":"study","goalText":"找 N2 學伴",
                   "evidenceQuote":"パクる"},
                  {"domain":"car","goalText":"找汽車技師",
                   "evidenceQuote":"BMW E46"}]}"#,
        )).unwrap();
        assert!(response.intent_candidate.is_none());
        assert!(response.intent_candidates.is_empty());
        assert!(crate::goal_draft::GoalStore::open(dir.path()).unwrap()
            .load(&asset.id).unwrap().is_none());
    }

    #[test]
    fn us01_multi_candidates_reject_forged_model_confirmation_and_large_set() {
        let dir=tempfile::tempdir().unwrap();
        let store=LocalStore::open(dir.path()).unwrap();
        let input="我想找人一起練 JLPT N2";
        let asset=store.save(text_drop(input)).unwrap();
        assert_eq!(store.save_model_suggestion(&asset.id,input,model_reply(
            r#"{"intentCandidates":[
                {"domain":"study","goalText":"N2","evidenceQuote":"我想找人","confirmed":true}]}"#
        )).unwrap_err(),"MODEL_OUTPUT_FORMAT");
        let items=vec![
            serde_json::json!({"domain":"study","goalText":"a","evidenceQuote":"我想找人"});
            4
        ];
        let raw=serde_json::json!({"contentHint":"N2","intentCandidates":items}).to_string();
        assert_eq!(store.save_model_suggestion(&asset.id,input,model_reply(&raw))
            .unwrap_err(),"MODEL_OUTPUT_UNAPPROVED_FIELDS");
    }

}
