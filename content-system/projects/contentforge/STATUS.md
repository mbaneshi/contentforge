# ContentForge — Content Kit

## Project Summary
Rust-native, single-binary content creation and multi-platform publishing platform. Draft, adapt, schedule, and publish to every platform from CLI, TUI, Web UI, or AI assistant (via MCP).

## Key Facts
- **Repo**: https://github.com/mbaneshi/contentforge
- **Local**: `/Users/bm/contentforge/`
- **Version**: v0.1.0 (pre-launch)
- **Stack**: Rust (11 crates) + SvelteKit 5 + SQLite
- **Tests**: 4 passing (contentforge-db only)
- **LOC**: ~7,000 Rust across 11 crates
- **Binary**: Single binary (`contentforge serve|tui|mcp|draft|adapt|publish|...`)
- **Compiles**: Yes (clean `cargo check`)

## Architecture (11 Crates)

| Crate | Status | LOC | Notes |
|---|---|---|---|
| `contentforge-core` | Complete | ~390 | Content, Platform (11), Schedule, Error types |
| `contentforge-db` | Complete | ~610 | SQLite WAL, 9 tables, 4 repos, 4 tests |
| `contentforge-publish` | Complete | ~1,050 | Publisher trait + 6 adapters |
| `contentforge-cli` | Complete | ~810 | Full command structure with clap |
| `contentforge-api` | Partial | ~640 | Routes defined, handlers implemented |
| `contentforge-tui` | Partial | ~2,360 | 5-tab TUI, compiles but incomplete |
| `contentforge-mcp` | Partial | ~500 | draft/list/show work, adapt/publish/schedule stubbed |
| `contentforge-app` | Complete | ~190 | Binary entry point, all modes wired |
| `contentforge-agent` | Stubs | ~210 | Traits defined, StubGenerator only |
| `contentforge-schedule` | Stubs | ~140 | Skeleton with 4 todo!() |
| `contentforge-analytics` | Stubs | ~95 | Types only, 4 todo!() |

## Platform Adapters (6 Built)

| Platform | Adapter | Auth | Status |
|---|---|---|---|
| DEV.to | Complete | API Key | Working end-to-end |
| Mastodon | Complete | Instance + Token | Built, untested live |
| Bluesky | Complete | App Password | Built, untested live |
| Twitter/X | Complete | Bearer Token | Built, deferred ($100+/mo API) |
| LinkedIn | Complete | OAuth2 | Built, deferred (partner gate) |
| Medium | Complete | Integration Token | Built, API deprecated 2026 |

## Remaining `todo!()` Stubs (8 total)
- `contentforge-schedule`: 4 stubs (publish logic, DB queries, status update, retry)
- `contentforge-analytics`: 4 stubs (pipeline summary, platform stats, dashboard, record metrics)

## Product Strategy
- **Free tier**: Full CRUD, 4 platforms, CLI + TUI + Web + API + MCP
- **Pro tier**: $49 one-time or $9/mo — pipelines, approval flows, job queue, encryption, analytics
- **Team tier**: $19/user/mo (future) — multi-workspace, RBAC
- **Target**: Solo developer creators building in public

## Build Plan Priority
1. Wire MCP tools to real DB (P1.1)
2. Wire scheduler to real publish (P1.2)
3. Mastodon + Bluesky live testing (P2)
4. Pipeline engine — Pro feature (P3)
5. MCP registry + Homebrew (P4)
6. Encryption + license + Stripe (P5)
7. Cron-triggered pipelines (P6)

## Content Hooks
- "I built a content engine in Rust that publishes to 6 platforms from one command"
- "Single binary, SQLite, MCP server — the developer's content pipeline"
- "Why I chose Rust over Node.js for a content publishing platform"
- "How MCP turns Claude Code into your content manager"

## Launch Status
- [x] Core architecture (11 crates)
- [x] SQLite schema + migrations
- [x] 6 platform adapters
- [x] CLI command structure
- [x] MCP server (partial)
- [x] TUI (partial)
- [x] API routes (partial)
- [x] Product strategy + research
- [ ] Fill todo!() stubs (scheduler + analytics)
- [ ] Complete MCP tools (adapt, publish, schedule)
- [ ] Agent pipeline (Claude CLI integration)
- [ ] Live test Mastodon + Bluesky
- [ ] Pipeline engine (Pro feature)
- [ ] License gating
- [ ] Homebrew formula
- [ ] Landing page (contentforge.dev)
- [ ] Launch post drafted
- [ ] Show HN + Reddit + DEV.to launch

## Priority: HIGH (this is the product)
