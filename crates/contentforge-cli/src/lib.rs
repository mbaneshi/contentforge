use anyhow::{bail, Context, Result};
use clap::Subcommand;
use contentforge_core::{
    Content, ContentStatus, ContentType, Platform, PlatformAccount, PlatformAdaptation,
    PlatformCredential,
};
use contentforge_db::repo::{AdaptationRepo, ContentRepo, PlatformAccountRepo, PublicationRepo};
use contentforge_db::DbPool;
use contentforge_publish::adapters::devto::DevToPublisher;
use contentforge_publish::adapters::linkedin::LinkedInPublisher;
use contentforge_publish::adapters::medium::MediumPublisher;
use contentforge_publish::adapters::twitter::TwitterPublisher;
use contentforge_publish::Publisher;
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
    /// Show the current content pipeline overview.
    Status,
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
    /// Add a new schedule entry.
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
        _ => bail!(
            "Platform {platform} is not configured. Run: contentforge platforms add {platform_str} --key <key>",
            platform_str = platform.to_string().to_lowercase()
        ),
    }
}

// ---------------------------------------------------------------------------
// Schedule handler (stub for Phase 1)
// ---------------------------------------------------------------------------

async fn handle_schedule(action: ScheduleAction, _db: &DbPool) -> Result<()> {
    match action {
        ScheduleAction::List => {
            println!("Schedule: (not yet implemented — coming in Phase 2)");
        }
        ScheduleAction::Add { .. } => {
            println!("Schedule add: (not yet implemented — coming in Phase 2)");
        }
        ScheduleAction::Cancel { .. } => {
            println!("Schedule cancel: (not yet implemented — coming in Phase 2)");
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
