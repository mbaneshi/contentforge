# Quick Start

This tutorial walks you through creating, adapting, and publishing your first content piece with ContentForge. It takes about 5 minutes.

## Prerequisites

- ContentForge installed ([Installation guide](installation.md))
- A DEV.to API key (the easiest platform to start with)

## Step 1: Get a DEV.to API Key

1. Go to [dev.to/settings/extensions](https://dev.to/settings/extensions)
2. Under "DEV Community API Keys", generate a new key
3. Copy the key

## Step 2: Configure the Platform

```bash
contentforge platforms add devto --api-key YOUR_API_KEY
```

Verify the connection:

```bash
contentforge platforms health
```

Expected output:

```
Platform    Status    Account
--------    ------    -------
DEV.to      OK       your-username
```

## Step 3: Create Content

Create a new article:

```bash
contentforge new \
  --title "Getting Started with Rust Error Handling" \
  --type article \
  --tags rust,error-handling,beginners
```

This creates a content piece in the `idea` status and opens your default editor. Write your article in Markdown:

```markdown
Error handling in Rust is one of the language's strongest features.
Unlike exceptions in other languages, Rust makes error handling
explicit and type-safe through the `Result` and `Option` types.

## The Result Type

The `Result<T, E>` type is an enum with two variants:

- `Ok(T)` -- the operation succeeded with value `T`
- `Err(E)` -- the operation failed with error `E`

...
```

Save and close the editor. Note the content ID that is printed (e.g., `a1b2c3d4-...`).

## Step 4: Adapt for the Platform

Generate a DEV.to-specific adaptation:

```bash
contentforge adapt a1b2c3d4 --platform devto
```

This creates an adaptation that:

- Keeps the Markdown format (DEV.to supports Markdown natively)
- Limits tags to 4 (DEV.to maximum)
- Adds any canonical URL if configured

To preview the adaptation:

```bash
contentforge show a1b2c3d4 --adaptation devto
```

## Step 5: Publish

Publish to DEV.to:

```bash
contentforge publish a1b2c3d4 --platform devto
```

Output:

```
Published to DEV.to
URL: https://dev.to/yourusername/getting-started-with-rust-error-handling-abc
```

The content status automatically changes to `published`.

## Step 6: Check Publication Status

```bash
contentforge show a1b2c3d4
```

This shows the content details, all adaptations, and all publication records with live URLs.

## Next Steps

Now that you have published your first piece, try these:

- **Adapt for more platforms:** `contentforge adapt a1b2c3d4 --platform twitter` to generate a tweet thread
- **Schedule for later:** `contentforge schedule a1b2c3d4 --platform twitter --at "2026-03-20T09:00:00Z"`
- **Use AI:** `contentforge generate "Write a blog post about async Rust"` to have AI draft content for you
- **Launch the TUI:** `contentforge tui` for a full interactive dashboard
- **Start the web UI:** `contentforge serve` and open `http://localhost:3000`

See the [Content Workflow guide](../guide/content-workflow.md) for the full content lifecycle.
