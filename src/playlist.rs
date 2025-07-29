use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tokio::task;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub url: String,
    pub group: Option<String>,
    pub logo: Option<String>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub tvg_id: Option<String>,
}

impl Channel {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            group: None,
            logo: None,
            language: None,
            country: None,
            tvg_id: None,
        }
    }

    pub fn with_metadata(
        name: String,
        url: String,
        group: Option<String>,
        logo: Option<String>,
        language: Option<String>,
        country: Option<String>,
        tvg_id: Option<String>,
    ) -> Self {
        Self {
            name,
            url,
            group,
            logo,
            language,
            country,
            tvg_id,
        }
    }

    pub fn display_name(&self) -> String {
        match &self.group {
            Some(group) => format!("[{}] {}", group.bright_blue(), self.name),
            None => self.name.clone(),
        }
    }
}

pub struct PlaylistParser {
    channels: Vec<Channel>,
    channel_map: HashMap<String, usize>,
    groups: HashMap<String, Vec<usize>>,
    parallel_processing: bool,
}

impl PlaylistParser {
    pub fn new(parallel_processing: bool) -> Self {
        Self {
            channels: Vec::new(),
            channel_map: HashMap::new(),
            groups: HashMap::new(),
            parallel_processing,
        }
    }

    pub async fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        info!("📂 Loading playlist: {}", path.display());

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read playlist file: {}", path.display()))?;

        if self.parallel_processing {
            self.parse_parallel(content).await?;
        } else {
            self.parse_sequential(content)?;
        }

        self.build_indices();
        Ok(())
    }

    async fn parse_parallel(&mut self, content: String) -> Result<()> {
        let start = Instant::now();
        info!("🚀 Using parallel processing for maximum speed...");

        // Create progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Parsing playlist...");

        let channels = task::spawn_blocking(move || {
            let lines: Vec<&str> = content.lines().collect();
            let mut channels = Vec::with_capacity(100_000);
            
            // Regex for parsing EXTINF metadata
            let extinf_regex = Regex::new(
                r#"#EXTINF:([^,]*),(?:.*tvg-name="([^"]*)")?(?:.*tvg-logo="([^"]*)")?(?:.*group-title="([^"]*)")?(?:.*tvg-language="([^"]*)")?(?:.*tvg-country="([^"]*)")?(?:.*tvg-id="([^"]*)")?(.*)$"#
            ).unwrap();

            let mut i = 1;
            let total_lines = lines.len();

            while i < total_lines {
                if let Some(line) = lines.get(i) {
                    if line.starts_with("#EXTINF:") {
                        if let Some(captures) = extinf_regex.captures(line) {
                            // Extract metadata
                            let tvg_name = captures.get(2).map(|m| m.as_str().to_string());
                            let logo = captures.get(3).map(|m| m.as_str().to_string());
                            let group = captures.get(4).map(|m| m.as_str().to_string());
                            let language = captures.get(5).map(|m| m.as_str().to_string());
                            let country = captures.get(6).map(|m| m.as_str().to_string());
                            let tvg_id = captures.get(7).map(|m| m.as_str().to_string());
                            
                            // Channel name is everything after the last comma
                            let name_part = captures.get(8)
                                .map(|m| m.as_str().trim())
                                .unwrap_or("Unknown Channel");
                            
                            let channel_name = tvg_name.unwrap_or_else(|| name_part.to_string());

                            // Get URL from previous line
                            if let Some(url_line) = lines.get(i - 1) {
                                if url_line.starts_with("http") {
                                    channels.push(Channel::with_metadata(
                                        channel_name,
                                        url_line.trim().to_string(),
                                        group,
                                        logo,
                                        language,
                                        country,
                                        tvg_id,
                                    ));
                                }
                            }
                        } else {
                            // Fallback parsing for simple format
                            if let Some(comma_pos) = line.find(',') {
                                let name = line[comma_pos + 1..].trim();
                                if let Some(url_line) = lines.get(i - 1) {
                                    if url_line.starts_with("http") {
                                        channels.push(Channel::new(
                                            name.to_string(),
                                            url_line.trim().to_string(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                i += 1;
            }

            channels
        }).await?;

        pb.finish_with_message("✅ Parsing complete!");

        let duration = start.elapsed();
        let channels_per_sec = channels.len() as f64 / duration.as_secs_f64();

        info!(
            "⚡ Parsed {} channels in {:?} ({:.0} channels/sec)",
            channels.len().to_string().bright_green().bold(),
            duration,
            channels_per_sec
        );

        self.channels = channels;
        Ok(())
    }

    fn parse_sequential(&mut self, content: String) -> Result<()> {
        let start = Instant::now();
        info!("📝 Using sequential processing...");

        let lines: Vec<&str> = content.lines().collect();
        let mut channels = Vec::with_capacity(50_000);

        let pb = ProgressBar::new(lines.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap(),
        );

        let mut i = 1;
        while i < lines.len() {
            if let Some(line) = lines.get(i) {
                if line.starts_with("#EXTINF:") {
                    if let Some(comma_pos) = line.find(',') {
                        let name = line[comma_pos + 1..].trim();
                        if let Some(url_line) = lines.get(i - 1) {
                            if url_line.starts_with("http") {
                                channels.push(Channel::new(
                                    name.to_string(),
                                    url_line.trim().to_string(),
                                ));
                            }
                        }
                    }
                }
            }
            
            if i % 1000 == 0 {
                pb.set_position(i as u64);
                pb.set_message(format!("Found {} channels", channels.len()));
            }
            
            i += 1;
        }

        pb.finish_with_message("✅ Parsing complete!");

        let duration = start.elapsed();
        info!("📊 Parsed {} channels in {:?}", channels.len(), duration);

        self.channels = channels;
        Ok(())
    }

    fn build_indices(&mut self) {
        info!("🔗 Building search indices...");
        
        // Build channel name -> index map
        for (idx, channel) in self.channels.iter().enumerate() {
            self.channel_map.insert(channel.name.clone(), idx);
        }

        // Build group -> channel indices map
        for (idx, channel) in self.channels.iter().enumerate() {
            if let Some(group) = &channel.group {
                self.groups
                    .entry(group.clone())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        debug!("Built indices for {} channels and {} groups", 
               self.channels.len(), self.groups.len());
    }

    pub fn get_channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn get_channel_by_name(&self, name: &str) -> Option<&Channel> {
        self.channel_map.get(name)
            .and_then(|&idx| self.channels.get(idx))
    }

    pub fn get_channels_by_group(&self, group: &str) -> Vec<&Channel> {
        self.groups.get(group)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&idx| self.channels.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn search_channels(&self, query: &str) -> Vec<&Channel> {
        use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
        
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<(i64, &Channel)> = self.channels
            .iter()
            .filter_map(|channel| {
                matcher.fuzzy_match(&channel.name, query)
                    .map(|score| (score, channel))
            })
            .collect();

        // Sort by score (higher is better)
        matches.sort_by(|a, b| b.0.cmp(&a.0));
        matches.into_iter().map(|(_, channel)| channel).collect()
    }

    pub fn get_statistics(&self) -> PlaylistStats {
        let mut stats = PlaylistStats::default();
        
        stats.total_channels = self.channels.len();
        stats.total_groups = self.groups.len();
        
        // Count channels per group
        for (group, channels) in &self.groups {
            stats.channels_per_group.insert(group.clone(), channels.len());
        }

        // Count by country/language if available
        for channel in &self.channels {
            if let Some(country) = &channel.country {
                *stats.countries.entry(country.clone()).or_insert(0) += 1;
            }
            if let Some(language) = &channel.language {
                *stats.languages.entry(language.clone()).or_insert(0) += 1;
            }
        }

        stats
    }
}

#[derive(Debug, Default)]
pub struct PlaylistStats {
    pub total_channels: usize,
    pub total_groups: usize,
    pub channels_per_group: HashMap<String, usize>,
    pub countries: HashMap<String, usize>,
    pub languages: HashMap<String, usize>,
}
