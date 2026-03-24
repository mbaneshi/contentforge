# MCP Registry Submission Guide

## 1. Official MCP Registry
- URL: https://registry.modelcontextprotocol.io
- Submit via PR to: https://github.com/modelcontextprotocol/servers
- Add entry pointing to our repo

## 2. Gradually AI
- URL: https://gradually.ai/en/mcp-servers/
- Submit: look for "Add Server" or contact form
- Use mcp-manifest.json data

## 3. APITracker
- URL: https://apitracker.io/mcp-servers
- Submit: look for submission form
- Use mcp-manifest.json data

## 4. MCP Server Finder
- URL: https://mcpserverfinder.com
- Submit: look for "Submit" button
- Use mcp-manifest.json data

## 5. PulseMCP
- URL: https://pulsemcp.com
- Submit: look for submission process

## Submission Data (copy from mcp-manifest.json)

**Name**: contentforge
**Description**: Developer content pipeline — draft, adapt, schedule, and publish to DEV.to, Mastodon, Bluesky, Twitter, LinkedIn from your AI assistant.
**Repository**: https://github.com/mbaneshi/contentforge
**Transport**: stdio
**Command**: `contentforge mcp`
**Install**: `git clone https://github.com/mbaneshi/contentforge && cd contentforge && cargo build --release`
**Categories**: social-media, content, publishing, developer-tools
**Tools**: draft_content, list_content, show_content, adapt_content, publish_content, schedule_content, pipeline_status

## Claude Code Setup (include in all submissions)

```bash
claude mcp add contentforge -- contentforge mcp
```

Or manually add to MCP config:
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
