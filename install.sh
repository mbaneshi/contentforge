#!/usr/bin/env bash
set -euo pipefail

# ContentForge installer
# Usage: curl -fsSL https://raw.githubusercontent.com/mbaneshi-labs/contentforge/main/install.sh | bash

REPO="mbaneshi-labs/contentforge"
BINARY="contentforge"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
            x86_64)        TARGET="x86_64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    Linux)
        case "$ARCH" in
            aarch64)       TARGET="aarch64-unknown-linux-gnu" ;;
            x86_64)        TARGET="x86_64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        echo "Try: cargo install contentforge"
        exit 1
        ;;
esac

# Get latest release tag
echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "No release found. Building from source instead..."
    echo ""
    echo "  git clone https://github.com/$REPO.git"
    echo "  cd contentforge && cargo build --release"
    echo "  cp target/release/contentforge $INSTALL_DIR/"
    exit 1
fi

echo "Installing ContentForge $LATEST for $TARGET..."

# Download
URL="https://github.com/$REPO/releases/download/$LATEST/${BINARY}-${TARGET}.tar.gz"
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

curl -fsSL "$URL" -o "$TMP_DIR/contentforge.tar.gz"
tar xzf "$TMP_DIR/contentforge.tar.gz" -C "$TMP_DIR"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
else
    echo "Need sudo to install to $INSTALL_DIR"
    sudo mv "$TMP_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
fi

chmod +x "$INSTALL_DIR/$BINARY"

echo ""
echo "ContentForge $LATEST installed to $INSTALL_DIR/$BINARY"
echo ""
echo "Get started:"
echo "  contentforge --help"
echo "  contentforge draft create \"My first post\" --body \"Hello world\""
echo ""
echo "Set up MCP for Claude Code:"
echo "  claude mcp add contentforge contentforge mcp"
