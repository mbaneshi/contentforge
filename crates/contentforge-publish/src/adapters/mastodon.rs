use crate::Publisher;
use async_trait::async_trait;
use chrono::Utc;
use contentforge_core::{Content, ContentForgeError, Platform, PlatformAdaptation, Publication};
use uuid::Uuid;

/// Mastodon API adapter.
///
/// API docs: https://docs.joinmastodon.org/methods/statuses/
/// Auth: OAuth 2.0 per-instance. Each Mastodon server has its own app registration.
/// Post endpoint: POST /api/v1/statuses
pub struct MastodonPublisher {
    client: reqwest::Client,
    instance_url: String,
    access_token: String,
}

impl MastodonPublisher {
    pub fn new(instance_url: String, access_token: String) -> Self {
        let instance_url = instance_url.trim_end_matches('/').to_string();
        Self {
            client: reqwest::Client::new(),
            instance_url,
            access_token,
        }
    }
}

#[derive(serde::Deserialize)]
struct StatusResponse {
    id: String,
    url: Option<String>,
}

#[async_trait]
impl Publisher for MastodonPublisher {
    fn platform(&self) -> Platform {
        Platform::Mastodon
    }

    async fn publish(
        &self,
        content: &Content,
        adaptation: &PlatformAdaptation,
    ) -> Result<Publication, ContentForgeError> {
        self.validate(adaptation)?;

        let resp = self
            .client
            .post(format!("{}/api/v1/statuses", self.instance_url))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .form(&[("status", &adaptation.body)])
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Mastodon,
                message: e.to_string(),
            })?;

        if resp.status().as_u16() == 429 {
            return Err(ContentForgeError::RateLimited {
                platform: Platform::Mastodon,
                retry_after_secs: 300,
            });
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Mastodon,
                message: format!("HTTP error: {body}"),
            });
        }

        let status: StatusResponse =
            resp.json()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Mastodon,
                    message: e.to_string(),
                })?;

        let url = status
            .url
            .unwrap_or_else(|| format!("{}/web/statuses/{}", self.instance_url, status.id));

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::Mastodon,
            url,
            platform_post_id: status.id,
            published_at: Utc::now(),
        })
    }

    async fn delete(&self, publication: &Publication) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .delete(format!(
                "{}/api/v1/statuses/{}",
                self.instance_url, publication.platform_post_id
            ))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Mastodon,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Mastodon,
                message: format!("Delete failed: {body}"),
            });
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .get(format!(
                "{}/api/v1/accounts/verify_credentials",
                self.instance_url
            ))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|_| ContentForgeError::AuthFailed(Platform::Mastodon))?;

        if resp.status().as_u16() == 401 {
            return Err(ContentForgeError::AuthFailed(Platform::Mastodon));
        }
        Ok(())
    }
}
