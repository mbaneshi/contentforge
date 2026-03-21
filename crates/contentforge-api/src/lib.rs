use std::sync::Arc;

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use contentforge_core::{
    Content, ContentStatus, ContentType, Platform, PlatformAccount, PlatformAdaptation,
    PlatformCredential,
};
use contentforge_db::repo::{AdaptationRepo, ContentRepo, PlatformAccountRepo, PublicationRepo};
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

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub publishers: Arc<PublisherRegistry>,
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

pub fn app_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/api/content", get(list_content).post(create_content))
        .route(
            "/api/content/{id}",
            get(get_content).put(update_content).delete(delete_content),
        )
        .route("/api/content/{id}/adapt", post(adapt_content))
        .route("/api/content/{id}/publish", post(publish_content))
        .route("/api/schedule", get(list_schedule).post(create_schedule))
        .route("/api/platforms", get(list_platforms).post(add_platform))
        .route("/api/platforms/{platform}", delete(remove_platform))
        .route("/api/analytics", get(analytics_dashboard))
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
    pub body: Option<String>,
    pub content_type: Option<ContentType>,
    pub tags: Option<Vec<String>>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<ContentStatus>,
    pub tags: Option<Vec<String>>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdaptRequest {
    pub platform: Platform,
    pub title: Option<String>,
    pub body: Option<String>,
    pub canonical_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub platform: Option<Platform>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub content_id: Uuid,
    pub platform: Platform,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddPlatformRequest {
    pub platform: Platform,
    pub key: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListContentQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContentResponse {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub content_type: ContentType,
    pub status: ContentStatus,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub adaptations: Vec<PlatformAdaptation>,
    pub publications: Vec<PublicationInfo>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct PublicationInfo {
    pub platform: Platform,
    pub url: String,
    pub published_at: String,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsDashboard {
    pub total_content: usize,
    pub by_status: Vec<StatusCount>,
    pub recent_publications: Vec<PublicationInfo>,
    pub platforms_configured: usize,
}

#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

fn api_err(status: StatusCode, msg: impl ToString) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": msg.to_string() })),
    )
}

fn to_content_response(c: &Content, db: &DbPool) -> ContentResponse {
    let pub_repo = PublicationRepo::new(db.clone());
    let publications = pub_repo
        .list_for_content(c.id)
        .unwrap_or_default()
        .into_iter()
        .map(|p| PublicationInfo {
            platform: p.platform,
            url: p.url,
            published_at: p.published_at.to_rfc3339(),
        })
        .collect();

    ContentResponse {
        id: c.id,
        title: c.title.clone(),
        body: c.body.clone(),
        content_type: c.content_type,
        status: c.status,
        tags: c.tags.clone(),
        project: c.project.clone(),
        adaptations: c.adaptations.clone(),
        publications,
        created_at: c.created_at.to_rfc3339(),
        updated_at: c.updated_at.to_rfc3339(),
    }
}

// ---------------------------------------------------------------------------
// Content handlers
// ---------------------------------------------------------------------------

async fn list_content(
    State(state): State<AppState>,
    Query(params): Query<ListContentQuery>,
) -> ApiResult<Vec<ContentResponse>> {
    let repo = ContentRepo::new(state.db.clone());

    let contents = if let Some(status_str) = params.status {
        let status: ContentStatus =
            serde_json::from_str(&format!("\"{status_str}\"")).map_err(|_| {
                api_err(
                    StatusCode::BAD_REQUEST,
                    format!("Invalid status: {status_str}"),
                )
            })?;
        repo.list_by_status(status)
    } else {
        repo.list_all()
    }
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Load adaptations for each
    let adapt_repo = AdaptationRepo::new(state.db.clone());
    let results: Vec<ContentResponse> = contents
        .iter()
        .map(|c| {
            let mut cr = to_content_response(c, &state.db);
            cr.adaptations = adapt_repo.list_for_content(c.id).unwrap_or_default();
            cr
        })
        .collect();

    Ok(Json(results))
}

async fn create_content(
    State(state): State<AppState>,
    Json(payload): Json<CreateContentRequest>,
) -> ApiResult<ContentResponse> {
    let ct = payload.content_type.unwrap_or(ContentType::Article);
    let mut content = Content::new(&payload.title, payload.body.unwrap_or_default(), ct);
    content.status = ContentStatus::Drafting;
    content.tags = payload.tags.unwrap_or_default();
    content.project = payload.project;

    let repo = ContentRepo::new(state.db.clone());
    repo.insert(&content)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let _ = state
        .ws_tx
        .send(serde_json::json!({"event": "content_created", "id": content.id}).to_string());

    Ok(Json(to_content_response(&content, &state.db)))
}

async fn get_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<ContentResponse> {
    let repo = ContentRepo::new(state.db.clone());
    let content = repo
        .get_by_id_full(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Content not found"))?;

    Ok(Json(to_content_response(&content, &state.db)))
}

async fn update_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateContentRequest>,
) -> ApiResult<ContentResponse> {
    let repo = ContentRepo::new(state.db.clone());
    let mut content = repo
        .get_by_id(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Content not found"))?;

    if let Some(title) = payload.title {
        content.title = title;
    }
    if let Some(body) = payload.body {
        repo.update_body(id, &body)
            .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        content.body = body;
    }
    if let Some(status) = payload.status {
        repo.update_status(id, status)
            .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        content.status = status;
    }

    let _ = state
        .ws_tx
        .send(serde_json::json!({"event": "content_updated", "id": id}).to_string());

    let content = repo
        .get_by_id_full(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Content not found"))?;

    Ok(Json(to_content_response(&content, &state.db)))
}

async fn delete_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
    let repo = ContentRepo::new(state.db.clone());
    repo.delete(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let _ = state
        .ws_tx
        .send(serde_json::json!({"event": "content_deleted", "id": id}).to_string());

    Ok(Json(serde_json::json!({ "deleted": id })))
}

// ---------------------------------------------------------------------------
// Adapt handler
// ---------------------------------------------------------------------------

async fn adapt_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdaptRequest>,
) -> ApiResult<ContentResponse> {
    let content_repo = ContentRepo::new(state.db.clone());
    let content = content_repo
        .get_by_id(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Content not found"))?;

    let body = payload.body.unwrap_or_else(|| content.body.clone());
    let adaptation = PlatformAdaptation {
        platform: payload.platform,
        title: payload.title.or(Some(content.title.clone())),
        body,
        thread_parts: None,
        canonical_url: payload.canonical_url,
        metadata: serde_json::json!({}),
    };

    let adapt_repo = AdaptationRepo::new(state.db.clone());
    adapt_repo
        .upsert(id, &adaptation)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if content.status == ContentStatus::Drafting || content.status == ContentStatus::Idea {
        content_repo
            .update_status(id, ContentStatus::Ready)
            .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    let content = content_repo
        .get_by_id_full(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Content not found"))?;

    Ok(Json(to_content_response(&content, &state.db)))
}

// ---------------------------------------------------------------------------
// Publish handler
// ---------------------------------------------------------------------------

async fn publish_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PublishRequest>,
) -> ApiResult<serde_json::Value> {
    let content_repo = ContentRepo::new(state.db.clone());
    let content = content_repo
        .get_by_id_full(id)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Content not found"))?;

    if content.adaptations.is_empty() {
        return Err(api_err(
            StatusCode::BAD_REQUEST,
            "No adaptations found. Adapt content for a platform first.",
        ));
    }

    let pub_repo = PublicationRepo::new(state.db.clone());
    let mut results = Vec::new();

    let adapts_to_publish: Vec<&PlatformAdaptation> = if let Some(platform) = payload.platform {
        content
            .adaptations
            .iter()
            .filter(|a| a.platform == platform)
            .collect()
    } else {
        content.adaptations.iter().collect()
    };

    for adaptation in adapts_to_publish {
        let publisher = state.publishers.get(adaptation.platform);
        match publisher {
            Some(p) => match p.publish(&content, adaptation).await {
                Ok(publication) => {
                    let _ = pub_repo.insert(&publication);
                    results.push(serde_json::json!({
                        "platform": adaptation.platform,
                        "status": "published",
                        "url": publication.url
                    }));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "platform": adaptation.platform,
                        "status": "failed",
                        "error": e.to_string()
                    }));
                }
            },
            None => {
                results.push(serde_json::json!({
                    "platform": adaptation.platform,
                    "status": "skipped",
                    "error": "Platform not configured"
                }));
            }
        }
    }

    content_repo
        .update_status(id, ContentStatus::Published)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let _ = state
        .ws_tx
        .send(serde_json::json!({"event": "content_published", "id": id}).to_string());

    Ok(Json(serde_json::json!({ "results": results })))
}

// ---------------------------------------------------------------------------
// Schedule handlers
// ---------------------------------------------------------------------------

async fn list_schedule(State(_state): State<AppState>) -> ApiResult<Vec<serde_json::Value>> {
    // TODO: implement with ScheduleRepo when scheduler is built
    Ok(Json(vec![]))
}

async fn create_schedule(
    State(_state): State<AppState>,
    Json(_payload): Json<CreateScheduleRequest>,
) -> ApiResult<serde_json::Value> {
    // TODO: implement with ScheduleRepo
    Ok(Json(
        serde_json::json!({ "status": "scheduled", "message": "Scheduling coming in Phase 4" }),
    ))
}

// ---------------------------------------------------------------------------
// Platform handlers
// ---------------------------------------------------------------------------

async fn list_platforms(State(state): State<AppState>) -> ApiResult<Vec<PlatformInfo>> {
    let repo = PlatformAccountRepo::new(state.db.clone());
    let accounts = repo
        .list_all()
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let infos: Vec<PlatformInfo> = accounts
        .into_iter()
        .map(|a| PlatformInfo {
            platform: a.platform,
            display_name: a.display_name,
            enabled: a.enabled,
            credential_type: match a.credential {
                PlatformCredential::ApiKey { .. } => "api_key".to_string(),
                PlatformCredential::OAuth2 { .. } => "oauth2".to_string(),
                PlatformCredential::IntegrationToken { .. } => "token".to_string(),
                PlatformCredential::Cookie { .. } => "cookie".to_string(),
                PlatformCredential::MastodonAuth { .. } => "mastodon".to_string(),
                PlatformCredential::BlueskyAuth { .. } => "bluesky".to_string(),
            },
        })
        .collect();

    Ok(Json(infos))
}

async fn add_platform(
    State(state): State<AppState>,
    Json(payload): Json<AddPlatformRequest>,
) -> ApiResult<serde_json::Value> {
    let credential = match payload.platform {
        Platform::DevTo => PlatformCredential::ApiKey { key: payload.key },
        Platform::Medium => PlatformCredential::IntegrationToken { token: payload.key },
        _ => PlatformCredential::OAuth2 {
            client_id: String::new(),
            client_secret: String::new(),
            access_token: Some(payload.key),
            refresh_token: None,
            expires_at: None,
        },
    };

    let account = PlatformAccount {
        id: Uuid::new_v4(),
        platform: payload.platform,
        display_name: payload.name.unwrap_or_else(|| "default".to_string()),
        credential,
        enabled: true,
        created_at: chrono::Utc::now(),
    };

    let repo = PlatformAccountRepo::new(state.db.clone());
    repo.insert(&account)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        serde_json::json!({ "added": payload.platform, "name": account.display_name }),
    ))
}

async fn remove_platform(
    State(state): State<AppState>,
    Path(platform_str): Path<String>,
) -> ApiResult<serde_json::Value> {
    let platform: Platform = platform_str
        .parse()
        .map_err(|e: String| api_err(StatusCode::BAD_REQUEST, e))?;

    let repo = PlatformAccountRepo::new(state.db.clone());
    repo.delete(platform)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "removed": platform })))
}

#[derive(Debug, Serialize)]
struct PlatformInfo {
    platform: Platform,
    display_name: String,
    enabled: bool,
    credential_type: String,
}

// ---------------------------------------------------------------------------
// Analytics handler
// ---------------------------------------------------------------------------

async fn analytics_dashboard(State(state): State<AppState>) -> ApiResult<AnalyticsDashboard> {
    let content_repo = ContentRepo::new(state.db.clone());
    let pub_repo = PublicationRepo::new(state.db.clone());
    let account_repo = PlatformAccountRepo::new(state.db.clone());

    let counts = content_repo
        .count_by_status()
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total: i64 = counts.iter().map(|(_, c)| c).sum();
    let by_status: Vec<StatusCount> = counts
        .into_iter()
        .map(|(s, c)| StatusCount {
            status: s.trim_matches('"').to_string(),
            count: c,
        })
        .collect();

    let recent = pub_repo.list_recent(10).unwrap_or_default();
    let recent_publications: Vec<PublicationInfo> = recent
        .into_iter()
        .map(|p| PublicationInfo {
            platform: p.platform,
            url: p.url,
            published_at: p.published_at.to_rfc3339(),
        })
        .collect();

    let platforms = account_repo.list_all().unwrap_or_default();

    Ok(Json(AnalyticsDashboard {
        total_content: total as usize,
        by_status,
        recent_publications,
        platforms_configured: platforms.len(),
    }))
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

    if let Some(file) = FrontendAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .header("content-type", mime.as_ref())
            .body(axum::body::Body::from(file.data.to_vec()))
            .unwrap();
    }

    match FrontendAssets::get("index.html") {
        Some(index) => Html(String::from_utf8_lossy(&index.data).to_string()).into_response(),
        None => Response::builder()
            .status(404)
            .body(axum::body::Body::from("Frontend not built"))
            .unwrap(),
    }
}
