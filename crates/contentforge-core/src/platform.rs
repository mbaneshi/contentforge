use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported publishing platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Twitter,
    LinkedIn,
    DevTo,
    Medium,
    YouTube,
    Instagram,
    Substack,
    HackerNews,
    Reddit,
    Mastodon,
    Bluesky,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Twitter => write!(f, "Twitter/X"),
            Self::LinkedIn => write!(f, "LinkedIn"),
            Self::DevTo => write!(f, "DEV.to"),
            Self::Medium => write!(f, "Medium"),
            Self::YouTube => write!(f, "YouTube"),
            Self::Instagram => write!(f, "Instagram"),
            Self::Substack => write!(f, "Substack"),
            Self::HackerNews => write!(f, "Hacker News"),
            Self::Reddit => write!(f, "Reddit"),
            Self::Mastodon => write!(f, "Mastodon"),
            Self::Bluesky => write!(f, "Bluesky"),
        }
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().replace(['-', '.', ' '], "_").as_str() {
            "twitter" | "x" => Ok(Self::Twitter),
            "linkedin" => Ok(Self::LinkedIn),
            "devto" | "dev_to" | "dev" => Ok(Self::DevTo),
            "medium" => Ok(Self::Medium),
            "youtube" | "yt" => Ok(Self::YouTube),
            "instagram" | "insta" | "ig" => Ok(Self::Instagram),
            "substack" => Ok(Self::Substack),
            "hackernews" | "hacker_news" | "hn" => Ok(Self::HackerNews),
            "reddit" => Ok(Self::Reddit),
            "mastodon" | "masto" | "fediverse" => Ok(Self::Mastodon),
            "bluesky" | "bsky" => Ok(Self::Bluesky),
            other => Err(format!("Unknown platform: '{other}'. Valid: twitter, linkedin, devto, mastodon, bluesky, medium, youtube, instagram, substack, hackernews, reddit")),
        }
    }
}

impl Platform {
    /// Character limit for the platform's primary content field.
    pub fn char_limit(&self) -> Option<usize> {
        match self {
            Self::Twitter => Some(280),
            Self::LinkedIn => Some(3000),
            Self::DevTo => None,
            Self::Medium => None,
            Self::YouTube => Some(5000),
            Self::Instagram => Some(2200),
            Self::Substack => None,
            Self::HackerNews => Some(2000),
            Self::Reddit => Some(40000),
            Self::Mastodon => Some(500),
            Self::Bluesky => Some(300),
        }
    }

    /// Whether the platform supports markdown natively.
    pub fn supports_markdown(&self) -> bool {
        matches!(
            self,
            Self::DevTo | Self::Medium | Self::Substack | Self::Reddit
        )
    }

    /// Whether the platform supports image attachments.
    pub fn supports_images(&self) -> bool {
        matches!(
            self,
            Self::Twitter
                | Self::LinkedIn
                | Self::DevTo
                | Self::Medium
                | Self::Instagram
                | Self::Substack
                | Self::Mastodon
                | Self::Bluesky
        )
    }

    /// Whether the platform supports thread/multi-part content.
    pub fn supports_threads(&self) -> bool {
        matches!(self, Self::Twitter | Self::Bluesky)
    }

    /// API difficulty rating for prioritization.
    pub fn integration_difficulty(&self) -> &'static str {
        match self {
            Self::DevTo => "easy",
            Self::Mastodon => "easy",
            Self::Bluesky => "easy",
            Self::Medium => "deprecated",
            Self::Twitter => "medium",
            Self::LinkedIn => "medium",
            Self::YouTube => "medium",
            Self::Reddit => "medium",
            Self::HackerNews => "medium",
            Self::Instagram => "hard",
            Self::Substack => "no-api",
        }
    }
}

/// Credentials for authenticating with a platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlatformCredential {
    ApiKey {
        key: String,
    },
    OAuth2 {
        client_id: String,
        client_secret: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    IntegrationToken {
        token: String,
    },
    Cookie {
        value: String,
    },
    /// Mastodon: instance URL + access token.
    MastodonAuth {
        instance_url: String,
        access_token: String,
    },
    /// Bluesky: handle + app password (used to create session).
    BlueskyAuth {
        handle: String,
        app_password: String,
    },
}

/// Configuration for a connected platform account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAccount {
    pub id: uuid::Uuid,
    pub platform: Platform,
    pub display_name: String,
    pub credential: PlatformCredential,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
