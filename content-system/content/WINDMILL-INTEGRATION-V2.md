# Windmill x ContentForge: Integration Roadmap (V2)

> **Goal**: Borrow Windmill's production-grade patterns to build a lightweight, embedded content pipeline — no external service dependency.
>
> **Date**: 2026-03-21 (updated 2026-03-22)
>
> **Context**: ContentForge is a single-user content system. Deploying a full Windmill instance (PostgreSQL + server + workers) is overkill. But Windmill's architecture — built in Rust + Svelte (our exact stack) — solves the same problems we have. This V2 borrows the patterns, not the service.
>
> **V1 vs V2**: V1 proposed running Windmill as an external service. V2 internalizes Windmill's design patterns into ContentForge's own codebase — lighter, self-contained, zero external dependencies beyond SQLite.
>
> **Reality Check (2026-03-22)**: ContentForge already exists as a working 11-crate Rust project at `/Users/bm/contentforge/`. Much of what V2 originally proposed to build from scratch already exists — SQLite schema, platform adapters, MCP server, CLI, credential storage. The Windmill patterns now inform the remaining gaps: **pipeline engine (Phase 3)**, **scheduler completion**, and **analytics**. See `projects/contentforge/STATUS.md` for the real inventory.

---

## Design Principle: Patterns, Not Infrastructure

Windmill is a 100K+ line Rust codebase solving multi-tenant workflow orchestration at scale. ContentForge needs ~5% of that capability. The right move is to study what Windmill got right and implement the minimal version.

| Windmill (full) | ContentForge (borrowed pattern) | Why not the full thing |
|---|---|---|
| PostgreSQL-backed job queue | SQLite-backed job queue | Single user, no concurrent workers needed |
| DAG workflow engine with visual editor | Rust state machine with enum steps | Content pipelines are linear with one branch point (platform adaptation) |
| Multi-tenant approval flows with RBAC | Simple `awaiting_review` job status | One reviewer: you |
| Cron + webhook server | `tokio-cron-scheduler` + CLI trigger | No always-on server needed |
| Encrypted resource store with typed templates | SQLite + `age` encryption | No RBAC, no workspace isolation needed |
| Multi-language script execution (Python, TS, Go, Bash) | Rust-native + Claude CLI subprocess | One language, one LLM |
| Svelte flow editor | TOML/YAML flow definitions | Flows change rarely, no UI needed |

**Target: ~500-800 lines of Rust for the core engine.**

---

## Current State (Truth)

| What works | What's manual | Critical gap |
|---|---|---|
| Brand identity (`brand/`) | Draft generation (`draft.sh` — local only) | No job queue or retry |
| 6 project kits with STATUS.md | Platform adaptation (manual per platform) | No scheduling runtime |
| 5 content templates | Publishing (copy-paste to each platform) | No approval workflow |
| Content calendar (calendar.md) | Calendar updates (manual after publish) | No credential storage |
| Git-based version control | Everything after draft generation | No pipeline orchestration |

---

## Architecture: The Embedded Content Engine

```
┌─────────────────────────────────────────────────┐
│                contentforge CLI                   │
│                                                   │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐ │
│  │ Scheduler  │  │ Pipeline  │  │  Job Queue   │ │
│  │ (cron)     │──▶ Engine    │──▶ (SQLite)     │ │
│  └───────────┘  │ (DAG)     │  │              │ │
│                  └─────┬─────┘  │ - pending    │ │
│                        │        │ - running    │ │
│                        ▼        │ - review     │ │
│               ┌────────────┐   │ - approved   │ │
│               │ Step Runner │   │ - published  │ │
│               │             │   │ - failed     │ │
│               │ - draft     │   │ - retrying   │ │
│               │ - adapt     │   └──────────────┘ │
│               │ - review    │                     │
│               │ - publish   │   ┌──────────────┐ │
│               │ - calendar  │   │  Credentials │ │
│               │             │   │  (SQLite +   │ │
│               └──────┬──────┘   │   age encrypt)│ │
│                      │          └──────────────┘ │
└──────────────────────┼───────────────────────────┘
                       │
        ┌──────────────┼──────────────┬──────────────┐
        ▼              ▼              ▼              ▼
   ┌────────┐   ┌──────────┐  ┌──────────┐  ┌──────────┐
   │ Claude  │   │ Platform │  │ Git Repo │  │ Calendar │
   │ CLI     │   │ APIs     │  │ (source  │  │ (auto-   │
   │ (draft) │   │ (publish)│  │ of truth)│  │  update) │
   └────────┘   └──────────┘  └──────────┘  └──────────┘
```

---

## Pattern 1: SQLite Job Queue with Retry

**Borrowed from**: Windmill's PostgreSQL job queue (`queue.rs`)

### Schema

```sql
CREATE TABLE jobs (
    id          TEXT PRIMARY KEY,  -- ulid
    pipeline    TEXT NOT NULL,     -- 'launch-post', 'weekly-ship'
    project     TEXT NOT NULL,     -- 'codeilus', 'glassforge'
    step        TEXT NOT NULL,     -- 'draft', 'adapt', 'review', 'publish'
    platform    TEXT,              -- 'substack', 'x', 'linkedin', null for non-platform steps
    status      TEXT NOT NULL DEFAULT 'pending',
        -- pending | running | awaiting_review | approved | published | failed | retrying
    payload     TEXT NOT NULL,     -- JSON: template, brand, draft content, etc.
    result      TEXT,              -- JSON: output of completed step
    attempts    INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    retry_after TEXT,              -- ISO 8601 timestamp for next retry
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    completed_at TEXT
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_pipeline ON jobs(pipeline, project);
CREATE INDEX idx_jobs_retry ON jobs(status, retry_after) WHERE status = 'retrying';
```

### Retry Logic (from Windmill's exponential backoff)

```rust
fn next_retry_delay(attempt: u32) -> Duration {
    let base = Duration::from_secs(30);
    let max = Duration::from_secs(600); // 10 min cap
    std::cmp::min(base * 2u32.pow(attempt - 1), max)
}
// Attempt 1: 30s, Attempt 2: 60s, Attempt 3: 120s
```

### Worker Loop

```rust
async fn run_worker(db: &SqlitePool) {
    loop {
        // 1. Pick next pending or retry-ready job
        let job = sqlx::query_as!(Job,
            "UPDATE jobs SET status = 'running', updated_at = datetime('now')
             WHERE id = (
                SELECT id FROM jobs
                WHERE status = 'pending'
                   OR (status = 'retrying' AND retry_after <= datetime('now'))
                ORDER BY created_at ASC LIMIT 1
             ) RETURNING *"
        ).fetch_optional(db).await?;

        match job {
            Some(job) => execute_step(&job, db).await,
            None => tokio::time::sleep(Duration::from_secs(5)).await,
        }
    }
}
```

**What Windmill does differently**: PostgreSQL SKIP LOCKED for concurrent workers, priority queues, worker groups. We don't need any of that — single worker, FIFO, SQLite WAL mode is enough.

---

## Pattern 2: DAG Pipeline Engine

**Borrowed from**: Windmill's flow engine (`flow.rs`)

### Flow Definition (TOML instead of Windmill's JSON)

```toml
[pipeline]
name = "launch-post"
description = "Full launch post pipeline: draft → adapt → review → publish"

[[steps]]
name = "draft"
runner = "claude"
template = "templates/launch-post.md"
inputs = ["brand/bio.md", "projects/{project}/STATUS.md"]

[[steps]]
name = "adapt"
runner = "claude"
parallel_over = ["substack", "x", "linkedin", "devto"]
depends_on = ["draft"]

[[steps]]
name = "review"
runner = "approval"
depends_on = ["adapt"]
# Pipeline suspends here until `contentforge approve <job_id>`

[[steps]]
name = "publish"
runner = "platform_api"
parallel_over = ["substack", "x", "linkedin", "devto"]
depends_on = ["review"]
retry = 3

[[steps]]
name = "update_calendar"
runner = "git"
depends_on = ["publish"]
```

### State Machine (Rust enum, not Windmill's JSON graph)

```rust
#[derive(Debug, Clone)]
enum PipelineState {
    Drafting,
    Adapting { platforms: Vec<Platform>, completed: Vec<Platform> },
    AwaitingReview { drafts: HashMap<Platform, String> },
    Publishing { platforms: Vec<Platform>, completed: Vec<Platform> },
    UpdatingCalendar,
    Done,
    Failed { step: String, error: String, can_retry: bool },
}

impl PipelineState {
    fn next(self, event: StepCompleted) -> PipelineState {
        match (self, event) {
            (Drafting, StepCompleted::Draft(content)) =>
                Adapting { platforms: all_platforms(), completed: vec![] },
            (Adapting { mut completed, platforms }, StepCompleted::Adapted(platform, _))
                if completed.len() + 1 == platforms.len() =>
                AwaitingReview { drafts: collect_drafts() },
            (AwaitingReview { drafts }, StepCompleted::Approved) =>
                Publishing { platforms: drafts.keys().collect(), completed: vec![] },
            // ...
            _ => Failed { step: "unknown".into(), error: "invalid transition".into(), can_retry: false },
        }
    }
}
```

**What Windmill does differently**: Full DAG with fan-out/fan-in, conditional branching, for-loops, error handlers per step. We only need linear + one parallel fan-out (platform adaptation).

---

## Pattern 3: Approval Suspension

**Borrowed from**: Windmill's approval steps (`approvals.rs`)

### How Windmill Does It

Windmill suspends a flow, generates a unique approval URL, sends notification, waits for HTTP POST to approve/reject.

### Our Lightweight Version

```rust
// When pipeline hits review step:
async fn suspend_for_review(job_id: &str, db: &SqlitePool) {
    sqlx::query!(
        "UPDATE jobs SET status = 'awaiting_review', updated_at = datetime('now') WHERE id = ?",
        job_id
    ).execute(db).await?;

    // Notify (simple options, pick one):
    // 1. Terminal notification via `notify-send` or `osascript`
    // 2. Write to stdout: "Draft ready for review. Run: contentforge review <job_id>"
    // 3. Optional: webhook to Slack/email
    notify_review_ready(job_id).await;
}

// CLI command to approve:
// $ contentforge approve <job_id>
// $ contentforge reject <job_id> --reason "tone is off"
async fn approve(job_id: &str, db: &SqlitePool) {
    sqlx::query!(
        "UPDATE jobs SET status = 'approved', updated_at = datetime('now') WHERE id = ?",
        job_id
    ).execute(db).await?;
    // Worker picks up approved job → moves to publish step
}
```

### Review Flow

```
$ contentforge review abc123

Pipeline: launch-post | Project: codeilus
Step: review | Created: 2026-03-21 14:30 UTC

── Substack (1,847 words) ──────────────────────────
I Scanned 3 iOS Apps for iOS 26 Readiness...
[full draft shown or opened in $EDITOR]

── X/Twitter (7 tweets) ────────────────────────────
🧵 1/7: I scanned 3 production iOS apps...
[thread shown]

── LinkedIn (412 words) ────────────────────────────
[draft shown]

Actions:
  contentforge approve abc123
  contentforge approve abc123 --only substack,x
  contentforge reject abc123 --reason "..."
  contentforge edit abc123 --platform substack
```

**What Windmill does differently**: Multi-user RBAC, approval URLs with tokens, timeout-based auto-reject. We need none of that — one reviewer, CLI-based.

---

## Pattern 4: Cron Scheduling

**Borrowed from**: Windmill's schedule engine

### Implementation

```rust
use tokio_cron_scheduler::{JobScheduler, Job};

async fn start_scheduler(db: SqlitePool) {
    let sched = JobScheduler::new().await.unwrap();

    // Weekly ship — every Friday at 06:00 UTC
    sched.add(Job::new_async("0 0 6 * * 5", move |_uuid, _lock| {
        let db = db.clone();
        Box::pin(async move {
            create_pipeline_job(&db, "weekly-ship", "all", None).await;
        })
    }).unwrap()).await.unwrap();

    // Bi-weekly deep post — every other Tuesday
    // Monthly rollup — 1st of month

    sched.start().await.unwrap();
}
```

### Schedule Config (`schedules.toml`)

```toml
[[schedule]]
name = "weekly-ship"
cron = "0 6 * * 5"          # Friday 6 AM UTC
pipeline = "weekly-ship"
project = "all"              # Aggregates all projects

[[schedule]]
name = "deep-post"
cron = "0 6 */14 * *"       # Every 14 days
pipeline = "build-story"
project = "most-active"      # Auto-selects project with most git activity

[[schedule]]
name = "monthly-rollup"
cron = "0 6 1 * *"           # 1st of month
pipeline = "results-post"
project = "all"
```

**What Windmill does differently**: Timezone-aware scheduling, error handlers per schedule, dynamic schedule creation via API. We use system timezone + static TOML config.

---

## Pattern 5: Credential Storage

**Borrowed from**: Windmill's resource/secret management

### Implementation

```rust
use age::secrecy::ExposeSecret;

// Store: encrypt with age, save to SQLite
async fn store_credential(db: &SqlitePool, name: &str, value: &str, key: &age::x25519::Identity) {
    let encrypted = age::encrypt(key.to_public(), value.as_bytes())?;
    sqlx::query!(
        "INSERT OR REPLACE INTO credentials (name, value, updated_at)
         VALUES (?, ?, datetime('now'))",
        name, encrypted
    ).execute(db).await?;
}

// Retrieve: decrypt on access
async fn get_credential(db: &SqlitePool, name: &str, key: &age::x25519::Identity) -> String {
    let row = sqlx::query!("SELECT value FROM credentials WHERE name = ?", name)
        .fetch_one(db).await?;
    age::decrypt(key, &row.value)?
}
```

```sql
CREATE TABLE credentials (
    name       TEXT PRIMARY KEY,  -- 'devto_api_key', 'twitter_oauth', etc.
    value      BLOB NOT NULL,     -- age-encrypted
    updated_at TEXT NOT NULL
);
```

### CLI

```
$ contentforge credentials set devto_api_key
Enter value: ********
Stored (encrypted with age).

$ contentforge credentials list
  devto_api_key     (updated 2026-03-21)
  twitter_oauth     (updated 2026-03-20)
  linkedin_oauth    (updated 2026-03-20)
  substack_token    (updated 2026-03-19)
```

**What Windmill does differently**: Per-workspace scoping, typed resource templates with schema validation, RBAC per secret. We need a flat key-value store with encryption.

---

## Pattern 6: Embedded MCP Server

**Borrowed from**: Windmill's auto-generated MCP endpoint (`mcp.rs`)

### Why Embedded, Not Windmill-Dependent

Windmill auto-exposes scripts as MCP tools via `/api/w/:ws/mcp/*`. Convenient, but it means:
- MCP only works if Windmill is running
- Another service to deploy and maintain
- ContentForge can't be a standalone binary

Same V2 philosophy: borrow the pattern, embed the server.

### What Windmill Does

1. Reads all scripts in a workspace
2. Extracts parameter schemas from function signatures
3. Exposes `list_tools` and `run_tool` over HTTP + stdio
4. Returns typed JSON results

### Our Lightweight Version

MCP is a simple JSON-RPC protocol over stdio or HTTP. We need:
- `tools/list` — enumerate ContentForge capabilities
- `tools/call` — execute a tool and return results

```rust
use rmcp::{Server, Tool, ToolResult}; // or hand-roll — MCP spec is ~200 lines

struct ContentForgeMcp {
    db: SqlitePool,
}

impl ContentForgeMcp {
    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::new("draft", "Generate content draft from template")
                .param("project", "string", "Project name (codeilus, glassforge, ...)", true)
                .param("content_type", "string", "Template type (launch-post, build-story, ...)", true),

            Tool::new("adapt", "Adapt draft for target platforms")
                .param("job_id", "string", "Job ID from draft step", true)
                .param("platforms", "array", "Target platforms", false),

            Tool::new("review", "Show drafts awaiting review")
                .param("job_id", "string", "Specific job ID (optional)", false),

            Tool::new("approve", "Approve drafts for publishing")
                .param("job_id", "string", "Job ID to approve", true)
                .param("only", "array", "Limit to specific platforms", false),

            Tool::new("reject", "Reject drafts with feedback")
                .param("job_id", "string", "Job ID to reject", true)
                .param("reason", "string", "Rejection reason", true),

            Tool::new("publish", "Publish approved drafts")
                .param("job_id", "string", "Job ID to publish", true),

            Tool::new("status", "Show pipeline status")
                .param("project", "string", "Filter by project", false)
                .param("status", "string", "Filter by status", false),

            Tool::new("schedule_list", "List active schedules"),

            Tool::new("schedule_run", "Manually trigger a schedule")
                .param("name", "string", "Schedule name", true),

            Tool::new("credentials_list", "List stored credentials"),
        ]
    }

    async fn call(&self, tool: &str, args: serde_json::Value) -> ToolResult {
        match tool {
            "draft" => {
                let project = args["project"].as_str().unwrap();
                let content_type = args["content_type"].as_str().unwrap();
                let job_id = create_pipeline_job(&self.db, content_type, project, None).await?;
                ToolResult::text(format!("Pipeline started. Job ID: {job_id}"))
            }
            "approve" => {
                let job_id = args["job_id"].as_str().unwrap();
                approve(job_id, &self.db).await?;
                ToolResult::text(format!("Job {job_id} approved. Publishing will begin."))
            }
            "status" => {
                let jobs = list_jobs(&self.db, args.get("project"), args.get("status")).await?;
                ToolResult::json(jobs)
            }
            // ... other tools map 1:1 to CLI commands
            _ => ToolResult::error(format!("Unknown tool: {tool}")),
        }
    }
}
```

### Transport: stdio (Primary) + HTTP (Optional)

```rust
// stdio — for Claude Code / Claude Desktop integration
// ContentForge registers as an MCP server in Claude's config
async fn serve_stdio(mcp: ContentForgeMcp) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    // Read JSON-RPC from stdin, write responses to stdout
    json_rpc_loop(stdin, stdout, mcp).await;
}

// HTTP — optional, for remote access or other MCP clients
async fn serve_http(mcp: ContentForgeMcp, port: u16) {
    let app = axum::Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(mcp);
    axum::serve(TcpListener::bind(("127.0.0.1", port)).await?, app).await?;
}
```

### Claude Code Integration

```json
// ~/.claude/mcp_servers.json
{
  "contentforge": {
    "command": "contentforge",
    "args": ["mcp", "serve"],
    "transport": "stdio"
  }
}
```

Now from any Claude session:
```
You: "Draft a launch post for Codeilus and schedule it for Thursday"

Claude calls via MCP:
  → contentforge.draft(project="codeilus", content_type="launch-post")
  → contentforge.status(job_id="abc123")  // check when draft is ready
  // ... you review and approve via CLI or MCP ...
  → contentforge.approve(job_id="abc123")
```

### What This Gives You vs Windmill MCP

| | Windmill MCP | Embedded MCP |
|---|---|---|
| Dependency | Windmill server must be running | Single binary, always available |
| Tool discovery | Auto-generated from scripts | Explicit tool definitions (more control) |
| Transport | HTTP only | stdio + HTTP (Claude Code prefers stdio) |
| Latency | HTTP round-trip to Windmill | In-process, near zero |
| Offline | No | Yes |
| Schema control | Inferred from script params | Hand-crafted, precise descriptions |

**~150-200 lines for the MCP server. The tools just delegate to the same functions the CLI uses.**

---

## CLI Interface

```
contentforge draft <project> <type>          # Generate draft from template
contentforge adapt <job_id>                   # Adapt draft for all platforms
contentforge review <job_id>                  # Show drafts for review
contentforge approve <job_id> [--only x,li]   # Approve for publishing
contentforge reject <job_id> --reason "..."   # Reject with feedback
contentforge publish <job_id>                 # Publish approved drafts
contentforge status                           # Pipeline status (all jobs)
contentforge schedule list                    # Show active schedules
contentforge schedule run <name>              # Manually trigger a schedule
contentforge credentials set <name>           # Store encrypted credential
contentforge credentials list                 # List stored credentials
contentforge mcp serve                        # Start MCP server (stdio)
contentforge mcp serve --http 8787            # Start MCP server (HTTP)
```

---

## Implementation Timeline

| Week | What | Effort | Deliverable |
|---|---|---|---|
| **1** | SQLite schema + job queue + worker loop + retry logic | 2-3 days | Jobs can be created, queued, executed, retried |
| **2** | Pipeline engine (state machine) + TOML flow definitions + `draft` step wired to Claude CLI | 2-3 days | `contentforge draft codeilus launch-post` creates and runs a pipeline |
| **3** | Approval flow + `review`/`approve`/`reject` CLI commands + platform adaptation step | 2-3 days | Full draft → adapt → review → approve cycle works |
| **4** | Cron scheduler + `schedules.toml` + weekly-ship auto-generation | 1-2 days | Friday weekly-ship drafts auto-generated |
| **5** | Credential storage (age encryption) + first platform integration (DEV.to publish) | 2-3 days | First automated publish to a real platform |
| **6** | Embedded MCP server (stdio + HTTP) + Claude Code integration | 1-2 days | Claude can drive the full pipeline as MCP tools |
| **7** | Remaining platform integrations (X, LinkedIn, Substack) + calendar.md auto-update | 3-4 days | Full multi-platform automated publishing |

**Total: ~700-1000 lines of core engine Rust + ~200 lines per platform integration.**

---

## Summary: V1 vs V2

| | V1 (Windmill as service) | V2 (Borrowed patterns) |
|---|---|---|
| **External deps** | PostgreSQL + Windmill server + workers | SQLite only |
| **Deployment** | docker-compose + separate service | Single binary |
| **Maintenance** | Windmill upgrades, PostgreSQL backups | Just your code |
| **Capability** | Everything Windmill offers | Exactly what ContentForge needs |
| **Multi-tenant** | Yes (workspaces) | No (single user) |
| **MCP** | Free via Windmill | Embedded — stdio + HTTP, ~150 lines |
| **Risk** | Over-engineered for single-user content | Under-engineered if ContentForge goes multi-tenant |
| **Right for** | GlassForge (enterprise, multi-tenant) | ContentForge (personal, lightweight) |

**Bottom line**: V2 gives ContentForge a production-quality content pipeline in a single binary. If ContentForge ever needs multi-tenant support, V1's Windmill deployment is still there via GlassForge. You get the best of both worlds without paying the infrastructure tax upfront.
