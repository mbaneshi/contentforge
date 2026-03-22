#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# ContentForge — Real API Testing Script
# =============================================================================
#
# Before running, set your API keys:
#
#   export DEVTO_API_KEY="your_devto_api_key"
#   export MASTODON_INSTANCE="https://mastodon.social"
#   export MASTODON_TOKEN="your_mastodon_access_token"
#   export BLUESKY_HANDLE="yourhandle.bsky.social"
#   export BLUESKY_APP_PASSWORD="your_app_password"
#
# How to get keys:
#   DEV.to:    https://dev.to/settings/extensions → Generate API Key
#   Mastodon:  Settings → Development → New Application → copy access token
#   Bluesky:   bsky.app → Settings → App Passwords → Add App Password
#
# Usage:
#   ./scripts/test-real-apis.sh          # Test all configured platforms
#   ./scripts/test-real-apis.sh devto    # Test DEV.to only
#   ./scripts/test-real-apis.sh mastodon # Test Mastodon only
#   ./scripts/test-real-apis.sh bluesky  # Test Bluesky only
# =============================================================================

CF="${CF_BIN:-./target/release/contentforge}"
DB="${CF_DB:-/tmp/contentforge-api-test.db}"
PLATFORM="${1:-all}"

echo "========================================="
echo "  ContentForge — Real API Test"
echo "  $(date +%Y-%m-%d\ %H:%M)"
echo "========================================="
echo ""

# Clean test DB
rm -f "$DB"

# Create test content
echo "Creating test draft..."
OUTPUT=$($CF --db "$DB" draft create "ContentForge API Test — $(date +%H:%M:%S)" \
  --body "This is an automated test post from ContentForge.

ContentForge is a Rust-native content creation and multi-platform publishing platform.

- Single binary, zero config
- CLI + TUI + Web + MCP
- 6 platform adapters
- Pipeline automation

GitHub: https://github.com/mbaneshi-labs/contentforge

(This post was published via ContentForge CLI — testing in progress, will be deleted shortly.)" \
  --tags "test,contentforge,rust" \
  --project "contentforge" 2>&1)

echo "$OUTPUT"
ID=$(echo "$OUTPUT" | grep "Full ID:" | awk '{print $3}' | head -c 8)

if [ -z "$ID" ]; then
    echo "ERROR: Failed to create draft"
    exit 1
fi

echo ""
echo "Draft ID: $ID"
echo ""

# --- DEV.to ---
test_devto() {
    if [ -z "${DEVTO_API_KEY:-}" ]; then
        echo "⏭  DEV.to: DEVTO_API_KEY not set, skipping"
        return
    fi

    echo "--- Testing DEV.to ---"
    $CF --db "$DB" platforms add devto --key "$DEVTO_API_KEY" --name "DEV.to Test"
    $CF --db "$DB" adapt "$ID" --platform devto
    echo "Publishing to DEV.to..."
    if $CF --db "$DB" publish "$ID" --platform devto 2>&1; then
        echo "✅ DEV.to: SUCCESS"
    else
        echo "❌ DEV.to: FAILED"
    fi
    echo ""
}

# --- Mastodon ---
test_mastodon() {
    if [ -z "${MASTODON_TOKEN:-}" ] || [ -z "${MASTODON_INSTANCE:-}" ]; then
        echo "⏭  Mastodon: MASTODON_INSTANCE or MASTODON_TOKEN not set, skipping"
        return
    fi

    echo "--- Testing Mastodon ---"
    $CF --db "$DB" platforms add mastodon --key "${MASTODON_INSTANCE}|${MASTODON_TOKEN}" --name "Mastodon Test"
    $CF --db "$DB" adapt "$ID" --platform mastodon
    echo "Publishing to Mastodon..."
    if $CF --db "$DB" publish "$ID" --platform mastodon 2>&1; then
        echo "✅ Mastodon: SUCCESS"
    else
        echo "❌ Mastodon: FAILED"
    fi
    echo ""
}

# --- Bluesky ---
test_bluesky() {
    if [ -z "${BLUESKY_HANDLE:-}" ] || [ -z "${BLUESKY_APP_PASSWORD:-}" ]; then
        echo "⏭  Bluesky: BLUESKY_HANDLE or BLUESKY_APP_PASSWORD not set, skipping"
        return
    fi

    echo "--- Testing Bluesky ---"
    $CF --db "$DB" platforms add bluesky --key "${BLUESKY_HANDLE}|${BLUESKY_APP_PASSWORD}" --name "Bluesky Test"
    $CF --db "$DB" adapt "$ID" --platform bluesky
    echo "Publishing to Bluesky..."
    if $CF --db "$DB" publish "$ID" --platform bluesky 2>&1; then
        echo "✅ Bluesky: SUCCESS"
    else
        echo "❌ Bluesky: FAILED"
    fi
    echo ""
}

# Run tests
case "$PLATFORM" in
    devto)    test_devto ;;
    mastodon) test_mastodon ;;
    bluesky)  test_bluesky ;;
    all)
        test_devto
        test_mastodon
        test_bluesky
        ;;
    *)
        echo "Unknown platform: $PLATFORM"
        echo "Usage: $0 [devto|mastodon|bluesky|all]"
        exit 1
        ;;
esac

# Summary
echo "========================================="
echo "  Test complete"
echo "========================================="
$CF --db "$DB" status
echo ""
echo "To clean up test posts, delete them from each platform manually."
echo "Test DB at: $DB"
