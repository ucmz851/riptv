#!/usr/bin/env bash

set -e

# ------------------------
# Installer for RIPTV
# ------------------------

BINARY_NAME="riptv"
INSTALL_PATH="/usr/local/bin"

# Check for required tools
command -v cargo >/dev/null 2>&1 || {
  echo "❌ Rust is not installed. Please install Rust: https://rustup.rs/"
  exit 1
}
command -v git >/dev/null 2>&1 || {
  echo "❌ Git is not installed. Please install Git."
  exit 1
}

# Clone latest repository if not already present
if [ ! -d "riptv" ]; then
  echo "🔄 Cloning RIPTV repository..."
  git clone https://github.com/ucmz851/riptv.git
fi

cd riptv

echo "⚡ Building RIPTV..."
cargo build --release

echo "📦 Installing binary to $INSTALL_PATH..."
sudo cp target/release/riptv "$INSTALL_PATH/"
sudo chmod +x "$INSTALL_PATH/$BINARY_NAME"

echo "✅ RIPTV installed successfully!"

# Usage message
echo
echo "🎬 To run RIPTV, use:"
echo "   riptv --playlist /path/to/playlist.m3u"
echo
