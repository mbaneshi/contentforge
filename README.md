<p align="center">
  <h1 align="center">ContentForge</h1>
  <p align="center">A Rust-native content creation and multi-platform publishing platform. Single binary. TUI + Web + CLI + MCP.</p>
</p>

<p align="center">
  <a href="https://github.com/mbaneshi-labs/contentforge/actions/workflows/ci.yml"><img src="https://github.com/mbaneshi-labs/contentforge/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.80%2B-orange.svg" alt="Rust 1.80+">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

---

## Why ContentForge?

You write a blog post. Now you need a tweet thread, a LinkedIn post, a DEV.to cross-post, and a Medium article -- each with different formatting, character limits, and tone. ContentForge lets you write once, adapt everywhere, and publish from a single binary with no external services required.

## Feature Highlights

- **Single binary** -- one `contentforge` binary runs CLI, TUI, Web UI, API server, and MCP server
- **Write once, publish everywhere** -- create content in Markdown, auto-adapt to each platform's constraints
- **Local-first** -- SQLite database, no cloud dependency, your data stays on your machine
- **AI-native** -- built-in LLM agent pipeline for drafting, adapting, thread splitting, and review
- **MCP server** -- use ContentForge as a tool from Claude Code or any MCP-compatible AI assistant
- **Scheduling engine** -- queue posts for optimal timing with cron-based recurring schedules
- **Analytics tracking** -- pull engagement metrics back from each platform into one dashboard
- **Privacy-respecting** -- credentials stored locally, no telemetry, no third-party analytics

## Architecture

```
                         ┌─────────────┐
                         │   Claude /   │
                         │  MCP Client  │
                         └──────┬───────┘
                                │ MCP (stdio/SSE)
                                │
┌──────────┐  ┌──────────┐  ┌──┴───────┐
│   TUI    │  │ SvelteKit│  │   CLI    │
│ (ratatui)│  │  Web UI  │  │  (clap)  │
└────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │              │
     └─────────────┼──────────────┘
                   │
            ┌──────┴──────┐
            │  Axum API   │
            │   Server    │
            └──────┬──────┘
                   │
     ┌─────────────┼─────────────┐
     │             │             │
┌────┴────┐  ┌────┴────┐  ┌────┴─────┐
│  Core   │  │   AI    │  │ Schedule │
│ Domain  │  │  Agent  │  │  Engine  │
└────┬────┘  └─────────┘  └──────────┘
     │
┌────┴────────────────────────────────┐
│          Platform Adapters          │
├─────────┬──────────┬────────┬───────┤
│ Twitter │ LinkedIn │ DEV.to │Medium │
│ YouTube │Instagram │ Reddit │ HN    │
└─────────┴──────────┴────────┴───────┘
     │
┌────┴────┐
│ SQLite  │
│ (WAL)   │
└─────────┘
```

## Quick Start

### Install via Homebrew (macOS / Linux)

```bash
brew install mbaneshi/tap/contentforge
```

### Install via Cargo

```bash
cargo install contentforge
```

### Build from Source

```bash
git clone https://github.com/mbaneshi-labs/contentforge.git
cd contentforge
cargo build --release
# Binary is at target/release/contentforge
```

## Usage

### CLI

```bash
# Create a new content piece
contentforge new --title "Rust Error Handling" --type article

# List all drafts
contentforge list --status drafting

# Adapt content for a platform
contentforge adapt <content-id> --platform twitter

# Publish to a specific platform
contentforge publish <content-id> --platform devto

# Publish to all adapted platforms
contentforge publish <content-id> --all

# Schedule for later
contentforge schedule <content-id> --platform twitter --at "2026-03-20T09:00:00Z"

# Check platform health
contentforge platforms health
```

### TUI (Terminal UI)

```bash
# Launch the interactive TUI
contentforge tui
```

The TUI provides a full dashboard with content list, editor, platform status, and schedule overview -- all navigable with keyboard shortcuts.

### Web UI

```bash
# Start the web server (serves embedded SvelteKit frontend)
contentforge serve --port 3000
```

Open `http://localhost:3000` for a rich web interface with drag-and-drop scheduling, live preview, and analytics charts.

### MCP Server (for Claude Code)

```bash
# Start as MCP server over stdio (add to Claude Code config)
contentforge mcp
```

Add to your Claude Code MCP configuration:

```json
{
  "mcpServers": {
    "contentforge": {
      "command": "contentforge",
      "args": ["mcp"]
    }
  }
}
```

Then ask Claude: *"Create a tweet thread about Rust error handling and publish it to Twitter."*

## Supported Platforms

| Platform      | Status     | Auth Method          | Features                        |
|---------------|------------|----------------------|---------------------------------|
| DEV.to        | Ready      | API Key              | Articles, tags, series, canonical URL |
| Twitter/X     | Ready      | OAuth 2.0 / Bearer   | Tweets, threads, media          |
| LinkedIn      | Ready      | OAuth 2.0            | Posts, articles                 |
| Medium        | Ready      | Integration Token    | Articles, tags, canonical URL   |
| YouTube       | Planned    | OAuth 2.0            | Video descriptions, metadata    |
| Instagram     | Planned    | Graph API            | Image posts, captions           |
| Substack      | Planned    | Cookie (fragile)     | Long-form articles              |
| Reddit        | Planned    | OAuth 2.0            | Posts, comments                 |
| Hacker News   | Planned    | Cookie               | Story submissions               |

## Tech Stack

| Component        | Technology                       |
|------------------|----------------------------------|
| Language         | Rust 1.80+                       |
| Async Runtime    | Tokio                            |
| Web Framework    | Axum 0.8                         |
| Database         | SQLite (rusqlite, WAL mode)      |
| TUI              | Ratatui + Crossterm              |
| CLI              | Clap 4 (derive)                  |
| Frontend         | SvelteKit (embedded via rust-embed) |
| AI/LLM           | rig-core                         |
| MCP              | rmcp 0.16                        |
| HTTP Client      | Reqwest (rustls)                 |
| Scheduling       | cron 0.15                        |
| Serialization    | serde + serde_json               |
| Error Handling   | thiserror + anyhow               |

## Workspace Crate Map

| Crate                      | Description                                              |
|----------------------------|----------------------------------------------------------|
| `contentforge-core`        | Domain types: Content, Platform, Schedule, errors         |
| `contentforge-db`          | SQLite persistence, migrations, repository pattern        |
| `contentforge-publish`     | Platform adapter trait (`Publisher`) and implementations   |
| `contentforge-agent`       | AI agent pipeline: generate, adapt, split, review         |
| `contentforge-schedule`    | Cron-based scheduling engine                              |
| `contentforge-analytics`   | Engagement metrics collection and aggregation             |
| `contentforge-api`         | Axum REST API + WebSocket server                          |
| `contentforge-cli`         | Clap-based CLI interface                                  |
| `contentforge-tui`         | Ratatui terminal UI                                       |
| `contentforge-mcp`         | MCP server (stdio + SSE transport)                        |
| `contentforge-app`         | Binary entry point, wires everything together             |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code conventions, and PR process.

## Documentation

- [Architecture](docs/architecture/ARCHITECTURE.md) -- deep dive into system design
- [North Star](NORTH_STAR.md) -- vision and product direction
- [Roadmap](ROADMAP.md) -- phased development plan
- [Full Documentation Site](https://mbaneshi.github.io/contentforge/) -- guides, reference, and tutorials

## License

MIT -- see [LICENSE](LICENSE) for details.
