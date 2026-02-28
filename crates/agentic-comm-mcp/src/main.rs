//! MCP server entry point for agentic-comm.
//!
//! Runs a stdio-based JSON-RPC 2.0 server implementing the MCP protocol
//! for agent-to-agent and agent-to-human communication.

mod config;
mod prompts;
mod protocol;
mod resources;
mod session;
mod tools;
mod types;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use protocol::handler::ProtocolHandler;
use protocol::stdio::run_stdio;
use session::manager::SessionManager;

#[tokio::main]
async fn main() {
    // Optional CLI arg: path to .acomm file
    let store_path: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);

    let session = SessionManager::new(store_path);
    let session = Arc::new(Mutex::new(session));
    let handler = Arc::new(ProtocolHandler::new(session));

    run_stdio(handler).await;
}
