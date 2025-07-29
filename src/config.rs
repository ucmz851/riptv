use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default playlist file path
    pub default_playlist: Option<String>,
    
    /// Media player command
    pub player_command: String,
    
    /// Additional arguments for the media player
    pub player_args: Option<Vec<String>>,
    
    /// Enable parallel processing for large playlists
    pub parallel_processing: bool,
    
    /// Maximum number of channels to show in search results
    pub max_search_results: usize,
    
    /// Enable fuzzy matching in search
    pub fuzzy_search: bool,
    
    /// UI preferences
    pub ui: UiConfig,
    
    /// Network settings
    pub network: NetworkConfig,
    
    /// Recently played channels (for quick access)
    pub recent_channels: Vec<String>,
    
    /// Favorite channels
    pub favorite_channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color scheme for the interface
    pub color_scheme: String,
    
    /// Show channel preview by default
    pub show_preview: bool,
    
    /// Preview window size (percentage)
    pub preview_size: String,
    
    /// Number of channels to display per page
    pub page_size: usize,
    
    /// Show group information in channel list
    pub show_groups: bool,
    
    /// Custom key bindings
    pub key_bindings: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Connection timeout in seconds
    pub timeout: u64,
    
    /// Number of retry attempts
    pub retry_attempts: u32,
    
    /// User agent string for HTTP requests
    pub user_agent: String,
    
    /// Enable HTTP redirects
    pub follow_redirects: bool,
    
    /// Maximum redirect count
    pub max_redirects: u32,
}

impl Default for Config {
    fn default() -> Self {
        let mut key_bindings = std::collections::HashMap::new();
        key_bindings.insert("quit".to_string(), "q,esc".to_string());
        key_bindings.insert("select".to_string(), "enter".to_string());
        key_bindings.insert("preview".to_string(), "tab".to_string());
        key_bindings.insert("up".to_string(), "up,ctrl-k".to_string());
        key_bindings.insert("down".to_string(), "down,ctrl-j".to_string());
        key_bindings.insert("page_up".to_string(), "page-up,ctrl-b".to_string());
        key_bindings.insert("page_down".to_string(), "page-down,ctrl-f".to_string());

        Self {
            default_playlist: None,
            player_command: "mpv".to_string(),
            player_args: Some(vec![
                "--cache=yes".to_string(),
                "--demuxer-max-bytes=100M".to_string(),
                "--demuxer-readahead-secs=30".to_string(),
                "--force-window=immediate".to_string(),
                "--no-terminal".to_string(),
                "--quiet".to_string(),
                "--hwdec=auto-safe".to_string(),
                "--vo=gpu".to_string(),
                "--profile=fast".to_string(),
            ]),
            parallel_processing: true,
            max_search_results: 100,
            fuzzy_search: true,
            ui: UiConfig {
                color_scheme: "dark".to_string(),
                show_preview: true,
                preview_size: "50%".to_string(),
                page_size: 20,
                show_groups: true,
                key_bindings,
            },
            network: NetworkConfig {
                timeout: 30,
                retry_attempts: 3,
                user_agent: "RIPTV/1.0 (Rust IPTV Player)".to_string(),
                follow_redirects: true,
                max_redirects: 5,
            },
            recent_channels: Vec::new(),
            favorite_channels: Vec::new(),
        }
    }
}

impl Config {
    /// Load configuration from file, creating default if not exists
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let config_file = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::default_config_path()?,
        };

        if config_file.exists() {
            debug!("Loading config from: {}", config_file.display());
            
            let content = fs::read_to_string(&config_file)
                .with_context(|| format!("Failed to read config file: {}", config_file.display()))?;
            
            let config: Config = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_file.display()))?;
            
            info!("✅ Configuration loaded from {}", config_file.display());
            Ok(config)
        } else {
            info!("No config file found, creating default configuration");
            let config = Config::default();
            
            // Create config directory if it doesn't exist
            if let Some(parent) = config_file.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
            }
            
            // Save default config
            config.save(Some(config_file.to_str().unwrap()))?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self, config_path: Option<&str>) -> Result<()> {
        let config_file = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::default_config_path()?,
        };

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        // Create parent directory if needed
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        fs::write(&config_file, content)
            .with_context(|| format!("Failed to write config file: {}", config_file.display()))?;

        info!("✅ Configuration saved to {}", config_file.display());
        Ok(())
    }

    /// Get the default configuration file path
    fn default_config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .context("Unable to determine config directory")?;
        
        Ok(config_dir.join("riptv").join("config.json"))
    }

    /// Add a channel to recent channels list
    pub fn add_recent_channel(&mut self, channel_name: String) {
        // Remove if already exists
        self.recent_channels.retain(|name| name != &channel_name);
        
        // Add to front
        self.recent_channels.insert(0, channel_name);
        
        // Keep only last 20
        if self.recent_channels.len() > 20 {
            self.recent_channels.truncate(20);
        }
    }

    /// Add a channel to favorites
    pub fn add_favorite_channel(&mut self, channel_name: String) {
        if !self.favorite_channels.contains(&channel_name) {
            self.favorite_channels.push(channel_name);
        }
    }

    /// Remove a channel from favorites
    pub fn remove_favorite_channel(&mut self, channel_name: &str) {
        self.favorite_channels.retain(|name| name != channel_name);
    }

    /// Check if a channel is in favorites
    pub fn is_favorite(&self, channel_name: &str) -> bool {
        self.favorite_channels.contains(&channel_name.to_string())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Check if player command exists
        if self.player_command.is_empty() {
            anyhow::bail!("Player command cannot be empty");
        }

        // Validate timeout values
        if self.network.timeout == 0 {
            anyhow::bail!("Network timeout must be greater than 0");
        }

        if self.network.retry_attempts == 0 {
            warn!("Retry attempts is 0, network failures will not be retried");
        }

        // Validate UI settings
        if self.ui.page_size == 0 {
            anyhow::bail!("Page size must be greater than 0");
        }

        if self.max_search_results == 0 {
            anyhow::bail!("Max search results must be greater than 0");
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    /// Get player command with arguments
    pub fn get_player_command(&self) -> Vec<String> {
        let mut cmd = vec![self.player_command.clone()];
        
        if let Some(args) = &self.player_args {
            cmd.extend(args.clone());
        }
        
        cmd
    }

    /// Export configuration as JSON string
    pub fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to export configuration as JSON")
    }

    /// Import configuration from JSON string
    pub fn import_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Failed to import configuration from JSON")
    }

    /// Reset to default configuration
    pub fn reset_to_default() -> Self {
        Self::default()
    }

    /// Get configuration file location for display
    pub fn config_file_location() -> Result<String> {
        Ok(Self::default_config_path()?.display().to_string())
    }
}
