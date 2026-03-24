# AI Agents

ContentForge includes a built-in AI agent pipeline that can generate, adapt, split, and review content using LLMs. AI features are opt-in and the tool is fully functional without them.

## Overview

The AI agent pipeline provides four capabilities:

1. **Generate** -- Create a full draft from a one-line prompt
2. **Adapt** -- Intelligently adapt content for a specific platform
3. **Split** -- Break long content into thread parts
4. **Review** -- Check content quality and suggest improvements

## Configuration

Configure your LLM provider in `~/.config/contentforge/config.toml`:

```toml
[agent]
provider = "anthropic"        # openai, anthropic, or local
model = "claude-sonnet-4-20250514"  # model identifier
api_key = "${ANTHROPIC_API_KEY}"
```

Supported providers:

| Provider    | Models                          | Config Key             |
|-------------|---------------------------------|------------------------|
| Anthropic   | Claude Sonnet, Claude Haiku     | `ANTHROPIC_API_KEY`    |
| OpenAI      | GPT-4o, GPT-4o-mini             | `OPENAI_API_KEY`       |
| Local       | Ollama, llama.cpp               | (no API key needed)    |

## Generate Content

Create a full draft from a prompt:

```bash
contentforge generate "Write a technical blog post about Rust's async/await model, targeting intermediate Rust developers"
```

The agent:

1. Generates a structured outline
2. Writes the full article in Markdown
3. Creates a `Content` entity with status `drafting`
4. Returns the content ID for further editing

### Options

```bash
# Specify content type
contentforge generate "Tweet about our new release" --type short_post

# Specify target length
contentforge generate "Blog post about error handling" --length 1500

# Specify tags
contentforge generate "Rust tutorial" --tags rust,tutorial,beginners

# Specify a project
contentforge generate "Changelog for v0.2.0" --project contentforge
```

## Adapt Content

Generate intelligent platform-specific adaptations:

```bash
contentforge adapt <content-id> --platform twitter --ai
```

Unlike simple truncation, AI adaptation:

- Rewrites content in the platform's native style
- Preserves the core message and key points
- Respects character limits naturally (not by cutting off)
- Adds platform-appropriate elements (hashtags for Twitter, formatting for LinkedIn)
- Generates thread breakpoints that read naturally

### Example: Article to Tweet Thread

Given a 1,500-word article about Rust error handling, the AI adapter might produce:

```
Tweet 1/7:
Rust's error handling is one of its killer features. Here's why the
Result type makes your code more reliable than try/catch -- a thread.

Tweet 2/7:
The Result<T, E> type forces you to handle errors at compile time.
No more "I forgot to catch that exception" bugs in production.

...

Tweet 7/7:
TL;DR: Rust error handling is verbose but intentional. The compiler
catches mistakes before your users do.

If you found this useful, check out the full article: [link]
```

## Split into Threads

For content that needs to be broken into thread parts:

```bash
contentforge split <content-id> --platform twitter
```

The agent:

1. Analyzes the content structure
2. Identifies natural break points (paragraph boundaries, topic shifts)
3. Ensures each part is under the platform's character limit
4. Adds thread markers (1/N) and continuity phrases
5. Stores the result as `thread_parts` in the adaptation

## Review Content

Get AI feedback on your content before publishing:

```bash
contentforge review <content-id>
```

The review agent checks:

- **Clarity**: Is the message clear and well-structured?
- **Grammar**: Are there any grammatical or spelling issues?
- **Engagement**: Will this perform well on the target platforms?
- **Platform fit**: Does the adaptation match platform conventions?
- **Hashtags**: Suggests relevant hashtags (for Twitter, LinkedIn)
- **Timing**: Suggests optimal posting times based on content type

### Review Output

```
Review for "Rust Error Handling" (article)

Quality: 8/10

Suggestions:
- Consider adding a concrete code example in the introduction
- The section on `?` operator could be shorter for social media adaptation
- Suggested hashtags: #RustLang #ErrorHandling #Programming

Platform Notes:
- Twitter thread: Good length (7 tweets), consider a hook in tweet 1
- LinkedIn: Add a question at the end to drive comments
- DEV.to: Add a cover image for better visibility
```

## Agent Pipeline

For fully automated content creation, chain the agents:

```bash
# Generate, adapt for all platforms, and schedule
contentforge pipeline \
  --prompt "Write about our v0.2.0 release" \
  --platforms twitter,linkedin,devto \
  --schedule "2026-03-20T09:00:00Z"
```

This runs the full pipeline:

1. **Generate** -- Creates the canonical content
2. **Adapt** -- Generates adaptations for each platform
3. **Review** -- Checks quality and makes improvements
4. **Schedule** -- Queues for publication at the specified time

## Using with MCP

The AI agent capabilities are also available via MCP, so Claude Code or other AI assistants can orchestrate the full pipeline. See the [MCP Integration guide](mcp-integration.md) for details.

## Cost Awareness

AI operations make LLM API calls which cost money. ContentForge shows estimated costs before executing:

```bash
contentforge generate "Blog post about Rust" --estimate
# Estimated: ~2,000 tokens input, ~3,000 tokens output
# Estimated cost: $0.02 (Claude Sonnet)
```

Use `--confirm` to skip the confirmation prompt:

```bash
contentforge generate "Blog post about Rust" --confirm
```
