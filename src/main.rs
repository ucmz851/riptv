use anyhow::Result;
use clap::Parser;
use colored::*;
use std::process;
use tracing::{info, warn, error};

mod config;
mod player;
mod playlist;
mod ui;
mod utils;

use config::Config;
use player::IptvPlayer;

#[derive(Parser)]
#[command(
    name = "riptv",
    about = "⚡ Blazing fast IPTV player written in Rust",
    version = "1.0.0",
    author = "Your Name"
)]
struct Args {
    /// Path to M3U playlist file
    #[arg(short, long, value_name = "FILE")]
    playlist: Option<String>,

    /// Media player command (default: mpv)
    #[arg(short = 'P', long, default_value = "mpv")]
    player: String,

    /// Enable parallel processing for large playlists
    #[arg(long)]
    parallel: bool,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// List available playlists and exit
    #[arg(long)]
    list: bool,

    /// Search channels by name
    #[arg(short, long)]
    search: Option<String>,

    /// Show statistics about the playlist
    #[arg(long)]
    stats: bool,
}

fn setup_logging(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("riptv={}", level))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}

fn print_banner() {
    println!("{}", "
    ██████╗ ██╗██████╗ ████████╗██╗   ██╗
    ██╔══██╗██║██╔══██╗╚══██╔══╝██║   ██║
    ██████╔╝██║██████╔╝   ██║   ██║   ██║
    ██╔══██╗██║██╔═══╝    ██║   ╚██╗ ██╔╝
    ██║  ██║██║██║        ██║    ╚████╔╝ 
    ╚═╝  ╚═╝╚═╝╚═╝        ╚═╝     ╚═══╝  
    ".bright_cyan());
    
    println!("{}", "⚡ Blazing Fast IPTV Player v1.0".bright_yellow().bold());
    println!("{}", "🦀 Written in Rust for Maximum Performance".bright_green());
    println!();
}

async fn run_app(args: Args) -> Result<()> {
    // Load configuration
    let config = Config::load(args.config.as_deref())?;
    
    // Determine playlist path
    let playlist_path = args.playlist
        .or(config.default_playlist.clone())
        .unwrap_or_else(|| {
            error!("No playlist specified. Use --playlist or set default in config.");
            process::exit(1);
        });

    info!("Using playlist: {}", playlist_path);

    // Create player instance
    let mut player = IptvPlayer::new(
        args.player.clone(),
        config,
        args.parallel,
    );

    // Handle special commands
    if args.list {
        player.list_playlists().await?;
        return Ok(());
    }

    // Load playlist
    player.load_playlist(&playlist_path).await?;

    if args.stats {
        player.show_statistics();
        return Ok(());
    }

    if let Some(search_term) = args.search {
        player.search_channels(&search_term).await?;
        return Ok(());
    }

    // Start interactive mode
    player.run_interactive().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    setup_logging(args.verbose);
    print_banner();

    if let Err(e) = run_app(args).await {
        error!("Application error: {}", e);
        
        // Print error chain
        let mut source = e.source();
        while let Some(err) = source {
            error!("  Caused by: {}", err);
            source = err.source();
        }
        
        process::exit(1);
    }
}
