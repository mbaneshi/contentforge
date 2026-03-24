# ContentForge

**A Rust-native content creation and multi-platform publishing platform. Single binary. TUI + Web + CLI + MCP.**

## What is ContentForge?

ContentForge lets you write content once in Markdown and publish adapted versions to multiple platforms -- Twitter/X, LinkedIn, DEV.to, Medium, and more -- from a single binary. No Docker, no cloud services, no SaaS subscriptions.

## Why ContentForge?

- **Write once, publish everywhere.** Create a blog post, and ContentForge generates a tweet thread, a LinkedIn post, and a DEV.to cross-post, each respecting platform-specific formatting and character limits.
- **Single binary.** One `cargo install` or `brew install` gives you CLI, TUI, Web UI, and MCP server. No infrastructure to manage.
- **Local-first.** Your content lives in a local SQLite database. No data leaves your machine until you publish.
- **AI-native.** Built-in LLM integration for drafting, adapting, and reviewing content. Exposes an MCP server so Claude Code can manage your content directly.
- **Developer-friendly.** TOML configuration, JSON output mode, full REST API, scriptable CLI.

## Quick Install

### Homebrew

```bash
brew install mbaneshi/tap/contentforge
```

### Cargo

```bash
cargo install contentforge
```

### From Source

```bash
git clone https://github.com/mbaneshi/contentforge.git
cd contentforge
cargo build --release
```

## First Steps

1. [Install ContentForge](getting-started/installation.md)
2. [Follow the Quick Start tutorial](getting-started/quickstart.md) (5 minutes)
3. [Configure your platform credentials](getting-started/configuration.md)

## Features at a Glance

| Feature              | Description                                                |
|----------------------|------------------------------------------------------------|
| Multi-platform       | DEV.to, Twitter/X, LinkedIn, Medium, and more              |
| Content adaptation   | Auto-adapt long-form to short-form, threads, platform limits|
| Scheduling           | Cron-based scheduling with retry logic                     |
| AI agents            | Generate, adapt, split, and review content with LLMs       |
| MCP server           | Use ContentForge from Claude Code or any MCP client        |
| Analytics            | Pull engagement metrics from all platforms into one view   |
| Multiple interfaces  | CLI, TUI, Web UI -- all from the same binary               |
