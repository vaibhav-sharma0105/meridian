//! Meridian MCP Server
//!
//! Exposes Meridian's data via the Model Context Protocol (MCP).
//! Configure in Claude Code's MCP settings to enable AI agents to query
//! your tasks, meetings, and projects.
//!
//! Protocol: JSON-RPC 2.0 over stdio
//! - Reads JSON lines from stdin
//! - Writes JSON lines to stdout
//! - Logs to stderr

use anyhow::Result;
use std::io::{self, BufRead, Write};
use tracing::{error, info};

mod protocol;
mod handlers;

fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is for MCP protocol)
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Meridian MCP server starting");
    info!("DB path: {:?}", meridian_lib::db::connection::get_db_path());

    run_server()
}

fn run_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to read line: {}", e);
                continue;
            }
        };

        if line.is_empty() {
            continue;
        }

        let request: protocol::Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                let error_response = protocol::Response::error(
                    None,
                    protocol::RpcError::parse_error(&e.to_string()),
                );
                writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                stdout.flush()?;
                continue;
            }
        };

        if let Some(response) = handlers::handle_request(request) {
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
    }

    Ok(())
}
