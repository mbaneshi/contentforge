# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1](https://github.com/mbaneshi/contentforge/compare/contentforge-v0.1.0...contentforge-v0.1.1) (2026-03-24)


### Features

* initial ContentForge scaffold — Phase 0 + Phase 1 ([01ea295](https://github.com/mbaneshi/contentforge/commit/01ea2958b9f662736e1648b4019c27c8ad03159f))
* launch infrastructure — test script, license worker, MCP submission guide ([fc080b4](https://github.com/mbaneshi/contentforge/commit/fc080b4a8e06a2bb94a8747d87cb5c514570ed5c))
* Phase 1+2 — working MCP tools, Mastodon + Bluesky adapters ([92aa5a8](https://github.com/mbaneshi/contentforge/commit/92aa5a8d4fddd1f51022ed06aa0f091cad7290b6))
* Phase 2 — SvelteKit web UI, ratatui TUI, clippy fixes ([e6d53a2](https://github.com/mbaneshi/contentforge/commit/e6d53a2d1ef6a7b03954f55d91a25f65849af296))
* Phase 3 — implement Axum REST API with real handlers ([ea920ff](https://github.com/mbaneshi/contentforge/commit/ea920ffe4d762d4edae48bb38e245979eeba3073))
* Phase 3 — pipeline engine with job queue, worker, approval flow ([9a6241d](https://github.com/mbaneshi/contentforge/commit/9a6241dd5eca1d6bdbb9f150d52fcf62d1d266bc))
* Phase 4 — MCP manifest, install script, updated README ([48bccad](https://github.com/mbaneshi/contentforge/commit/48bccad4b27b0a0d8f9305a9bc1f338f8ba382f4))
* Phase 5 — license gating with Ed25519 validation ([88a7e88](https://github.com/mbaneshi/contentforge/commit/88a7e88f152606a11f8c2a1a6d68f177f2c6c976))
* Phase 6 — cron scheduling, working scheduler, schedule CLI ([c2626c6](https://github.com/mbaneshi/contentforge/commit/c2626c69b885d33d3b2620374a82e9840dc7fc41))


### Bug Fixes

* CI — add frontend build placeholder, fix clippy, format code ([100af72](https://github.com/mbaneshi/contentforge/commit/100af72c199babe2b8513ba23daafbbb50b24e57))
* CI workflows, migrate docs to mdBook, update repo URLs ([1620c63](https://github.com/mbaneshi/contentforge/commit/1620c6319da23adc512c60f6495c3de6e9932b78))
* switch release-please to simple type for workspace compatibility ([85e6eb8](https://github.com/mbaneshi/contentforge/commit/85e6eb8b9d4a9074a2fcbba89032a5b76e201b6a))

## [Unreleased]

### Added
- Workspace structure with 11 modular crates
- Core domain types: `Content`, `ContentStatus`, `ContentType`, `Platform`, `PlatformAdaptation`, `Publication`
- Platform credential model: `PlatformCredential` (API key, OAuth2, integration token, cookie)
- Schedule types: `ScheduleEntry`, `ScheduleStatus`, `RecurringSchedule`
- Error handling with `ContentForgeError` (thiserror) and `anyhow`
- SQLite database layer with WAL mode and migration framework
- Database schema: content, adaptations, media, platform_accounts, publications, schedule, recurring_schedules, analytics
- Content repository (`ContentRepo`) with insert, get, list, update status, delete
- `Publisher` trait with validate, publish, delete, health_check methods
- `PublisherRegistry` for managing and dispatching to multiple platform adapters
- DEV.to adapter: create articles with tags, series, and canonical URL support
- Twitter/X adapter: single tweets and threaded replies with rate limit detection
- LinkedIn adapter: posts via REST API with proper versioning headers
- Medium adapter: articles via integration token with Markdown content format
- Platform metadata: character limits, markdown support, image support, thread support, integration difficulty ratings

[Unreleased]: https://github.com/mbaneshi/contentforge/compare/v0.1.0...HEAD
