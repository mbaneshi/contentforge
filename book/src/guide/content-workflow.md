# Content Workflow

ContentForge follows a structured content lifecycle from initial idea to published post with tracked analytics. This guide walks through each stage.

## The Content Lifecycle

```
Idea  -->  Drafting  -->  Review  -->  Ready  -->  Scheduled  -->  Published  -->  Archived
```

Each piece of content has a `status` field that tracks its position in this lifecycle. Status transitions can happen manually (via CLI/TUI/Web) or automatically (e.g., publishing changes status to `Published`).

## Stage 1: Idea

Capture an idea before you forget it. Ideas are lightweight -- just a title and optional notes.

```bash
contentforge new --title "Why Rust's borrow checker is your friend" --type article
```

Or capture just a one-liner:

```bash
contentforge new --title "Thread about error handling patterns" --type thread
```

### Content Types

| Type         | Description                                    | Best for                     |
|--------------|------------------------------------------------|------------------------------|
| `article`    | Long-form content in Markdown                  | DEV.to, Medium, Substack     |
| `thread`     | Multi-part content (e.g., tweet threads)       | Twitter/X                    |
| `short_post` | Short-form text                                | LinkedIn, single tweets      |
| `video`      | Video content metadata                         | YouTube                      |
| `image_post` | Image with caption                             | Instagram                    |
| `link_share` | URL with commentary                            | LinkedIn, Twitter, Reddit    |

## Stage 2: Drafting

Write the canonical version of your content in Markdown. This is the "source of truth" from which all platform adaptations are derived.

```bash
# Open in your configured editor
contentforge edit <content-id>

# Or set the body directly
contentforge edit <content-id> --body "Full markdown content here..."
```

Add tags for organization:

```bash
contentforge tag <content-id> --add rust,error-handling,tutorial
```

Associate with a project:

```bash
contentforge edit <content-id> --project contentforge
```

## Stage 3: Adapt

Generate platform-specific versions of your content. Each adaptation respects the target platform's constraints (character limits, formatting, tag limits).

```bash
# Adapt for a specific platform
contentforge adapt <content-id> --platform twitter
contentforge adapt <content-id> --platform devto
contentforge adapt <content-id> --platform linkedin

# Adapt for all configured platforms at once
contentforge adapt <content-id> --all
```

### What Adaptation Does

| Source Type | Target Platform | Adaptation                                    |
|-------------|-----------------|-----------------------------------------------|
| Article     | Twitter         | Splits into a thread, each tweet under 280 chars |
| Article     | LinkedIn        | Extracts key points, trims to 3,000 chars     |
| Article     | DEV.to          | Keeps Markdown, limits to 4 tags              |
| Article     | Medium          | Keeps Markdown, limits to 5 tags              |
| Thread      | LinkedIn        | Combines thread into a single post            |
| Short Post  | Twitter         | Validates under 280 chars                     |

### AI-Powered Adaptation

If an AI agent is configured, adaptations are generated intelligently:

```bash
contentforge adapt <content-id> --platform twitter --ai
```

The AI agent will:

1. Read the canonical content
2. Understand the target platform's style and constraints
3. Generate a platform-native version (not just truncation)
4. Preserve the core message while adjusting tone

### Preview Adaptations

```bash
# See the adaptation for a specific platform
contentforge show <content-id> --adaptation twitter

# See all adaptations
contentforge show <content-id> --adaptations
```

## Stage 4: Review (Optional)

Mark content as ready for review:

```bash
contentforge status <content-id> --set review
```

If using AI, request a review:

```bash
contentforge review <content-id>
```

The AI agent checks for:

- Grammar and clarity
- Platform-specific best practices
- Hashtag suggestions
- Engagement optimization tips

## Stage 5: Schedule

Schedule content for publication at a specific time:

```bash
# Schedule for a specific time
contentforge schedule <content-id> --platform twitter --at "2026-03-20T09:00:00Z"

# Schedule for multiple platforms
contentforge schedule <content-id> --platform twitter --at "2026-03-20T09:00:00"
contentforge schedule <content-id> --platform linkedin --at "2026-03-20T09:30:00"

# View the schedule
contentforge schedule list
```

### Recurring Schedules

Set up recurring publication rules:

```bash
# Every Friday at 9 AM, publish from the "ready" queue
contentforge schedule recurring \
  --name "weekly-roundup" \
  --cron "0 9 * * FRI" \
  --platforms twitter,linkedin
```

### The Scheduling Engine

The scheduling engine runs as a background process (via `contentforge daemon` or as part of `contentforge serve`). It:

1. Polls the schedule table every 30 seconds (configurable)
2. Finds entries where `scheduled_at <= now` and `status = pending`
3. Publishes via the appropriate adapter
4. Updates the schedule entry status to `published` or `failed`
5. Retries failed publishes up to 3 times (configurable) with exponential backoff

## Stage 6: Publish

Publish immediately or let the scheduler handle it:

```bash
# Publish to a specific platform now
contentforge publish <content-id> --platform devto

# Publish to all adapted platforms
contentforge publish <content-id> --all
```

After publishing, a `Publication` record is created with:

- The live URL on the platform
- The platform-specific post ID
- The publication timestamp

## Stage 7: Track

After publication, ContentForge can pull engagement metrics:

```bash
# View analytics for a content piece
contentforge analytics <content-id>

# View analytics across all published content
contentforge analytics --summary
```

Metrics tracked (where supported by platform API):

- Views / Impressions
- Likes / Reactions
- Shares / Reposts
- Comments / Replies
- Link clicks

## Bulk Operations

```bash
# List all content in a specific status
contentforge list --status drafting

# Publish all ready content to all platforms
contentforge publish --status ready --all

# Archive all content older than 90 days
contentforge archive --older-than 90d
```

## Content Organization

### Tags

Tags help organize content by topic:

```bash
contentforge list --tag rust
contentforge list --tag error-handling
```

### Projects

Group content by project:

```bash
contentforge list --project contentforge
contentforge list --project codeilus
```
