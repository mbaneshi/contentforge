# CLI Reference

Complete reference for all ContentForge CLI commands.

## Global Options

```
contentforge [OPTIONS] <COMMAND>

Options:
  --config <PATH>      Path to config file (default: ~/.config/contentforge/config.toml)
  --db <PATH>          Path to database file (default: ~/.local/share/contentforge/contentforge.db)
  --format <FORMAT>    Output format: text, json (default: text)
  -v, --verbose        Increase log verbosity (can be repeated: -vv, -vvv)
  -h, --help           Print help
  -V, --version        Print version
```

## Commands

### `new`

Create a new content piece.

```bash
contentforge new [OPTIONS]
```

| Option           | Type   | Required | Description                          |
|------------------|--------|----------|--------------------------------------|
| `--title <TEXT>` | string | yes      | Content title                        |
| `--type <TYPE>`  | string | yes      | Content type (article, thread, short_post, video, image_post, link_share) |
| `--body <TEXT>`  | string | no       | Content body (opens editor if omitted) |
| `--tags <TAGS>`  | string | no       | Comma-separated tags                 |
| `--project <NAME>` | string | no    | Associated project                   |
| `--no-edit`      | flag   | no       | Skip opening the editor              |

**Examples:**

```bash
# Create an article and open the editor
contentforge new --title "Rust Error Handling Guide" --type article --tags rust,errors

# Create with inline body
contentforge new --title "Quick tip" --type short_post --body "Use ? operator for clean error propagation" --no-edit

# Create and associate with a project
contentforge new --title "v0.2.0 Release Notes" --type article --project contentforge
```

---

### `list`

List content with optional filters.

```bash
contentforge list [OPTIONS]
```

| Option             | Type   | Description                          |
|--------------------|--------|--------------------------------------|
| `--status <STATUS>`| string | Filter by status                     |
| `--type <TYPE>`    | string | Filter by content type               |
| `--project <NAME>` | string | Filter by project                   |
| `--tag <TAG>`      | string | Filter by tag                        |
| `--limit <N>`      | number | Maximum results (default: 50)        |
| `--offset <N>`     | number | Skip first N results                 |

**Examples:**

```bash
# List all drafts
contentforge list --status drafting

# List articles tagged "rust"
contentforge list --type article --tag rust

# JSON output for scripting
contentforge list --status ready --format json
```

---

### `show`

Display details of a content piece.

```bash
contentforge show <ID> [OPTIONS]
```

| Option               | Type   | Description                          |
|----------------------|--------|--------------------------------------|
| `--adaptations`      | flag   | Show all platform adaptations        |
| `--adaptation <PLAT>`| string | Show adaptation for a specific platform |
| `--publications`     | flag   | Show publication records             |

**Examples:**

```bash
# Show content details
contentforge show a1b2c3d4

# Show with all adaptations
contentforge show a1b2c3d4 --adaptations

# Show Twitter adaptation only
contentforge show a1b2c3d4 --adaptation twitter
```

---

### `edit`

Edit a content piece.

```bash
contentforge edit <ID> [OPTIONS]
```

| Option           | Type   | Description                          |
|------------------|--------|--------------------------------------|
| `--title <TEXT>` | string | Update the title                     |
| `--body <TEXT>`  | string | Update the body (opens editor if omitted) |
| `--tags <TAGS>`  | string | Replace tags (comma-separated)       |
| `--project <NAME>` | string | Update project association         |

---

### `status`

Change the status of a content piece.

```bash
contentforge status <ID> --set <STATUS>
```

Valid statuses: `idea`, `drafting`, `review`, `ready`, `scheduled`, `published`, `archived`.

---

### `adapt`

Create a platform-specific adaptation.

```bash
contentforge adapt <ID> [OPTIONS]
```

| Option             | Type   | Required | Description                       |
|--------------------|--------|----------|-----------------------------------|
| `--platform <PLAT>`| string | yes*     | Target platform                   |
| `--all`            | flag   | yes*     | Adapt for all configured platforms|
| `--ai`             | flag   | no       | Use AI for intelligent adaptation |

*One of `--platform` or `--all` is required.

**Examples:**

```bash
# Adapt for Twitter
contentforge adapt a1b2c3d4 --platform twitter

# AI-powered adaptation for LinkedIn
contentforge adapt a1b2c3d4 --platform linkedin --ai

# Adapt for all platforms
contentforge adapt a1b2c3d4 --all
```

---

### `publish`

Publish content to a platform.

```bash
contentforge publish <ID> [OPTIONS]
```

| Option             | Type   | Required | Description                       |
|--------------------|--------|----------|-----------------------------------|
| `--platform <PLAT>`| string | yes*     | Target platform                   |
| `--all`            | flag   | yes*     | Publish to all adapted platforms  |

**Examples:**

```bash
# Publish to DEV.to
contentforge publish a1b2c3d4 --platform devto

# Publish to all adapted platforms
contentforge publish a1b2c3d4 --all
```

---

### `schedule`

Schedule content for future publication.

```bash
contentforge schedule <SUBCOMMAND>
```

#### `schedule add`

```bash
contentforge schedule add <ID> --platform <PLAT> --at <DATETIME>
```

| Option             | Type   | Required | Description                       |
|--------------------|--------|----------|-----------------------------------|
| `--platform <PLAT>`| string | yes      | Target platform                   |
| `--at <DATETIME>`  | string | yes      | ISO 8601 datetime                 |

#### `schedule list`

```bash
contentforge schedule list [OPTIONS]
```

| Option             | Type   | Description                       |
|--------------------|--------|-----------------------------------|
| `--status <STATUS>`| string | Filter by schedule status (pending, published, failed) |
| `--platform <PLAT>`| string | Filter by platform               |

#### `schedule cancel`

```bash
contentforge schedule cancel <SCHEDULE-ID>
```

#### `schedule recurring`

```bash
contentforge schedule recurring --name <NAME> --cron <EXPR> --platforms <PLATS>
```

**Examples:**

```bash
# Schedule a tweet for tomorrow morning
contentforge schedule add a1b2c3d4 --platform twitter --at "2026-03-20T09:00:00Z"

# List pending schedules
contentforge schedule list --status pending

# Set up a weekly recurring schedule
contentforge schedule recurring --name "weekly-roundup" --cron "0 9 * * FRI" --platforms twitter,linkedin
```

---

### `platforms`

Manage platform accounts.

```bash
contentforge platforms <SUBCOMMAND>
```

#### `platforms add`

```bash
contentforge platforms add <PLATFORM> [OPTIONS]
```

Platform-specific options:

- `devto`: `--api-key <KEY>`
- `twitter`: `--bearer-token <TOKEN>`
- `linkedin`: `--access-token <TOKEN> --author-urn <URN>`
- `medium`: `--token <TOKEN>`

#### `platforms list`

```bash
contentforge platforms list
```

#### `platforms health`

```bash
contentforge platforms health [--platform <PLAT>]
```

#### `platforms remove`

```bash
contentforge platforms remove <PLATFORM>
```

---

### `analytics`

View engagement metrics.

```bash
contentforge analytics [OPTIONS]
```

| Option             | Type   | Description                       |
|--------------------|--------|-----------------------------------|
| `<ID>`             | string | Content ID (omit for summary)     |
| `--summary`        | flag   | Show aggregate metrics            |
| `--platform <PLAT>`| string | Filter by platform               |

---

### `generate`

Generate content using AI.

```bash
contentforge generate <PROMPT> [OPTIONS]
```

| Option             | Type   | Description                       |
|--------------------|--------|-----------------------------------|
| `--type <TYPE>`    | string | Content type (default: article)   |
| `--tags <TAGS>`    | string | Comma-separated tags              |
| `--project <NAME>` | string | Associated project               |
| `--length <N>`     | number | Target word count                 |
| `--estimate`       | flag   | Show cost estimate only           |
| `--confirm`        | flag   | Skip confirmation prompt          |

---

### `review`

Get AI feedback on content.

```bash
contentforge review <ID>
```

---

### `serve`

Start the web server.

```bash
contentforge serve [OPTIONS]
```

| Option           | Type   | Description                       |
|------------------|--------|-----------------------------------|
| `--host <HOST>`  | string | Bind address (default: 127.0.0.1) |
| `--port <PORT>`  | number | Port number (default: 3000)       |
| `--cors`         | flag   | Enable CORS headers               |

---

### `tui`

Launch the terminal UI.

```bash
contentforge tui
```

---

### `mcp`

Start the MCP server.

```bash
contentforge mcp [OPTIONS]
```

| Option   | Type | Description                       |
|----------|------|-----------------------------------|
| `--sse`  | flag | Use SSE transport instead of stdio|

---

### `daemon`

Run the scheduling daemon.

```bash
contentforge daemon [OPTIONS]
```

| Option              | Type   | Description                       |
|---------------------|--------|-----------------------------------|
| `--interval <SECS>` | number | Poll interval in seconds (default: 30) |

---

### `doctor`

Diagnose configuration and environment issues.

```bash
contentforge doctor
```

---

### `completions`

Generate shell completions.

```bash
contentforge completions <SHELL>
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.
