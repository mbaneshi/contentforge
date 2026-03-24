# Platform Setup Guide

This guide covers how to set up credentials and configure each supported publishing platform.

## DEV.to

**Status:** Ready | **Auth:** API Key | **Difficulty:** Easy

### Get Your API Key

1. Log in to [dev.to](https://dev.to)
2. Go to **Settings** > **Extensions** > **DEV Community API Keys**
3. Enter a description (e.g., "ContentForge") and click **Generate API Key**
4. Copy the key immediately (it will not be shown again)

### Configure

```bash
contentforge platforms add devto --api-key YOUR_API_KEY
```

### Capabilities

| Feature         | Supported |
|-----------------|-----------|
| Articles        | Yes       |
| Tags (max 4)    | Yes       |
| Series          | Yes       |
| Canonical URL   | Yes       |
| Images          | Yes       |
| Delete          | Yes       |

### Limitations

- Maximum 4 tags per article
- Tags must be lowercase, no spaces (use hyphens)
- Rate limit: 30 requests per 30 seconds
- Article body is Markdown

---

## Twitter/X

**Status:** Ready | **Auth:** OAuth 2.0 / Bearer Token | **Difficulty:** Medium

### Get API Access

1. Go to the [Twitter Developer Portal](https://developer.x.com/en/portal)
2. Create a project and app
3. Under **Authentication**, set up OAuth 2.0 (User context) with the `tweet.read` and `tweet.write` scopes
4. Generate a Bearer Token for app-only access, or complete the OAuth 2.0 flow for user context

> **Warning: Twitter API Tiers**
>
> - **Free tier:** 500 tweets/month, 1 app
> - **Basic ($100/month):** 10,000 tweets/month
> - **Pro ($5,000/month):** 300,000 tweets/month
>
> ContentForge tracks rate limits and will report `RateLimited` errors with retry-after times.

### Configure

```bash
contentforge platforms add twitter --bearer-token YOUR_BEARER_TOKEN
```

### Capabilities

| Feature            | Supported |
|--------------------|-----------|
| Single tweets      | Yes       |
| Threads (chains)   | Yes       |
| Media attachments  | Planned   |
| Delete             | Yes       |

### Limitations

- 280 characters per tweet
- Threads are published as reply chains (sequential API calls)
- Rate limit: 200 tweets per 15 minutes (user context)

---

## LinkedIn

**Status:** Ready | **Auth:** OAuth 2.0 | **Difficulty:** Medium

### Get API Access

1. Go to [LinkedIn Developer Portal](https://www.linkedin.com/developers/)
2. Create an app
3. Under **Products**, request access to **Share on LinkedIn** and **Sign In with LinkedIn using OpenID Connect**
4. Generate an OAuth 2.0 access token with the `w_member_social` scope

> **Note: Access Token Expiry**
>
> LinkedIn access tokens expire after 60 days. You will need to refresh them periodically. ContentForge will report `AuthFailed` when the token expires.

### Find Your Author URN

You need your LinkedIn person URN (e.g., `urn:li:person:ABC123`). To find it:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" https://api.linkedin.com/v2/userinfo
```

The `sub` field in the response is your person ID. Your URN is `urn:li:person:{sub}`.

### Configure

```bash
contentforge platforms add linkedin \
  --access-token YOUR_ACCESS_TOKEN \
  --author-urn "urn:li:person:YOUR_ID"
```

### Capabilities

| Feature         | Supported |
|-----------------|-----------|
| Text posts      | Yes       |
| Articles        | Planned   |
| Images          | Planned   |
| Delete          | Yes       |

### Limitations

- 3,000 character limit for post text
- Rich media requires additional API permissions
- LinkedIn API versioning: ContentForge uses the `202501` version header

---

## Medium

**Status:** Ready | **Auth:** Integration Token | **Difficulty:** Easy

### Get Your Integration Token

1. Log in to [Medium](https://medium.com)
2. Go to **Settings** > **Security and apps** > **Integration tokens**
3. Enter a description and generate a token
4. Copy the token

> **Warning: Medium API Status**
>
> The Medium API is officially deprecated but still functional for creating posts. There is no guarantee of continued availability. Medium does not support post deletion via API.

### Configure

```bash
contentforge platforms add medium --token YOUR_INTEGRATION_TOKEN
```

### Capabilities

| Feature         | Supported |
|-----------------|-----------|
| Articles        | Yes       |
| Tags (max 5)    | Yes       |
| Canonical URL   | Yes       |
| Markdown        | Yes       |
| Delete          | No (API limitation) |

### Limitations

- Maximum 5 tags per article
- No deletion support (Medium API limitation)
- API is deprecated but functional

---

## YouTube

**Status:** Planned | **Auth:** OAuth 2.0 | **Difficulty:** Medium

YouTube support is planned for Phase 3. It will allow updating video descriptions and metadata (not video uploads).

---

## Instagram

**Status:** Planned | **Auth:** Graph API | **Difficulty:** Hard

Instagram support is planned for Phase 3. It requires a Facebook Business account and Instagram Professional account, plus the Instagram Graph API.

---

## Reddit

**Status:** Planned | **Auth:** OAuth 2.0 | **Difficulty:** Medium

Reddit support is planned for Phase 3 for submitting self-posts and links to subreddits.

---

## Hacker News

**Status:** Planned | **Auth:** Cookie-based | **Difficulty:** Medium

Hacker News support is planned for Phase 3. Since HN has no official API for posting, this will use authenticated web requests.

---

## Substack

**Status:** Planned | **Auth:** Cookie-based | **Difficulty:** Fragile

Substack support is planned but marked as fragile. Substack has no public API; integration relies on reverse-engineered web endpoints that may break without notice.

---

## Checking Platform Health

After configuring platforms, verify they are working:

```bash
# Check all platforms
contentforge platforms health

# Check a specific platform
contentforge platforms health --platform devto
```

This calls each platform's health check endpoint (typically a "get current user" API call) to verify that credentials are valid and the account is accessible.
