use crate::Publisher;
use async_trait::async_trait;
use chrono::Utc;
use contentforge_core::{Content, ContentForgeError, Platform, PlatformAdaptation, Publication};
use uuid::Uuid;

/// LinkedIn API adapter.
///
/// API docs: https://learn.microsoft.com/en-us/linkedin/marketing/community-management/shares/posts-api
/// Auth: OAuth 2.0 with `w_member_social` scope.
pub struct LinkedInPublisher {
    client: reqwest::Client,
    access_token: String,
    /// LinkedIn person URN (e.g., "urn:li:person:ABC123").
    author_urn: String,
}

impl LinkedInPublisher {
    const BASE_URL: &'static str = "https://api.linkedin.com/rest";

    pub fn new(access_token: String, author_urn: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token,
            author_urn,
        }
    }
}

#[async_trait]
impl Publisher for LinkedInPublisher {
    fn platform(&self) -> Platform {
        Platform::LinkedIn
    }

    async fn publish(
        &self,
        content: &Content,
        adaptation: &PlatformAdaptation,
    ) -> Result<Publication, ContentForgeError> {
        self.validate(adaptation)?;

        let payload = serde_json::json!({
            "author": &self.author_urn,
            "commentary": &adaptation.body,
            "visibility": "PUBLIC",
            "distribution": {
                "feedDistribution": "MAIN_FEED",
                "targetEntities": [],
                "thirdPartyDistributionChannels": []
            },
            "lifecycleState": "PUBLISHED",
            "isReshareDisabledByAuthor": false
        });

        let resp = self
            .client
            .post(format!("{}/posts", Self::BASE_URL))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("LinkedIn-Version", "202501")
            .header("X-Restli-Protocol-Version", "2.0.0")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::LinkedIn,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::LinkedIn,
                message: body,
            });
        }

        // LinkedIn returns the post ID in the x-restli-id header.
        let post_id = resp
            .headers()
            .get("x-restli-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::LinkedIn,
            url: format!("https://www.linkedin.com/feed/update/{post_id}"),
            platform_post_id: post_id,
            published_at: Utc::now(),
        })
    }

    async fn delete(&self, publication: &Publication) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .delete(format!(
                "{}/posts/{}",
                Self::BASE_URL,
                publication.platform_post_id
            ))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("LinkedIn-Version", "202501")
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::LinkedIn,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::LinkedIn,
                message: format!("Delete failed: {body}"),
            });
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .get("https://api.linkedin.com/v2/userinfo")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(|_| ContentForgeError::AuthFailed(Platform::LinkedIn))?;

        if resp.status().as_u16() == 401 {
            return Err(ContentForgeError::AuthFailed(Platform::LinkedIn));
        }
        Ok(())
    }
}
