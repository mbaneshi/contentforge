use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use contentforge_api::AppState;
use contentforge_cli::CliCommand;
use contentforge_mcp::ContentForgeMcp;
use contentforge_publish::PublisherRegistry;

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "contentforge",
    about = "ContentForge — AI-powered content pipeline. Create, adapt, schedule, and publish across platforms.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Path to the SQLite database file.
    #[arg(long, default_value = "contentforge.db", global = true)]
    db: PathBuf,
}

#[derive(Subcommand)]
enum Command {
    /// Start the Axum web server (API + embedded frontend).
    Serve {
        /// Address to bind to.
        #[arg(short, long, default_value = "127.0.0.1:3000")]
        bind: String,
    },
    /// Start the terminal UI.
    Tui,
    /// Start the MCP stdio server (for AI assistant integration).
    Mcp,

    // --- CLI subcommands delegated to contentforge-cli ---
    /// Create, list, or show drafts.
    Draft {
        #[command(subcommand)]
        action: contentforge_cli::DraftAction,
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
        action: contentforge_cli::ScheduleAction,
    },
    /// List, add, or check platform integrations.
    Platforms {
        #[command(subcommand)]
        action: contentforge_cli::PlatformAction,
    },
    /// Run, list, and manage automated pipelines (Pro feature).
    Pipeline {
        #[command(subcommand)]
        action: contentforge_cli::PipelineAction,
    },
    /// Manage your ContentForge license.
    License {
        #[command(subcommand)]
        action: contentforge_cli::LicenseAction,
    },
    /// Show the current content pipeline overview.
    Status,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    // Initialize database.
    let db = contentforge_db::init_db(&cli.db)?;

    match cli.command {
        // Default (no subcommand) → serve
        None => run_serve(db, "127.0.0.1:3000".to_string()).await,

        Some(Command::Serve { bind }) => run_serve(db, bind).await,

        Some(Command::Tui) => {
            contentforge_tui::run(db)?;
            Ok(())
        }

        Some(Command::Mcp) => run_mcp(db).await,

        // Delegate CLI subcommands
        Some(Command::Draft { action }) => {
            contentforge_cli::execute(CliCommand::Draft { action }, db).await
        }
        Some(Command::Adapt {
            id,
            platform,
            title,
            canonical_url,
        }) => {
            contentforge_cli::execute(
                CliCommand::Adapt {
                    id,
                    platform,
                    title,
                    canonical_url,
                },
                db,
            )
            .await
        }
        Some(Command::Publish { id, platform }) => {
            contentforge_cli::execute(CliCommand::Publish { id, platform }, db).await
        }
        Some(Command::Schedule { action }) => {
            contentforge_cli::execute(CliCommand::Schedule { action }, db).await
        }
        Some(Command::Platforms { action }) => {
            contentforge_cli::execute(CliCommand::Platforms { action }, db).await
        }
        Some(Command::Pipeline { action }) => {
            contentforge_cli::execute(CliCommand::Pipeline { action }, db).await
        }
        Some(Command::License { action }) => {
            contentforge_cli::execute(CliCommand::License { action }, db).await
        }
        Some(Command::Status) => contentforge_cli::execute(CliCommand::Status, db).await,
    }
}

// ---------------------------------------------------------------------------
// Serve (Axum)
// ---------------------------------------------------------------------------

async fn run_serve(db: contentforge_db::DbPool, bind: String) -> Result<()> {
    let publishers = PublisherRegistry::new();
    let state = AppState::new(db, publishers);
    let app = contentforge_api::app_router(state);

    let addr: SocketAddr = bind.parse()?;
    tracing::info!(%addr, "Starting ContentForge web server");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// MCP stdio server
// ---------------------------------------------------------------------------

async fn run_mcp(db: contentforge_db::DbPool) -> Result<()> {
    use rmcp::ServiceExt;
    use std::sync::Arc;

    let publishers = Arc::new(PublisherRegistry::new());
    let server = ContentForgeMcp::new(db, publishers);

    tracing::info!("Starting ContentForge MCP server on stdio");

    let transport = rmcp::transport::io::stdio();
    let running = server.serve(transport).await?;
    running.waiting().await?;

    Ok(())
}
