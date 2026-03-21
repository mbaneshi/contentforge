# ContentForge — Product Strategy

> **Date**: 2026-03-21
> **Status**: Pre-launch
> **Author**: Mehdi Baneshi
> **Repo**: https://github.com/mbaneshi-labs/contentforge

---

## 1. What ContentForge Is

A Rust-native, single-binary content creation and multi-platform publishing platform. Write once, adapt per platform, publish everywhere — from CLI, TUI, Web UI, or AI assistant (via MCP).

**One-liner**: The developer's content engine — draft, adapt, schedule, and publish to every platform from a single binary.

## 2. Problem Statement

Developers and technical creators have the skills to build but no system to be visible. The existing tools fail them:

| Tool | Problem |
|---|---|
| **Buffer/Hootsuite** | Built for marketers, not developers. No markdown, no CLI, no automation |
| **Postiz** (OSS, 27k stars) | Node.js + PostgreSQL + Redis + Temporal — 4 services to self-host |
| **Typefully** | Closed source, X/Threads only, SaaS lock-in |
| **n8n** | General workflow engine, not purpose-built for content |
| **Manual copy-paste** | What 90% of devs actually do — doesn't scale |

**Gap**: No single-binary, developer-native, self-hosted content platform exists. No CLI. No TUI. No MCP. No Rust performance.

## 3. Target Users

### Primary: Solo Developer Creators (TAM: ~2M globally)
- Ship projects but nobody knows about them
- Comfortable with CLI/terminal
- Want automation, not another SaaS dashboard
- Price-sensitive — prefer one-time purchase over subscription

### Secondary: Developer Advocates / DevRel (TAM: ~50K)
- Manage content across 5-10 platforms
- Need scheduling, analytics, team features
- Budget: company pays, $20-100/mo is easy

### Tertiary: Small Dev Teams / Indie Hackers (TAM: ~500K)
- Building in public
- Need shared content pipeline
- 2-5 people

## 4. Competitive Landscape

| Product | Stack | Self-host | CLI | TUI | MCP | AI-native | Pricing |
|---|---|---|---|---|---|---|---|
| **Buffer** | SaaS | No | No | No | No | Add-on | $6-120/mo |
| **Postiz** | Node.js | Yes (complex) | No | No | No | Yes | Free / $20/mo |
| **Mixpost** | PHP/Laravel | Yes | No | No | No | Paid | Free / $299 |
| **Typefully** | SaaS | No | No | No | No | Yes | $12-29/mo |
| **ContentForge** | Rust | Yes (1 binary) | Yes | Yes | Yes | Yes | Free / $49 |

**Unique advantages**:
1. Single binary — `brew install contentforge` and done
2. CLI + TUI + Web — use however you prefer
3. MCP server — Claude Code can drive the entire pipeline
4. Rust performance — sub-millisecond operations
5. SQLite — zero infrastructure, works offline
6. MIT licensed — truly open, no AGPL/BSL tricks

## 5. Product Tiers

### Free (MIT, open source forever)
- Full content CRUD (create, list, edit, delete)
- 4 platform adapters (DEV.to, Twitter/X, LinkedIn, Medium)
- CLI + TUI + Web UI + API server
- Manual publish workflow (draft → adapt → publish, one at a time)
- 5 built-in templates + 5 custom templates
- MCP server (stdio transport — works with Claude Code)
- Basic cron scheduling (single publishes)
- SQLite storage with WAL mode

### Pro ($49 one-time or $9/mo)
- Everything in Free, plus:
- **Pipeline engine** — TOML-defined automated workflows (draft → adapt → review → approve → publish → update-calendar)
- **Cron-triggered pipelines** — "every Friday at 8 AM, auto-generate weekly-ship"
- **Approval flows** — pipeline suspends for review, approve/reject via CLI or MCP
- **Job queue with retry** — exponential backoff, max retries, failure recovery
- **Encrypted credential store** — `age`-encrypted API keys in SQLite
- **MCP HTTP transport** — remote access, team sharing
- **Unlimited custom templates and pipelines**
- **YouTube + Instagram adapters**
- **Analytics** — pull engagement metrics from platforms
- **Priority support** via GitHub

### Team ($19/user/mo — future)
- Everything in Pro, plus:
- Multi-workspace (multiple brands)
- Shared pipeline definitions
- Team approval workflows (RBAC)
- Audit log

## 6. Revenue Projections

### Conservative (Year 1)
| Metric | Value |
|---|---|
| Free users | 1,000 |
| Pro conversion rate | 3% |
| Pro users | 30 |
| Average revenue per Pro user | $49 (one-time) |
| **Year 1 revenue** | **$1,470** |

### Moderate (Year 1, with marketing)
| Metric | Value |
|---|---|
| Free users | 5,000 |
| Pro conversion rate | 5% |
| Pro users | 250 |
| Revenue mix | 60% one-time ($49), 40% monthly ($9/mo × 12) |
| **Year 1 revenue** | **$18,090** |

### Optimistic (Year 1-2, with DevRel/Team tier)
| Metric | Value |
|---|---|
| Free users | 20,000 |
| Pro users | 1,000 |
| Team users | 50 teams × 3 users |
| **Year 2 revenue** | **$90,000+** |

## 7. Technical Architecture

### Current State (Built)
```
contentforge (single binary, ~15MB)
├── contentforge-core       — Domain types (Content, Platform, Schedule)     ✅
├── contentforge-db         — SQLite + migrations + 4 repos                 ✅
├── contentforge-publish    — Publisher trait + 4 adapters (DEV.to, X, LI, Medium) ✅
├── contentforge-cli        — Full CLI (draft, adapt, publish, platforms, status)  ✅
├── contentforge-tui        — 5-tab terminal UI                              ✅
├── contentforge-api        — Axum REST API (12 endpoints) + WebSocket       ✅
├── contentforge-mcp        — MCP server (rmcp, stdio)                       ✅ (stubbed)
├── contentforge-agent      — AI agent pipeline traits                       ✅ (stubbed)
├── contentforge-schedule   — Scheduler crate                                ✅ (stubbed)
├── contentforge-analytics  — Analytics types                                ✅ (stubbed)
├── contentforge-app        — Binary entry point                             ✅
└── frontend/               — SvelteKit 5 + Tailwind 4 (8 pages)            ✅
```

### Next to Build (Pro features)
```
├── contentforge-pipeline   — Job queue + worker + state machine + retry
├── contentforge-core/
│   ├── template.rs         — Template engine with variable substitution
│   ├── pipeline.rs         — Pipeline definitions + state machine
│   └── license.rs          — Tier gating (Ed25519 offline validation)
├── templates/              — Built-in content templates (ship with binary)
├── pipelines/              — Built-in pipeline definitions (TOML)
└── encrypted credentials   — age-encrypted credential store
```

### License Gating (Technical)
```rust
pub enum Tier { Free, Pro, Team }

// Ed25519 signed license key — offline validation, no phone-home
// Gate at feature entry points only (pipeline, analytics, premium adapters)
// Free features are FULL features, not crippled versions
```

## 8. Go-to-Market Strategy

### Phase 1: Build Audience (Week 1-4)
- Ship v0.1.0 as open-source free tool
- Post about building ContentForge (build in public)
- Launch on: Hacker News (Show HN), Reddit (r/rust, r/programming), DEV.to
- Get first 100-500 GitHub stars
- Collect feedback, fix issues

### Phase 2: Establish Value (Week 5-8)
- Ship v0.2.0 with pipeline engine (Pro feature)
- Create landing page at contentforge.dev
- Write 3 case studies: "How I publish to 5 platforms in 30 seconds"
- YouTube demo video (60 seconds)
- Engage with commenters, build relationships

### Phase 3: Monetize (Week 9-12)
- Add Stripe checkout for Pro license
- Ship license key validation
- Launch Pro tier
- Product Hunt launch
- Dev.to / Hashnode sponsorship outreach

### Phase 4: Scale (Month 4+)
- Team tier
- Template marketplace
- Integration partnerships
- Conference talks / podcasts

## 9. Distribution Channels

| Channel | Type | Expected Impact |
|---|---|---|
| **Hacker News (Show HN)** | Launch | High (developer audience, Rust projects do well) |
| **Reddit r/rust** | Community | High (Rust community supports Rust projects) |
| **Reddit r/programming** | Broad | Medium |
| **DEV.to** | Article series | Medium-High (target audience lives here) |
| **Twitter/X** | Ongoing | Medium (build-in-public audience) |
| **LinkedIn** | Professional | Medium (DevRel audience) |
| **Product Hunt** | Launch | Medium-High (dev tools category) |
| **YouTube** | Demo video | Medium |
| **Homebrew** | Distribution | High (frictionless install) |
| **crates.io** | Rust ecosystem | Medium |

## 10. Key Metrics to Track

| Metric | Target (Month 3) | Target (Month 6) |
|---|---|---|
| GitHub stars | 500 | 2,000 |
| Homebrew installs | 200 | 1,000 |
| Weekly active CLI users | 50 | 200 |
| Pro conversions | 10 | 50 |
| Monthly revenue | $490 | $2,000 |
| Content published via ContentForge | 500 posts | 5,000 posts |

## 11. Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Platform APIs break/change | High | Adapter pattern isolates changes; community can contribute fixes |
| Postiz adds CLI/TUI | Medium | ContentForge's Rust perf + single binary + MCP are hard to replicate in Node.js |
| Low conversion to Pro | Medium | Keep Pro price low ($49 one-time), consider "pay what you want" |
| Twitter/X API pricing increases | Medium | Free tier (500 posts/mo) sufficient for most users |
| Solo maintainer burnout | High | Open source community; keep scope minimal; Pro revenue funds time |

## 12. Success Criteria (6 months)

- [ ] 1,000+ GitHub stars
- [ ] 50+ paying Pro users
- [ ] $2,000+ monthly revenue
- [ ] Used by at least 3 recognizable dev advocates
- [ ] Featured in at least 1 Rust newsletter
- [ ] ContentForge used to publish its own marketing content (dogfooding)
