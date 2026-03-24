use anyhow::{bail, Context, Result};
use clap::Subcommand;
use contentforge_core::{
    Content, ContentStatus, ContentType, Platform, PlatformAccount, PlatformAdaptation,
    PlatformCredential,
};
use contentforge_db::repo::{AdaptationRepo, ContentRepo, PlatformAccountRepo, PublicationRepo};
use contentforge_db::DbPool;
use contentforge_publish::adapters::bluesky::BlueskyPublisher;
use contentforge_publish::adapters::devto::DevToPublisher;
use contentforge_publish::adapters::linkedin::LinkedInPublisher;
use contentforge_publish::adapters::mastodon::MastodonPublisher;
use contentforge_publish::adapters::medium::MediumPublisher;
use contentforge_publish::adapters::twitter::TwitterPublisher;
use contentforge_publish::Publisher;
use std::str::FromStr;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// CLI Subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    /// Create, list, or show drafts.
    Draft {
        #[command(subcommand)]
        action: DraftAction,
    },
    /// Adapt content for a specific platform.
    Adapt {
        /// Content UUID (or short prefix).
        id: String,
        /// Target platform (devto, twitter, linkedin, medium, etc.).
        #[arg(short, long)]
        platform: String,
        /// Optional custom title for this platform.
        #[arg(long)]
        title: Option<String>,
        /// Optional canonical URL for cross-posting SEO.
        #[arg(long)]
        canonical_url: Option<String>,
    },
    /// Publish content to a platform.
    Publish {
        /// Content UUID (or short prefix).
        id: String,
        /// Target platform.
        #[arg(short, long)]
        platform: String,
    },
    /// Manage the publishing schedule.
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// List, add, or check platform integrations.
    Platforms {
        #[command(subcommand)]
        action: PlatformAction,
    },
    /// Run, list, and manage automated pipelines (Pro feature).
    Pipeline {
        #[command(subcommand)]
        action: PipelineAction,
    },
    /// Manage your ContentForge license.
    License {
        #[command(subcommand)]
        action: LicenseAction,
    },
    /// Show the current content pipeline overview.
    Status,
}

// ---------------------------------------------------------------------------
// License subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
pub enum LicenseAction {
    /// Activate a license key.
    Activate {
        /// Your license key.
        key: String,
    },
    /// Show current license status.
    Status,
    /// Deactivate the current license.
    Deactivate,
}

// ---------------------------------------------------------------------------
// Pipeline subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
pub enum PipelineAction {
    /// Run a pipeline on a content item.
    Run {
        /// Content UUID (or short prefix).
        content_id: String,
        /// Pipeline name (publish-all, adapt-review-publish).
        #[arg(short = 'P', long, default_value = "publish-all")]
        pipeline: String,
        /// Target platforms (comma-separated). Default: all configured.
        #[arg(short, long)]
        platforms: Option<String>,
        /// Skip the review/approval step.
        #[arg(long)]
        skip_review: bool,
    },
    /// List pipeline jobs.
    List {
        /// Filter by status.
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show details of a specific job.
    Show {
        /// Job UUID (or short prefix).
        id: String,
    },
    /// Approve a job awaiting review.
    Approve {
        /// Job UUID (or short prefix).
        id: String,
    },
    /// Reject a job awaiting review.
    Reject {
        /// Job UUID (or short prefix).
        id: String,
        /// Reason for rejection.
        #[arg(short, long)]
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Draft subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
pub enum DraftAction {
    /// Create a new draft.
    Create {
        /// Title of the draft.
        title: String,
        /// Markdown body content.
        #[arg(short, long)]
        body: Option<String>,
        /// Read body from a file.
        #[arg(short = 'f', long)]
        file: Option<String>,
        /// Comma-separated tags.
        #[arg(short, long)]
        tags: Option<String>,
        /// Content type (article, thread, short_post, video, image_post, link_share).
        #[arg(short = 'T', long, default_value = "article")]
        content_type: String,
        /// Associated project name.
        #[arg(long)]
        project: Option<String>,
    },
    /// List existing drafts (all statuses).
    List {
        /// Filter by status (idea, drafting, review, ready, scheduled, published).
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show a draft's full details.
    Show {
        /// Content UUID (or short prefix).
        id: String,
    },
    /// Delete a draft.
    Delete {
        /// Content UUID (or short prefix).
        id: String,
    },
}

// ---------------------------------------------------------------------------
// Schedule subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
pub enum ScheduleAction {
    /// List pending schedule entries.
    List,
    /// Add a one-off schedule entry.
    Add {
        /// Content UUID.
        #[arg(short, long)]
        content_id: String,
        /// Target platform.
        #[arg(short, long)]
        platform: String,
        /// Scheduled time (RFC 3339 format).
        #[arg(short, long)]
        at: String,
    },
    /// Cancel a schedule entry.
    Cancel {
        /// Schedule entry UUID.
        id: String,
    },
    /// Add a recurring cron schedule (Pro feature).
    Cron {
        /// Schedule name (e.g., "weekly-ship").
        name: String,
        /// Cron expression (e.g., "0 0 8 * * FRI" for Friday 8 AM).
        #[arg(short, long)]
        expr: String,
        /// Pipeline template to run (default: publish-all).
        #[arg(short, long, default_value = "publish-all")]
        pipeline: String,
        /// Target platforms (comma-separated).
        #[arg(short = 'P', long, default_value = "devto")]
        platforms: String,
    },
    /// List recurring cron schedules.
    CronList,
    /// Remove a recurring cron schedule.
    CronRemove {
        /// Schedule name.
        name: String,
    },
}

// ---------------------------------------------------------------------------
// Platform subcommands
// ---------------------------------------------------------------------------

#[derive(Debug, Subcommand)]
pub enum PlatformAction {
    /// List configured platforms.
    List,
    /// Add a new platform credential.
    Add {
        /// Platform name (devto, twitter, linkedin, medium).
        platform: String,
        /// API key or token value.
        #[arg(short, long)]
        key: String,
        /// Display name for this account.
        #[arg(short, long, default_value = "default")]
        name: String,
    },
    /// Remove a platform credential.
    Remove {
        /// Platform name.
        platform: String,
    },
    /// Check health/auth for all configured platforms.
    Check,
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

pub async fn execute(cmd: CliCommand, db: DbPool) -> Result<()> {
    match cmd {
        CliCommand::Draft { action } => handle_draft(action, &db).await,
        CliCommand::Adapt {
            id,
            platform,
            title,
            canonical_url,
        } => handle_adapt(&id, &platform, title, canonical_url, &db).await,
        CliCommand::Publish { id, platform } => handle_publish(&id, &platform, &db).await,
        CliCommand::Schedule { action } => handle_schedule(action, &db).await,
        CliCommand::Platforms { action } => handle_platforms(action, &db).await,
        CliCommand::Pipeline { action } => handle_pipeline(action, &db).await,
        CliCommand::License { action } => handle_license(action, &db).await,
        CliCommand::Status => handle_status(&db).await,
    }
}

// ---------------------------------------------------------------------------
// Draft handlers
// ---------------------------------------------------------------------------

async fn handle_draft(action: DraftAction, db: &DbPool) -> Result<()> {
    let repo = ContentRepo::new(db.clone());

    match action {
        DraftAction::Create {
            title,
            body,
            file,
            tags,
            content_type,
            project,
        } => {
            let ct: ContentType = content_type
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;

            let body_text = if let Some(path) = file {
                std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read file: {path}"))?
            } else {
                body.unwrap_or_default()
            };

            let mut content = Content::new(&title, body_text, ct);
            content.status = ContentStatus::Drafting;
            content.project = project;

            if let Some(tags_str) = tags {
                content.tags = tags_str.split(',').map(|t| t.trim().to_string()).collect();
            }

            repo.insert(&content)?;

            let short_id = &content.id.to_string()[..8];
            println!("Created draft: {short_id}");
            println!("  Title: {}", content.title);
            println!("  Type:  {ct}");
            if !content.tags.is_empty() {
                println!("  Tags:  {}", content.tags.join(", "));
            }
            println!("\n  Full ID: {}", content.id);
        }

        DraftAction::List { status } => {
            let contents = if let Some(status_str) = status {
                let s: ContentStatus = serde_json::from_str(&format!("\"{status_str}\""))
                    .map_err(|_| anyhow::anyhow!("Invalid status: {status_str}"))?;
                repo.list_by_status(s)?
            } else {
                repo.list_all()?
            };

            if contents.is_empty() {
                println!("No content found.");
                return Ok(());
            }

            println!(
                "{:<10} {:<12} {:<10} {:<40} TAGS",
                "ID", "STATUS", "TYPE", "TITLE"
            );
            println!("{}", "-".repeat(90));

            for c in &contents {
                let short_id = &c.id.to_string()[..8];
                let tags = if c.tags.is_empty() {
                    "-".to_string()
                } else {
                    c.tags.join(", ")
                };
                let title = if c.title.len() > 38 {
                    format!("{}...", &c.title[..35])
                } else {
                    c.title.clone()
                };
                println!(
                    "{:<10} {:<12} {:<10} {:<40} {}",
                    short_id, c.status, c.content_type, title, tags
                );
            }
            println!("\n{} item(s)", contents.len());
        }

        DraftAction::Show { id } => {
            let uuid = resolve_uuid(&id, db)?;
            let content = repo
                .get_by_id_full(uuid)?
                .ok_or_else(|| anyhow::anyhow!("Content not found: {id}"))?;

            let pub_repo = PublicationRepo::new(db.clone());
            let publications = pub_repo.list_for_content(uuid)?;

            println!("=== {} ===", content.title);
            println!("ID:      {}", content.id);
            println!("Status:  {}", content.status);
            println!("Type:    {}", content.content_type);
            if !content.tags.is_empty() {
                println!("Tags:    {}", content.tags.join(", "));
            }
            if let Some(ref p) = content.project {
                println!("Project: {p}");
            }
            println!("Created: {}", content.created_at.format("%Y-%m-%d %H:%M"));
            println!("Updated: {}", content.updated_at.format("%Y-%m-%d %H:%M"));

            println!("\n--- Body ---");
            println!("{}", content.body);

            if !content.adaptations.is_empty() {
                println!("\n--- Adaptations ---");
                for a in &content.adaptations {
                    let chars = a.body.len();
                    let limit_info = a
                        .platform
                        .char_limit()
                        .map(|l| format!(" ({chars}/{l} chars)"))
                        .unwrap_or_default();
                    println!("  {} — {} chars{limit_info}", a.platform, chars);
                }
            }

            if !publications.is_empty() {
                println!("\n--- Publications ---");
                for p in &publications {
                    println!(
                        "  {} — {} ({})",
                        p.platform,
                        p.url,
                        p.published_at.format("%Y-%m-%d %H:%M")
                    );
                }
            }
        }

        DraftAction::Delete { id } => {
            let uuid = resolve_uuid(&id, db)?;
            repo.delete(uuid)?;
            println!("Deleted: {}", &id[..8.min(id.len())]);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Adapt handler
// ---------------------------------------------------------------------------

async fn handle_adapt(
    id: &str,
    platform_str: &str,
    custom_title: Option<String>,
    canonical_url: Option<String>,
    db: &DbPool,
) -> Result<()> {
    let uuid = resolve_uuid(id, db)?;
    let platform: Platform = platform_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let content_repo = ContentRepo::new(db.clone());
    let content = content_repo
        .get_by_id(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Content not found: {id}"))?;

    // For now, adapt by copying the body (markdown-native platforms like DEV.to).
    // For Twitter, split into thread parts.
    let (body, thread_parts) = if platform == Platform::Twitter {
        let parts = split_into_thread(&content.body);
        (parts.first().cloned().unwrap_or_default(), Some(parts))
    } else {
        (content.body.clone(), None)
    };

    let adaptation = PlatformAdaptation {
        platform,
        title: custom_title.or(Some(content.title.clone())),
        body,
        thread_parts,
        canonical_url,
        metadata: serde_json::json!({}),
    };

    // Validate character limits.
    if let Some(limit) = platform.char_limit() {
        if adaptation.body.len() > limit && adaptation.thread_parts.is_none() {
            println!(
                "Warning: Body is {} chars, platform limit is {limit}. Consider editing.",
                adaptation.body.len()
            );
        }
    }

    let adapt_repo = AdaptationRepo::new(db.clone());
    adapt_repo.upsert(uuid, &adaptation)?;

    // Update status to Ready if it was just Drafting.
    if content.status == ContentStatus::Drafting || content.status == ContentStatus::Idea {
        content_repo.update_status(uuid, ContentStatus::Ready)?;
    }

    let short_id = &uuid.to_string()[..8];
    println!("Adapted {short_id} for {platform}");
    if let Some(ref parts) = adaptation.thread_parts {
        println!("  Thread: {} tweets", parts.len());
    } else {
        println!("  Body: {} chars", adaptation.body.len());
    }

    Ok(())
}

/// Simple thread splitter: split on double newlines, then merge to stay under 280 chars.
fn split_into_thread(body: &str) -> Vec<String> {
    let paragraphs: Vec<&str> = body.split("\n\n").collect();
    let mut tweets = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }
        if current.len() + trimmed.len() + 1 > 270 {
            if !current.is_empty() {
                tweets.push(current.trim().to_string());
                current = String::new();
            }
            // If single paragraph exceeds 270, chunk it.
            if trimmed.len() > 270 {
                for chunk in trimmed
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(265)
                    .map(|c| c.iter().collect::<String>())
                {
                    tweets.push(chunk);
                }
                continue;
            }
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(trimmed);
    }
    if !current.is_empty() {
        tweets.push(current.trim().to_string());
    }

    // Add thread numbering if more than 1.
    let total = tweets.len();
    if total > 1 {
        tweets = tweets
            .into_iter()
            .enumerate()
            .map(|(i, t)| format!("{t}\n\n{}/{total}", i + 1))
            .collect();
    }

    tweets
}

// ---------------------------------------------------------------------------
// Publish handler
// ---------------------------------------------------------------------------

async fn handle_publish(id: &str, platform_str: &str, db: &DbPool) -> Result<()> {
    let uuid = resolve_uuid(id, db)?;
    let platform: Platform = platform_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    let content_repo = ContentRepo::new(db.clone());
    let content = content_repo
        .get_by_id_full(uuid)?
        .ok_or_else(|| anyhow::anyhow!("Content not found: {id}"))?;

    let adaptation = content.adaptation_for(platform).ok_or_else(|| {
        anyhow::anyhow!(
            "No adaptation for {platform}. Run: contentforge adapt {id} --platform {platform_str}"
        )
    })?;

    // Get credentials — try DB first, then env var fallback.
    let account_repo = PlatformAccountRepo::new(db.clone());
    let publisher = build_publisher(platform, &account_repo)?;

    println!("Publishing to {platform}...");

    let publication = publisher
        .publish(&content, adaptation)
        .await
        .map_err(|e| anyhow::anyhow!("Publish failed: {e}"))?;

    // Save publication record.
    let pub_repo = PublicationRepo::new(db.clone());
    pub_repo.insert(&publication)?;

    // Update status.
    content_repo.update_status(uuid, ContentStatus::Published)?;

    let short_id = &uuid.to_string()[..8];
    println!("Published {short_id} to {platform}");
    println!("  URL: {}", publication.url);

    Ok(())
}

fn build_publisher(
    platform: Platform,
    account_repo: &PlatformAccountRepo,
) -> Result<Box<dyn Publisher>> {
    // Try DB credentials first.
    if let Some(account) = account_repo.get_by_platform(platform)? {
        return match (platform, account.credential) {
            (Platform::DevTo, PlatformCredential::ApiKey { key }) => {
                Ok(Box::new(DevToPublisher::new(key)))
            }
            (Platform::Twitter, PlatformCredential::OAuth2 { access_token, .. }) => {
                let token = access_token
                    .ok_or_else(|| anyhow::anyhow!("Twitter OAuth2 access_token is missing"))?;
                Ok(Box::new(TwitterPublisher::new(token)))
            }
            (Platform::LinkedIn, PlatformCredential::OAuth2 { access_token, .. }) => {
                let token = access_token
                    .ok_or_else(|| anyhow::anyhow!("LinkedIn OAuth2 access_token is missing"))?;
                // TODO: author_urn should come from account metadata.
                Ok(Box::new(LinkedInPublisher::new(token, String::new())))
            }
            (Platform::Medium, PlatformCredential::IntegrationToken { token }) => {
                Ok(Box::new(MediumPublisher::new(token)))
            }
            (
                Platform::Mastodon,
                PlatformCredential::MastodonAuth {
                    instance_url,
                    access_token,
                },
            ) => Ok(Box::new(MastodonPublisher::new(instance_url, access_token))),
            (
                Platform::Bluesky,
                PlatformCredential::BlueskyAuth {
                    handle,
                    app_password,
                },
            ) => Ok(Box::new(BlueskyPublisher::new(handle, app_password))),
            _ => bail!("Unsupported credential type for {platform}"),
        };
    }

    // Fallback: environment variables.
    match platform {
        Platform::DevTo => {
            let key = std::env::var("DEVTO_API_KEY")
                .context("DEV.to: Set DEVTO_API_KEY env var or run: contentforge platforms add devto --key <key>")?;
            Ok(Box::new(DevToPublisher::new(key)))
        }
        Platform::Twitter => {
            let token = std::env::var("TWITTER_BEARER_TOKEN")
                .context("Twitter: Set TWITTER_BEARER_TOKEN env var or run: contentforge platforms add twitter --key <token>")?;
            Ok(Box::new(TwitterPublisher::new(token)))
        }
        Platform::Medium => {
            let token = std::env::var("MEDIUM_TOKEN")
                .context("Medium: Set MEDIUM_TOKEN env var or run: contentforge platforms add medium --key <token>")?;
            Ok(Box::new(MediumPublisher::new(token)))
        }
        Platform::Mastodon => {
            let instance = std::env::var("MASTODON_INSTANCE")
                .context("Mastodon: Set MASTODON_INSTANCE and MASTODON_TOKEN env vars")?;
            let token = std::env::var("MASTODON_TOKEN")
                .context("Mastodon: Set MASTODON_TOKEN env var")?;
            Ok(Box::new(MastodonPublisher::new(instance, token)))
        }
        Platform::Bluesky => {
            let handle = std::env::var("BLUESKY_HANDLE")
                .context("Bluesky: Set BLUESKY_HANDLE and BLUESKY_APP_PASSWORD env vars")?;
            let password = std::env::var("BLUESKY_APP_PASSWORD")
                .context("Bluesky: Set BLUESKY_APP_PASSWORD env var")?;
            Ok(Box::new(BlueskyPublisher::new(handle, password)))
        }
        _ => bail!(
            "Platform {platform} is not configured. Run: contentforge platforms add {platform_str} --key <key>",
            platform_str = platform.to_string().to_lowercase()
        ),
    }
}

// ---------------------------------------------------------------------------
// Schedule handlers
// ---------------------------------------------------------------------------

async fn handle_schedule(action: ScheduleAction, db: &DbPool) -> Result<()> {
    match action {
        ScheduleAction::List => {
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut stmt = conn.prepare(
                "SELECT s.id, s.content_id, s.platform, s.scheduled_at, s.status, s.retries, c.title
                 FROM schedule s LEFT JOIN content c ON s.content_id = c.id
                 ORDER BY s.scheduled_at ASC LIMIT 20",
            )?;
            #[allow(clippy::type_complexity)]
            let rows: Vec<(
                String,
                String,
                String,
                String,
                String,
                i32,
                Option<String>,
            )> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i32>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            if rows.is_empty() {
                println!("No scheduled entries.");
                return Ok(());
            }

            println!(
                "{:<10} {:<30} {:<12} {:<20} STATUS",
                "ID", "TITLE", "PLATFORM", "SCHEDULED AT"
            );
            println!("{}", "-".repeat(85));
            for (id, _cid, platform, at, status, _retries, title) in &rows {
                let short_id = &id[..8.min(id.len())];
                let platform_clean = platform.trim_matches('"');
                let title_str = title.as_deref().unwrap_or("(unknown)");
                let title_short = if title_str.len() > 28 {
                    format!("{}...", &title_str[..25])
                } else {
                    title_str.to_string()
                };
                println!(
                    "{:<10} {:<30} {:<12} {:<20} {}",
                    short_id, title_short, platform_clean, at, status
                );
            }
        }

        ScheduleAction::Add {
            content_id,
            platform,
            at,
        } => {
            let uuid = resolve_uuid(&content_id, db)?;
            let p: Platform = platform.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            let scheduled_at = chrono::DateTime::parse_from_rfc3339(&at).map_err(|e| {
                anyhow::anyhow!("Invalid date: {e}. Use RFC 3339: 2026-03-25T08:00:00Z")
            })?;

            let entry_id = uuid::Uuid::new_v4();
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            conn.execute(
                "INSERT INTO schedule (id, content_id, platform, scheduled_at, status, retries, created_at)
                 VALUES (?1, ?2, ?3, ?4, 'pending', 0, datetime('now'))",
                rusqlite::params![
                    entry_id.to_string(),
                    uuid.to_string(),
                    serde_json::to_string(&p)?,
                    scheduled_at.to_rfc3339(),
                ],
            )?;

            let short_id = &entry_id.to_string()[..8];
            println!("Scheduled: {short_id}");
            println!("  Content:  {}", &content_id[..8.min(content_id.len())]);
            println!("  Platform: {p}");
            println!("  Time:     {}", scheduled_at.format("%Y-%m-%d %H:%M UTC"));
        }

        ScheduleAction::Cancel { id } => {
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            conn.execute(
                "UPDATE schedule SET status = 'cancelled' WHERE id LIKE ?1",
                [format!("{id}%")],
            )?;
            println!("Cancelled: {}", &id[..8.min(id.len())]);
        }

        ScheduleAction::Cron {
            name,
            expr,
            pipeline,
            platforms,
        } => {
            // Pro feature gate
            let license = load_license(db);
            if let Err(msg) = license.require_pro("Cron scheduling") {
                println!("{msg}");
                return Ok(());
            }

            // Validate cron expression
            if cron::Schedule::from_str(&expr).is_err() {
                bail!("Invalid cron expression: '{expr}'. Example: '0 0 8 * * FRI' (Friday 8 AM)");
            }

            let platform_list: Vec<String> =
                platforms.split(',').map(|s| s.trim().to_string()).collect();

            let id = uuid::Uuid::new_v4();
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            conn.execute(
                "INSERT INTO recurring_schedules (id, name, cron_expr, template, platforms, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, datetime('now'))",
                rusqlite::params![
                    id.to_string(),
                    name,
                    expr,
                    pipeline,
                    serde_json::to_string(&platform_list)?,
                ],
            )?;

            println!("Cron schedule created: {name}");
            println!("  Cron:      {expr}");
            println!("  Pipeline:  {pipeline}");
            println!("  Platforms: {}", platform_list.join(", "));
        }

        ScheduleAction::CronList => {
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut stmt = conn.prepare(
                "SELECT name, cron_expr, template, platforms, enabled FROM recurring_schedules ORDER BY name",
            )?;
            let rows: Vec<(String, String, Option<String>, String, bool)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i32>(4)? != 0,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            if rows.is_empty() {
                println!("No recurring schedules.");
                println!("\nCreate one (Pro feature):");
                println!("  contentforge schedule cron weekly-ship --expr '0 0 8 * * FRI' --platforms devto,mastodon");
                return Ok(());
            }

            println!(
                "{:<20} {:<25} {:<15} {:<8} PLATFORMS",
                "NAME", "CRON", "PIPELINE", "ACTIVE"
            );
            println!("{}", "-".repeat(80));
            for (name, cron_expr, template, platforms, enabled) in &rows {
                let pipeline = template.as_deref().unwrap_or("publish-all");
                let status = if *enabled { "yes" } else { "no" };
                println!(
                    "{:<20} {:<25} {:<15} {:<8} {}",
                    name, cron_expr, pipeline, status, platforms
                );
            }
        }

        ScheduleAction::CronRemove { name } => {
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            let deleted =
                conn.execute("DELETE FROM recurring_schedules WHERE name = ?1", [&name])?;
            if deleted > 0 {
                println!("Removed cron schedule: {name}");
            } else {
                println!("No schedule found with name: {name}");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Platforms handler
// ---------------------------------------------------------------------------

async fn handle_platforms(action: PlatformAction, db: &DbPool) -> Result<()> {
    let repo = PlatformAccountRepo::new(db.clone());

    match action {
        PlatformAction::List => {
            let accounts = repo.list_all()?;
            if accounts.is_empty() {
                println!("No platforms configured.");
                println!("\nAdd one:");
                println!("  contentforge platforms add devto --key <API_KEY>");
                println!("  contentforge platforms add twitter --key <BEARER_TOKEN>");
                println!("  contentforge platforms add medium --key <INTEGRATION_TOKEN>");
                return Ok(());
            }

            println!(
                "{:<12} {:<20} {:<10} CREDENTIAL",
                "PLATFORM", "NAME", "STATUS"
            );
            println!("{}", "-".repeat(60));

            for account in &accounts {
                let status = if account.enabled {
                    "active"
                } else {
                    "disabled"
                };
                let cred_type = match &account.credential {
                    PlatformCredential::ApiKey { .. } => "api_key",
                    PlatformCredential::OAuth2 { .. } => "oauth2",
                    PlatformCredential::IntegrationToken { .. } => "token",
                    PlatformCredential::Cookie { .. } => "cookie",
                    PlatformCredential::MastodonAuth { .. } => "mastodon",
                    PlatformCredential::BlueskyAuth { .. } => "bluesky",
                };
                println!(
                    "{:<12} {:<20} {:<10} {}",
                    account.platform, account.display_name, status, cred_type
                );
            }
        }

        PlatformAction::Add {
            platform,
            key,
            name,
        } => {
            let p: Platform = platform.parse().map_err(|e: String| anyhow::anyhow!(e))?;

            let credential = match p {
                Platform::DevTo => PlatformCredential::ApiKey { key },
                Platform::Twitter
                | Platform::LinkedIn
                | Platform::YouTube
                | Platform::Instagram => PlatformCredential::OAuth2 {
                    client_id: String::new(),
                    client_secret: String::new(),
                    access_token: Some(key),
                    refresh_token: None,
                    expires_at: None,
                },
                Platform::Medium => PlatformCredential::IntegrationToken { token: key },
                Platform::Substack => PlatformCredential::Cookie { value: key },
                Platform::Mastodon => {
                    // key format: "instance_url|access_token"
                    let parts: Vec<&str> = key.splitn(2, '|').collect();
                    if parts.len() != 2 {
                        bail!("Mastodon: use --key 'https://mastodon.social|your_access_token'");
                    }
                    PlatformCredential::MastodonAuth {
                        instance_url: parts[0].to_string(),
                        access_token: parts[1].to_string(),
                    }
                }
                Platform::Bluesky => {
                    // key format: "handle|app_password"
                    let parts: Vec<&str> = key.splitn(2, '|').collect();
                    if parts.len() != 2 {
                        bail!("Bluesky: use --key 'yourhandle.bsky.social|your_app_password'");
                    }
                    PlatformCredential::BlueskyAuth {
                        handle: parts[0].to_string(),
                        app_password: parts[1].to_string(),
                    }
                }
                _ => PlatformCredential::ApiKey { key },
            };

            let account = PlatformAccount {
                id: uuid::Uuid::new_v4(),
                platform: p,
                display_name: name,
                credential,
                enabled: true,
                created_at: chrono::Utc::now(),
            };

            repo.insert(&account)?;
            println!("Added platform: {p} ({})", account.display_name);
        }

        PlatformAction::Remove { platform } => {
            let p: Platform = platform.parse().map_err(|e: String| anyhow::anyhow!(e))?;
            repo.delete(p)?;
            println!("Removed platform: {p}");
        }

        PlatformAction::Check => {
            let accounts = repo.list_all()?;
            if accounts.is_empty() {
                println!("No platforms configured.");
                return Ok(());
            }

            for account in &accounts {
                let publisher = build_publisher(account.platform, &repo);
                match publisher {
                    Ok(p) => {
                        print!("Checking {}... ", account.platform);
                        match p.health_check().await {
                            Ok(()) => println!("OK"),
                            Err(e) => println!("FAILED: {e}"),
                        }
                    }
                    Err(e) => println!("{}: FAILED to build publisher: {e}", account.platform),
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Status handler
// ---------------------------------------------------------------------------

async fn handle_status(db: &DbPool) -> Result<()> {
    let content_repo = ContentRepo::new(db.clone());
    let pub_repo = PublicationRepo::new(db.clone());

    println!("=== ContentForge Pipeline ===\n");

    // Content counts by status.
    let counts = content_repo.count_by_status()?;
    if counts.is_empty() {
        println!("No content yet. Create your first draft:");
        println!("  contentforge draft create \"My first post\" --body \"Hello world\"");
        return Ok(());
    }

    println!("Content by status:");
    for (status, count) in &counts {
        let clean_status = status.trim_matches('"');
        println!("  {clean_status:<12} {count}");
    }

    let total: i64 = counts.iter().map(|(_, c)| c).sum();
    println!("  {:<12} {total}", "total");

    // Recent publications.
    let recent = pub_repo.list_recent(5)?;
    if !recent.is_empty() {
        println!("\nRecent publications:");
        for p in &recent {
            println!(
                "  {} — {} ({})",
                p.platform,
                p.url,
                p.published_at.format("%Y-%m-%d %H:%M")
            );
        }
    }

    // Platform check.
    let account_repo = PlatformAccountRepo::new(db.clone());
    let accounts = account_repo.list_all()?;
    println!("\nConfigured platforms: {}", accounts.len());
    for a in &accounts {
        let status = if a.enabled { "active" } else { "disabled" };
        println!("  {} ({})", a.platform, status);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Utility: resolve short UUID prefix to full UUID
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Pipeline handlers
// ---------------------------------------------------------------------------

async fn handle_pipeline(action: PipelineAction, db: &DbPool) -> Result<()> {
    let queue = contentforge_pipeline::JobQueue::new(db.clone());

    match action {
        PipelineAction::Run {
            content_id,
            pipeline,
            platforms,
            skip_review,
        } => {
            // Pro feature gate
            let license = load_license(db);
            if let Err(msg) = license.require_pro("Pipeline automation") {
                println!("{msg}");
                return Ok(());
            }

            let uuid = resolve_uuid(&content_id, db)?;

            let platform_list: Vec<String> = platforms
                .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|| vec!["devto".to_string()]);

            let payload = serde_json::json!({
                "platforms": platform_list,
                "skip_review": skip_review,
            });

            let job = queue.enqueue(
                &pipeline,
                "adapt", // first step
                Some(uuid),
                None,
                None,
                payload,
                None,
            )?;

            let short_id = &job.id.to_string()[..8];
            println!("Pipeline '{pipeline}' started");
            println!("  Job ID:    {short_id}");
            println!("  Content:   {}", &content_id[..8.min(content_id.len())]);
            println!("  Platforms: {}", platform_list.join(", "));
            if skip_review {
                println!("  Review:    skipped");
            }
            println!("\nTrack: contentforge pipeline show {short_id}");
        }

        PipelineAction::List { status } => {
            let jobs = queue.list(status.as_deref(), None, 20)?;

            if jobs.is_empty() {
                println!("No pipeline jobs.");
                println!("\nStart one:");
                println!("  contentforge pipeline run <content_id>");
                return Ok(());
            }

            println!(
                "{:<10} {:<20} {:<10} {:<15} {:<10} CREATED",
                "ID", "PIPELINE", "STEP", "STATUS", "ATTEMPTS"
            );
            println!("{}", "-".repeat(85));

            for job in &jobs {
                let short_id = &job.id.to_string()[..8];
                println!(
                    "{:<10} {:<20} {:<10} {:<15} {:<10} {}",
                    short_id,
                    job.pipeline,
                    job.step,
                    job.status,
                    format!("{}/{}", job.attempts, job.max_retries),
                    job.created_at.format("%Y-%m-%d %H:%M")
                );
            }
            println!("\n{} job(s)", jobs.len());
        }

        PipelineAction::Show { id } => {
            let uuid = resolve_job_uuid(&id, db)?;
            let job = queue
                .get(uuid)?
                .ok_or_else(|| anyhow::anyhow!("Job not found: {id}"))?;

            println!("=== Pipeline Job ===");
            println!("ID:        {}", job.id);
            println!("Pipeline:  {}", job.pipeline);
            println!("Step:      {}", job.step);
            println!("Status:    {}", job.status);
            if let Some(ref project) = job.project {
                println!("Project:   {project}");
            }
            if let Some(ref platform) = job.platform {
                println!("Platform:  {platform}");
            }
            if let Some(content_id) = job.content_id {
                println!("Content:   {}", &content_id.to_string()[..8]);
            }
            println!("Attempts:  {}/{}", job.attempts, job.max_retries);
            println!("Created:   {}", job.created_at.format("%Y-%m-%d %H:%M"));
            if let Some(ref err) = job.error {
                println!("Error:     {err}");
            }
            if let Some(ref result) = job.result {
                println!("Result:    {}", serde_json::to_string_pretty(result)?);
            }
        }

        PipelineAction::Approve { id } => {
            let uuid = resolve_job_uuid(&id, db)?;
            let job = queue
                .get(uuid)?
                .ok_or_else(|| anyhow::anyhow!("Job not found: {id}"))?;

            if job.status != contentforge_pipeline::JobStatus::AwaitingReview {
                bail!(
                    "Job {} is not awaiting review (status: {})",
                    &id[..8.min(id.len())],
                    job.status
                );
            }

            queue.approve(uuid)?;

            // Enqueue the next step (publish)
            queue.enqueue(
                &job.pipeline,
                "publish",
                job.content_id,
                job.project.as_deref(),
                job.platform.as_deref(),
                job.payload.clone(),
                Some(job.id),
            )?;

            println!("Approved: {}", &id[..8.min(id.len())]);
            println!("Publishing will begin shortly.");
        }

        PipelineAction::Reject { id, reason } => {
            let uuid = resolve_job_uuid(&id, db)?;
            queue.reject(uuid, &reason)?;
            println!("Rejected: {} — {reason}", &id[..8.min(id.len())]);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// License handlers
// ---------------------------------------------------------------------------

async fn handle_license(action: LicenseAction, db: &DbPool) -> Result<()> {
    match action {
        LicenseAction::Activate { key } => {
            let license = contentforge_core::License::validate(&key);
            match license.tier {
                contentforge_core::Tier::Free => {
                    println!("Invalid license key. Tier remains: Free");
                    println!("\nGet a Pro license: https://contentforge.dev/pro");
                }
                tier => {
                    // Store the key in the database
                    let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
                    conn.execute(
                        "INSERT OR REPLACE INTO platform_accounts (id, platform, display_name, credential, enabled, created_at)
                         VALUES ('license', '\"license\"', 'ContentForge License', ?1, 1, datetime('now'))",
                        [&key],
                    )?;
                    println!("License activated!");
                    println!("  Tier:    {tier}");
                    println!("  Email:   {}", license.email);
                    if let Some(ref exp) = license.expires_at {
                        println!("  Expires: {exp}");
                    }
                }
            }
        }

        LicenseAction::Status => {
            let license = load_license(db);
            println!("ContentForge License");
            println!("  Tier:  {}", license.tier);
            if !license.email.is_empty() {
                println!("  Email: {}", license.email);
            }
            if let Some(ref exp) = license.expires_at {
                println!("  Expires: {exp}");
            }
            if license.tier == contentforge_core::Tier::Free {
                println!("\nUpgrade to Pro: https://contentforge.dev/pro");
                println!("Pro features: pipeline automation, approval flows, cron scheduling, encrypted credentials");
            }
        }

        LicenseAction::Deactivate => {
            let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            conn.execute("DELETE FROM platform_accounts WHERE id = 'license'", [])?;
            println!("License deactivated. Tier: Free");
        }
    }
    Ok(())
}

/// Load the license from the database, or return Free tier.
fn load_license(db: &DbPool) -> contentforge_core::License {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => return contentforge_core::License::free(),
    };

    let key: Option<String> = conn
        .query_row(
            "SELECT credential FROM platform_accounts WHERE id = 'license'",
            [],
            |row| row.get(0),
        )
        .ok();

    match key {
        Some(k) => contentforge_core::License::validate(&k),
        None => contentforge_core::License::free(),
    }
}

// ---------------------------------------------------------------------------
// Utility: resolve short job UUID prefix
// ---------------------------------------------------------------------------

fn resolve_job_uuid(id: &str, db: &DbPool) -> Result<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(id) {
        return Ok(uuid);
    }
    let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut stmt = conn.prepare("SELECT id FROM jobs WHERE id LIKE ?1")?;
    let pattern = format!("{id}%");
    let matches: Vec<String> = stmt
        .query_map([pattern], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    match matches.len() {
        0 => bail!("No job found matching '{id}'"),
        1 => Ok(Uuid::parse_str(&matches[0])?),
        n => bail!("Ambiguous job ID '{id}' matches {n} items."),
    }
}

// ---------------------------------------------------------------------------
// Utility: resolve short content UUID prefix
// ---------------------------------------------------------------------------

fn resolve_uuid(id: &str, db: &DbPool) -> Result<Uuid> {
    // Try direct parse first.
    if let Ok(uuid) = Uuid::parse_str(id) {
        return Ok(uuid);
    }

    // Prefix match: find content where ID starts with the given prefix.
    let conn = db.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut stmt = conn.prepare("SELECT id FROM content WHERE id LIKE ?1")?;
    let pattern = format!("{id}%");
    let matches: Vec<String> = stmt
        .query_map([pattern], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    match matches.len() {
        0 => bail!("No content found matching '{id}'"),
        1 => Ok(Uuid::parse_str(&matches[0])?),
        n => bail!("Ambiguous ID '{id}' matches {n} items. Provide more characters."),
    }
}
