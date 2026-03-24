# MCP Integration

ContentForge implements the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) and can be used as a tool server for AI assistants like Claude Code.

## What is MCP?

MCP is a protocol that allows AI assistants to use external tools. When ContentForge runs as an MCP server, an AI assistant can create content, adapt it for platforms, schedule publication, and check analytics -- all through natural language.

## Setup with Claude Code

### Step 1: Start the MCP Server

Add ContentForge to your Claude Code MCP configuration. Edit your Claude Code settings (typically `~/.config/claude-code/mcp.json` or the project-level `.mcp.json`):

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

### Step 2: Verify

In Claude Code, the ContentForge tools will appear automatically. You can verify by asking Claude:

> "What ContentForge tools are available?"

### Transport Modes

ContentForge supports two MCP transport modes:

| Mode  | Command                  | Use Case                     |
|-------|--------------------------|------------------------------|
| stdio | `contentforge mcp`       | Claude Code, local AI tools  |
| SSE   | `contentforge mcp --sse` | Web-based AI clients         |

For Claude Code, stdio (the default) is the correct choice.

## Available Tools

### `create_content`

Create a new content piece.

**Parameters:**

| Parameter      | Type   | Required | Description                        |
|---------------|--------|----------|------------------------------------|
| `title`        | string | yes      | Content title                      |
| `body`         | string | yes      | Markdown body                      |
| `content_type` | string | yes      | article, thread, short_post, etc.  |
| `tags`         | array  | no       | Tags for organization              |
| `project`      | string | no       | Associated project name            |

**Example prompt:**

> "Create a new article titled 'Understanding Rust Lifetimes' with tags rust and tutorial."

### `list_content`

List content filtered by status, project, or type.

**Parameters:**

| Parameter      | Type   | Required | Description                        |
|---------------|--------|----------|------------------------------------|
| `status`       | string | no       | Filter by status (idea, drafting, published, etc.) |
| `project`      | string | no       | Filter by project                  |
| `content_type` | string | no       | Filter by content type             |
| `limit`        | number | no       | Max results (default 20)           |

**Example prompt:**

> "List all my draft articles."

### `get_content`

Get a specific content piece by ID.

**Parameters:**

| Parameter | Type   | Required | Description   |
|-----------|--------|----------|---------------|
| `id`      | string | yes      | Content UUID  |

### `adapt_content`

Generate a platform-specific adaptation.

**Parameters:**

| Parameter  | Type   | Required | Description                       |
|-----------|--------|----------|-----------------------------------|
| `id`       | string | yes      | Content UUID                      |
| `platform` | string | yes      | Target platform (twitter, devto, etc.) |
| `use_ai`   | boolean| no       | Use AI for intelligent adaptation |

**Example prompt:**

> "Adapt my latest article for Twitter as a thread."

### `publish`

Publish content to a platform.

**Parameters:**

| Parameter  | Type   | Required | Description                       |
|-----------|--------|----------|-----------------------------------|
| `id`       | string | yes      | Content UUID                      |
| `platform` | string | yes      | Target platform                   |

**Example prompt:**

> "Publish the Rust lifetimes article to DEV.to."

### `schedule`

Schedule content for future publication.

**Parameters:**

| Parameter      | Type   | Required | Description                       |
|---------------|--------|----------|-----------------------------------|
| `id`           | string | yes      | Content UUID                      |
| `platform`     | string | yes      | Target platform                   |
| `scheduled_at` | string | yes      | ISO 8601 datetime                 |

**Example prompt:**

> "Schedule the Twitter thread for tomorrow at 9 AM EST."

### `list_platforms`

List configured platform accounts and their health status.

**Parameters:** None

### `get_analytics`

Get engagement metrics for published content.

**Parameters:**

| Parameter | Type   | Required | Description                    |
|-----------|--------|----------|--------------------------------|
| `id`      | string | no       | Content UUID (omit for all)    |

## Example Conversations

### Full Publishing Workflow

> **You:** "Write a blog post about why developers should use Rust for CLI tools, then publish it to DEV.to and create a Twitter thread for it."
>
> **Claude:** *Uses `create_content` to draft the article, then `adapt_content` for DEV.to and Twitter, then `publish` to both platforms. Reports the live URLs.*

### Scheduled Campaign

> **You:** "I have three articles ready. Schedule them for this week -- one on Monday, Wednesday, and Friday at 9 AM on LinkedIn and Twitter."
>
> **Claude:** *Uses `list_content` to find ready articles, then `schedule` for each platform and date.*

### Analytics Check

> **You:** "How did my posts perform this week?"
>
> **Claude:** *Uses `get_analytics` and summarizes views, likes, and engagement across platforms.*

## Security Considerations

- MCP over stdio communicates only with the parent process (Claude Code). No network exposure.
- Platform credentials are read from your local config. The MCP server does not accept credentials as tool parameters.
- All publish actions are explicit -- the AI must call the `publish` tool; no automatic publishing happens.
- Audit: All operations are logged via the standard tracing framework. Set `RUST_LOG=contentforge=info` to see MCP tool invocations.
