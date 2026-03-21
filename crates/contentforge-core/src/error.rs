use crate::platform::Platform;

#[derive(Debug, thiserror::Error)]
pub enum ContentForgeError {
    #[error("Content not found: {0}")]
    ContentNotFound(uuid::Uuid),

    #[error("Platform {platform} publish failed: {message}")]
    PublishFailed { platform: Platform, message: String },

    #[error("Platform {0} not configured")]
    PlatformNotConfigured(Platform),

    #[error("Rate limited on {platform}, retry after {retry_after_secs}s")]
    RateLimited {
        platform: Platform,
        retry_after_secs: u64,
    },

    #[error("Authentication failed for {0}")]
    AuthFailed(Platform),

    #[error("Content exceeds {platform} character limit ({limit}): got {actual}")]
    ContentTooLong {
        platform: Platform,
        limit: usize,
        actual: usize,
    },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Schedule error: {0}")]
    Schedule(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ContentForgeError>;
