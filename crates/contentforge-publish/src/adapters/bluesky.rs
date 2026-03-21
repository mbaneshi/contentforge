use crate::Publisher;
use async_trait::async_trait;
use chrono::Utc;
use contentforge_core::{Content, ContentForgeError, Platform, PlatformAdaptation, Publication};
use uuid::Uuid;

/// Bluesky / AT Protocol adapter.
///
/// API docs: https://docs.bsky.app/
/// Auth: Handle + App Password → createSession → JWT access token
/// Post endpoint: POST /xrpc/com.atproto.repo.createRecord
const BSKY_API: &str = "https://bsky.social/xrpc";

pub struct BlueskyPublisher {
    client: reqwest::Client,
    handle: String,
    app_password: String,
}

impl BlueskyPublisher {
    pub fn new(handle: String, app_password: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            handle,
            app_password,
        }
    }

    /// Create a session and get access token + DID.
    async fn create_session(&self) -> Result<(String, String), ContentForgeError> {
        let resp = self
            .client
            .post(format!("{BSKY_API}/com.atproto.server.createSession"))
            .json(&serde_json::json!({
                "identifier": self.handle,
                "password": self.app_password,
            }))
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Bluesky,
                message: format!("Session creation failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let _body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::AuthFailed(Platform::Bluesky));
        }

        let data: serde_json::Value =
            resp.json()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Bluesky,
                    message: e.to_string(),
                })?;

        let access_jwt = data["accessJwt"]
            .as_str()
            .ok_or(ContentForgeError::AuthFailed(Platform::Bluesky))?
            .to_string();

        let did = data["did"]
            .as_str()
            .ok_or(ContentForgeError::AuthFailed(Platform::Bluesky))?
            .to_string();

        Ok((access_jwt, did))
    }
}

#[async_trait]
impl Publisher for BlueskyPublisher {
    fn platform(&self) -> Platform {
        Platform::Bluesky
    }

    async fn publish(
        &self,
        content: &Content,
        adaptation: &PlatformAdaptation,
    ) -> Result<Publication, ContentForgeError> {
        self.validate(adaptation)?;

        let (token, did) = self.create_session().await?;

        let now = Utc::now().to_rfc3339();
        let record = serde_json::json!({
            "repo": did,
            "collection": "app.bsky.feed.post",
            "record": {
                "$type": "app.bsky.feed.post",
                "text": adaptation.body,
                "createdAt": now,
            }
        });

        let resp = self
            .client
            .post(format!("{BSKY_API}/com.atproto.repo.createRecord"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&record)
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Bluesky,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Bluesky,
                message: body,
            });
        }

        let data: serde_json::Value =
            resp.json()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Bluesky,
                    message: e.to_string(),
                })?;

        let uri = data["uri"].as_str().unwrap_or_default().to_string();
        // Extract rkey from URI: at://did:plc:xxx/app.bsky.feed.post/rkey
        let rkey = uri.rsplit('/').next().unwrap_or_default();
        let url = format!("https://bsky.app/profile/{}/post/{rkey}", self.handle);

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::Bluesky,
            url,
            platform_post_id: uri,
            published_at: Utc::now(),
        })
    }

    async fn delete(&self, publication: &Publication) -> Result<(), ContentForgeError> {
        let (token, did) = self.create_session().await?;

        // Extract rkey from AT URI
        let rkey = publication
            .platform_post_id
            .rsplit('/')
            .next()
            .unwrap_or_default();

        let resp = self
            .client
            .post(format!("{BSKY_API}/com.atproto.repo.deleteRecord"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({
                "repo": did,
                "collection": "app.bsky.feed.post",
                "rkey": rkey,
            }))
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Bluesky,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Bluesky,
                message: format!("Delete failed: {body}"),
            });
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ContentForgeError> {
        self.create_session().await?;
        Ok(())
    }
}
