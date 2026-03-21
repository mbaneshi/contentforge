use std::sync::Arc;

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use contentforge_core::ContentType;
use contentforge_db::DbPool;
use contentforge_publish::PublisherRegistry;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Embedded SvelteKit frontend
// ---------------------------------------------------------------------------

#[derive(RustEmbed)]
#[folder = "../../frontend/build"]
#[prefix = ""]
struct FrontendAssets;

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

/// Shared state injected into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub publishers: Arc<PublisherRegistry>,
    /// Broadcast channel for real-time WebSocket updates.
    pub ws_tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(db: DbPool, publishers: PublisherRegistry) -> Self {
        let (ws_tx, _) = broadcast::channel(256);
        Self {
            db,
            publishers: Arc::new(publishers),
            ws_tx,
        }
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build the full application router.
pub fn app_router(state: AppState) -> Router {
    let api = Router::new()
        // Content CRUD
        .route("/api/content", get(list_content).post(create_content))
        .route(
            "/api/content/{id}",
            get(get_content).put(update_content).delete(delete_content),
        )
        // Publish
        .route("/api/content/{id}/publish", post(publish_content))
        // Schedule
        .route("/api/schedule", get(list_schedule).post(create_schedule))
        // Platforms
        .route("/api/platforms", get(list_platforms))
        // Analytics
        .route("/api/analytics", get(analytics_dashboard))
        // WebSocket
        .route("/ws", get(ws_handler));

    let spa = Router::new().fallback(get(serve_frontend));

    api.merge(spa)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    pub title: String,
    pub body: String,
    pub content_type: ContentType,
    pub tags: Option<Vec<String>>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub content_id: Uuid,
    pub platform: contentforge_core::Platform,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsDashboard {
    pub total_content: usize,
    pub published_count: usize,
    pub scheduled_count: usize,
    pub draft_count: usize,
}

// ---------------------------------------------------------------------------
// Content handlers
// ---------------------------------------------------------------------------

async fn list_content(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: query all content via ContentRepo
    Json(serde_json::json!({ "content": [] }))
}

async fn create_content(
    State(_state): State<AppState>,
    Json(_payload): Json<CreateContentRequest>,
) -> impl IntoResponse {
    // TODO: create content via ContentRepo
    Json(serde_json::json!({ "status": "created" }))
}

async fn get_content(State(_state): State<AppState>, Path(_id): Path<Uuid>) -> impl IntoResponse {
    // TODO: fetch by id
    Json(serde_json::json!({ "content": null }))
}

async fn update_content(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_payload): Json<UpdateContentRequest>,
) -> impl IntoResponse {
    // TODO: update content
    Json(serde_json::json!({ "status": "updated" }))
}

async fn delete_content(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: delete content
    Json(serde_json::json!({ "status": "deleted" }))
}

// ---------------------------------------------------------------------------
// Publish handler
// ---------------------------------------------------------------------------

async fn publish_content(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: look up content, call publishers.publish_all()
    Json(serde_json::json!({ "status": "published", "results": [] }))
}

// ---------------------------------------------------------------------------
// Schedule handlers
// ---------------------------------------------------------------------------

async fn list_schedule(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: list pending schedule entries
    Json(serde_json::json!({ "schedule": [] }))
}

async fn create_schedule(
    State(_state): State<AppState>,
    Json(_payload): Json<CreateScheduleRequest>,
) -> impl IntoResponse {
    // TODO: create schedule entry
    Json(serde_json::json!({ "status": "scheduled" }))
}

// ---------------------------------------------------------------------------
// Platform handler
// ---------------------------------------------------------------------------

async fn list_platforms(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: return configured platforms from PublisherRegistry
    Json(serde_json::json!({ "platforms": [] }))
}

// ---------------------------------------------------------------------------
// Analytics handler
// ---------------------------------------------------------------------------

async fn analytics_dashboard(State(_state): State<AppState>) -> impl IntoResponse {
    let dashboard = AnalyticsDashboard {
        total_content: 0,
        published_count: 0,
        scheduled_count: 0,
        draft_count: 0,
    };
    Json(dashboard)
}

// ---------------------------------------------------------------------------
// WebSocket handler
// ---------------------------------------------------------------------------

async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        let mut rx = state.ws_tx.subscribe();
        while let Ok(msg) = rx.recv().await {
            if socket
                .send(axum::extract::ws::Message::Text(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    })
}

// ---------------------------------------------------------------------------
// SPA fallback (embedded frontend)
// ---------------------------------------------------------------------------

async fn serve_frontend(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try exact asset match first.
    if let Some(file) = FrontendAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .header("content-type", mime.as_ref())
            .body(axum::body::Body::from(file.data.to_vec()))
            .unwrap();
    }

    // SPA fallback: serve index.html for all non-asset routes.
    match FrontendAssets::get("index.html") {
        Some(index) => Html(String::from_utf8_lossy(&index.data).to_string()).into_response(),
        None => Response::builder()
            .status(404)
            .body(axum::body::Body::from("Frontend not built"))
            .unwrap(),
    }
}
