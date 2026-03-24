# ContentForge — Launch Plan

> **Goal**: First paying user within 30 days
> **Date**: 2026-03-22
> **Status**: Product technically complete. Need sales infrastructure + distribution.

---

## Week 1: Test + Polish (Days 1-3)

### Day 1: Real API Testing
```bash
# DEV.to (already works — verify with real key)
export DEVTO_API_KEY=<real_key>
contentforge draft create "ContentForge: I built a content CLI in Rust" \
  --body "..." --tags "rust,opensource,devtools"
contentforge adapt <id> --platform devto
contentforge publish <id> --platform devto
# → Verify live on DEV.to

# Mastodon
contentforge platforms add mastodon --key 'https://mastodon.social|<token>'
contentforge adapt <id> --platform mastodon
contentforge publish <id> --platform mastodon
# → Verify live on Mastodon

# Bluesky
contentforge platforms add bluesky --key 'mbaneshi.bsky.social|<app_password>'
contentforge adapt <id> --platform bluesky
contentforge publish <id> --platform bluesky
# → Verify live on Bluesky
```

**Deliverable**: 3 platforms verified working with real APIs.

### Day 2: Build + Embed SvelteKit Frontend
```bash
cd frontend
npm install
npm run build
# Build output goes to frontend/build/
# rust-embed picks it up automatically

cd ..
cargo build --release
# Binary now serves the real web UI at localhost:3000
```

**Deliverable**: `contentforge serve` shows a working web dashboard.

### Day 3: MCP Integration Test
```bash
# Add ContentForge as MCP server in Claude Code
claude mcp add contentforge -- /Users/bm/contentforge/target/release/contentforge mcp

# Test from Claude Code:
# "Create a draft about Rust error handling and adapt it for DEV.to"
# → Claude calls draft_content + adapt_content via MCP
# → Verify content appears in `contentforge draft list`
```

**Deliverable**: Claude Code can drive ContentForge end-to-end via MCP.

---

## Week 1: Landing Page (Days 3-4)

### Option A: Static site on Cloudflare Pages (fastest)
- Single HTML page with Tailwind CDN
- Hosted at contentforge.dev (buy domain)
- Sections: hero, features, platforms, pricing, install, FAQ

### Option B: Use the docs site (free, already deployed)
- mbaneshi.github.io/contentforge already works
- Add pricing section + Stripe checkout link
- Custom domain later

### Content for Landing Page

```
Hero:
  "The developer's content engine"
  "Draft, adapt, schedule, and publish to every platform — from a single binary."
  [Install] [GitHub] [Docs]

Problem:
  "You write a blog post. Now copy-paste to Twitter, LinkedIn, DEV.to, Mastodon..."
  "ContentForge: write once, publish everywhere."

How it works (3 steps):
  1. contentforge draft create "My post" --body "..."
  2. contentforge adapt <id> --platform devto,mastodon,bluesky
  3. contentforge publish <id> --platform devto

Platforms:
  DEV.to ✅ | Mastodon ✅ | Bluesky ✅ | Twitter ✅ | LinkedIn ✅

Interfaces:
  CLI | TUI | Web UI | MCP (Claude Code)

Pricing:
  Free: Full CRUD, 6 adapters, CLI+TUI+Web+MCP, one-off scheduling
  Pro ($9/mo or $99/yr): Pipeline automation, cron scheduling, approval flows

Install:
  curl -fsSL https://raw.githubusercontent.com/.../install.sh | bash
  # or
  git clone ... && cargo build --release

CTA:
  [Get Started — Free] [Upgrade to Pro]
```

**Deliverable**: Landing page live at contentforge.dev or mbaneshi.github.io/contentforge.

---

## Week 1: Stripe + License Issuer (Day 5)

### Architecture
```
User clicks "Buy Pro" on landing page
  → Stripe Checkout ($9/mo or $99/yr)
  → Stripe webhook fires on payment success
  → Cloudflare Worker receives webhook
  → Worker signs license key with Ed25519 private key
  → Worker emails license key to user
  → User runs: contentforge license activate <KEY>
```

### Stripe Setup
1. Create Stripe account (if not exists)
2. Create Product: "ContentForge Pro"
3. Create Prices:
   - $9/month recurring
   - $99/year recurring (save $9)
4. Create Checkout Session link for landing page
5. Set up webhook endpoint

### License Issuer (Cloudflare Worker)
```javascript
// ~50 lines: receive Stripe webhook, sign license, email to customer
export default {
  async fetch(request, env) {
    const sig = request.headers.get('stripe-signature');
    const body = await request.text();

    // Verify Stripe signature
    // Extract customer email from event
    // Sign license payload with Ed25519 private key
    // Send license key via email (Resend API or similar)

    return new Response('OK');
  }
}
```

### Key Generation (one-time)
```bash
# Generate the signing keypair
# Private key → store in Cloudflare Worker secrets
# Public key → already embedded in contentforge binary (license.rs)
```

**Deliverable**: User can pay → receive license key → activate Pro features.

---

## Week 2: Distribution (Days 6-10)

### Day 6: MCP Registry Submissions
- [ ] Submit to official MCP Registry (registry.modelcontextprotocol.io)
- [ ] Submit to Gradually AI (gradually.ai/en/mcp-servers)
- [ ] Submit to APITracker (apitracker.io/mcp-servers)
- [ ] Submit to MCP Server Finder (mcpserverfinder.com)
- [ ] Submit to PulseMCP (pulsemcp.com)

Use `mcp-manifest.json` (already created) as the submission source.

### Day 7: Homebrew Tap
```bash
# Create the tap repo
gh repo create mbaneshi/homebrew-tap --public

# Create formula (after first GitHub release)
# The release workflow already handles updating the formula
```

### Day 8: First Release (v0.1.0)
```bash
# Trigger release-please
git commit --allow-empty -m "chore: release v0.1.0"
git push
# → release-please creates a PR
# → Merge PR → release workflow builds binaries for 4 targets
# → Homebrew formula updated automatically
```

### Day 9: Show HN Post
```
Title: Show HN: ContentForge – A Rust CLI for multi-platform content publishing

Body:
Hi HN,

I built ContentForge because I was tired of copy-pasting blog posts
to 5 different platforms. It's a single Rust binary that lets you:

- Write content in Markdown
- Adapt it per platform (DEV.to, Mastodon, Bluesky, Twitter, LinkedIn)
- Schedule and publish from CLI, TUI, web UI, or Claude Code via MCP

Key differentiators:
- Single binary: `brew install contentforge` (or curl installer)
- Zero infrastructure: SQLite, no Docker/PG/Redis
- MCP server: Claude Code can draft and publish for you
- Pipeline automation: adapt → review → approve → publish (Pro)

Free and open source (MIT). Pro tier ($9/mo) for automation.

GitHub: https://github.com/mbaneshi/contentforge
Install: curl -fsSL ... | bash

Happy to answer questions about the Rust architecture
(12 crates, Axum, ratatui, SvelteKit, rmcp).
```

**Timing**: Post Tuesday-Thursday, 8-10 AM EST for best HN engagement.

### Day 9: Reddit Posts (same day as HN)
- r/rust — "I built a multi-platform content publisher in Rust (12 crates, single binary)"
- r/selfhosted — "ContentForge: self-hosted, single-binary content scheduler (SQLite, no Docker)"
- r/programming — "Show r/programming: CLI tool for cross-platform content publishing"
- r/buildinpublic — "I built ContentForge to solve my own content distribution problem"

### Day 10: DEV.to Launch Article
```
Title: "I Built a Multi-Platform Content Publisher in Rust — Here's the Architecture"

Outline:
1. Problem: copy-pasting to 5 platforms
2. Solution: single Rust binary with 12 crates
3. Architecture diagram + crate map
4. How the pipeline engine works
5. MCP integration (Claude Code can publish for you)
6. Try it: installation + first content piece
7. What's next

Published VIA ContentForge itself (dogfooding).
```

---

## Week 2-3: Content Loop

### Ongoing Posts (use ContentForge to create and publish these)
| Post | Platform | When |
|------|----------|------|
| "I published this post using my own CLI tool" | DEV.to + X + LinkedIn | Day 10 |
| "How the pipeline engine works (Windmill patterns in 800 LOC)" | DEV.to + Substack | Day 12 |
| "6 platform APIs in 2026: what works, what's broken" | DEV.to + HN | Day 14 |
| "MCP as a growth lever for developer tools" | DEV.to + LinkedIn | Day 17 |
| Weekly ship update | X + LinkedIn | Every Friday |
| "From 0 to first paying user: building ContentForge in public" | Substack | Day 21 |

### Engagement Strategy
- Reply to every comment on HN/Reddit/DEV.to within 2 hours
- Thank every GitHub star with a follow
- Engage in r/buildinpublic with progress updates
- Share screenshots/GIFs of TUI and web UI

---

## Week 3-4: Iterate + Revenue

### Track Metrics
| Metric | Target (Day 30) |
|--------|----------------|
| GitHub stars | 100-500 |
| Homebrew installs | 50+ |
| MCP registry views | 500+ |
| DEV.to article views | 2,000+ |
| Pro conversions | 1-5 |
| Monthly revenue | $9-45 |

### Iterate Based on Feedback
- Fix bugs reported on GitHub issues
- Add most-requested platform (probably Threads or Reddit)
- Improve whatever users complain about
- Write follow-up content about what you learned

### Revenue Optimization
- If Pro conversions are low: consider lower price ($5/mo) or longer trial
- If free adoption is high: add more Pro-only features
- If MCP drives traffic: write more "AI-native tool" content
- If HN drives traffic: post more technical deep-dives

---

## Checklist Summary

### Before Launch (Week 1)
- [ ] Test DEV.to with real API key
- [ ] Test Mastodon with real token
- [ ] Test Bluesky with real app password
- [ ] Build and embed SvelteKit frontend
- [ ] Test MCP with Claude Code
- [ ] Landing page live
- [ ] Stripe checkout working
- [ ] License issuer deployed (Cloudflare Worker)
- [ ] Domain purchased (contentforge.dev)

### Launch Week (Week 2)
- [ ] Submit to 5 MCP registries
- [ ] Create Homebrew tap repo
- [ ] Cut v0.1.0 release (triggers binary builds)
- [ ] Post on Hacker News (Show HN)
- [ ] Post on Reddit (4 subreddits)
- [ ] Publish launch article on DEV.to
- [ ] Tweet thread from @mbaneshi_
- [ ] LinkedIn post
- [ ] Mastodon post (via ContentForge)
- [ ] Bluesky post (via ContentForge)

### Post-Launch (Week 3-4)
- [ ] Reply to all comments within 2 hours
- [ ] Fix reported bugs
- [ ] Weekly ship updates
- [ ] Follow-up articles
- [ ] Track metrics dashboard
- [ ] First paying user celebration post
