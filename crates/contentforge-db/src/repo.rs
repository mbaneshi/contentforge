use crate::DbPool;
use anyhow::Result;
use contentforge_core::{
    Content, ContentStatus, ContentType, Platform, PlatformAccount, PlatformAdaptation,
    PlatformCredential, Publication,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extension trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>>;
}

impl<T> OptionalExt<T> for std::result::Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

/// Parse a datetime string that may be RFC 3339 or SQLite's `datetime('now')` format.
fn parse_datetime(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| {
            // SQLite datetime('now') format: "YYYY-MM-DD HH:MM:SS"
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
                .unwrap_or_else(|_| chrono::Utc::now())
        })
}

fn parse_content_row(row: &rusqlite::Row) -> rusqlite::Result<Content> {
    Ok(Content {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
        title: row.get(1)?,
        body: row.get(2)?,
        content_type: serde_json::from_str::<ContentType>(&row.get::<_, String>(3)?).unwrap(),
        status: serde_json::from_str::<ContentStatus>(&row.get::<_, String>(4)?).unwrap(),
        tags: serde_json::from_str(&row.get::<_, String>(5)?).unwrap(),
        project: row.get(6)?,
        adaptations: Vec::new(),
        media: Vec::new(),
        created_at: parse_datetime(&row.get::<_, String>(7)?),
        updated_at: parse_datetime(&row.get::<_, String>(8)?),
    })
}

// ---------------------------------------------------------------------------
// ContentRepo
// ---------------------------------------------------------------------------

pub struct ContentRepo {
    db: DbPool,
}

impl ContentRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn insert(&self, content: &Content) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "INSERT INTO content (id, title, body, content_type, status, tags, project, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                content.id.to_string(),
                content.title,
                content.body,
                serde_json::to_string(&content.content_type)?,
                serde_json::to_string(&content.status)?,
                serde_json::to_string(&content.tags)?,
                content.project,
                content.created_at.to_rfc3339(),
                content.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: Uuid) -> Result<Option<Content>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT id, title, body, content_type, status, tags, project, created_at, updated_at
             FROM content WHERE id = ?1",
        )?;
        stmt.query_row([id.to_string()], parse_content_row)
            .optional()
    }

    pub fn get_by_id_full(&self, id: Uuid) -> Result<Option<Content>> {
        let mut content = match self.get_by_id(id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let adapt_repo = AdaptationRepo::new(self.db.clone());
        content.adaptations = adapt_repo.list_for_content(id)?;
        Ok(Some(content))
    }

    pub fn list_all(&self) -> Result<Vec<Content>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT id, title, body, content_type, status, tags, project, created_at, updated_at
             FROM content ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], parse_content_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_by_status(&self, status: ContentStatus) -> Result<Vec<Content>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let status_str = serde_json::to_string(&status)?;
        let mut stmt = conn.prepare(
            "SELECT id, title, body, content_type, status, tags, project, created_at, updated_at
             FROM content WHERE status = ?1 ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([status_str], parse_content_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn update_status(&self, id: Uuid, status: ContentStatus) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE content SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![serde_json::to_string(&status)?, id.to_string()],
        )?;
        Ok(())
    }

    pub fn update_body(&self, id: Uuid, body: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "UPDATE content SET body = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![body, id.to_string()],
        )?;
        Ok(())
    }

    pub fn count_by_status(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt =
            conn.prepare("SELECT status, count(*) FROM content GROUP BY status ORDER BY status")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute("DELETE FROM content WHERE id = ?1", [id.to_string()])?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// AdaptationRepo
// ---------------------------------------------------------------------------

pub struct AdaptationRepo {
    db: DbPool,
}

impl AdaptationRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn upsert(&self, content_id: Uuid, adaptation: &PlatformAdaptation) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "INSERT OR REPLACE INTO adaptations (content_id, platform, title, body, thread_parts, canonical_url, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                content_id.to_string(),
                serde_json::to_string(&adaptation.platform)?,
                adaptation.title,
                adaptation.body,
                adaptation.thread_parts.as_ref().map(|p| serde_json::to_string(p).unwrap()),
                adaptation.canonical_url,
                serde_json::to_string(&adaptation.metadata)?,
            ],
        )?;
        Ok(())
    }

    pub fn get_for_platform(
        &self,
        content_id: Uuid,
        platform: Platform,
    ) -> Result<Option<PlatformAdaptation>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT platform, title, body, thread_parts, canonical_url, metadata
             FROM adaptations WHERE content_id = ?1 AND platform = ?2",
        )?;
        let platform_str = serde_json::to_string(&platform)?;
        stmt.query_row(
            rusqlite::params![content_id.to_string(), platform_str],
            |row| {
                Ok(PlatformAdaptation {
                    platform: serde_json::from_str::<Platform>(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    body: row.get(2)?,
                    thread_parts: row
                        .get::<_, Option<String>>(3)?
                        .map(|s| serde_json::from_str(&s).unwrap()),
                    canonical_url: row.get(4)?,
                    metadata: serde_json::from_str(&row.get::<_, String>(5)?).unwrap(),
                })
            },
        )
        .optional()
    }

    pub fn list_for_content(&self, content_id: Uuid) -> Result<Vec<PlatformAdaptation>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT platform, title, body, thread_parts, canonical_url, metadata
             FROM adaptations WHERE content_id = ?1",
        )?;
        let rows = stmt
            .query_map([content_id.to_string()], |row| {
                Ok(PlatformAdaptation {
                    platform: serde_json::from_str::<Platform>(&row.get::<_, String>(0)?).unwrap(),
                    title: row.get(1)?,
                    body: row.get(2)?,
                    thread_parts: row
                        .get::<_, Option<String>>(3)?
                        .map(|s| serde_json::from_str(&s).unwrap()),
                    canonical_url: row.get(4)?,
                    metadata: serde_json::from_str(&row.get::<_, String>(5)?).unwrap(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

// ---------------------------------------------------------------------------
// PublicationRepo
// ---------------------------------------------------------------------------

pub struct PublicationRepo {
    db: DbPool,
}

impl PublicationRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn insert(&self, pub_record: &Publication) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "INSERT INTO publications (id, content_id, platform, url, platform_post_id, published_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                pub_record.id.to_string(),
                pub_record.content_id.to_string(),
                serde_json::to_string(&pub_record.platform)?,
                pub_record.url,
                pub_record.platform_post_id,
                pub_record.published_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_for_content(&self, content_id: Uuid) -> Result<Vec<Publication>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT id, content_id, platform, url, platform_post_id, published_at
             FROM publications WHERE content_id = ?1 ORDER BY published_at DESC",
        )?;
        let rows = stmt
            .query_map([content_id.to_string()], |row| {
                Ok(Publication {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    content_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    platform: serde_json::from_str::<Platform>(&row.get::<_, String>(2)?).unwrap(),
                    url: row.get(3)?,
                    platform_post_id: row.get(4)?,
                    published_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<Publication>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT id, content_id, platform, url, platform_post_id, published_at
             FROM publications ORDER BY published_at DESC LIMIT ?1",
        )?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(Publication {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    content_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                    platform: serde_json::from_str::<Platform>(&row.get::<_, String>(2)?).unwrap(),
                    url: row.get(3)?,
                    platform_post_id: row.get(4)?,
                    published_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

// ---------------------------------------------------------------------------
// PlatformAccountRepo
// ---------------------------------------------------------------------------

pub struct PlatformAccountRepo {
    db: DbPool,
}

impl PlatformAccountRepo {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn insert(&self, account: &PlatformAccount) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        conn.execute(
            "INSERT OR REPLACE INTO platform_accounts (id, platform, display_name, credential, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                account.id.to_string(),
                serde_json::to_string(&account.platform)?,
                account.display_name,
                serde_json::to_string(&account.credential)?,
                account.enabled as i32,
                account.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_platform(&self, platform: Platform) -> Result<Option<PlatformAccount>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let platform_str = serde_json::to_string(&platform)?;
        let mut stmt = conn.prepare(
            "SELECT id, platform, display_name, credential, enabled, created_at
             FROM platform_accounts WHERE platform = ?1 AND enabled = 1",
        )?;
        stmt.query_row([platform_str], |row| {
            Ok(PlatformAccount {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                platform: serde_json::from_str::<Platform>(&row.get::<_, String>(1)?).unwrap(),
                display_name: row.get(2)?,
                credential: serde_json::from_str::<PlatformCredential>(&row.get::<_, String>(3)?)
                    .unwrap(),
                enabled: row.get::<_, i32>(4)? != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
        })
        .optional()
    }

    pub fn list_all(&self) -> Result<Vec<PlatformAccount>> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut stmt = conn.prepare(
            "SELECT id, platform, display_name, credential, enabled, created_at
             FROM platform_accounts ORDER BY platform",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PlatformAccount {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    platform: serde_json::from_str::<Platform>(&row.get::<_, String>(1)?).unwrap(),
                    display_name: row.get(2)?,
                    credential: serde_json::from_str::<PlatformCredential>(
                        &row.get::<_, String>(3)?,
                    )
                    .unwrap(),
                    enabled: row.get::<_, i32>(4)? != 0,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn delete(&self, platform: Platform) -> Result<()> {
        let conn = self.db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let platform_str = serde_json::to_string(&platform)?;
        conn.execute(
            "DELETE FROM platform_accounts WHERE platform = ?1",
            [platform_str],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contentforge_core::ContentType;

    fn test_db() -> DbPool {
        crate::init_memory_db().unwrap()
    }

    #[test]
    fn test_content_crud() {
        let db = test_db();
        let repo = ContentRepo::new(db);

        let content = Content::new("Test Post", "Hello world", ContentType::Article);
        repo.insert(&content).unwrap();

        let loaded = repo.get_by_id(content.id).unwrap().unwrap();
        assert_eq!(loaded.title, "Test Post");
        assert_eq!(loaded.body, "Hello world");

        repo.update_status(content.id, ContentStatus::Drafting)
            .unwrap();
        let updated = repo.get_by_id(content.id).unwrap().unwrap();
        assert_eq!(updated.status, ContentStatus::Drafting);

        let list = repo.list_all().unwrap();
        assert_eq!(list.len(), 1);

        repo.delete(content.id).unwrap();
        assert!(repo.get_by_id(content.id).unwrap().is_none());
    }

    #[test]
    fn test_adaptation_upsert() {
        let db = test_db();
        let content_repo = ContentRepo::new(db.clone());
        let adapt_repo = AdaptationRepo::new(db);

        let content = Content::new("Test", "body", ContentType::Article);
        content_repo.insert(&content).unwrap();

        let adaptation = PlatformAdaptation {
            platform: Platform::DevTo,
            title: Some("DEV title".to_string()),
            body: "adapted body".to_string(),
            thread_parts: None,
            canonical_url: None,
            metadata: serde_json::json!({}),
        };

        adapt_repo.upsert(content.id, &adaptation).unwrap();

        let loaded = adapt_repo
            .get_for_platform(content.id, Platform::DevTo)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.body, "adapted body");
        assert_eq!(loaded.title.unwrap(), "DEV title");

        let all = adapt_repo.list_for_content(content.id).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_platform_account() {
        let db = test_db();
        let repo = PlatformAccountRepo::new(db);

        let account = PlatformAccount {
            id: Uuid::new_v4(),
            platform: Platform::DevTo,
            display_name: "My DEV.to".to_string(),
            credential: PlatformCredential::ApiKey {
                key: "test-key".to_string(),
            },
            enabled: true,
            created_at: chrono::Utc::now(),
        };

        repo.insert(&account).unwrap();

        let loaded = repo.get_by_platform(Platform::DevTo).unwrap().unwrap();
        assert_eq!(loaded.display_name, "My DEV.to");

        let all = repo.list_all().unwrap();
        assert_eq!(all.len(), 1);
    }
}
