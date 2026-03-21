use crate::Publisher;
use async_trait::async_trait;
use chrono::Utc;
use contentforge_core::{Content, ContentForgeError, Platform, PlatformAdaptation, Publication};
use uuid::Uuid;

/// Medium API adapter.
///
/// API docs: https://github.com/Medium/medium-api-docs
/// Auth: Self-issued integration token from medium.com/me/settings
/// Note: The Medium API is deprecated but still functional for posting.
pub struct MediumPublisher {
    client: reqwest::Client,
    integration_token: String,
    author_id: Option<String>,
}

impl MediumPublisher {
    const BASE_URL: &'static str = "https://api.medium.com/v1";

    pub fn new(integration_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            integration_token,
            author_id: None,
        }
    }

    /// Fetch and cache the author ID.
    #[allow(dead_code)]
    async fn ensure_author_id(&mut self) -> Result<String, ContentForgeError> {
        if let Some(ref id) = self.author_id {
            return Ok(id.clone());
        }

        let resp = self
            .client
            .get(format!("{}/me", Self::BASE_URL))
            .header(
                "Authorization",
                format!("Bearer {}", self.integration_token),
            )
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Medium,
                message: e.to_string(),
            })?;

        let data: serde_json::Value =
            resp.json()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Medium,
                    message: e.to_string(),
                })?;

        let id = data["data"]["id"]
            .as_str()
            .ok_or(ContentForgeError::AuthFailed(Platform::Medium))?
            .to_string();

        self.author_id = Some(id.clone());
        Ok(id)
    }
}

#[async_trait]
impl Publisher for MediumPublisher {
    fn platform(&self) -> Platform {
        Platform::Medium
    }

    async fn publish(
        &self,
        content: &Content,
        adaptation: &PlatformAdaptation,
    ) -> Result<Publication, ContentForgeError> {
        let author_id = self
            .author_id
            .as_ref()
            .ok_or(ContentForgeError::AuthFailed(Platform::Medium))?;

        let title = adaptation
            .title
            .as_deref()
            .unwrap_or(&content.title)
            .to_string();

        let payload = serde_json::json!({
            "title": title,
            "contentFormat": "markdown",
            "content": &adaptation.body,
            "tags": content.tags.iter().take(5).collect::<Vec<_>>(),
            "canonicalUrl": adaptation.canonical_url,
            "publishStatus": "public"
        });

        let resp = self
            .client
            .post(format!("{}/users/{}/posts", Self::BASE_URL, author_id))
            .header(
                "Authorization",
                format!("Bearer {}", self.integration_token),
            )
            .json(&payload)
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Medium,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Medium,
                message: body,
            });
        }

        let data: serde_json::Value =
            resp.json()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Medium,
                    message: e.to_string(),
                })?;

        let post_id = data["data"]["id"].as_str().unwrap_or("unknown").to_string();
        let url = data["data"]["url"]
            .as_str()
            .unwrap_or("https://medium.com")
            .to_string();

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::Medium,
            url,
            platform_post_id: post_id,
            published_at: Utc::now(),
        })
    }

    async fn delete(&self, _publication: &Publication) -> Result<(), ContentForgeError> {
        // Medium API does not support deleting posts.
        Err(ContentForgeError::PublishFailed {
            platform: Platform::Medium,
            message: "Medium API does not support post deletion".to_string(),
        })
    }

    async fn health_check(&self) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .get(format!("{}/me", Self::BASE_URL))
            .header(
                "Authorization",
                format!("Bearer {}", self.integration_token),
            )
            .send()
            .await
            .map_err(|_| ContentForgeError::AuthFailed(Platform::Medium))?;

        if resp.status().as_u16() == 401 {
            return Err(ContentForgeError::AuthFailed(Platform::Medium));
        }
        Ok(())
    }
}
