# Content System

This repo is the **single source of truth** for all content creation, personal branding, and launch campaigns across all projects and platforms.

## How It Works

```
1. BUILD something → 2. Document it in projects/ → 3. Draft content from templates/
→ 4. Publish across platforms → 5. Track in calendar.md
```

## Directory Structure

| Directory | Purpose |
|-----------|---------|
| `brand/` | Canonical identity — bio, links, niche, assets |
| `projects/` | Per-project content kits (talking points, screenshots, launch plans) |
| `templates/` | Reusable content formats per platform |
| `content/drafts/` | Work-in-progress content |
| `content/published/` | Final published content (with metadata) |
| `scripts/` | Automation scripts for content generation and publishing |
| `calendar.md` | Content calendar — what, when, where |

## Content Workflow

### Step 1: Project ships or hits a milestone
- Update `projects/<name>/STATUS.md`
- Add screenshots/GIFs to `projects/<name>/assets/`

### Step 2: Generate content
```bash
# Use Claude Code to draft from template
# Example: generate a launch post for Codeilus
./scripts/draft.sh codeilus launch-post
```

### Step 3: Adapt per platform
Each piece of content gets adapted:
- **Substack** — full article (1000-2000 words)
- **DEV.to** — same article, canonical URL points to Substack
- **X/Twitter** — thread version (5-8 tweets)
- **LinkedIn** — professional angle (300-500 words)
- **YouTube** — script for demo video

### Step 4: Publish and track
- Move from `content/drafts/` to `content/published/`
- Update `calendar.md` with links and dates

## Content Types

| Type | When to Use | Template |
|------|------------|----------|
| `launch-post` | First public announcement of a project | `templates/launch-post.md` |
| `build-story` | Technical deep-dive on how you built it | `templates/build-story.md` |
| `results-post` | Show real data/results from using the tool | `templates/results-post.md` |
| `weekly-ship` | Weekly summary of what you shipped | `templates/weekly-ship.md` |
| `tip-post` | Quick tip or insight from your work | `templates/tip-post.md` |

## Automation

- `scripts/draft.sh` — Generate a draft from a template + project data
- `scripts/status.sh` — Show content pipeline status
- `scripts/publish-checklist.sh` — Pre-publish checklist per platform

## ContentForge (The Product)

This repo documents the content workflow. **ContentForge** (`/Users/bm/contentforge/`) is the Rust binary that automates it.

| This repo (content-mbaneshi) | ContentForge (contentforge/) |
|---|---|
| Brand, templates, calendar | CLI, TUI, API, MCP server |
| Manual workflow docs | Automated pipeline engine |
| Project STATUS.md files | SQLite + 6 platform adapters |
| Shell scripts (draft.sh) | `contentforge draft/adapt/publish` |

See `projects/contentforge/STATUS.md` for current build status.
