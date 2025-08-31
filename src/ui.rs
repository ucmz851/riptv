use anyhow::Result;
use colored::*;
use skim::prelude::*;
use std::borrow::Cow;
use std::io::Cursor;
use tracing::debug;

use crate::config::Config;
use crate::playlist::Channel;
use crate::utils::terminal;

#[derive(Debug, Clone)]
pub struct ChannelItem {
    pub channel: Channel,
    pub display_text: String,
}

impl SkimItem for ChannelItem {
fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.display_text)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut preview = String::new();

        preview.push_str(&format!("🎬 {}\n", self.channel.name.bright_cyan().bold()));
        preview.push_str(&format!("🔗 {}\n\n", self.channel.url.bright_white()));

        if let Some(group) = &self.channel.group {
            preview.push_str(&format!("📁 Group: {}\n", group.bright_blue()));
        }

        if let Some(country) = &self.channel.country {
            preview.push_str(&format!("🌍 Country: {}\n", country.bright_green()));
        }

        if let Some(language) = &self.channel.language {
            preview.push_str(&format!("🗣️ Language: {}\n", language.bright_yellow()));
        }

        if let Some(logo) = &self.channel.logo {
            preview.push_str(&format!("🖼️ Logo: {}\n", logo.bright_magenta()));
        }

        preview.push_str("\n📋 Controls:\n");
        preview.push_str("  Enter - Play channel\n");
        preview.push_str("  Esc   - Exit\n");
        preview.push_str("  Tab   - Toggle preview\n");
        preview.push_str("  Ctrl+C - Quit");

        ItemPreview::Text(preview)
    }
}

pub struct ChannelSelector {
    channels: Vec<ChannelItem>,
    config: Config,
}

impl ChannelSelector {
    pub fn new(channels: Vec<Channel>, config: &Config) -> Self {
        let channel_items: Vec<ChannelItem> = channels
            .into_iter()
            .map(|channel| {
                let display_text = match &channel.group {
                    Some(group) => format!("[{}] {}", group, channel.name),
                    None => channel.name.clone(),
                };

                ChannelItem {
                    channel,
                    display_text,
                }
            })
            .collect();

        Self {
            channels: channel_items,
            config: config.clone(),
        }
    }

    pub async fn select_channel(&mut self) -> Result<Option<Channel>> {
        debug!("Starting channel selection with {} channels", self.channels.len());

        terminal::init_terminal();
        let result = self.run_selection().await;
        terminal::restore_terminal();

        result
    }

    async fn run_selection(&mut self) -> Result<Option<Channel>> {
        let logo_header = r#"
██████╗ ██╗██████╗ ████████╗██╗   ██╗
██╔══██╗██║██╔══██╗╚══██╔══╝██║   ██║
██████╔╝██║██████╔╝   ██║   ██║   ██║
██╔══██╗██║██╔═══╝    ██║   ╚██╗ ██╔╝
██║  ██║██║██║        ██║    ╚████╔╝ 
╚═╝  ╚═╝╚═╝╚═╝        ╚═╝     ╚═══╝
⚡ RIPTV - Blazing Fast IPTV Player v1.0
🦀 Written in Rust for Maximum Performance
Use arrows or Ctrl-J/K to navigate channels
Press Tab for preview, Enter to play, Esc to quit
"#;

        let options = SkimOptionsBuilder::default()
            .height(Some("70%"))
            .multi(false)
            .prompt(Some("⚡ RIPTV > "))
            .preview(Some(""))
            .preview_window(Some("right:50%:wrap"))
            .header(Some(logo_header))
            .bind(vec![
                "ctrl-j:down",
                "ctrl-k:up",
                "ctrl-d:half-page-down",
                "ctrl-u:half-page-up",
                "ctrl-f:page-down",
                "ctrl-b:page-up",
                "alt-enter:accept",
                "ctrl-c:abort",
            ])
            .reverse(true)
            .build()?;

        let input = self
            .channels
            .iter()
            .map(|item| item.display_text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(input));

        let output = Skim::run_with(&options, Some(items));

        match output {
            Some(output) => {
                if output.is_abort {
                    debug!("User aborted selection");
                    return Ok(None);
                }

                if let Some(selected_item) = output.selected_items.first() {
                    let selected_text = selected_item.output();
                    debug!("User selected: {}", selected_text);

                    for item in &self.channels {
                        if item.display_text == selected_text {
                            return Ok(Some(item.channel.clone()));
                        }
                    }
                }

                Ok(None)
            }
            None => {
                debug!("No selection made");
                Ok(None)
            }
        }
    }
}

impl Drop for ChannelSelector {
    fn drop(&mut self) {
        debug!("ChannelSelector being dropped, ensuring terminal cleanup");
        terminal::ensure_clean_terminal();
    }
}

// -----------------------------------
// Helper UI functions
// -----------------------------------

pub fn show_welcome_message() {
    println!("{}", "🎉 Welcome to RIPTV!".bright_magenta().bold());
    println!("{}", "The blazing fast IPTV player written in Rust.".bright_cyan());
    println!();

    println!("{}", "🚀 Features:".bright_yellow().bold());
    println!("  ⚡ Lightning-fast playlist parsing");
    println!("  🔍 Fuzzy search with real-time filtering");
    println!("  🎬 Optimized media playback");
    println!("  📊 Detailed playlist statistics");
    println!("  💾 Channel history and favorites");
    println!("  🎨 Beautiful terminal interface");
    println!();

    println!("{}", "📝 Quick Tips:".bright_green().bold());
    println!("  • Type to search channels in real-time");
    println!("  • Use arrow keys or Ctrl+J/K to navigate");
    println!("  • Press Tab to toggle preview panel");
    println!("  • Press Enter to play selected channel");
    println!("  • Press Esc or Ctrl+C to quit");
    println!();
}

pub fn show_loading_animation(message: &str) {
    use std::io::{self, Write};
    use std::thread;
    use std::time::Duration;

    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

    print!("{} ", message);
    io::stdout().flush().unwrap();

    for _ in 0..20 {
        for &ch in &spinner_chars {
            print!("\r{} {}", message, ch.to_string().bright_cyan());
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    }

    println!("\r{} ✅", message);
}

pub fn confirm_action(message: &str) -> bool {
    use std::io::{self, Write};

    print!("{} [y/N]: ", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let input = input.trim().to_lowercase();
            input == "y" || input == "yes"
        }
        Err(_) => false,
    }
}

pub fn display_error(error: &str) {
    eprintln!("{} {}", "❌ Error:".bright_red().bold(), error);
}

pub fn display_warning(warning: &str) {
    println!("{} {}", "⚠️ Warning:".bright_yellow().bold(), warning);
}

pub fn display_success(message: &str) {
    println!("{} {}", "✅ Success:".bright_green().bold(), message);
}

pub fn display_info(message: &str) {
    println!("{} {}", "ℹ️ Info:".bright_blue().bold(), message);
}
