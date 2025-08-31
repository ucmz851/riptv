use std::time::Duration;

/// Terminal control escape sequences and utilities
pub mod terminal {
    use std::io::{self, Write};
    
    /// Terminal escape sequences
    pub const ENTER_ALTERNATE_SCREEN: &str = "\x1B[?1049h";
    pub const EXIT_ALTERNATE_SCREEN: &str = "\x1B[?1049l";
    pub const HIDE_CURSOR: &str = "\x1B[?25l";
    pub const SHOW_CURSOR: &str = "\x1B[?25h";
    pub const RESET_COLORS: &str = "\x1B[0m";
    pub const RESET_TERMINAL: &str = "\x1Bc";  // RIS - Reset to Initial State
    pub const CLEAR_SCREEN: &str = "\x1B[2J";
    pub const MOVE_CURSOR_HOME: &str = "\x1B[H";
    pub const SOFT_RESET: &str = "\x1B[!p";
    
    /// Ensure terminal is in a clean state
    pub fn ensure_clean_terminal() {
        print!("{}{}{}{}", 
            EXIT_ALTERNATE_SCREEN,
            SHOW_CURSOR, 
            RESET_COLORS,
            RESET_TERMINAL
        );
        
        let _ = io::stdout().flush();
        
        // Give terminal time to process
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    /// Initialize terminal for TUI mode
    pub fn init_terminal() {
        print!("{}{}", ENTER_ALTERNATE_SCREEN, HIDE_CURSOR);
        let _ = io::stdout().flush();
    }
    
    /// Restore terminal from TUI mode
    pub fn restore_terminal() {
        ensure_clean_terminal();
    }
    
    /// Emergency terminal reset (call this in panic handlers)
    pub fn emergency_terminal_reset() {
        // More aggressive reset sequence
        print!("{}", SOFT_RESET);      // Soft reset
        print!("{}", EXIT_ALTERNATE_SCREEN);  // Exit alternate screen
        print!("{}", SHOW_CURSOR);    // Show cursor
        print!("{}", RESET_COLORS);      // Reset attributes
        print!("{}", RESET_TERMINAL);        // Full reset
        print!("\x0C");         // Form feed (clear screen)
        
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
        
        // Give more time for emergency reset
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    #[cfg(unix)]
    pub fn reset_terminal_unix() {
        use std::process::Command;
        
        // Use system commands as a fallback
        let _ = Command::new("reset").output();
        let _ = Command::new("stty").arg("sane").output();
    }
    
    /// Comprehensive terminal cleanup
    pub fn comprehensive_cleanup() {
        ensure_clean_terminal();
        
        #[cfg(unix)]
        reset_terminal_unix();
        
        // Give the terminal extra time to process the escape sequences
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Format duration in a human-readable format
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Sanitize channel name for filename usage
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect()
}

/// Extract domain from URL
pub fn extract_domain(url: &str) -> Option<String> {
    if let Ok(parsed) = url::Url::parse(url) {
        parsed.host_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// Check if URL is valid
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Truncate string to specified length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Calculate similarity between two strings (simple implementation)
pub fn string_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    
    if a_lower == b_lower {
        return 1.0;
    }
    
    if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
        return 0.8;
    }
    
    // Simple character-based similarity
    let mut matches = 0;
    let min_len = a_lower.len().min(b_lower.len());
    
    for (ca, cb) in a_lower.chars().zip(b_lower.chars()) {
        if ca == cb {
            matches += 1;
        }
    }
    
    matches as f64 / min_len as f64
}

/// Parse M3U metadata from EXTINF line
pub fn parse_extinf_metadata(extinf_line: &str) -> ExtinfMetadata {
    let mut metadata = ExtinfMetadata::default();
    
    // Extract basic info after comma
    if let Some(comma_pos) = extinf_line.find(',') {
        let after_comma = &extinf_line[comma_pos + 1..];
        
        // Look for various attributes
        if let Some(tvg_name) = extract_attribute(extinf_line, "tvg-name") {
            metadata.tvg_name = Some(tvg_name);
        }
        
        if let Some(tvg_logo) = extract_attribute(extinf_line, "tvg-logo") {
            metadata.tvg_logo = Some(tvg_logo);
        }
        
        if let Some(group_title) = extract_attribute(extinf_line, "group-title") {
            metadata.group_title = Some(group_title);
        }
        
        if let Some(tvg_language) = extract_attribute(extinf_line, "tvg-language") {
            metadata.tvg_language = Some(tvg_language);
        }
        
        if let Some(tvg_country) = extract_attribute(extinf_line, "tvg-country") {
            metadata.tvg_country = Some(tvg_country);
        }
        
        if let Some(tvg_id) = extract_attribute(extinf_line, "tvg-id") {
            metadata.tvg_id = Some(tvg_id);
        }
        
        // Channel name is everything after attributes
        metadata.channel_name = after_comma.trim().to_string();
        
        // If we have tvg-name, prefer that
        if let Some(ref tvg_name) = metadata.tvg_name {
            if !tvg_name.is_empty() {
                metadata.channel_name = tvg_name.clone();
            }
        }
    }
    
    metadata
}

fn extract_attribute(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, attr_name);
    if let Ok(re) = regex::Regex::new(&pattern) {
        if let Some(captures) = re.captures(line) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

#[derive(Debug, Default)]
pub struct ExtinfMetadata {
    pub channel_name: String,
    pub tvg_name: Option<String>,
    pub tvg_logo: Option<String>,
    pub group_title: Option<String>,
    pub tvg_language: Option<String>,
    pub tvg_country: Option<String>,
    pub tvg_id: Option<String>,
}

/// Create progress callback for long operations
pub fn create_progress_callback<F>(total: usize, callback: F) -> impl FnMut(usize)
where
    F: FnMut(f64),
{
    let mut callback = callback;
    move |current: usize| {
        let progress = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        callback(progress);
    }
}

/// Simple retry mechanism
pub async fn retry_async<F, Fut, T, E>(
    mut operation: F,
    max_attempts: u32,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_attempts {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

/// Get system information for debugging
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        family: std::env::consts::FAMILY.to_string(),
        exe_suffix: std::env::consts::EXE_SUFFIX.to_string(),
    }
}

#[derive(Debug)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub exe_suffix: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file<name>"), "test_file_name_");
        assert_eq!(sanitize_filename("normal_name"), "normal_name");
    }

    #[test]
    fn test_string_similarity() {
        assert_eq!(string_similarity("test", "test"), 1.0);
        assert!(string_similarity("testing", "test") > 0.7);
        assert!(string_similarity("abc", "xyz") < 0.5);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com/stream.m3u8"));
        assert!(is_valid_url("http://test.tv/channel"));
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_terminal_cleanup() {
        // Test that terminal utilities don't panic
        terminal::ensure_clean_terminal();
        terminal::emergency_terminal_reset();
    }
}
