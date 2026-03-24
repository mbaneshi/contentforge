<p align="center">
  <h1 align="center">ContentForge</h1>
  <p align="center">A Rust-native content creation and multi-platform publishing platform.<br>Single binary. CLI + TUI + Web + MCP.</p>
</p>

<p align="center">
  <a href="https://github.com/mbaneshi/contentforge/actions/workflows/ci.yml"><img src="https://github.com/mbaneshi/contentforge/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.80%2B-orange.svg" alt="Rust 1.80+">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <img src="https://img.shields.io/badge/platforms-6-brightgreen.svg" alt="6 Platforms">
</p>

---

## What is ContentForge?

Write content once. Adapt for each platform. Publish everywhere — from your terminal, a web dashboard, or your AI assistant via MCP.

```bash
# The hero workflow
contentforge draft create "What I shipped this week" --body "..." --tags "rust,ai"
contentforge adapt <id> --platform devto
contentforge adapt <id> --platform mastodon
contentforge adapt <id> --platform bluesky
contentforge publish <id> --platform devto   # → live on DEV.to

# Or let the pipeline do it all
contentforge pipeline run <id> --platforms devto,mastodon,bluesky --skip-review

# Or use Claude Code via MCP
# Claude: "Draft a post about my latest Rust project and publish to DEV.to and Bluesky"
```

## Why ContentForge?

| Problem | ContentForge |
|---------|-------------|
| Content scattered across 5+ platforms | Write once, adapt per platform |
| Manual copy-paste for cross-posting | CLI/TUI/Web/MCP — automate everything |
| Complex self-hosting (Docker + PG + Redis) | Single binary + SQLite, zero config |
| No developer-native tools (all web dashboards) | CLI-first, git-friendly, terminal-native |
| Existing tools don't work with AI assistants | Built-in MCP server for Claude Code |

## Quick Start

### Install

```bash
# From source (recommended for now)
git clone https://github.com/mbaneshi/contentforge.git
cd contentforge
cargo build --release
# Binary: target/release/contentforge

# Or via the install script
curl -fsSL https://raw.githubusercontent.com/mbaneshi/contentforge/main/install.sh | bash
```

### First Content Piece (2 minutes)

```bash
# 1. Create a draft
contentforge draft create "I built a content pipeline in Rust" \
  --body "## Why\n\nI was tired of copy-pasting to 5 platforms..." \
  --tags "rust,devtools,opensource"

# 2. Add your DEV.to API key (once)
contentforge platforms add devto --key <YOUR_DEVTO_API_KEY>

# 3. Adapt and publish
contentforge adapt <id> --platform devto
contentforge publish <id> --platform devto
# → Your post is live on DEV.to
```

### Pipeline Automation

```bash
# Run a full pipeline: adapt → review → approve → publish
contentforge pipeline run <id> --pipeline adapt-review-publish --platforms devto,mastodon

# Check status
contentforge pipeline list

# Approve when ready
contentforge pipeline approve <job_id>
```

## Interfaces

### CLI
```bash
contentforge draft create/list/show/delete    # Content CRUD
contentforge adapt <id> --platform <platform> # Adapt for a platform
contentforge publish <id> --platform <platform>  # Publish
contentforge pipeline run/list/show/approve/reject  # Automated pipelines
contentforge platforms add/list/remove/check  # Manage credentials
contentforge status                           # Pipeline overview
```

### TUI (Terminal UI)
```bash
contentforge tui
# 5 tabs: Dashboard | Drafts | Adapt | Publish | Platforms
# Navigate: Tab/1-5, j/k, Enter, q to quit, ? for help
```

### Web UI
```bash
contentforge serve --bind 127.0.0.1:3000
# Dashboard, draft editor, publish controls, analytics
```

### MCP Server (for Claude Code / Claude Desktop)
```bash
# Add to Claude Code
claude mcp add contentforge -- contentforge mcp

# Or manually in ~/.claude.json
```

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

**Available MCP tools:**
| Tool | What It Does |
|------|-------------|
| `draft_content` | Create a new draft |
| `list_content` | List content by status |
| `show_content` | Full content details |
| `adapt_content` | Adapt for a platform |
| `publish_content` | Publish to a platform |
| `schedule_content` | Schedule for later |
| `pipeline_status` | Pipeline overview |

## Supported Platforms

| Platform | Status | Auth | Notes |
|----------|--------|------|-------|
| **DEV.to** | Working | API Key | Free, stable, markdown-native |
| **Mastodon** | Working | OAuth per-instance | Free, 500 char limit, FOSS community |
| **Bluesky** | Working | Handle + App Password | Free, 300 char limit, growing dev audience |
| **Twitter/X** | Working | OAuth 2.0 / Bearer | $100+/mo for write access (user brings own keys) |
| **LinkedIn** | Working | OAuth 2.0 | Requires LinkedIn Partner approval for write |
| **Medium** | Deprecated | Integration Token | API deprecated in 2026, limited support |

## Architecture

```
contentforge (single binary)
├── CLI (clap)        ── draft/adapt/publish/pipeline/platforms/status
├── TUI (ratatui)     ── 5-tab terminal dashboard
├── Web (SvelteKit)   ── 8-page web UI via Axum
├── API (Axum)        ── 12 REST endpoints + WebSocket
├── MCP (rmcp)        ── 7 tools for AI assistant integration
├── Pipeline Engine   ── Job queue + worker + retry + approval
├── 6 Adapters        ── DEV.to, Mastodon, Bluesky, Twitter, LinkedIn, Medium
└── SQLite (WAL)      ── Zero-config persistence
```

**12 Rust crates** in a clean workspace architecture:

| Crate | Purpose |
|-------|---------|
| `contentforge-core` | Domain types, platform definitions, errors |
| `contentforge-db` | SQLite persistence, migrations, repositories |
| `contentforge-publish` | Publisher trait + 6 platform adapters |
| `contentforge-pipeline` | Job queue, worker loop, retry, approval flow |
| `contentforge-cli` | CLI commands and handlers |
| `contentforge-tui` | Terminal UI with ratatui |
| `contentforge-api` | Axum REST API + embedded SvelteKit |
| `contentforge-mcp` | MCP server with 7 working tools |
| `contentforge-agent` | AI content generation pipeline |
| `contentforge-schedule` | Cron scheduling engine |
| `contentforge-analytics` | Engagement metrics |
| `contentforge-app` | Binary entry point |

## Documentation

- [Product Strategy](docs/PRODUCT_STRATEGY.md) — vision, pricing, go-to-market
- [Build Plan](docs/BUILD_PLAN.md) — prioritized implementation plan
- [Architecture](docs/architecture/ARCHITECTURE.md) — system design deep-dive
- [Full Docs Site](https://mbaneshi.github.io/contentforge/) — guides and reference

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, code conventions, and how to add a new platform adapter.

## License

MIT — see [LICENSE](LICENSE) for details.
