#!/usr/bin/env bash
set -e

# ------------------------
# RIPTV Installer
# ------------------------

BINARY_NAME="riptv"
INSTALL_PATH="/usr/local/bin"
REPO_URL="https://github.com/ucmz851/riptv.git"

echo "🎬 RIPTV Installer Starting..."

# Check for required tools
command -v cargo >/dev/null 2>&1 || {
  echo "❌ Rust is not installed. Install it: https://rustup.rs/"
  exit 1
}
command -v git >/dev/null 2>&1 || {
  echo "❌ Git is not installed. Install it first."
  exit 1
}

# Create temporary directory for building
TMP_DIR=$(mktemp -d)
echo "🔄 Cloning RIPTV repository into $TMP_DIR..."
git clone "$REPO_URL" "$TMP_DIR"
cd "$TMP_DIR"

# Build release
echo "⚡ Building RIPTV (release)..."
cargo build --release

# Install binary
echo "📦 Installing binary to $INSTALL_PATH..."
sudo install -Dm755 "target/release/$BINARY_NAME" "$INSTALL_PATH/$BINARY_NAME"

# Cleanup
cd ~
rm -rf "$TMP_DIR"

echo "✅ RIPTV installed successfully!"
echo
echo "🎬 Run RIPTV with:"
echo "   $BINARY_NAME --playlist /path/to/playlist.m3u"
echo
echo "🦀 Enjoy blazing fast IPTV playback!"
