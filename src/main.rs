//! webpuppet-mcp - MCP server for browser automation
//!
//! This binary provides an MCP server that exposes webpuppet functionality
//! to AI assistants like GitHub Copilot and Claude Desktop.

use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use webpuppet::{PermissionGuard, PermissionPolicy};
use webpuppet_mcp::McpServer;

/// MCP server for webpuppet browser automation.
#[derive(Parser, Debug)]
#[command(name = "webpuppet-mcp")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in stdio mode (standard MCP transport).
    #[arg(long, default_value = "true")]
    stdio: bool,

    /// Permission policy (secure, permissive, readonly).
    #[arg(long, default_value = "secure")]
    policy: String,

    /// Show browser window (non-headless mode).
    /// When enabled, browser automation will be visible to the user.
    #[arg(long)]
    visible: bool,

    /// Enable verbose logging.
    #[arg(short, long)]
    verbose: bool,

    /// Log output file (default: stderr).
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    // Set up logging
    let filter = if args.verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    // Log to stderr (not stdout, which is used for MCP protocol)
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!(
        "Starting {} v{}",
        webpuppet_mcp::server::SERVER_NAME,
        webpuppet_mcp::server::SERVER_VERSION
    );

    // Set up permissions
    let permissions = match args.policy.to_lowercase().as_str() {
        "secure" => PermissionGuard::new(PermissionPolicy::secure()),
        "permissive" => PermissionGuard::new(PermissionPolicy::permissive()),
        "readonly" => PermissionGuard::new(PermissionPolicy::read_only()),
        _ => {
            tracing::error!("Unknown policy: {}. Using 'secure'.", args.policy);
            PermissionGuard::secure()
        }
    };

    tracing::info!("Using '{}' permission policy", args.policy);

    // Create server with visible browser if requested
    let server = if args.visible {
        tracing::info!("Browser will be visible (non-headless mode)");
        McpServer::with_visible_browser(permissions)
    } else {
        McpServer::with_permissions(permissions)
    };

    if args.stdio {
        match server.run_stdio().await {
            Ok(()) => {
                tracing::info!("Server exited cleanly");
                ExitCode::SUCCESS
            }
            Err(e) => {
                tracing::error!("Server error: {}", e);
                ExitCode::FAILURE
            }
        }
    } else {
        tracing::error!("Only stdio mode is currently supported");
        ExitCode::FAILURE
    }
}
