# Crate Map

ContentForge is organized as a Cargo workspace with 11 crates. This page describes what each crate does and how they relate.

## Dependency Diagram

```
contentforge-app
├── contentforge-cli
├── contentforge-tui
├── contentforge-mcp
├── contentforge-api
│   ├── contentforge-publish
│   ├── contentforge-agent
│   ├── contentforge-schedule
│   └── contentforge-analytics
├── contentforge-db
└── contentforge-core (shared by all)
```

## Crate Descriptions

### contentforge-core

**Path:** `crates/contentforge-core`
**Dependencies:** serde, chrono, uuid, thiserror

The foundation crate with zero internal dependencies. Contains all domain types:

- `Content` -- the central entity (title, body, status, tags, adaptations, media)
- `ContentStatus` -- lifecycle enum (Idea, Drafting, Review, Ready, Scheduled, Published, Archived)
- `ContentType` -- what kind of content (Article, Thread, ShortPost, Video, ImagePost, LinkShare)
- `Platform` -- supported platforms enum with metadata (char limits, markdown support, etc.)
- `PlatformAdaptation` -- platform-specific version of content
- `PlatformCredential` -- authentication credential types (API key, OAuth2, token, cookie)
- `PlatformAccount` -- a configured platform connection
- `Publication` -- record of a successful publish
- `ScheduleEntry` -- a scheduled publication event
- `RecurringSchedule` -- cron-based recurring rules
- `ContentForgeError` -- the project-wide error type
- `MediaAttachment` -- attached files with MIME types

---

### contentforge-db

**Path:** `crates/contentforge-db`
**Dependencies:** contentforge-core, rusqlite, serde_json, anyhow

SQLite persistence layer:

- `init_db(path)` -- opens/creates database with WAL mode, runs migrations
- `init_memory_db()` -- in-memory database for testing
- `ContentRepo` -- CRUD operations for content (insert, get_by_id, list_by_status, update_status, delete)
- Migration framework in `migrations.rs` with append-only SQL files

Schema tables: content, adaptations, media, platform_accounts, publications, schedule, recurring_schedules, analytics.

---

### contentforge-publish

**Path:** `crates/contentforge-publish`
**Dependencies:** contentforge-core, reqwest, async-trait, serde_json

Platform adapter trait and implementations:

- `Publisher` trait -- the interface every adapter implements (platform, validate, publish, delete, health_check)
- `PublisherRegistry` -- container for all configured adapters with `get()` and `publish_all()`
- `DevToPublisher` -- DEV.to Forem API adapter
- `TwitterPublisher` -- Twitter/X API v2 adapter (single tweets + threads)
- `LinkedInPublisher` -- LinkedIn REST API adapter
- `MediumPublisher` -- Medium API adapter

---

### contentforge-agent

**Path:** `crates/contentforge-agent`
**Dependencies:** contentforge-core, rig-core, serde_json, tokio

AI agent pipeline:

- Content generation from prompts
- Intelligent platform adaptation (not just truncation)
- Thread splitting with natural break points
- Content review and quality scoring
- Uses `rig-core` for LLM provider abstraction

---

### contentforge-schedule

**Path:** `crates/contentforge-schedule`
**Dependencies:** contentforge-core, contentforge-db, contentforge-publish, cron, chrono, tokio

Scheduling engine:

- Polls the schedule table on a configurable interval
- Dispatches due entries to the appropriate publisher
- Handles retries with exponential backoff
- Manages recurring schedules via cron expressions
- Runs as a background Tokio task

---

### contentforge-analytics

**Path:** `crates/contentforge-analytics`
**Dependencies:** contentforge-core, contentforge-db, reqwest

Engagement metrics collection:

- Periodically fetches metrics from platform APIs (views, likes, shares, comments, clicks)
- Stores snapshots in the analytics table
- Provides aggregation queries for dashboards

---

### contentforge-api

**Path:** `crates/contentforge-api`
**Dependencies:** contentforge-core, contentforge-db, contentforge-publish, contentforge-agent, contentforge-schedule, contentforge-analytics, axum, tower-http, rust-embed

The Axum HTTP server:

- REST endpoints under `/api/` for all CRUD operations
- WebSocket at `/api/ws` for real-time updates
- Embedded SvelteKit static files served for all non-API routes
- CORS configuration for development
- Request validation and error responses

---

### contentforge-cli

**Path:** `crates/contentforge-cli`
**Dependencies:** contentforge-core, contentforge-db, contentforge-publish, contentforge-api, clap

Command-line interface:

- `new` -- create content
- `list` -- list content with filters
- `show` -- display content details
- `edit` -- edit content body
- `adapt` -- create platform adaptations
- `publish` -- publish to platforms
- `schedule` -- schedule future publications
- `platforms` -- manage platform accounts
- `analytics` -- view engagement metrics
- `serve` -- start the web server
- `tui` -- launch the TUI
- `mcp` -- start the MCP server
- `daemon` -- run the scheduling daemon
- `doctor` -- diagnose configuration issues

---

### contentforge-tui

**Path:** `crates/contentforge-tui`
**Dependencies:** contentforge-core, contentforge-db, contentforge-api, ratatui, crossterm

Terminal user interface:

- Dashboard with content list, platform status, and schedule overview
- Content editor with Markdown preview
- Platform adaptation preview
- Schedule management
- Keyboard-driven navigation

---

### contentforge-mcp

**Path:** `crates/contentforge-mcp`
**Dependencies:** contentforge-core, contentforge-db, contentforge-publish, contentforge-agent, rmcp

MCP server implementation:

- stdio transport for Claude Code
- SSE transport for web clients
- Exposes tools: create_content, list_content, adapt_content, publish, schedule, list_platforms, get_analytics
- Handles JSON-RPC requests per the MCP specification

---

### contentforge-app

**Path:** `crates/contentforge-app`
**Dependencies:** all other crates, tokio, tracing-subscriber

Binary entry point:

- Parses CLI arguments
- Initializes logging (tracing-subscriber with env-filter)
- Initializes the database
- Dispatches to the appropriate interface (CLI, TUI, Web, MCP)
- Wires all crates together
