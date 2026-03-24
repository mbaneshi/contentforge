# MCP Tools Reference

Complete reference for all MCP tools exposed by ContentForge.

## Overview

ContentForge implements the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) and exposes the following tools when running as an MCP server (`contentforge mcp`).

## Tools

### create_content

Create a new content piece.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "The title of the content piece"
    },
    "body": {
      "type": "string",
      "description": "The content body in Markdown format"
    },
    "content_type": {
      "type": "string",
      "enum": ["article", "thread", "short_post", "video", "image_post", "link_share"],
      "description": "The type of content"
    },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Tags for organizing the content"
    },
    "project": {
      "type": "string",
      "description": "The project this content belongs to"
    }
  },
  "required": ["title", "body", "content_type"]
}
```

**Output:** JSON object with the created content's ID, title, and status.

---

### list_content

List content pieces with optional filtering.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "status": {
      "type": "string",
      "enum": ["idea", "drafting", "review", "ready", "scheduled", "published", "archived"],
      "description": "Filter by content status"
    },
    "content_type": {
      "type": "string",
      "description": "Filter by content type"
    },
    "project": {
      "type": "string",
      "description": "Filter by project name"
    },
    "limit": {
      "type": "number",
      "description": "Maximum number of results (default: 20)"
    }
  }
}
```

**Output:** JSON array of content summaries (id, title, status, content_type, tags, updated_at).

---

### get_content

Get full details of a specific content piece.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "The UUID of the content piece"
    }
  },
  "required": ["id"]
}
```

**Output:** Full content object including body, adaptations, media, and publication records.

---

### adapt_content

Generate a platform-specific adaptation of content.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "The UUID of the content to adapt"
    },
    "platform": {
      "type": "string",
      "enum": ["twitter", "linkedin", "devto", "medium", "youtube", "instagram", "reddit", "hackernews", "substack"],
      "description": "Target platform for adaptation"
    },
    "use_ai": {
      "type": "boolean",
      "description": "Use AI for intelligent adaptation (default: false)"
    }
  },
  "required": ["id", "platform"]
}
```

**Output:** The generated adaptation object with platform-specific body, title, and thread parts (if applicable).

---

### publish

Publish content to a specific platform.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "The UUID of the content to publish"
    },
    "platform": {
      "type": "string",
      "description": "Target platform"
    }
  },
  "required": ["id", "platform"]
}
```

**Output:** Publication record with the live URL, platform post ID, and publication timestamp.

---

### schedule

Schedule content for future publication.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "The UUID of the content to schedule"
    },
    "platform": {
      "type": "string",
      "description": "Target platform"
    },
    "scheduled_at": {
      "type": "string",
      "description": "ISO 8601 datetime for when to publish"
    }
  },
  "required": ["id", "platform", "scheduled_at"]
}
```

**Output:** Schedule entry with the schedule ID, content ID, platform, and scheduled time.

---

### list_platforms

List all configured platform accounts and their health status.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {}
}
```

**Output:** Array of platform accounts with platform name, display name, enabled status, and health check result.

---

### get_analytics

Get engagement metrics for published content.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "description": "Content UUID (omit for summary of all content)"
    }
  }
}
```

**Output:** Analytics data including views, likes, shares, comments, and clicks per platform.

## Transport Configuration

### stdio (default)

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

### SSE

```json
{
  "mcpServers": {
    "contentforge": {
      "url": "http://localhost:3000/mcp/sse"
    }
  }
}
```

## Error Handling

MCP tool calls return errors in the standard MCP error format:

```json
{
  "isError": true,
  "content": [
    {
      "type": "text",
      "text": "Platform Twitter/X not configured. Run 'contentforge platforms add twitter' to set up credentials."
    }
  ]
}
```

Error messages are designed to be human-readable and actionable, so the AI assistant can relay them to the user or take corrective action.
