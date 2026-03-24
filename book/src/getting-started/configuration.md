# Configuration

ContentForge uses a TOML configuration file and supports environment variables for sensitive values like API keys.

## Configuration File Location

ContentForge looks for its configuration file at:

| OS      | Path                                               |
|---------|----------------------------------------------------|
| macOS   | `~/.config/contentforge/config.toml`               |
| Linux   | `~/.config/contentforge/config.toml`               |
| Windows | `%APPDATA%\contentforge\config.toml`               |

You can also specify a custom path:

```bash
contentforge --config /path/to/config.toml <command>
```

## Configuration File Format

```toml
# General settings
[general]
# Default editor for content creation
editor = "nvim"
# Default output format (text, json)
output_format = "text"
# Database location (default: ~/.local/share/contentforge/contentforge.db)
database_path = "~/.local/share/contentforge/contentforge.db"

# DEV.to configuration
[platforms.devto]
enabled = true
api_key = "${DEVTO_API_KEY}"  # Reference environment variable

# Twitter/X configuration
[platforms.twitter]
enabled = true
bearer_token = "${TWITTER_BEARER_TOKEN}"

# LinkedIn configuration
[platforms.linkedin]
enabled = true
access_token = "${LINKEDIN_ACCESS_TOKEN}"
author_urn = "urn:li:person:YOUR_ID"

# Medium configuration
[platforms.medium]
enabled = true
integration_token = "${MEDIUM_TOKEN}"

# Scheduling
[schedule]
# Check interval in seconds
poll_interval = 30
# Maximum retry attempts for failed publishes
max_retries = 3
# Default timezone for schedule display
timezone = "America/New_York"

# AI agent settings
[agent]
# LLM provider (openai, anthropic, local)
provider = "anthropic"
# Model to use
model = "claude-sonnet-4-20250514"
# API key for the LLM provider
api_key = "${ANTHROPIC_API_KEY}"

# Web server settings
[server]
# Host to bind to
host = "127.0.0.1"
# Port number
port = 3000
# Enable CORS (useful for development)
cors = false
```

## Environment Variables

Sensitive values like API keys should be set as environment variables rather than stored in the config file. ContentForge supports `${VAR_NAME}` syntax in the config file to reference environment variables.

Set them in your shell profile:

```bash
# ~/.bashrc or ~/.zshrc
export DEVTO_API_KEY="your-devto-api-key"
export TWITTER_BEARER_TOKEN="your-twitter-bearer-token"
export LINKEDIN_ACCESS_TOKEN="your-linkedin-access-token"
export MEDIUM_TOKEN="your-medium-integration-token"
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

Alternatively, use a `.env` file in the ContentForge config directory:

```bash
# ~/.config/contentforge/.env
DEVTO_API_KEY=your-devto-api-key
TWITTER_BEARER_TOKEN=your-twitter-bearer-token
```

## Platform Credential Management

### Add a Platform via CLI

```bash
# DEV.to (API key)
contentforge platforms add devto --api-key YOUR_KEY

# Twitter (bearer token)
contentforge platforms add twitter --bearer-token YOUR_TOKEN

# LinkedIn (OAuth)
contentforge platforms add linkedin --access-token YOUR_TOKEN --author-urn "urn:li:person:ABC123"

# Medium (integration token)
contentforge platforms add medium --token YOUR_TOKEN
```

### List Configured Platforms

```bash
contentforge platforms list
```

### Check Platform Health

```bash
contentforge platforms health
```

This calls each platform's health check endpoint to verify credentials are valid.

### Remove a Platform

```bash
contentforge platforms remove twitter
```

## Database Location

By default, the SQLite database is stored at:

| OS      | Path                                                |
|---------|-----------------------------------------------------|
| macOS   | `~/.local/share/contentforge/contentforge.db`       |
| Linux   | `~/.local/share/contentforge/contentforge.db`       |
| Windows | `%LOCALAPPDATA%\contentforge\contentforge.db`       |

Override with the `database_path` setting in the config file or the `--db` flag:

```bash
contentforge --db /path/to/database.db list
```

## Logging

Control log verbosity with the `RUST_LOG` environment variable:

```bash
# Minimal logging
RUST_LOG=warn contentforge serve

# Debug logging for contentforge crates only
RUST_LOG=contentforge=debug contentforge serve

# Trace logging (very verbose)
RUST_LOG=trace contentforge serve
```
