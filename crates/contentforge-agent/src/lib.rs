use anyhow::{Context, Result};
use async_trait::async_trait;
use contentforge_core::{Content, ContentType, Platform, PlatformAdaptation};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

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
// Claude CLI Agent — uses Claude Max subscription via `claude -p`
// ---------------------------------------------------------------------------

/// Configuration for the Claude CLI subprocess.
#[derive(Debug, Clone)]
pub struct ClaudeCliConfig {
    /// Path to the `claude` binary. Defaults to "claude" (found via PATH).
    pub claude_path: String,
    /// Use Max subscription instead of API credits.
    pub use_max_subscription: bool,
    /// Timeout in seconds for each CLI invocation.
    pub timeout_secs: u64,
}

impl Default for ClaudeCliConfig {
    fn default() -> Self {
        Self {
            claude_path: "claude".to_string(),
            use_max_subscription: true,
            timeout_secs: 120,
        }
    }
}

/// Invoke `claude -p` with the given prompt, using Max subscription env vars.
async fn claude_cli(config: &ClaudeCliConfig, prompt: &str) -> Result<String> {
    let mut cmd = Command::new(&config.claude_path);
    cmd.arg("-p")
        .arg(prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if config.use_max_subscription {
        // Force Claude Max subscription — $0 API credits.
        // Source: claude_max Python package (pip install claude_max).
        // Caveat: undocumented env vars, may break in future CLI updates.
        cmd.env_remove("ANTHROPIC_API_KEY");
        cmd.env_remove("CLAUDECODE"); // avoid recursion detection
        cmd.env("CLAUDE_CODE_ENTRYPOINT", "sdk-max");
        cmd.env("CLAUDE_USE_SUBSCRIPTION", "true");
        cmd.env("CLAUDE_BYPASS_BALANCE_CHECK", "true");
    }

    tracing::debug!(
        use_max = config.use_max_subscription,
        timeout = config.timeout_secs,
        "Spawning claude CLI"
    );

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(config.timeout_secs),
        cmd.output(),
    )
    .await
    .context("claude CLI timed out")?
    .context("failed to spawn claude CLI")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If Max subscription fails, suggest falling back to API key
        if stderr.contains("subscription")
            || stderr.contains("unauthorized")
            || stderr.contains("entrypoint")
        {
            anyhow::bail!(
                "Claude Max subscription failed ({}). Set ANTHROPIC_API_KEY to use API credits instead. stderr: {}",
                output.status,
                stderr.trim()
            );
        }
        anyhow::bail!("claude CLI failed ({}): {}", output.status, stderr.trim());
    }

    let response = String::from_utf8(output.stdout)
        .context("claude CLI output is not valid UTF-8")?
        .trim()
        .to_string();

    if response.is_empty() {
        anyhow::bail!("claude CLI returned empty response");
    }

    Ok(response)
}

// ---------------------------------------------------------------------------
// Claude CLI Generator
// ---------------------------------------------------------------------------

/// Generates content drafts using `claude -p` with Max subscription.
pub struct ClaudeGenerator {
    pub config: ClaudeCliConfig,
}

impl ClaudeGenerator {
    pub fn new(config: ClaudeCliConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContentGenerator for ClaudeGenerator {
    async fn generate(&self, request: &GenerateRequest) -> Result<Content> {
        let constraints = request
            .constraints
            .as_deref()
            .unwrap_or("Write in a clear, engaging developer voice. Be concise.");

        let prompt = format!(
            "You are a content writer for a developer who builds in public.\n\
             Generate a {} about: {}\n\
             Constraints: {}\n\
             Output ONLY the content in markdown. No preamble, no explanation.",
            request.content_type, request.prompt, constraints
        );

        let body = claude_cli(&self.config, &prompt).await?;

        // Extract title from first markdown heading, or use the prompt
        let title = body
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l.trim_start_matches("# ").to_string())
            .unwrap_or_else(|| format!("Draft: {}", request.prompt));

        let mut content = Content::new(title, body, request.content_type);
        content.status = contentforge_core::ContentStatus::Drafting;
        Ok(content)
    }
}

// ---------------------------------------------------------------------------
// Claude CLI Adapter
// ---------------------------------------------------------------------------

/// Adapts content for specific platforms using `claude -p`.
pub struct ClaudeAdapter {
    pub config: ClaudeCliConfig,
}

impl ClaudeAdapter {
    pub fn new(config: ClaudeCliConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContentAdapter for ClaudeAdapter {
    async fn adapt(&self, request: &AdaptRequest) -> Result<PlatformAdaptation> {
        let char_limit = request
            .platform
            .char_limit()
            .map(|l| format!("Maximum {} characters.", l))
            .unwrap_or_default();

        let supports_md = if request.platform.supports_markdown() {
            "Markdown is supported."
        } else {
            "Plain text only, no markdown."
        };

        let thread_instruction = if request.platform.supports_threads() {
            "Split into a thread. Separate each post with '---THREAD---'."
        } else {
            ""
        };

        let prompt = format!(
            "Adapt the following content for {}.\n\
             Rules:\n\
             - {} {} {}\n\
             - Match the platform's tone and conventions.\n\
             - Keep the core message but optimize for the platform.\n\
             - Output ONLY the adapted content, no explanation.\n\n\
             Source content:\n{}",
            request.platform, char_limit, supports_md, thread_instruction, request.body
        );

        let adapted = claude_cli(&self.config, &prompt).await?;

        let thread_parts =
            if request.platform.supports_threads() && adapted.contains("---THREAD---") {
                Some(
                    adapted
                        .split("---THREAD---")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            } else {
                None
            };

        Ok(PlatformAdaptation {
            platform: request.platform,
            title: None,
            body: adapted,
            thread_parts,
            canonical_url: None,
            metadata: serde_json::Value::Null,
        })
    }
}

// ---------------------------------------------------------------------------
// Claude CLI Thread Splitter
// ---------------------------------------------------------------------------

/// Splits content into threads using Claude CLI.
pub struct ClaudeThreadSplitter {
    pub config: ClaudeCliConfig,
}

impl ClaudeThreadSplitter {
    pub fn new(config: ClaudeCliConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ThreadSplitter for ClaudeThreadSplitter {
    async fn split(&self, text: &str, max_chars: usize) -> Result<Vec<String>> {
        let prompt = format!(
            "Split the following content into a thread where each part is at most {} characters.\n\
             Separate each part with '---THREAD---'.\n\
             Each part should be self-contained and engaging.\n\
             Number them (1/, 2/, etc.).\n\
             Output ONLY the thread parts, no explanation.\n\n{}",
            max_chars, text
        );

        let response = claude_cli(&self.config, &prompt).await?;

        let parts: Vec<String> = response
            .split("---THREAD---")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            anyhow::bail!("Thread splitter returned no parts");
        }

        Ok(parts)
    }
}

// ---------------------------------------------------------------------------
// Claude CLI Reviewer
// ---------------------------------------------------------------------------

/// Reviews content quality using Claude CLI.
pub struct ClaudeReviewer {
    pub config: ClaudeCliConfig,
}

impl ClaudeReviewer {
    pub fn new(config: ClaudeCliConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ContentReviewer for ClaudeReviewer {
    async fn review(&self, content: &Content) -> Result<ReviewFeedback> {
        let prompt = format!(
            "Review the following content for publishing quality.\n\
             Respond in EXACTLY this JSON format, nothing else:\n\
             {{\"score\": 0.0-1.0, \"suggestions\": [\"...\"], \"ready_to_publish\": true/false}}\n\n\
             Title: {}\nType: {}\n\n{}",
            content.title, content.content_type, content.body
        );

        let response = claude_cli(&self.config, &prompt).await?;

        // Try to parse JSON from the response — Claude may wrap it in markdown code blocks
        let json_str = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let feedback: ReviewFeedback =
            serde_json::from_str(json_str).context("Failed to parse review feedback JSON")?;

        Ok(feedback)
    }
}

// ---------------------------------------------------------------------------
// Factory — create a full pipeline with Claude CLI agents
// ---------------------------------------------------------------------------

/// Create a full agent pipeline backed by Claude CLI with Max subscription.
pub fn claude_pipeline() -> AgentPipeline {
    claude_pipeline_with_config(ClaudeCliConfig::default())
}

/// Create a full agent pipeline with custom config.
pub fn claude_pipeline_with_config(config: ClaudeCliConfig) -> AgentPipeline {
    AgentPipeline::new(
        Box::new(ClaudeGenerator::new(config.clone())),
        Box::new(ClaudeAdapter::new(config.clone())),
        Box::new(ClaudeThreadSplitter::new(config.clone())),
        Box::new(ClaudeReviewer::new(config)),
    )
}

// ---------------------------------------------------------------------------
// Stub implementations (for testing / offline / CI)
// ---------------------------------------------------------------------------

/// A placeholder generator that returns a skeleton draft.
pub struct StubGenerator;

#[async_trait]
impl ContentGenerator for StubGenerator {
    async fn generate(&self, request: &GenerateRequest) -> Result<Content> {
        Ok(Content::new(
            format!("Draft: {}", request.prompt),
            format!(
                "# {}\n\nGenerated draft content for this topic.",
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_pipeline() {
        let pipeline = AgentPipeline::new(
            Box::new(StubGenerator),
            Box::new(StubAdapter),
            Box::new(StubThreadSplitter),
            Box::new(StubReviewer),
        );

        let request = GenerateRequest {
            prompt: "Test content".to_string(),
            content_type: ContentType::Article,
            constraints: None,
        };

        let content = pipeline
            .run(request, &[Platform::DevTo, Platform::Mastodon])
            .await
            .unwrap();

        assert_eq!(content.adaptations.len(), 2);
        assert_eq!(content.adaptations[0].platform, Platform::DevTo);
        assert_eq!(content.adaptations[1].platform, Platform::Mastodon);
    }

    #[tokio::test]
    async fn test_stub_thread_splitter() {
        let splitter = StubThreadSplitter;
        let parts = splitter.split("Hello World!", 5).await.unwrap();
        assert_eq!(parts, vec!["Hello", " Worl", "d!"]);
    }
}
