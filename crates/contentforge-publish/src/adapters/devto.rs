use crate::Publisher;
use async_trait::async_trait;
use contentforge_core::{
    Content, ContentForgeError, Platform, PlatformAdaptation, Publication,
};
use chrono::Utc;
use uuid::Uuid;

/// DEV.to API adapter.
///
/// API docs: https://developers.forem.com/api/v1
/// Auth: API key from Settings > Extensions > DEV Community API Keys
pub struct DevToPublisher {
    client: reqwest::Client,
    api_key: String,
}

impl DevToPublisher {
    const BASE_URL: &'static str = "https://dev.to/api";

    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

#[derive(serde::Serialize)]
struct CreateArticle {
    article: ArticleBody,
}

#[derive(serde::Serialize)]
struct ArticleBody {
    title: String,
    body_markdown: String,
    published: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    series: Option<String>,
}

#[derive(serde::Deserialize)]
struct ArticleResponse {
    id: i64,
    url: String,
}

#[async_trait]
impl Publisher for DevToPublisher {
    fn platform(&self) -> Platform {
        Platform::DevTo
    }

    async fn publish(
        &self,
        content: &Content,
        adaptation: &PlatformAdaptation,
    ) -> Result<Publication, ContentForgeError> {
        self.validate(adaptation)?;

        let title = adaptation
            .title
            .as_deref()
            .unwrap_or(&content.title)
            .to_string();

        let payload = CreateArticle {
            article: ArticleBody {
                title,
                body_markdown: adaptation.body.clone(),
                published: true,
                tags: if content.tags.is_empty() {
                    None
                } else {
                    Some(content.tags.iter().take(4).cloned().collect())
                },
                canonical_url: adaptation.canonical_url.clone(),
                series: None,
            },
        };

        let resp = self
            .client
            .post(format!("{}/articles", Self::BASE_URL))
            .header("api-key", &self.api_key)
            .header("Accept", "application/vnd.forem.api-v1+json")
            .header("User-Agent", "contentforge/0.1.0")
            .json(&payload)
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::DevTo,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::DevTo,
                message: format!("HTTP {status}: {body}"),
            });
        }

        let article: ArticleResponse =
            resp.json().await.map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::DevTo,
                message: e.to_string(),
            })?;

        Ok(Publication {
            id: Uuid::new_v4(),
            content_id: content.id,
            platform: Platform::DevTo,
            url: article.url,
            platform_post_id: article.id.to_string(),
            published_at: Utc::now(),
        })
    }

    async fn delete(&self, publication: &Publication) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .delete(format!(
                "{}/articles/{}",
                Self::BASE_URL,
                publication.platform_post_id
            ))
            .header("api-key", &self.api_key)
            .header("Accept", "application/vnd.forem.api-v1+json")
            .header("User-Agent", "contentforge/0.1.0")
            .send()
            .await
            .map_err(|e| ContentForgeError::PublishFailed {
                platform: Platform::DevTo,
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ContentForgeError::PublishFailed {
                platform: Platform::DevTo,
                message: format!("Delete failed: {body}"),
            });
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<(), ContentForgeError> {
        let resp = self
            .client
            .get(format!("{}/articles/me", Self::BASE_URL))
            .header("api-key", &self.api_key)
            .header("Accept", "application/vnd.forem.api-v1+json")
            .header("User-Agent", "contentforge/0.1.0")
            .send()
            .await
            .map_err(|_| ContentForgeError::AuthFailed(Platform::DevTo))?;

        if resp.status().as_u16() == 401 {
            return Err(ContentForgeError::AuthFailed(Platform::DevTo));
        }

        Ok(())
    }
}
