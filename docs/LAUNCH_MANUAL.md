# ContentForge — Launch Manual

> Step-by-step instructions to go from "built" to "live and selling."
> Estimated total time: 2-3 hours.

---

## Step 1: Get DEV.to API Key (2 min)

1. Open browser: https://dev.to/settings/extensions
2. Scroll down to "DEV Community API Keys"
3. Type a description: `contentforge`
4. Click **Generate API Key**
5. Copy the key (looks like: `dKz7...`)
6. Run in terminal:

```bash
export DEVTO_API_KEY="paste_your_key_here"
```

---

## Step 2: Get Mastodon Access Token (3 min)

1. Open your Mastodon instance (e.g., https://mastodon.social)
2. Log in
3. Go to: **Settings → Development → New Application**
4. Application name: `contentforge`
5. Scopes: check `write:statuses` (at minimum)
6. Click **Submit**
7. Click on the app name you just created
8. Copy **"Your access token"** (starts with something like: `Bx3z...`)
9. Note your instance URL (e.g., `https://mastodon.social`)
10. Run in terminal:

```bash
export MASTODON_INSTANCE="https://mastodon.social"
export MASTODON_TOKEN="paste_your_token_here"
```

---

## Step 3: Get Bluesky App Password (2 min)

1. Open browser: https://bsky.app
2. Log in
3. Click your avatar → **Settings**
4. Click **"App Passwords"** (left sidebar)
5. Click **"Add App Password"**
6. Name: `contentforge`
7. Click **Create App Password**
8. Copy the password (looks like: `xxxx-xxxx-xxxx-xxxx`)
9. Note your handle (e.g., `mbaneshi.bsky.social`)
10. Run in terminal:

```bash
export BLUESKY_HANDLE="mbaneshi.bsky.social"
export BLUESKY_APP_PASSWORD="paste_your_app_password_here"
```

---

## Step 4: Run the API Test (5 min)

```bash
cd /Users/bm/contentforge
./scripts/test-real-apis.sh
```

This will:
- Create a test draft
- Publish to each platform where you set the env vars
- Show `✅` or `❌` for each

To test one platform at a time:

```bash
./scripts/test-real-apis.sh devto
./scripts/test-real-apis.sh mastodon
./scripts/test-real-apis.sh bluesky
```

**Expected output:**
```
✅ DEV.to: SUCCESS
✅ Mastodon: SUCCESS
✅ Bluesky: SUCCESS
```

If any fail, check:
- Is the API key/token correct?
- Is the Mastodon instance URL correct (include `https://`)?
- For Bluesky, is the handle in `user.bsky.social` format?

**Clean up:** Delete the test posts from each platform manually after testing.

---

## Step 5: Preview Landing Page (1 min)

```bash
open /Users/bm/contentforge/landing/index.html
```

Check it looks good in your browser. To serve it locally with hot reload:

```bash
npx serve /Users/bm/contentforge/landing
# Opens at http://localhost:3000
```

---

## Step 6: Deploy Landing Page (5 min)

### Option A: Cloudflare Pages (recommended)

Via dashboard:
1. Open browser: https://dash.cloudflare.com
2. Log in
3. Go to: **Workers & Pages → Create → Pages**
4. Choose **"Direct Upload"**
5. Project name: `contentforge`
6. Drag and drop the file: `/Users/bm/contentforge/landing/index.html`
7. Click **Deploy**
8. You'll get a URL like: `contentforge.pages.dev`

Via CLI:
```bash
npx wrangler pages deploy /Users/bm/contentforge/landing --project-name contentforge
```

### Option B: GitHub Pages (free)

```bash
# Create a separate repo for the landing page
gh repo create mbaneshi/contentforge.dev --public
cd /tmp && git clone git@github.com:mbaneshi/contentforge.dev.git
cp /Users/bm/contentforge/landing/index.html /tmp/contentforge.dev/index.html
cd /tmp/contentforge.dev
git add . && git commit -m "Landing page" && git push
# Enable GitHub Pages: repo Settings → Pages → Source: main branch
```

---

## Step 7: Buy Domain (5 min, optional but recommended)

### Using Cloudflare Registrar:
1. Go to: https://dash.cloudflare.com → **Domain Registration**
2. Search for: `contentforge.dev`
3. Buy it (~$12/year for `.dev`)
4. Connect to Cloudflare Pages:
   - Go to **Pages → contentforge → Custom domains → Add**
   - Enter: `contentforge.dev`
   - Follow DNS setup instructions

### Alternative registrars:
- Namecheap: https://namecheap.com
- Porkbun: https://porkbun.com (cheapest `.dev` domains)
- Google Domains → Squarespace Domains

---

## Step 8: Set Up Stripe (15 min)

### 8.1: Create Account & Product

1. Open browser: https://dashboard.stripe.com
2. Sign up or log in
3. Go to: **Products → + Add Product**
   - Name: `ContentForge Pro`
   - Description: `Pipeline automation, cron scheduling, approval flows, encrypted credentials`

4. Add two prices:
   - Click **Add a price**:
     - Amount: `$9.00`
     - Billing period: `Monthly`
     - Click **Save**
   - Click **Add another price**:
     - Amount: `$99.00`
     - Billing period: `Yearly`
     - Click **Save**

### 8.2: Create Payment Link

1. Go to: **Payment Links → + Create payment link**
2. Select **"ContentForge Pro"** → choose the `$9/month` price
3. After payment: redirect to `https://contentforge.dev/thanks` (or your landing page)
4. Click **Create Link**
5. Copy the link (looks like: `https://buy.stripe.com/xxx`)

### 8.3: Update Landing Page

```bash
nvim /Users/bm/contentforge/landing/index.html
```

Search and replace:
```
Old: https://buy.stripe.com/contentforge-pro
New: https://buy.stripe.com/your_actual_link
```

Save and re-deploy:
```bash
npx wrangler pages deploy /Users/bm/contentforge/landing --project-name contentforge
```

---

## Step 9: Deploy License Worker (20 min)

### 9.1: Install Wrangler

```bash
npm install -g wrangler
wrangler login
```

### 9.2: Generate Ed25519 Keypair

```bash
node -e "
const crypto = require('crypto');
const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519');
const privBytes = privateKey.export({ type: 'pkcs8', format: 'der' }).slice(-32);
const pubBytes = publicKey.export({ type: 'spki', format: 'der' }).slice(-32);
console.log('=== PRIVATE KEY (store in Worker secret) ===');
console.log(Buffer.from(privBytes).toString('base64'));
console.log('');
console.log('=== PUBLIC KEY (embed in license.rs) ===');
console.log('let public_key_bytes: [u8; 32] = [');
console.log('    ' + Array.from(pubBytes).map(b => '0x' + b.toString(16).padStart(2, '0')).join(', '));
console.log('];');
"
```

**Save both outputs.** The private key goes to the Worker. The public key goes into the Rust binary.

### 9.3: Update Rust Binary with Real Public Key

```bash
nvim /Users/bm/contentforge/crates/contentforge-core/src/license.rs
```

Find the line:
```rust
let public_key_bytes: [u8; 32] = [
    0xd7, 0x5a, 0x98, ...
```

Replace the bytes with the PUBLIC KEY output from step 9.2.

Rebuild:
```bash
cd /Users/bm/contentforge
cargo build --release
```

### 9.4: Set Worker Secrets

```bash
cd /Users/bm/contentforge/infra/license-worker

# Paste the PRIVATE KEY from step 9.2 when prompted
wrangler secret put ED25519_PRIVATE_KEY
```

### 9.5: Set Up Stripe Webhook

1. Go to: Stripe Dashboard → **Developers → Webhooks → + Add endpoint**
2. Endpoint URL: `https://contentforge-license.<your-subdomain>.workers.dev/webhook`
3. Events to listen for: select `checkout.session.completed`
4. Click **Add endpoint**
5. Copy the **Signing secret** (starts with `whsec_...`)

```bash
# Paste the signing secret when prompted
wrangler secret put STRIPE_WEBHOOK_SECRET
```

### 9.6: Set Up Email (Resend)

1. Go to: https://resend.com → sign up
2. Get your API key from the dashboard
3. Add your domain (`contentforge.dev`) for email verification

```bash
# Paste your Resend API key when prompted
wrangler secret put RESEND_API_KEY
```

### 9.7: Deploy

```bash
wrangler deploy
```

### 9.8: Verify

```bash
# Health check
curl https://contentforge-license.<your-subdomain>.workers.dev/health
# Should return: {"status":"ok","service":"contentforge-license"}
```

Test with Stripe CLI (optional):
```bash
# Install Stripe CLI: brew install stripe/stripe-cli/stripe
stripe listen --forward-to https://contentforge-license.<your-subdomain>.workers.dev/webhook
stripe trigger checkout.session.completed
```

---

## Step 10: Submit to MCP Registries (30 min)

### Registry 1: Official MCP Registry
- URL: https://registry.modelcontextprotocol.io
- How: Submit a PR to https://github.com/modelcontextprotocol/servers
- Add an entry for `contentforge` pointing to our repo

### Registry 2: Gradually AI
- URL: https://gradually.ai/en/mcp-servers/
- How: Look for "Add Server" or contact form
- Submit with data from `mcp-manifest.json`

### Registry 3: APITracker
- URL: https://apitracker.io/mcp-servers
- How: Look for submission form

### Registry 4: MCP Server Finder
- URL: https://mcpserverfinder.com
- How: Look for "Submit" button

### Registry 5: PulseMCP
- URL: https://pulsemcp.com
- How: Look for submission process

### Submission Data (use for all registries)

```
Name:        contentforge
Description: Developer content pipeline — draft, adapt, schedule, and publish
             to DEV.to, Mastodon, Bluesky, Twitter, LinkedIn from your AI assistant.
Repository:  https://github.com/mbaneshi/contentforge
Transport:   stdio
Command:     contentforge mcp
License:     MIT
Categories:  social-media, content, publishing, developer-tools

Tools:
  - draft_content:    Create a new content draft
  - list_content:     List content by status
  - show_content:     Full content details
  - adapt_content:    Adapt for a platform
  - publish_content:  Publish to a platform
  - schedule_content: Schedule for later
  - pipeline_status:  Pipeline overview

Setup:
  claude mcp add contentforge -- contentforge mcp
```

---

## Step 11: Cut v0.1.0 Release (10 min)

```bash
cd /Users/bm/contentforge

# Trigger release-please by pushing
git commit --allow-empty -m "chore: prepare release v0.1.0"
git push
```

This triggers the release workflow which:
1. release-please creates a PR with CHANGELOG
2. **You merge the PR** (in GitHub)
3. Workflow auto-builds binaries for 4 targets:
   - `x86_64-unknown-linux-gnu`
   - `aarch64-unknown-linux-gnu`
   - `x86_64-apple-darwin` (Intel Mac)
   - `aarch64-apple-darwin` (Apple Silicon)
4. Uploads binaries + SHA256 checksums to GitHub Releases
5. Install script (`install.sh`) now works

Verify:
```bash
# After release is published:
gh release list
gh release view v0.1.0
```

---

## Step 12: Launch Posts (1 hour)

### 12.1: Hacker News

1. Open: https://news.ycombinator.com/submit
2. Title: `Show HN: ContentForge – A Rust CLI for multi-platform content publishing`
3. URL: `https://github.com/mbaneshi/contentforge`
4. After posting, add this comment:

```
Hi HN,

I built ContentForge because I was tired of copy-pasting blog posts
to 5 different platforms. It's a single Rust binary that lets you:

- Write content in Markdown
- Adapt it per platform (DEV.to, Mastodon, Bluesky, Twitter, LinkedIn)
- Schedule and publish from CLI, TUI, web UI, or Claude Code via MCP

Key differentiators vs Postiz/Buffer/Typefully:
- Single binary: brew install or curl installer (no Docker/PG/Redis)
- Zero infrastructure: SQLite, works offline
- MCP server: Claude Code can draft and publish for you
- Pipeline automation: adapt → review → approve → publish (Pro)

Free and open source (MIT). Pro tier ($9/mo) for automation.

GitHub: https://github.com/mbaneshi/contentforge

Happy to answer questions about the Rust architecture
(12 crates, Axum, ratatui, SvelteKit, rmcp).
```

**Best time to post**: Tuesday-Thursday, 8-10 AM EST.

### 12.2: Reddit (post to all 4 on same day)

**r/rust:**
- Title: `I built a multi-platform content publisher in Rust (12 crates, single binary)`
- Link: https://github.com/mbaneshi/contentforge

**r/selfhosted:**
- Title: `ContentForge: self-hosted, single-binary content scheduler (SQLite, no Docker)`
- Link: https://github.com/mbaneshi/contentforge

**r/programming:**
- Title: `Show r/programming: CLI tool for cross-platform content publishing`
- Link: https://github.com/mbaneshi/contentforge

**r/buildinpublic:**
- Title: `I built ContentForge to solve my own content distribution problem — here's the architecture`
- Self post with details

### 12.3: DEV.to (publish via ContentForge itself)

```bash
contentforge draft create "I Built a Multi-Platform Content Publisher in Rust" \
  --body "$(cat <<'ARTICLE'
## The Problem

I was tired of writing a blog post and then manually copy-pasting it to Twitter, LinkedIn, DEV.to, and Mastodon...

(write the full article)
ARTICLE
)" \
  --tags "rust,opensource,devtools,showdev"

contentforge adapt <id> --platform devto
contentforge publish <id> --platform devto
```

### 12.4: Social Posts (via ContentForge)

```bash
# Mastodon
contentforge draft create "I just launched ContentForge — a Rust CLI for cross-platform content publishing. Write once, publish to DEV.to, Mastodon, Bluesky, and more. Single binary, zero config. GitHub: https://github.com/mbaneshi/contentforge" --tags "rust,opensource"
contentforge adapt <id> --platform mastodon
contentforge publish <id> --platform mastodon

# Bluesky
contentforge adapt <id> --platform bluesky
contentforge publish <id> --platform bluesky
```

---

## Checklist

```
Phase 1: API Keys & Testing
  [ ] Step 1:  DEV.to API key exported
  [ ] Step 2:  Mastodon token exported
  [ ] Step 3:  Bluesky app password exported
  [ ] Step 4:  ./scripts/test-real-apis.sh passes (✅ all 3)

Phase 2: Landing Page
  [ ] Step 5:  Landing page previewed in browser
  [ ] Step 6:  Landing page deployed (Cloudflare Pages or GitHub Pages)
  [ ] Step 7:  Domain purchased (contentforge.dev) — optional

Phase 3: Revenue Infrastructure
  [ ] Step 8:  Stripe product created, payment link generated
  [ ] Step 8:  Landing page updated with real Stripe link
  [ ] Step 9:  Ed25519 keypair generated
  [ ] Step 9:  Public key embedded in license.rs, binary rebuilt
  [ ] Step 9:  Worker secrets set (private key, Stripe, Resend)
  [ ] Step 9:  License worker deployed and health-checked

Phase 4: Distribution
  [ ] Step 10: Submitted to 5 MCP registries
  [ ] Step 11: v0.1.0 release cut, binaries built

Phase 5: Launch
  [ ] Step 12: Show HN posted (Tue-Thu, 8-10 AM EST)
  [ ] Step 12: Reddit posted (4 subreddits)
  [ ] Step 12: DEV.to article published (via ContentForge)
  [ ] Step 12: Mastodon + Bluesky announcements (via ContentForge)
```

---

## After Launch

- Reply to every HN/Reddit comment within 2 hours
- Fix bugs reported on GitHub issues same day
- Post weekly ship updates every Friday (via ContentForge)
- Track: GitHub stars, Homebrew installs, Pro conversions
- Target: 1-5 Pro users ($9-45/mo) within 30 days
