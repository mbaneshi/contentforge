use crate::Publisher;
use async_trait::async_trait;
use chrono::Utc;
use contentforge_core::{Content, ContentForgeError, Platform, PlatformAdaptation, Publication};
use uuid::Uuid;

/// Twitter/X API v2 adapter.
///
/// API docs: https://docs.x.com/x-api
/// Auth: OAuth 2.0 (user context) for posting.
/// Rate limit: 200 tweets per 15 minutes (user), 500/month (free tier).
pub struct TwitterPublisher {
    client: reqwest::Client,
    bearer_token: String,
}

impl TwitterPublisher {
    const BASE_URL: &'static str = "https://api.x.com/2";

    pub fn new(bearer_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            bearer_token,
        }
    }

    /// Publish a thread by chaining replies.
    async fn publish_thread(
        &self,
        parts: &[String],
        content: &Content,
    ) -> Result<Publication, ContentForgeError> {
        let mut previous_id: Option<String> = None;
        let mut first_url = String::new();
        let mut first_id = String::new();

        for (i, part) in parts.iter().enumerate() {
            let mut body = serde_json::json!({ "text": part });

            if let Some(ref reply_to) = previous_id {
                body["reply"] = serde_json::json!({
                    "in_reply_to_tweet_id": reply_to
                });
            }

            let resp = self
                .client
                .post(format!("{}/tweets", Self::BASE_URL))
                .header("Authorization", format!("Bearer {}", self.bearer_token))
                .json(&body)
                .send()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Twitter,
                    message: e.to_string(),
                })?;

            if resp.status().as_u16() == 429 {
                return Err(ContentForgeError::RateLimited {
                    platform: Platform::Twitter,
                    retry_after_secs: 900, // 15 minutes
                });
            }

            if !resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(ContentForgeError::PublishFailed {
                    platform: Platform::Twitter,
                    message: format!("HTTP error on tweet {}: {body}", i + 1),
                });
            }

            let data: serde_json::Value =
                resp.json()
                    .await
                    .map_err(|e| ContentForgeError::PublishFailed {
                        platform: Platform::Twitter,
                        message: e.to_string(),
                    })?;

            let tweet_id = data["data"]["id"].as_str().unwrap_or_default().to_string();

            if i == 0 {
                first_id = tweet_id.clone();
                // Construct URL (username not available here, use generic).
                first_url = format!("https://x.com/i/status/{tweet_id}");
            }

            previous_id = Some(tweet_id);
        }

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::Twitter,
            url: first_url,
            platform_post_id: first_id,
            published_at: Utc::now(),
        })
    }
}

#[async_trait]
impl Publisher for TwitterPublisher {
    fn platform(&self) -> Platform {
        Platform::Twitter
    }

    async fn publish(
        &self,
        content: &Content,
        adaptation: &PlatformAdaptation,
    ) -> Result<Publication, ContentForgeError> {
        // If it's a thread, publish as a chain.
        if let Some(ref parts) = adaptation.thread_parts {
            return self.publish_thread(parts, content).await;
        }

        // Single tweet.
        self.validate(adaptation)?;

        let body = serde_json::json!({ "text": &adaptation.body });

        let resp = self
            .client
            .post(format!("{}/tweets", Self::BASE_URL))
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Twitter,
                message: e.to_string(),
            })?;

        if resp.status().as_u16() == 429 {
            return Err(ContentForgeError::RateLimited {
                platform: Platform::Twitter,
                retry_after_secs: 900,
            });
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Twitter,
                message: body,
            });
        }

        let data: serde_json::Value =
            resp.json()
                .await
                .map_err(|e| ContentForgeError::PublishFailed {
                    platform: Platform::Twitter,
                    message: e.to_string(),
                })?;

        let tweet_id = data["data"]["id"].as_str().unwrap_or_default().to_string();

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::Twitter,
            url: format!("https://x.com/i/status/{tweet_id}"),
            platform_post_id: tweet_id,
            published_at: Utc::now(),
        })
    }

    async fn delete(&self, publication: &Publication) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .delete(format!(
                "{}/tweets/{}",
                Self::BASE_URL,
                publication.platform_post_id
            ))
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::Twitter,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::Twitter,
                message: format!("Delete failed: {body}"),
            });
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .get(format!("{}/users/me", Self::BASE_URL))
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .send()
            .await
            .map_err(|_| ContentForgeError::AuthFailed(Platform::Twitter))?;

        if resp.status().as_u16() == 401 {
            return Err(ContentForgeError::AuthFailed(Platform::Twitter));
        }
        Ok(())
    }
}
