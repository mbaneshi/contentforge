use serde::{Deserialize, Serialize};

/// Product tier — determines which features are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Free,
    Pro,
    Team,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "Free"),
            Self::Pro => write!(f, "Pro"),
            Self::Team => write!(f, "Team"),
        }
    }
}

/// A validated license.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub tier: Tier,
    pub email: String,
    pub issued_at: String,
    pub expires_at: Option<String>,
    /// The raw license key string.
    pub key: String,
}

impl License {
    /// Create a free-tier license (no key needed).
    pub fn free() -> Self {
        Self {
            tier: Tier::Free,
            email: String::new(),
            issued_at: String::new(),
            expires_at: None,
            key: String::new(),
        }
    }

    /// Validate a license key.
    ///
    /// License key format: base64-encoded JSON payload + "." + base64-encoded Ed25519 signature.
    /// The public key is embedded in the binary.
    ///
    /// For now, we use a simple format that can be validated offline:
    /// `CF-PRO-{email}-{issued_date}-{signature}`
    pub fn validate(key: &str) -> Self {
        // Try to decode the key
        if let Some(license) = Self::decode_key(key) {
            return license;
        }

        // Invalid key = free tier
        tracing::warn!("Invalid license key, defaulting to Free tier");
        Self::free()
    }

    fn decode_key(key: &str) -> Option<Self> {
        // Format: base64(json_payload).base64(signature)
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        if parts.len() != 2 {
            return None;
        }

        let payload_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[0])
                .ok()?;

        let payload: LicensePayload = serde_json::from_slice(&payload_bytes).ok()?;

        // Verify signature using embedded public key
        let signature_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[1])
                .ok()?;

        if signature_bytes.len() != 64 {
            return None;
        }

        let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes.try_into().ok()?);

        let public_key = Self::public_key()?;
        use ed25519_dalek::Verifier;
        public_key.verify(parts[0].as_bytes(), &signature).ok()?;

        // Check expiration
        if let Some(ref exp) = payload.expires_at {
            if let Ok(exp_dt) = chrono::DateTime::parse_from_rfc3339(exp) {
                if exp_dt < chrono::Utc::now() {
                    tracing::warn!("License expired on {exp}");
                    return None;
                }
            }
        }

        Some(License {
            tier: payload.tier,
            email: payload.email,
            issued_at: payload.issued_at,
            expires_at: payload.expires_at,
            key: key.to_string(),
        })
    }

    /// The embedded public key for license verification.
    /// This key is compiled into the binary — only we hold the private key.
    fn public_key() -> Option<ed25519_dalek::VerifyingKey> {
        // This is a placeholder key. In production, generate a keypair:
        //   let keypair = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        //   let public = keypair.verifying_key();
        // Embed the public key bytes here, keep private key in your Stripe webhook server.
        let public_key_bytes: [u8; 32] = [
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xe5, 0x6e, 0x53, 0x5d, 0xea, 0x0c,
            0x5c, 0x32, 0x93, 0x0a, 0x72, 0x7f, 0xeb, 0x84, 0x73, 0x3a, 0xf4, 0x28, 0xd7, 0xce,
            0x99, 0x13, 0x6f, 0x50,
        ];
        ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes).ok()
    }

    /// Check if a feature requires Pro tier.
    pub fn require_pro(&self, feature: &str) -> Result<(), String> {
        match self.tier {
            Tier::Pro | Tier::Team => Ok(()),
            Tier::Free => Err(format!(
                "'{feature}' requires ContentForge Pro ($9/mo or $99/year).\n\
                 Upgrade: https://contentforge.dev/pro\n\
                 Activate: contentforge license activate <KEY>"
            )),
        }
    }

    /// Check if a feature requires Team tier.
    pub fn require_team(&self, feature: &str) -> Result<(), String> {
        match self.tier {
            Tier::Team => Ok(()),
            _ => Err(format!(
                "'{feature}' requires ContentForge Team ($19/user/mo).\n\
                 Learn more: https://contentforge.dev/team"
            )),
        }
    }
}

/// Internal payload structure inside a license key.
#[derive(Debug, Serialize, Deserialize)]
struct LicensePayload {
    tier: Tier,
    email: String,
    issued_at: String,
    expires_at: Option<String>,
}

// License issuance (signing) is done by the licensing server, not the client binary.
// See docs/PRODUCT_STRATEGY.md for the Stripe webhook architecture.
