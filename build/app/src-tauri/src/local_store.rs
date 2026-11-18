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
             CREATE INDEX IF NOT EXISTS idx_assets_fingerprint ON local_assets(sha256);",
        )
        .map_err(|_| store_error("DATABASE_MIGRATION_FAILED"))?;
        let store = Self { root: root.to_path_buf() };
        // 每次重新開啟儲存時修復中斷的刪除操作；以 SQLite 寫入鎖序列化恢復。
        store.recover_pending_deletes()?;
        Ok(store)
    }

    fn connect(&self) -> Result<Connection, String> {
        Connection::open(self.root.join("cakami_private.sqlite"))
            .map_err(|_| store_error("DATABASE_UNAVAILABLE"))
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
            // DB 已提交成功；若磁碟清理失敗，下一次 open 會再清理孤兒暫存檔。
            let _ = fs::remove_file(pending);
        }
        Ok(true)
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

}
