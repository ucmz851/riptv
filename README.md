````markdown
# 🦀 RIPTV - Blazing Fast IPTV Player

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/ucmz851/riptv)

⚡ **RIPTV** is a lightning-fast IPTV player written in Rust, designed to handle large playlists with instant search and blazing performance.

> **Currently supported on Linux only.**  
> macOS and Windows support are **under development**.

---

## 🚀 Features

* ⚡ **Lightning Fast**: Parse large playlists in seconds
* 🔍 **Fuzzy Search**: Real-time filtering with intelligent matching
* 🎬 **Optimized Playback**: Tuned for streaming performance on Linux
* 📊 **Playlist Statistics**: Quick overview of channels & categories
* 💾 **History & Favorites**: Track recently played channels
* 🎨 **Modern Terminal UI**: Beautiful TUI interface
* 🔧 **Configurable**: Playlist, player, and UI options
* 🦀 **Memory Safe**: 100% Rust reliability

---

## 📸 Screenshot

<p align="center">
  <img src="docs/screenshot.png" alt="RIPTV Screenshot" width="800">
</p>

> 🎨 *RIPTV TUI in action: blazing-fast search & playback right from your terminal.*

---

## 📦 Installation (Linux)

### ⚡ One-Liner Install (Recommended)

Just copy-paste this into your terminal:

```bash
curl -sSL https://raw.githubusercontent.com/ucmz851/riptv/main/install.sh | bash
````

This will:
✅ Download the repo
✅ Build with Cargo (Rust)
✅ Install `riptv` into `/usr/local/bin`

> You may need to enter your **sudo password** during installation.

---

### 🛠️ Manual Build (Alternative)

If you prefer manual installation:

```bash
# Install prerequisites
sudo apt install mpv git curl -y   # Debian/Ubuntu
# Make sure Rust is installed: https://rustup.rs/

# Clone and build
git clone https://github.com/ucmz851/riptv.git
cd riptv
cargo build --release

# Run directly
./target/release/riptv --playlist your_playlist.m3u
```

---

## 🎯 Quick Start

```bash
# Play a playlist
riptv --playlist playlist.m3u

# Search for a channel
riptv --playlist playlist.m3u --search "BBC"

# Show playlist statistics
riptv --playlist playlist.m3u --stats

# Use a different player
riptv --playlist playlist.m3u --player vlc
```

---

## ⚙️ Configuration

Config file: `~/.config/riptv/config.json`

Example:

```json
{
  "default_playlist": "/path/to/playlist.m3u",
  "player_command": "mpv",
  "player_args": ["--cache=yes", "--hwdec=auto-safe"],
  "parallel_processing": true,
  "max_search_results": 100,
  "fuzzy_search": true
}
```

---

## 🐛 Limitations

* Works only on **Linux** right now
* Windows/macOS builds are coming soon
* Advanced features (remote control, EPG, recording) in progress

---

## 🤝 Contributing

1. Fork the repo
2. Create a branch (`git checkout -b feature-name`)
3. Commit changes (`git commit -m "feat: add new feature"`)
4. Push to branch (`git push origin feature-name`)
5. Open a Pull Request

---

## 📜 License

MIT License. See [LICENSE](LICENSE) for details.

---

**Made with ❤️ and 🦀 Rust**
```
