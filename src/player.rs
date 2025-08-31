use anyhow::{Context, Result};
use colored::*;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::playlist::{Channel, PlaylistParser};
use crate::ui::ChannelSelector;
use crate::utils::format_duration;

pub struct IptvPlayer {
    parser: PlaylistParser,
    player_cmd: String,
    config: Config,
    history: Vec<String>,
    favorites: Vec<String>,
    last_played: Option<Instant>,
    current_player_process: Option<Child>,
}

impl IptvPlayer {
    pub fn new(player_cmd: String, config: Config, parallel: bool) -> Self {
        Self {
            parser: PlaylistParser::new(parallel),
            player_cmd,
            config,
            history: Vec::new(),
            favorites: Vec::new(),
            last_played: None,
            current_player_process: None,
        }
    }

    pub async fn load_playlist(&mut self, path: &str) -> Result<()> {
        self.parser.parse_file(path).await
            .with_context(|| format!("Failed to load playlist: {}", path))?;

        let channels = self.parser.get_channels();
        if channels.is_empty() {
            warn!("⚠️ No channels found in playlist");
        } else {
            info!("✅ Successfully loaded {} channels", channels.len().to_string().bright_green().bold());
        }

        Ok(())
    }

    pub async fn list_playlists(&self) -> Result<()> {
        println!("{}", "📋 Available Playlists:".bright_cyan().bold());
        
        let common_paths = [".", "~/Downloads", "~/Documents", "/tmp"];

        for path in &common_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "m3u" || ext == "m3u8" {
                            println!("  📺 {}", entry.path().display().to_string().bright_white());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn show_statistics(&self) {
        let stats = self.parser.get_statistics();
        
        println!("{}", "📊 Playlist Statistics".bright_cyan().bold());
        println!("{}", "═".repeat(50).bright_blue());
        
        println!("🎯 Total Channels: {}", stats.total_channels.to_string().bright_green().bold());
        println!("📁 Total Groups: {}", stats.total_groups.to_string().bright_yellow().bold());
        
        if !stats.channels_per_group.is_empty() {
            println!("\n{}", "📋 Top Groups:".bright_magenta());
            let mut groups: Vec<_> = stats.channels_per_group.iter().collect();
            groups.sort_by(|a, b| b.1.cmp(a.1));
            for (group, count) in groups.iter().take(10) {
                println!("  📺 {} ({} channels)", group.bright_white(), count.to_string().bright_green());
            }
        }

        if !stats.countries.is_empty() {
            println!("\n{}", "🌍 Countries:".bright_blue());
            let mut countries: Vec<_> = stats.countries.iter().collect();
            countries.sort_by(|a, b| b.1.cmp(a.1));
            for (country, count) in countries.iter().take(10) {
                println!("  🏳️ {} ({} channels)", country.bright_white(), count.to_string().bright_green());
            }
        }

        if !stats.languages.is_empty() {
            println!("\n{}", "🗣️ Languages:".bright_cyan());
            let mut languages: Vec<_> = stats.languages.iter().collect();
            languages.sort_by(|a, b| b.1.cmp(a.1));
            for (language, count) in languages.iter().take(10) {
                println!("  🔤 {} ({} channels)", language.bright_white(), count.to_string().bright_green());
            }
        }
    }

    pub async fn search_channels(&self, query: &str) -> Result<()> {
        info!("🔍 Searching for: '{}'", query.bright_yellow());
        let results = self.parser.search_channels(query);
        
        if results.is_empty() {
            println!("{}", "❌ No channels found matching your search.".bright_red());
            return Ok(());
        }

        println!("{}", format!("🎯 Found {} matching channels:", results.len()).bright_green().bold());
        println!("{}", "─".repeat(60).bright_blue());

        for (i, channel) in results.iter().enumerate().take(20) {
            let index = format!("{:2}", i + 1).bright_blue();
            let name = channel.display_name();
            println!("{}. {}", index, name);
        }

        if results.len() > 20 {
            println!("{}", format!("... and {} more channels", results.len() - 20).bright_yellow());
        }

        Ok(())
    }

    pub async fn run_interactive(&mut self) -> Result<()> {
        let channels = self.parser.get_channels().to_vec();
        if channels.is_empty() {
            error!("No channels available for playback");
            return Ok(());
        }

        info!("🚀 Starting interactive mode with {} channels", channels.len());
        let mut selector = ChannelSelector::new(channels, &self.config);

        loop {
            match selector.select_channel().await? {
                Some(channel) => {
                    self.add_to_history(&channel.name);

                    if let Err(e) = self.play_channel(&channel).await {
                        error!("Failed to play channel '{}': {}", channel.name, e);
                        println!("{}", format!("❌ Error playing channel: {}", e).bright_red());
                        println!("{}", "Press any key to continue...".bright_yellow());
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).ok();
                    }

                    println!("{}", "🔄 Returning to channel selection...".bright_cyan());
                }
                None => {
                    println!("{}", "👋 Thanks for using RIPTV!".bright_magenta().bold());
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn run_interactive_with_shutdown(&mut self, running: Arc<AtomicBool>) -> Result<()> {
        let channels = self.parser.get_channels().to_vec();
        if channels.is_empty() {
            error!("No channels available for playback");
            return Ok(());
        }

        info!("🚀 Starting interactive mode with {} channels", channels.len());
        let mut selector = ChannelSelector::new(channels, &self.config);

        loop {
            if !running.load(Ordering::Relaxed) {
                debug!("Shutdown requested, exiting interactive mode");
                break;
            }

            match selector.select_channel().await? {
                Some(channel) => {
                    self.add_to_history(&channel.name);
                    if let Err(e) = self.play_channel(&channel).await {
                        error!("Failed to play channel '{}': {}", channel.name, e);
                        println!("{}", format!("❌ Error playing channel: {}", e).bright_red());
                        println!("{}", "Press any key to continue...".bright_yellow());
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).ok();
                    }

                    println!("{}", "🔄 Returning to channel selection...".bright_cyan());
                }
                None => {
                    println!("{}", "👋 Thanks for using RIPTV!".bright_magenta().bold());
                    break;
                }
            }
        }

        self.cleanup().await?;
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        debug!("Performing player cleanup");

        if let Some(mut child) = self.current_player_process.take() {
            debug!("Terminating media player process");
            let _ = child.kill();
            let _ = child.wait();
        }

        debug!("Player cleanup completed");
        Ok(())
    }

    async fn play_channel(&mut self, channel: &Channel) -> Result<()> {
        info!("🎬 Playing: {}", channel.name.bright_green().bold());

        if let Some(group) = &channel.group {
            info!("📁 Group: {}", group.bright_blue());
        }

        self.validate_player()?;
        let start_time = Instant::now();
        self.last_played = Some(start_time);

        let mut cmd = Command::new(&self.player_cmd);
        cmd.arg(&channel.url);

        // Optimized player arguments
        cmd.args(&[
            "--cache=yes",
            "--demuxer-max-bytes=100M",
            "--demuxer-readahead-secs=30",
            "--force-window=immediate",
            "--no-terminal",
            "--quiet",
            "--really-quiet",
            "--hwdec=auto-safe",
            "--vo=gpu",
            "--gpu-context=auto",
            "--profile=fast",
            "--network-timeout=10",
            "--stream-buffer-size=1024k",
            "--demuxer-thread=yes",
        ]);

        if let Some(extra_args) = &self.config.player_args {
            for arg in extra_args {
                cmd.arg(arg);
            }
        }

        #[cfg(unix)]
        {
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        }

        debug!("Executing: {} {}", self.player_cmd, channel.url);
        let child = cmd.spawn()
            .with_context(|| format!("Failed to start media player: {}", self.player_cmd))?;
        self.current_player_process = Some(child);

        println!("{}", "🎥 Player started. Controls:".bright_cyan());
        println!("   {} Quit player", "q".bright_white().bold());
        println!("   {} Toggle fullscreen", "f".bright_white().bold());
        println!("   {} Volume up/down", "9/0".bright_white().bold());
        println!("   {} Seek backward/forward", "←/→".bright_white().bold());

        // Wait for process to finish
        if let Some(ref mut process) = self.current_player_process {
            let status = process.wait().with_context(|| "Failed to wait for media player")?;
            self.current_player_process = None;

            let duration = start_time.elapsed();
            if status.success() {
                info!("✅ Playback finished (duration: {})", format_duration(duration));
            } else {
                warn!("⚠️ Player exited with error code: {:?}", status.code());
            }
        }

        Ok(())
    }

    fn validate_player(&self) -> Result<()> {
        let output = Command::new("which").arg(&self.player_cmd).output();
        match output {
            Ok(o) if o.status.success() => Ok(debug!("Player found: {}", self.player_cmd)),
            _ => {
                let output = Command::new("where").arg(&self.player_cmd).output();
                match output {
                    Ok(o) if o.status.success() => Ok(debug!("Player found: {}", self.player_cmd)),
                    _ => anyhow::bail!(
                        "Media player '{}' not found. Please install {} or specify a different player with --player",
                        self.player_cmd,
                        self.player_cmd
                    ),
                }
            }
        }
    }

    fn add_to_history(&mut self, channel_name: &str) {
        self.history.retain(|name| name != channel_name);
        self.history.insert(0, channel_name.to_string());
        if self.history.len() > 50 {
            self.history.truncate(50);
        }
    }

    pub fn get_history(&self) -> &[String] { &self.history }
    pub fn get_favorites(&self) -> &[String] { &self.favorites }

    pub fn add_favorite(&mut self, channel_name: &str) {
        if !self.favorites.contains(&channel_name.to_string()) {
            self.favorites.push(channel_name.to_string());
        }
    }

    pub fn remove_favorite(&mut self, channel_name: &str) {
        self.favorites.retain(|name| name != channel_name);
    }
}

impl Drop for IptvPlayer {
    fn drop(&mut self) {
        debug!("IptvPlayer being dropped, performing emergency cleanup");
        if let Some(mut child) = self.current_player_process.take() {
            let _ = child.kill();
        }
    }
}
