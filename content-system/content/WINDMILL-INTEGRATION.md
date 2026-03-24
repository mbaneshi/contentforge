# Windmill x ContentForge: Integration Roadmap

> **Goal**: Automate the content pipeline — from project milestone to multi-platform publish — using Windmill as the orchestration layer.
>
> **Date**: 2026-03-21
>
> **Context**: ContentForge is a git-based content system with brand identity, project kits (6 projects), templates (5 types), 3 shell scripts, and a manual workflow. Content is adapted for 7 platforms (Substack, DEV.to, X, LinkedIn, YouTube, Reddit, HN). Everything is manual today — drafting, adapting, publishing, and calendar tracking.

---

## Current ContentForge State (Truth)

| What works | What's manual | Critical gap |
|---|---|---|
| Brand identity (`brand/`) | Draft generation (`draft.sh` — local only) | No automated publishing to any platform |
| 6 project kits with STATUS.md | Platform adaptation (manual per platform) | No scheduling runtime for recurring content |
| 5 content templates | Publishing (copy-paste to each platform) | No approval workflow for review before publish |
| Content calendar (calendar.md) | Calendar updates (manual after publish) | No secret management for platform API keys |
| Git-based version control | Status tracking (`status.sh` — local only) | No retry/monitoring for failed publishes |
| Publish checklist script | Everything after draft generation | No MCP integration for Claude-driven content ops |

---

## Priority 1: Content Pipeline as a Windmill Flow

**Value**: VERY HIGH | **Effort**: LOW-MEDIUM | **Impact**: Immediate

### Why this first

- The core pain is manual repetition: draft → adapt × 5 platforms → publish × 5 → update calendar. This is pure orchestration work.
- Windmill flows model this naturally as a DAG with parallel branches and an approval gate.
- Your 3 shell scripts (`draft.sh`, `status.sh`, `publish-checklist.sh`) become Windmill scripts with retry, logging, and monitoring for free.

### What you get

```
Trigger (manual / webhook / schedule)
  │
  ▼
Read project STATUS.md + brand/bio.md + template
  │
  ▼
Claude generates base draft (Windmill script wrapping Claude CLI)
  │
  ├──▶ Adapt for Substack (1000-2000 words)
  ├──▶ Adapt for X/Twitter (5-8 tweet thread)
  ├──▶ Adapt for LinkedIn (300-500 words)
  ├──▶ Adapt for DEV.to (cross-post format)
  └──▶ Adapt for YouTube (script outline)
  │
  ▼
 ╔══════════════════════╗
 ║   APPROVAL GATE      ║
 ║  You review all 5    ║
 ║  drafts before any   ║
 ║  publishing happens  ║
 ╚══════════════════════╝
  │
  ▼
Publish to approved platforms (parallel)
  │
  ▼
Update calendar.md with links + dates
  │
  ▼
Commit and push calendar update
```

### How

1. Wrap `draft.sh` as a Windmill Bash script with parameters: `project_name`, `content_type`, `template`
2. Create platform adapter scripts (one per platform) that take base draft + platform config → adapted output
3. Compose as a Windmill flow with `for_each` parallel over platforms
4. Add approval step between adaptation and publishing
5. Final step: update `calendar.md` and git commit

### Replaces

- Manual draft → adapt → publish cycle
- `publish-checklist.sh` (Windmill enforces the checklist as flow steps)

---

## Priority 2: Scheduling for Recurring Content

**Value**: VERY HIGH | **Effort**: LOW | **Impact**: Consistency

### Why

- `calendar.md` defines recurring commitments:
  - Every Friday: weekly ship thread (X + LinkedIn)
  - Every 2 weeks: deep technical post (Substack + DEV.to)
  - Monthly: rollup post (Substack)
- Today these are reminders you have to remember and act on. Missed weeks = broken consistency = lost audience.
- Windmill cron scheduling makes these automatic.

### What you get

- **Friday 6 AM UTC**: Windmill triggers → scans all `projects/*/STATUS.md` for week's changes → generates weekly-ship draft → notifies you for review
- **Bi-weekly Tuesday 6 AM UTC**: Triggers deep technical post generation from most active project → draft ready for review
- **1st of month**: Generates monthly rollup from all project statuses → draft ready

### How

1. Create `weekly-ship-generator` script: reads git log + STATUS.md changes from past week → generates draft from `templates/weekly-ship.md`
2. Schedule via Windmill:
   ```json
   {
     "path": "f/contentforge/weekly_ship",
     "script_path": "f/contentforge/generate_weekly_ship",
     "schedule": "0 6 * * 5",
     "timezone": "UTC"
   }
   ```
3. Output → approval gate → publish flow (reuses P1 pipeline)

### Replaces

- Manual calendar checking
- Risk of missed recurring content

---

## Priority 3: Multi-Platform Publishing with Secrets

**Value**: HIGH | **Effort**: MEDIUM | **Impact**: True automation

### Why

- Without actual platform API integrations, "automation" stops at draft generation.
- Each platform needs credentials: Substack API token, X OAuth, LinkedIn OAuth, DEV.to API key.
- Windmill provides encrypted per-resource storage with typed templates.

### What you get

- Platform publish scripts that actually post content:
  - **DEV.to**: `POST /api/articles` with API key
  - **X/Twitter**: OAuth 2.0 thread posting
  - **LinkedIn**: Share API with OAuth
  - **Substack**: API or headless publish
- Credentials encrypted at rest, accessed via `wm.get_resource("f/contentforge/devto_key")`
- Failed publishes retry automatically (3 attempts, 30s delay)
- Partial success handling: if X fails but LinkedIn succeeds, only retry X

### How

1. Define resource types: `devto_credentials`, `twitter_credentials`, `linkedin_credentials`, `substack_credentials`
2. Create publish scripts per platform (TypeScript or Python — Windmill supports both)
3. Wire into the P1 flow as the post-approval parallel step
4. Error handler: on failure → Slack/email notification + auto-retry

### Replaces

- Manual copy-paste publishing
- No credential management

---

## Priority 4: MCP — Claude as Content Operator

**Value**: HIGH | **Effort**: LOW | **Impact**: Developer experience

### Why

- `draft.sh` only works locally. You can't say "draft a launch post for Codeilus" from Claude Code and have it actually happen end-to-end.
- Windmill auto-exposes all scripts as MCP tools.
- This means Claude can discover and invoke your entire content pipeline as tools.

### What you get

From any Claude session:
```
You: "Draft a launch post for Codeilus, schedule it for Thursday on Substack and X"

Claude calls:
  → contentforge_generate_draft(project="codeilus", type="launch-post")
  → contentforge_adapt_platforms(draft_id="...", platforms=["substack", "x"])
  → contentforge_schedule_publish(draft_id="...", date="2026-03-27", platforms=["substack", "x"])
```

### How

1. Deploy content scripts as Windmill scripts (done in P1)
2. Windmill auto-exposes via `/api/w/:workspace/mcp/list_tools` and `/api/w/:workspace/mcp/run_tool`
3. Add Windmill MCP endpoint to Claude Code config
4. Claude discovers tools automatically

### Replaces

- Local-only `draft.sh`
- Need to build custom MCP server

---

## Priority 5: Content Analytics & Feedback Loop

**Value**: MEDIUM-HIGH | **Effort**: MEDIUM | **Impact**: Content quality over time

### Why

- Currently no tracking of what performs well. You publish and forget.
- Windmill can schedule periodic analytics pulls from each platform.
- Feed performance data back into content generation to improve over time.

### What you get

- Weekly analytics pull: engagement per post per platform
- Auto-update `projects/*/STATUS.md` with content performance
- Identify what content types and topics perform best
- Feed top-performing patterns back into template selection

### How

1. Create analytics scripts per platform (pull engagement via API)
2. Schedule weekly: `0 8 * * 1` (Monday morning)
3. Aggregate into `content/analytics/` or append to project STATUS.md
4. Use results to prioritize next week's content calendar

---

## What NOT to Use Windmill For

| Capability | Why not |
|---|---|
| **Replacing git-based content storage** | Git gives you version history, branching, collaboration. Windmill is for orchestration, not storage. |
| **Replacing templates/** | Templates are source-of-truth markdown. Windmill reads them, doesn't replace them. |
| **Replacing brand/** | Brand identity is static reference data. Keep it in git. |
| **Content editing / writing** | Claude does the writing. Windmill orchestrates when and where it happens. |
| **Replacing calendar.md** | Human-readable calendar stays. Windmill syncs to it after publish. |

---

## Deployment Architecture

```
┌─────────────────────────────────────────────────┐
│              You (Review & Approve)              │
└──────────────────┬──────────────────────────────┘
                   │ Approval via Windmill UI / Slack / Email
         ┌─────────▼──────────┐
         │     Windmill        │
         │  (PostgreSQL +      │◄──── Orchestration: flows, schedules,
         │   Server + Workers) │      approvals, secrets, MCP, audit
         └─────────┬──────────┘
                   │
    ┌──────────────┼──────────────┬──────────────┐
    ▼              ▼              ▼              ▼
┌────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ Claude  │  │ Platform │  │ Git Repo │  │ Analytics│
│ CLI     │  │ APIs     │  │ (source  │  │ APIs     │
│ (draft) │  │ (publish)│  │ of truth)│  │ (metrics)│
└────────┘  └──────────┘  └──────────┘  └──────────┘
                │
    ┌───────────┼───────────┬───────────┬───────────┐
    ▼           ▼           ▼           ▼           ▼
 Substack    DEV.to     X/Twitter   LinkedIn    YouTube
```

---

## Shared Infrastructure with GlassForge

Both ContentForge and GlassForge use the **same Windmill deployment**:

| Capability | GlassForge | ContentForge |
|---|---|---|
| **Flows** | scan → plan → migrate → verify | draft → adapt → approve → publish |
| **Approval gates** | Human reviews AI code changes | Human reviews AI content drafts |
| **Job queue** | Migration tasks with retry | Publish tasks with retry |
| **Secrets** | Claude API key, GitHub tokens | Platform API keys (X, LinkedIn, etc.) |
| **Scheduling** | Nightly re-scans | Weekly ship, bi-weekly posts |
| **MCP** | `glassforge scan/migrate/verify` | `contentforge draft/publish/schedule` |
| **Audit** | Who approved which migration | Who approved which post |

One Windmill instance, two workspaces (`glassforge`, `contentforge`), full isolation.

---

## Implementation Timeline

| Week | What | Effort | Unblocks |
|---|---|---|---|
| **1** | Deploy Windmill (docker-compose, shared with GlassForge). Wrap `draft.sh` as Windmill script. Test MCP endpoint with Claude Code. | 2 days | MCP integration, all subsequent work |
| **2** | Build full content flow: trigger → generate → adapt (parallel) → approval → save to `content/drafts/`. | 2-3 days | Automated draft pipeline |
| **3** | Add scheduling: weekly-ship (Friday), bi-weekly deep post (Tuesday). Auto-generate from STATUS.md + git log. | 1-2 days | Recurring content consistency |
| **4** | Platform API integrations: DEV.to + X first (easiest APIs). Store credentials as Windmill resources. | 3-4 days | Actual automated publishing |
| **5** | Add LinkedIn + Substack publishing. Calendar.md auto-update after publish. | 2-3 days | Full multi-platform automation |
| **6+** | Analytics feedback loop. Content performance tracking. | Ongoing | Data-driven content strategy |

---

## Summary: Impact Matrix

| Priority | Feature | Value | Effort | Impact | Replaces |
|---|---|---|---|---|---|
| **P1** | Content Pipeline Flow | Very High | Low-Medium | Immediate | Manual draft-adapt-publish cycle |
| **P2** | Scheduling | Very High | Low | Consistency | Manual calendar reminders |
| **P3** | Platform Publishing + Secrets | High | Medium | True automation | Copy-paste publishing, no credential mgmt |
| **P4** | MCP Integration | High | Low | DX | Local-only draft.sh |
| **P5** | Analytics Feedback | Medium-High | Medium | Quality | No performance tracking |

**Bottom line**: Windmill turns ContentForge from "scripts + discipline" into an automated content pipeline where your only job is the creative review. Same Windmill instance as GlassForge — one orchestrator for your entire ecosystem. P1-P2 ship in one week. Full multi-platform automation by week 5.
