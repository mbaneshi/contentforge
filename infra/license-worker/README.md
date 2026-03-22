# ContentForge License Issuer

Cloudflare Worker that receives Stripe webhooks and issues Ed25519-signed license keys.

## Setup

### 1. Prerequisites
- Cloudflare account with Workers enabled
- Stripe account with a product and price created
- Resend account for transactional email (resend.com)
- Domain: contentforge.dev (for email sender)

### 2. Generate Ed25519 Keypair

```bash
# Generate keypair (run once, save the output)
node -e "
const crypto = require('crypto');
const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519');
const privBytes = privateKey.export({ type: 'pkcs8', format: 'der' }).slice(-32);
const pubBytes = publicKey.export({ type: 'spki', format: 'der' }).slice(-32);
console.log('Private key (base64, store in Worker secret):');
console.log(Buffer.from(privBytes).toString('base64'));
console.log('');
console.log('Public key (hex, embed in contentforge-core/src/license.rs):');
console.log('let public_key_bytes: [u8; 32] = [');
console.log(Array.from(pubBytes).map(b => '0x' + b.toString(16).padStart(2, '0')).join(', '));
console.log('];');
"
```

### 3. Update the Rust Binary

Replace the placeholder public key in `/Users/bm/contentforge/crates/contentforge-core/src/license.rs`
with the actual public key bytes from step 2.

### 4. Deploy

```bash
cd infra/license-worker

# Set secrets
wrangler secret put STRIPE_WEBHOOK_SECRET
wrangler secret put ED25519_PRIVATE_KEY
wrangler secret put RESEND_API_KEY

# Deploy
wrangler deploy
```

### 5. Configure Stripe Webhook

In Stripe Dashboard → Developers → Webhooks → Add endpoint:
- URL: `https://contentforge-license.<your-subdomain>.workers.dev/webhook`
- Events: `checkout.session.completed`

### 6. Create Stripe Products

```
Product: ContentForge Pro
  Price 1: $9/month (recurring)
  Price 2: $99/year (recurring, save $9)
```

Create a Checkout link for the landing page.

## Architecture

```
User clicks "Buy Pro" on contentforge.dev
  → Stripe Checkout
  → Payment succeeds
  → Stripe webhook → this Worker
  → Worker signs license key with Ed25519 private key
  → Worker emails key via Resend
  → User runs: contentforge license activate <KEY>
  → Binary verifies with embedded public key (offline, no phone-home)
```

## Testing

```bash
# Health check
curl https://contentforge-license.<subdomain>.workers.dev/health

# Simulate webhook (for testing — use Stripe CLI in dev)
stripe trigger checkout.session.completed \
  --add checkout_session:customer_details[email]=test@example.com
```
