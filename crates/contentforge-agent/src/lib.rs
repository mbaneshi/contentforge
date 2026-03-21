use anyhow::Result;
use async_trait::async_trait;
use contentforge_core::{Content, ContentType, Platform, PlatformAdaptation};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Content Generator
// ---------------------------------------------------------------------------

/// Request to generate a new content draft from an idea or prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// The idea, topic, or prompt to generate content from.
    pub prompt: String,
    /// The desired content type.
    pub content_type: ContentType,
    /// Optional constraints (tone, length, audience, etc.).
    pub constraints: Option<String>,
}

/// Generates a draft from an idea or prompt using an LLM.
#[async_trait]
pub trait ContentGenerator: Send + Sync {
    async fn generate(&self, request: &GenerateRequest) -> Result<Content>;
}

// ---------------------------------------------------------------------------
// Content Adapter
// ---------------------------------------------------------------------------

/// Request to adapt content for a specific platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptRequest {
    /// The source content to adapt.
    pub content_id: uuid::Uuid,
    /// The source body (markdown).
    pub body: String,
    /// Target platform.
    pub platform: Platform,
}

/// Adapts canonical content for a specific platform's requirements.
#[async_trait]
pub trait ContentAdapter: Send + Sync {
    async fn adapt(&self, request: &AdaptRequest) -> Result<PlatformAdaptation>;
}

// ---------------------------------------------------------------------------
// Thread Splitter
// ---------------------------------------------------------------------------

/// Splits long-form content into a tweet thread.
#[async_trait]
pub trait ThreadSplitter: Send + Sync {
    /// Split the given text into tweet-sized parts.
    async fn split(&self, text: &str, max_chars: usize) -> Result<Vec<String>>;
}

// ---------------------------------------------------------------------------
// Content Reviewer
// ---------------------------------------------------------------------------

/// Feedback from the AI content reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFeedback {
    /// Overall quality score (0.0 to 1.0).
    pub score: f64,
    /// Suggestions for improvement.
    pub suggestions: Vec<String>,
    /// Whether the content is ready to publish as-is.
    pub ready_to_publish: bool,
}

/// Reviews content and provides improvement suggestions.
#[async_trait]
pub trait ContentReviewer: Send + Sync {
    async fn review(&self, content: &Content) -> Result<ReviewFeedback>;
}

// ---------------------------------------------------------------------------
// Agent Pipeline
// ---------------------------------------------------------------------------

/// The full AI agent pipeline combining all capabilities.
pub struct AgentPipeline {
    pub generator: Box<dyn ContentGenerator>,
    pub adapter: Box<dyn ContentAdapter>,
    pub splitter: Box<dyn ThreadSplitter>,
    pub reviewer: Box<dyn ContentReviewer>,
}

impl AgentPipeline {
    pub fn new(
        generator: Box<dyn ContentGenerator>,
        adapter: Box<dyn ContentAdapter>,
        splitter: Box<dyn ThreadSplitter>,
        reviewer: Box<dyn ContentReviewer>,
    ) -> Self {
        Self {
            generator,
            adapter,
            splitter,
            reviewer,
        }
    }

    /// Generate content, review it, and adapt for the given platforms.
    pub async fn run(&self, request: GenerateRequest, platforms: &[Platform]) -> Result<Content> {
        tracing::info!(prompt = %request.prompt, "Starting agent pipeline");

        // Step 1: Generate draft
        let mut content = self.generator.generate(&request).await?;
        tracing::info!(id = %content.id, "Draft generated");

        // Step 2: Review
        let feedback = self.reviewer.review(&content).await?;
        tracing::info!(score = feedback.score, "Review complete");

        if !feedback.ready_to_publish {
            tracing::warn!(
                suggestions = ?feedback.suggestions,
                "Content needs improvement before publishing"
            );
        }

        // Step 3: Adapt for each platform
        for &platform in platforms {
            let adapt_req = AdaptRequest {
                content_id: content.id,
                body: content.body.clone(),
                platform,
            };
            let adaptation = self.adapter.adapt(&adapt_req).await?;
            content.adaptations.push(adaptation);
            tracing::info!(%platform, "Adaptation created");
        }

        Ok(content)
    }
}

// ---------------------------------------------------------------------------
// Stub implementations (for testing / initial wiring)
// ---------------------------------------------------------------------------

/// A placeholder generator that returns a skeleton draft.
pub struct StubGenerator;

#[async_trait]
impl ContentGenerator for StubGenerator {
    async fn generate(&self, request: &GenerateRequest) -> Result<Content> {
        Ok(Content::new(
            format!("Draft: {}", request.prompt),
            format!(
                "# {}\n\nTODO: flesh out content for this topic.",
                request.prompt
            ),
            request.content_type,
        ))
    }
}

/// A placeholder adapter that copies the body verbatim.
pub struct StubAdapter;

#[async_trait]
impl ContentAdapter for StubAdapter {
    async fn adapt(&self, request: &AdaptRequest) -> Result<PlatformAdaptation> {
        Ok(PlatformAdaptation {
            platform: request.platform,
            title: None,
            body: request.body.clone(),
            thread_parts: None,
            canonical_url: None,
            metadata: serde_json::Value::Null,
        })
    }
}

/// A placeholder thread splitter that chunks by character count.
pub struct StubThreadSplitter;

#[async_trait]
impl ThreadSplitter for StubThreadSplitter {
    async fn split(&self, text: &str, max_chars: usize) -> Result<Vec<String>> {
        let parts: Vec<String> = text
            .chars()
            .collect::<Vec<_>>()
            .chunks(max_chars)
            .map(|chunk| chunk.iter().collect())
            .collect();
        Ok(parts)
    }
}

/// A placeholder reviewer that always approves.
pub struct StubReviewer;

#[async_trait]
impl ContentReviewer for StubReviewer {
    async fn review(&self, _content: &Content) -> Result<ReviewFeedback> {
        Ok(ReviewFeedback {
            score: 0.8,
            suggestions: vec!["Consider adding more examples.".into()],
            ready_to_publish: true,
        })
    }
}
