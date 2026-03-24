# Architecture Overview

This page provides a high-level architecture overview for contributors. For the full deep-dive, see [ARCHITECTURE.md](https://github.com/mbaneshi/contentforge/blob/main/docs/architecture/ARCHITECTURE.md) in the repository.

## Design Principles

1. **Layered architecture** -- interfaces at the top, domain logic in the middle, infrastructure at the bottom. Dependencies flow downward only.
2. **Single binary** -- all interfaces (CLI, TUI, Web, MCP) compile into one binary. No separate deployments.
3. **Trait-based extensibility** -- the `Publisher` trait is the extension point for platform adapters. Adding a new platform requires implementing one trait.
4. **SQLite only** -- no external database. The embedded SQLite database with WAL mode handles concurrent reads from the API server.

## System Layers

### Interface Layer

The top layer provides four ways to interact with ContentForge:

| Interface | Crate               | Technology        | Purpose                          |
|-----------|---------------------|-------------------|----------------------------------|
| CLI       | `contentforge-cli`  | Clap 4 (derive)   | Scriptable command-line access   |
| TUI       | `contentforge-tui`  | Ratatui + Crossterm| Interactive terminal dashboard  |
| Web       | `contentforge-api`  | Axum + SvelteKit  | Browser-based rich interface     |
| MCP       | `contentforge-mcp`  | rmcp              | AI assistant integration         |

All four interfaces share the same domain logic and database. They are different frontends to the same system.

### Service Layer

The `contentforge-api` crate provides the Axum HTTP server that all interfaces ultimately use (the CLI calls domain logic directly for simple operations but shares the same crate dependencies).

Key responsibilities:

- REST endpoints for CRUD operations
- WebSocket connections for real-time updates
- Static file serving for the embedded SvelteKit frontend
- Request validation and error mapping

### Domain Layer

The core business logic lives in three crates:

- **`contentforge-core`** -- Data types (`Content`, `Platform`, `ScheduleEntry`, etc.) and error types. Zero dependencies on infrastructure.
- **`contentforge-agent`** -- AI pipeline that uses LLMs for content generation, adaptation, and review. Depends on `rig-core`.
- **`contentforge-schedule`** -- Scheduling engine that polls for due entries and dispatches to publishers.

### Infrastructure Layer

- **`contentforge-db`** -- SQLite database with migrations, WAL mode, and repository pattern.
- **`contentforge-publish`** -- Platform adapter trait and implementations. Makes HTTP calls to platform APIs.
- **`contentforge-analytics`** -- Pulls engagement metrics from platform APIs.

## Data Flow

```
User Input (any interface)
        |
        v
   Content CRUD (contentforge-db)
        |
        v
   AI Adaptation (contentforge-agent, optional)
        |
        v
   Platform Adaptation stored (contentforge-db)
        |
        v
   Schedule or Publish (contentforge-schedule / contentforge-publish)
        |
        v
   Platform API call (contentforge-publish adapters)
        |
        v
   Publication record stored (contentforge-db)
        |
        v
   Analytics collected (contentforge-analytics)
```

## Key Traits

### Publisher

```rust
#[async_trait]
pub trait Publisher: Send + Sync {
    fn platform(&self) -> Platform;
    fn validate(&self, adaptation: &PlatformAdaptation) -> Result<(), ContentForgeError>;
    async fn publish(&self, content: &Content, adaptation: &PlatformAdaptation) -> Result<Publication, ContentForgeError>;
    async fn delete(&self, publication: &Publication) -> Result<(), ContentForgeError>;
    async fn health_check(&self) -> Result<(), ContentForgeError>;
}
```

This is the only trait you need to implement to add a new platform. The `PublisherRegistry` manages all registered adapters and provides `publish_all()` for multi-platform publishing.

## Error Handling

Two-tier approach:

- **Library crates** use `ContentForgeError` (thiserror) with typed variants for each error category (publish failed, rate limited, auth failed, content too long, etc.)
- **Application boundaries** use `anyhow::Error` for wrapping with context

See `crates/contentforge-core/src/error.rs` for the full error type.
