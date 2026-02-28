//! Stdio transport — reads JSON-RPC from stdin, writes responses to stdout.

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use crate::protocol::handler::ProtocolHandler;
use crate::types::JsonRpcMessage;

/// Run the stdio transport loop.
pub async fn run_stdio(handler: Arc<ProtocolHandler>) {
    let stdin = tokio::io::stdin();
    let stdout = Arc::new(Mutex::new(tokio::io::stdout()));
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => {
                // EOF — clean up
                handler.cleanup().await;
                break;
            }
            Err(e) => {
                eprintln!("Stdin read error: {e}");
                handler.cleanup().await;
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: JsonRpcMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(e) => {
                // Send parse error response
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {e}")
                    }
                });
                let mut out = stdout.lock().await;
                let _ = out
                    .write_all(format!("{}\n", error_response).as_bytes())
                    .await;
                let _ = out.flush().await;
                continue;
            }
        };

        if let Some(response) = handler.handle_message(msg).await {
            let mut out = stdout.lock().await;
            let response_str = serde_json::to_string(&response).unwrap_or_default();
            let _ = out
                .write_all(format!("{response_str}\n").as_bytes())
                .await;
            let _ = out.flush().await;
        }

        if handler.shutdown_requested() {
            handler.cleanup().await;
            break;
        }
    }
}
