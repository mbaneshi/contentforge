use std::sync::Arc;

use contentforge_core::{Content, ContentStatus, ContentType, Platform, PlatformAdaptation};
use contentforge_db::repo::{AdaptationRepo, ContentRepo, PlatformAccountRepo, PublicationRepo};
use contentforge_db::DbPool;
use contentforge_publish::PublisherRegistry;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ServerInfo;
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// MCP Server
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ContentForgeMcp {
    db: DbPool,
    #[allow(dead_code)]
    publishers: Arc<PublisherRegistry>,
    tool_router: ToolRouter<Self>,
}

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DraftContentParams {
    /// Title for the new draft.
    pub title: String,
    /// Body content in markdown.
    pub body: String,
    /// Content type: article, thread, short_post, video, image_post, link_share.
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// Tags for organization (e.g., ["rust", "ai", "devtools"]).
    pub tags: Option<Vec<String>>,
    /// Associated project name (e.g., "codeilus", "contentforge").
    pub project: Option<String>,
}

fn default_content_type() -> String {
    "article".to_string()
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListContentParams {
    /// Filter by status: idea, drafting, review, ready, scheduled, published, archived. Leave empty for all.
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AdaptContentParams {
    /// UUID (or short prefix) of the content to adapt.
    pub content_id: String,
    /// Target platform: devto, twitter, linkedin, mastodon, bluesky, etc.
    pub platform: String,
    /// Optional custom title for this platform.
    pub title: Option<String>,
    /// Optional canonical URL for cross-posting SEO.
    pub canonical_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PublishContentParams {
    /// UUID (or short prefix) of the content to publish.
    pub content_id: String,
    /// Target platform: devto, twitter, linkedin, mastodon, bluesky, etc.
    pub platform: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ScheduleContentParams {
    /// UUID (or short prefix) of the content to schedule.
    pub content_id: String,
    /// Target platform.
    pub platform: String,
    /// Scheduled time in RFC 3339 format (e.g., "2026-03-25T08:00:00Z").
    pub scheduled_at: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ShowContentParams {
    /// UUID (or short prefix) of the content to show.
    pub content_id: String,
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

#[tool_router]
impl ContentForgeMcp {
    pub fn new(db: DbPool, publishers: Arc<PublisherRegistry>) -> Self {
        Self {
            db,
            publishers,
            tool_router: Self::tool_router(),
        }
    }

    /// Create a new content draft. Returns the created content with its UUID.
    #[tool]
    fn draft_content(&self, Parameters(params): Parameters<DraftContentParams>) -> String {
        let ct: ContentType = params.content_type.parse().unwrap_or(ContentType::Article);

        let mut content = Content::new(&params.title, &params.body, ct);
        content.status = ContentStatus::Drafting;
        content.tags = params.tags.unwrap_or_default();
        content.project = params.project;

        let repo = ContentRepo::new(self.db.clone());
        match repo.insert(&content) {
            Ok(()) => serde_json::json!({
                "status": "created",
                "id": content.id.to_string(),
                "title": content.title,
                "content_type": params.content_type,
                "tags": content.tags,
                "message": format!("Draft created. Use adapt_content with id '{}' to adapt for a platform.", &content.id.to_string()[..8])
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "error": format!("Failed to create draft: {e}")
            })
            .to_string(),
        }
    }

    /// List content in the pipeline. Optionally filter by status.
    #[tool]
    fn list_content(&self, Parameters(params): Parameters<ListContentParams>) -> String {
        let repo = ContentRepo::new(self.db.clone());

        let contents = if let Some(ref status_str) = params.status {
            match serde_json::from_str::<ContentStatus>(&format!("\"{status_str}\"")) {
                Ok(status) => repo.list_by_status(status),
                Err(_) => {
                    return serde_json::json!({
                        "error": format!("Invalid status: '{}'. Valid: idea, drafting, review, ready, scheduled, published, archived", status_str)
                    }).to_string();
                }
            }
        } else {
            repo.list_all()
        };

        match contents {
            Ok(items) => {
                let summary: Vec<serde_json::Value> = items
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "id": &c.id.to_string()[..8],
                            "full_id": c.id.to_string(),
                            "title": c.title,
                            "status": format!("{}", c.status),
                            "type": format!("{}", c.content_type),
                            "tags": c.tags,
                            "project": c.project,
                            "updated": c.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "count": summary.len(),
                    "content": summary,
                })
                .to_string()
            }
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    /// Show full details of a specific content item including adaptations and publications.
    #[tool]
    fn show_content(&self, Parameters(params): Parameters<ShowContentParams>) -> String {
        let uuid = match resolve_uuid(&params.content_id, &self.db) {
            Ok(id) => id,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let repo = ContentRepo::new(self.db.clone());
        match repo.get_by_id_full(uuid) {
            Ok(Some(content)) => {
                let adapt_list: Vec<String> = content
                    .adaptations
                    .iter()
                    .map(|a| format!("{}", a.platform))
                    .collect();

                let pub_repo = PublicationRepo::new(self.db.clone());
                let pubs = pub_repo.list_for_content(uuid).unwrap_or_default();
                let pub_list: Vec<serde_json::Value> = pubs
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "platform": format!("{}", p.platform),
                            "url": p.url,
                            "published_at": p.published_at.to_rfc3339(),
                        })
                    })
                    .collect();

                serde_json::json!({
                    "id": content.id.to_string(),
                    "title": content.title,
                    "body": content.body,
                    "status": format!("{}", content.status),
                    "type": format!("{}", content.content_type),
                    "tags": content.tags,
                    "project": content.project,
                    "adaptations": adapt_list,
                    "publications": pub_list,
                    "created": content.created_at.to_rfc3339(),
                    "updated": content.updated_at.to_rfc3339(),
                })
                .to_string()
            }
            Ok(None) => {
                serde_json::json!({ "error": format!("Content not found: {}", params.content_id) })
                    .to_string()
            }
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    /// Adapt content for a specific platform. Copies the markdown body as the adaptation (suitable for DEV.to, Mastodon, etc.).
    #[tool]
    fn adapt_content(&self, Parameters(params): Parameters<AdaptContentParams>) -> String {
        let uuid = match resolve_uuid(&params.content_id, &self.db) {
            Ok(id) => id,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let platform: Platform = match params.platform.parse() {
            Ok(p) => p,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let content_repo = ContentRepo::new(self.db.clone());
        let content = match content_repo.get_by_id(uuid) {
            Ok(Some(c)) => c,
            Ok(None) => {
                return serde_json::json!({ "error": "Content not found" }).to_string();
            }
            Err(e) => return serde_json::json!({ "error": e.to_string() }).to_string(),
        };

        let adaptation = PlatformAdaptation {
            platform,
            title: params.title.or(Some(content.title.clone())),
            body: content.body.clone(),
            thread_parts: None,
            canonical_url: params.canonical_url,
            metadata: serde_json::json!({}),
        };

        let adapt_repo = AdaptationRepo::new(self.db.clone());
        if let Err(e) = adapt_repo.upsert(uuid, &adaptation) {
            return serde_json::json!({ "error": e.to_string() }).to_string();
        }

        if content.status == ContentStatus::Drafting || content.status == ContentStatus::Idea {
            let _ = content_repo.update_status(uuid, ContentStatus::Ready);
        }

        let char_info = platform
            .char_limit()
            .map(|l| format!(" ({}/{} chars)", adaptation.body.len(), l))
            .unwrap_or_default();

        serde_json::json!({
            "status": "adapted",
            "content_id": &uuid.to_string()[..8],
            "platform": format!("{}", platform),
            "body_length": adaptation.body.len(),
            "message": format!("Adapted for {}{char_info}. Use publish_content to publish.", platform)
        })
        .to_string()
    }

    /// Publish adapted content to a platform. Content must be adapted first.
    #[tool]
    fn publish_content(&self, Parameters(params): Parameters<PublishContentParams>) -> String {
        let uuid = match resolve_uuid(&params.content_id, &self.db) {
            Ok(id) => id,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let platform: Platform = match params.platform.parse() {
            Ok(p) => p,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let content_repo = ContentRepo::new(self.db.clone());
        let content = match content_repo.get_by_id_full(uuid) {
            Ok(Some(c)) => c,
            Ok(None) => {
                return serde_json::json!({ "error": "Content not found" }).to_string();
            }
            Err(e) => return serde_json::json!({ "error": e.to_string() }).to_string(),
        };

        let adaptation = match content.adaptation_for(platform) {
            Some(a) => a,
            None => {
                return serde_json::json!({
                    "error": format!("No adaptation for {}. Run adapt_content first.", platform)
                })
                .to_string();
            }
        };

        // Try to get publisher from env vars
        let account_repo = PlatformAccountRepo::new(self.db.clone());
        let env_key = match platform {
            Platform::DevTo => std::env::var("DEVTO_API_KEY").ok(),
            Platform::Twitter => std::env::var("TWITTER_BEARER_TOKEN").ok(),
            _ => None,
        };

        // Check DB credentials
        let has_creds = account_repo
            .get_by_platform(platform)
            .ok()
            .flatten()
            .is_some();

        if env_key.is_none() && !has_creds {
            return serde_json::json!({
                "error": format!("No credentials for {}. Set env var or use the CLI: contentforge platforms add {}", platform, params.platform)
            }).to_string();
        }

        // For now, return guidance since MCP tools are sync and publish requires async
        // The user should use the CLI for actual publishing until we add async MCP support
        serde_json::json!({
            "status": "ready_to_publish",
            "content_id": &uuid.to_string()[..8],
            "platform": format!("{}", platform),
            "adaptation_chars": adaptation.body.len(),
            "message": format!(
                "Content is ready. Publish via CLI:\n  contentforge publish {} --platform {}",
                &uuid.to_string()[..8], params.platform
            ),
            "credentials_found": has_creds || env_key.is_some(),
        })
        .to_string()
    }

    /// Schedule content for publication at a specific future time.
    #[tool]
    fn schedule_content(&self, Parameters(params): Parameters<ScheduleContentParams>) -> String {
        let uuid = match resolve_uuid(&params.content_id, &self.db) {
            Ok(id) => id,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let platform: Platform = match params.platform.parse() {
            Ok(p) => p,
            Err(e) => return serde_json::json!({ "error": e }).to_string(),
        };

        let scheduled_at = match chrono::DateTime::parse_from_rfc3339(&params.scheduled_at) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(e) => {
                return serde_json::json!({
                    "error": format!("Invalid date format: {e}. Use RFC 3339: 2026-03-25T08:00:00Z")
                })
                .to_string();
            }
        };

        // Insert into schedule table
        let entry_id = Uuid::new_v4();
        let conn = match self.db.lock() {
            Ok(c) => c,
            Err(e) => return serde_json::json!({ "error": e.to_string() }).to_string(),
        };

        let result = conn.execute(
            "INSERT INTO schedule (id, content_id, platform, scheduled_at, status, retries, created_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', 0, datetime('now'))",
            rusqlite::params![
                entry_id.to_string(),
                uuid.to_string(),
                serde_json::to_string(&platform).unwrap_or_default(),
                scheduled_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => serde_json::json!({
                "status": "scheduled",
                "schedule_id": &entry_id.to_string()[..8],
                "content_id": &uuid.to_string()[..8],
                "platform": format!("{}", platform),
                "scheduled_at": scheduled_at.to_rfc3339(),
                "message": format!("Scheduled for {} on {}", platform, scheduled_at.format("%Y-%m-%d %H:%M UTC"))
            })
            .to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    /// Show current pipeline status: content counts by stage, configured platforms, and recent publications.
    #[tool]
    fn pipeline_status(&self) -> String {
        let content_repo = ContentRepo::new(self.db.clone());
        let pub_repo = PublicationRepo::new(self.db.clone());
        let account_repo = PlatformAccountRepo::new(self.db.clone());

        let counts = content_repo.count_by_status().unwrap_or_default();
        let total: i64 = counts.iter().map(|(_, c)| c).sum();
        let status_map: serde_json::Map<String, serde_json::Value> = counts
            .iter()
            .map(|(s, c)| (s.trim_matches('"').to_string(), serde_json::Value::from(*c)))
            .collect();

        let recent = pub_repo.list_recent(5).unwrap_or_default();
        let recent_pubs: Vec<serde_json::Value> = recent
            .iter()
            .map(|p| {
                serde_json::json!({
                    "platform": format!("{}", p.platform),
                    "url": p.url,
                    "published_at": p.published_at.format("%Y-%m-%d %H:%M").to_string(),
                })
            })
            .collect();

        let platforms = account_repo.list_all().unwrap_or_default();
        let platform_names: Vec<String> = platforms
            .iter()
            .map(|a| format!("{}", a.platform))
            .collect();

        serde_json::json!({
            "total_content": total,
            "by_status": status_map,
            "configured_platforms": platform_names,
            "recent_publications": recent_pubs,
        })
        .to_string()
    }
}

// ---------------------------------------------------------------------------
// ServerHandler implementation
// ---------------------------------------------------------------------------

#[tool_handler]
impl ServerHandler for ContentForgeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "ContentForge: Developer content pipeline. Create drafts, adapt for platforms (DEV.to, Mastodon, Bluesky, Twitter, LinkedIn), schedule, and publish — all from your AI assistant.\n\nTypical workflow:\n1. draft_content → create a post\n2. adapt_content → adapt for devto, mastodon, etc.\n3. publish_content → publish (or schedule_content for later)\n4. pipeline_status → see overview".into()
            ),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Utility: resolve short UUID prefix
// ---------------------------------------------------------------------------

fn resolve_uuid(id: &str, db: &DbPool) -> Result<Uuid, String> {
    if let Ok(uuid) = Uuid::parse_str(id) {
        return Ok(uuid);
    }

    let conn = db.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let mut stmt = conn
        .prepare("SELECT id FROM content WHERE id LIKE ?1")
        .map_err(|e| format!("Query error: {e}"))?;
    let pattern = format!("{id}%");
    let matches: Vec<String> = stmt
        .query_map([pattern], |row| row.get(0))
        .map_err(|e| format!("Query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    match matches.len() {
        0 => Err(format!("No content found matching '{id}'")),
        1 => Uuid::parse_str(&matches[0]).map_err(|e| format!("UUID parse error: {e}")),
        n => Err(format!(
            "Ambiguous ID '{id}' matches {n} items. Provide more characters."
        )),
    }
}
