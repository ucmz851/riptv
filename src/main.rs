use anyhow::Result;
use clap::Parser;
use colored::*;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, error, debug};
use tokio::signal;

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

/// Restore terminal to normal state
fn cleanup_terminal() {
    debug!("Cleaning up terminal state");
    
    use utils::terminal::*;
    use std::io::{self, Write};
    
    // Print escape sequences to restore terminal
    print!("{}{}{}{}", 
        EXIT_ALTERNATE_SCREEN,
        SHOW_CURSOR,
        RESET_COLORS,
        RESET_TERMINAL
    );
    
    // Flush output immediately
    let _ = io::stdout().flush();
    
    // Give terminal time to process escape sequences
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    debug!("Terminal cleanup completed");
}

/// Setup signal handlers for graceful shutdown
async fn setup_signal_handlers(running: Arc<AtomicBool>) -> Result<()> {
    tokio::select! {
        _ = signal::ctrl_c() => {
            debug!("Received Ctrl+C signal");
            running.store(false, Ordering::Relaxed);
        }
        _ = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate())?;
                let mut sigint = signal(SignalKind::interrupt())?;
                
                tokio::select! {
                    _ = sigterm.recv() => {
                        debug!("Received SIGTERM");
                    }
                    _ = sigint.recv() => {
                        debug!("Received SIGINT");
                    }
                }
            }
            #[cfg(not(unix))]
            {
                // On Windows, just wait indefinitely
                std::future::pending::<()>().await;
            }
            Ok::<(), anyhow::Error>(())
        } => {
            running.store(false, Ordering::Relaxed);
        }
    }
    
    cleanup_terminal();
    Ok(())
}

async fn run_app(args: Args) -> Result<()> {
    // Create a flag for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // Setup signal handlers in background
    tokio::spawn(async move {
        if let Err(e) = setup_signal_handlers(running_clone).await {
            error!("Signal handler error: {}", e);
        }
    });
    
    // Load configuration
    let config = Config::load(args.config.as_deref())?;
    
    // Determine playlist path
    let playlist_path = args.playlist
        .or(config.default_playlist.clone())
        .unwrap_or_else(|| {
            error!("No playlist specified. Use --playlist or set default in config.");
            cleanup_terminal();
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
        cleanup_terminal();
        return Ok(());
    }

    // Load playlist
    player.load_playlist(&playlist_path).await?;

    if args.stats {
        player.show_statistics();
        cleanup_terminal();
        return Ok(());
    }

    if let Some(search_term) = args.search {
        player.search_channels(&search_term).await?;
        cleanup_terminal();
        return Ok(());
    }

    // Start interactive mode with graceful shutdown support
    let result = player.run_interactive_with_shutdown(running).await;
    
    // Always cleanup on exit
    cleanup_terminal();
    
    result
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    setup_logging(args.verbose);
    
    // Setup panic handler for emergency cleanup
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panicked: {}", panic_info);
        cleanup_terminal();
        // Additional emergency cleanup
        utils::terminal::emergency_terminal_reset();
    }));
    
    print_banner();

    if let Err(e) = run_app(args).await {
        error!("Application error: {}", e);
        
        // Print error chain
        let mut source = e.source();
        while let Some(err) = source {
            error!("  Caused by: {}", err);
            source = err.source();
        }
        
        // Ensure terminal is cleaned up even on error
        cleanup_terminal();
        process::exit(1);
    }
}
