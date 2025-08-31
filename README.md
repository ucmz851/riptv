# 🦀 RIPTV - Blazing Fast IPTV Player

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/riptv)

⚡ **RIPTV** is a lightning-fast IPTV player written in Rust, designed to handle large playlists with instant search and blazing performance.

> **Currently fully supported on Linux only.** macOS and Windows support are **under development**.

## 🚀 Features

* ⚡ **Lightning Fast**: Parse large playlists in seconds
* 🔍 **Fuzzy Search**: Real-time filtering with intelligent matching
* 🎬 **Optimized Playback**: Tuned for streaming performance on Linux
* 📊 **Playlist Statistics**: Basic stats available
* 💾 **History & Favorites**: Track recently played channels and save favorites
* 🎨 **Modern Terminal UI**: Beautiful TUI for Linux terminals
* 🔧 **Configurable**: Customize player, playlist, and UI
* 🦀 **Memory Safe**: Written in Rust for reliability

> **Note:** Some advanced features (cross-platform support, detailed stats, remote web interface) are under development.

## 📦 Installation

### Prerequisites

1. **Rust** (1.70 or later): [Install Rust](https://rustup.rs/)
2. **Media Player** (Linux only currently):

   * **mpv** (recommended): `sudo apt install mpv`
   * VLC or ffplay may work but not fully tested

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/riptv.git
cd riptv

# Build release version
cargo build --release

# Binary is located at:
./target/release/riptv
```

## 🎯 Quick Start (Linux Only)

```bash
# Play a playlist
./target/release/riptv --playlist your_playlist.m3u

# Search for a channel
./target/release/riptv --playlist playlist.m3u --search "BBC"

# Show playlist statistics
./target/release/riptv --playlist playlist.m3u --stats

# Use a different player (Linux)
./target/release/riptv --playlist playlist.m3u --player vlc
```

### Controls

* **Channel Selector:**

  * Type to search
  * ↑/↓ or Ctrl+K/J to navigate
  * Tab: toggle preview panel
  * Enter: play channel
  * Esc/Ctrl+C: quit

* **Media Player (mpv):**

  * q: quit
  * f: fullscreen
  * 9/0: volume down/up
  * ←/→: seek
  * Space: pause/resume

## ⚙️ Configuration

Config file is located at:

* **Linux:** `~/.config/riptv/config.json`
* **Windows/macOS:** Not supported yet

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

## 🐛 Known Limitations

* **Windows/macOS:** Not yet supported
* **Some advanced features:** Under development (remote control, EPG, recording)
* Fully functional and stable on **Linux only**

## 🤝 Contributing

1. Fork the repo
2. Create a branch (`git checkout -b feature-name`)
3. Commit changes (`git commit -m "Add feature"`)
4. Push to branch (`git push origin feature-name`)
5. Open a Pull Request

## 📜 License

MIT License. See [LICENSE](LICENSE) for details.

---

**Made with ❤️ and 🦀 Rust**

*Compile with `cargo build --release` for best performance on Linux.*

---

If you want, I can also rewrite the **Installation & Quick Start sections** to make it crystal clear **Linux only**, and remove all Windows/macOS examples, so users don’t get confused.

Do you want me to do that?
