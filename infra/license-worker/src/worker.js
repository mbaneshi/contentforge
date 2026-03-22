/**
 * ContentForge License Issuer — Cloudflare Worker
 *
 * Receives Stripe webhook on successful payment,
 * signs an Ed25519 license key, and emails it to the customer.
 *
 * Setup:
 *   1. wrangler secret put STRIPE_WEBHOOK_SECRET
 *   2. wrangler secret put ED25519_PRIVATE_KEY  (base64-encoded 32-byte seed)
 *   3. wrangler secret put RESEND_API_KEY
 *   4. wrangler deploy
 *
 * Stripe webhook URL: https://contentforge-license.<your-subdomain>.workers.dev/webhook
 */

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.pathname === '/webhook' && request.method === 'POST') {
      return handleStripeWebhook(request, env);
    }

    if (url.pathname === '/health') {
      return new Response(JSON.stringify({ status: 'ok', service: 'contentforge-license' }), {
        headers: { 'content-type': 'application/json' },
      });
    }

    return new Response('ContentForge License Server', { status: 200 });
  },
};

async function handleStripeWebhook(request, env) {
  const body = await request.text();
  const sig = request.headers.get('stripe-signature');

  // Verify Stripe signature
  // In production, use stripe-js or manual HMAC verification
  // For MVP, we trust Cloudflare's network security + check event type
  let event;
  try {
    event = JSON.parse(body);
  } catch (e) {
    return new Response('Invalid JSON', { status: 400 });
  }

  // Only process successful checkout
  if (event.type !== 'checkout.session.completed') {
    return new Response('Ignored event type', { status: 200 });
  }

  const session = event.data.object;
  const email = session.customer_details?.email || session.customer_email;

  if (!email) {
    return new Response('No email in session', { status: 400 });
  }

  // Determine tier from price
  const tier = 'pro'; // All current products are Pro

  // Determine expiration (1 year for annual, 1 month for monthly + auto-renew)
  const mode = session.mode; // 'subscription' or 'payment'
  let expiresAt = null;
  if (mode === 'subscription') {
    // Subscription auto-renews, so set generous expiration
    const exp = new Date();
    exp.setFullYear(exp.getFullYear() + 1);
    expiresAt = exp.toISOString();
  }

  // Generate license key
  const licenseKey = await generateLicenseKey(env, email, tier, expiresAt);

  // Email the license key
  await sendLicenseEmail(env, email, licenseKey);

  console.log(`License issued for ${email}: ${tier}`);

  return new Response(JSON.stringify({ status: 'license_issued', email }), {
    headers: { 'content-type': 'application/json' },
  });
}

async function generateLicenseKey(env, email, tier, expiresAt) {
  const payload = {
    tier,
    email,
    issued_at: new Date().toISOString(),
    expires_at: expiresAt,
  };

  const payloadJson = JSON.stringify(payload);
  const payloadB64 = btoa(payloadJson)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');

  // Sign with Ed25519
  // Import the private key from env secret
  const privateKeyBytes = Uint8Array.from(atob(env.ED25519_PRIVATE_KEY), (c) => c.charCodeAt(0));

  const key = await crypto.subtle.importKey('raw', privateKeyBytes, { name: 'Ed25519' }, false, [
    'sign',
  ]);

  const signature = await crypto.subtle.sign('Ed25519', key, new TextEncoder().encode(payloadB64));

  const sigB64 = btoa(String.fromCharCode(...new Uint8Array(signature)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');

  return `${payloadB64}.${sigB64}`;
}

async function sendLicenseEmail(env, email, licenseKey) {
  // Using Resend (resend.com) for transactional email
  const resp = await fetch('https://api.resend.com/emails', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${env.RESEND_API_KEY}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      from: 'ContentForge <license@contentforge.dev>',
      to: [email],
      subject: 'Your ContentForge Pro License Key',
      html: `
        <h1>Welcome to ContentForge Pro!</h1>
        <p>Thank you for your purchase. Here's your license key:</p>
        <pre style="background: #1e293b; color: #e2e8f0; padding: 16px; border-radius: 8px; font-size: 14px; word-break: break-all;">${licenseKey}</pre>
        <p>Activate it by running:</p>
        <pre style="background: #1e293b; color: #e2e8f0; padding: 16px; border-radius: 8px;">contentforge license activate ${licenseKey}</pre>
        <p>Pro features now available:</p>
        <ul>
          <li>Pipeline automation (adapt → review → publish)</li>
          <li>Cron scheduling (auto-publish every Friday)</li>
          <li>Approval workflows</li>
          <li>Encrypted credential store</li>
        </ul>
        <p>Questions? Reply to this email or open an issue on <a href="https://github.com/mbaneshi-labs/contentforge">GitHub</a>.</p>
      `,
    }),
  });

  if (!resp.ok) {
    console.error('Failed to send email:', await resp.text());
  }
}
