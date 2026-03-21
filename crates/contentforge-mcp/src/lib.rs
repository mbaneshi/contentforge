use std::sync::Arc;

use contentforge_db::DbPool;
use contentforge_publish::PublisherRegistry;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ServerInfo;
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MCP Server
// ---------------------------------------------------------------------------

/// ContentForge MCP server exposing content pipeline tools to AI assistants.
#[derive(Clone)]
#[allow(dead_code)]
pub struct ContentForgeMcp {
    db: DbPool,
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
    /// Content type: article, thread, short_post, etc.
    pub content_type: String,
    /// Optional tags for organization.
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListContentParams {
    /// Filter by status: idea, drafting, review, ready, scheduled, published, archived.
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AdaptContentParams {
    /// UUID of the content to adapt.
    pub content_id: String,
    /// Target platform (twitter, linkedin, devto, medium, etc.).
    pub platform: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PublishContentParams {
    /// UUID of the content to publish.
    pub content_id: String,
    /// Target platform.
    pub platform: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ScheduleContentParams {
    /// UUID of the content to schedule.
    pub content_id: String,
    /// Target platform.
    pub platform: String,
    /// Scheduled time in RFC 3339 format.
    pub scheduled_at: String,
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

    /// Create a new content draft with title, body, and content type.
    #[tool]
    fn draft_content(&self, Parameters(params): Parameters<DraftContentParams>) -> String {
        // TODO: Parse content_type, create Content, insert via ContentRepo
        tracing::info!(title = %params.title, "MCP: draft_content called");
        serde_json::json!({
            "status": "created",
            "title": params.title,
        })
        .to_string()
    }

    /// List content items in the pipeline, optionally filtered by status.
    #[tool]
    fn list_content(&self, Parameters(params): Parameters<ListContentParams>) -> String {
        // TODO: Query ContentRepo, filter by status
        tracing::info!(status = ?params.status, "MCP: list_content called");
        serde_json::json!({
            "content": [],
        })
        .to_string()
    }

    /// Adapt content for a specific platform using AI.
    #[tool]
    fn adapt_content(&self, Parameters(params): Parameters<AdaptContentParams>) -> String {
        // TODO: Load content, use ContentAdapter to generate adaptation
        tracing::info!(
            content_id = %params.content_id,
            platform = %params.platform,
            "MCP: adapt_content called"
        );
        serde_json::json!({
            "status": "adapted",
            "content_id": params.content_id,
            "platform": params.platform,
        })
        .to_string()
    }

    /// Publish content to a platform (e.g., twitter, linkedin, devto).
    #[tool]
    fn publish_content(&self, Parameters(params): Parameters<PublishContentParams>) -> String {
        // TODO: Load content + adaptation, call publisher
        tracing::info!(
            content_id = %params.content_id,
            platform = %params.platform,
            "MCP: publish_content called"
        );
        serde_json::json!({
            "status": "published",
            "content_id": params.content_id,
            "platform": params.platform,
        })
        .to_string()
    }

    /// Schedule content for publication at a specific time.
    #[tool]
    fn schedule_content(&self, Parameters(params): Parameters<ScheduleContentParams>) -> String {
        // TODO: Create ScheduleEntry and insert into DB
        tracing::info!(
            content_id = %params.content_id,
            platform = %params.platform,
            scheduled_at = %params.scheduled_at,
            "MCP: schedule_content called"
        );
        serde_json::json!({
            "status": "scheduled",
            "content_id": params.content_id,
            "platform": params.platform,
            "scheduled_at": params.scheduled_at,
        })
        .to_string()
    }

    /// Show current content pipeline status with counts by stage.
    #[tool]
    fn pipeline_status(&self) -> String {
        // TODO: Query DB for content counts by status
        tracing::info!("MCP: pipeline_status called");
        serde_json::json!({
            "ideas": 0,
            "drafts": 0,
            "in_review": 0,
            "ready": 0,
            "scheduled": 0,
            "published": 0,
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
            instructions: Some("ContentForge: AI-powered content pipeline management. Use these tools to create, adapt, publish, and schedule content across multiple platforms.".into()),
            ..Default::default()
        }
    }
}
