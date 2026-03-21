use crate::platform::Platform;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// The lifecycle status of a content piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    /// Initial idea or outline.
    Idea,
    /// Being written.
    Drafting,
    /// Ready for review.
    Review,
    /// Approved, waiting to be scheduled.
    Ready,
    /// Scheduled for future publishing.
    Scheduled,
    /// Published to at least one platform.
    Published,
    /// Archived / no longer active.
    Archived,
}

/// The type of content being created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// A Twitter/X thread.
    Thread,
    /// A short social post (LinkedIn, single tweet).
    ShortPost,
    /// A long-form article (Substack, DEV.to, Medium).
    Article,
    /// A video (YouTube).
    Video,
    /// An image post (Instagram).
    ImagePost,
    /// A link share with commentary.
    LinkShare,
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "thread" => Ok(Self::Thread),
            "short_post" | "short" | "post" => Ok(Self::ShortPost),
            "article" => Ok(Self::Article),
            "video" => Ok(Self::Video),
            "image_post" | "image" => Ok(Self::ImagePost),
            "link_share" | "link" => Ok(Self::LinkShare),
            other => Err(format!("Unknown content type: '{other}'. Valid: thread, short_post, article, video, image_post, link_share")),
        }
    }
}

impl std::fmt::Display for ContentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idea => write!(f, "idea"),
            Self::Drafting => write!(f, "drafting"),
            Self::Review => write!(f, "review"),
            Self::Ready => write!(f, "ready"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Published => write!(f, "published"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thread => write!(f, "thread"),
            Self::ShortPost => write!(f, "short_post"),
            Self::Article => write!(f, "article"),
            Self::Video => write!(f, "video"),
            Self::ImagePost => write!(f, "image_post"),
            Self::LinkShare => write!(f, "link_share"),
        }
    }
}

/// A single piece of content — the central domain entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: Uuid,
    /// Internal title (not necessarily the published title).
    pub title: String,
    /// The canonical body in Markdown.
    pub body: String,
    pub content_type: ContentType,
    pub status: ContentStatus,
    /// Tags/topics for organization.
    pub tags: Vec<String>,
    /// Related project (e.g., "codeilus", "claude-forge").
    pub project: Option<String>,
    /// Platform-specific adaptations.
    pub adaptations: Vec<PlatformAdaptation>,
    /// Attached media (images, videos).
    pub media: Vec<MediaAttachment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A platform-specific version of the content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAdaptation {
    pub platform: Platform,
    /// Platform-specific title (may differ from canonical).
    pub title: Option<String>,
    /// Platform-specific body (adapted from canonical).
    pub body: String,
    /// For threads: each element is one tweet.
    pub thread_parts: Option<Vec<String>>,
    /// Canonical URL to set (for cross-posting SEO).
    pub canonical_url: Option<String>,
    /// Platform-specific metadata.
    pub metadata: serde_json::Value,
}

/// An attached media file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    pub id: Uuid,
    /// Local file path.
    pub path: String,
    /// MIME type.
    pub mime_type: String,
    /// Alt text for accessibility.
    pub alt_text: Option<String>,
}

/// Record of a successful publication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    pub id: Uuid,
    pub content_id: Uuid,
    pub platform: Platform,
    /// The URL where the content was published.
    pub url: String,
    /// Platform-specific post ID.
    pub platform_post_id: String,
    pub published_at: DateTime<Utc>,
}

impl Content {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        content_type: ContentType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            body: body.into(),
            content_type,
            status: ContentStatus::Idea,
            tags: Vec::new(),
            project: None,
            adaptations: Vec::new(),
            media: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the adaptation for a specific platform, if it exists.
    pub fn adaptation_for(&self, platform: Platform) -> Option<&PlatformAdaptation> {
        self.adaptations.iter().find(|a| a.platform == platform)
    }

    /// Check if this content has been adapted for a given platform.
    pub fn is_adapted_for(&self, platform: Platform) -> bool {
        self.adaptations.iter().any(|a| a.platform == platform)
    }
}
