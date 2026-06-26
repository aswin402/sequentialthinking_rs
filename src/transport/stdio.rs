use crate::server::SequentialThinkingServer;
use crate::tools::ToolRegistry;
use crate::transport::{handle_message, Transport};
use std::io::{self, Write};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

pub struct StdioTransport;

impl Transport for StdioTransport {
    async fn run(
        self,
        server: Arc<Mutex<SequentialThinkingServer>>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Result<(), String> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        tracing::info!("Sequential Thinking MCP Server running on stdio");

        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            // Acquire lock on server for this request
            let mut server_lock = server.lock().await;
            
            if let Some(response) = handle_message(&line, &tool_registry, &mut server_lock) {
                if let Ok(serialized) = serde_json::to_string(&response) {
                    println!("{}", serialized);
                    let _ = io::stdout().flush();
                }
            }
        }
        Ok(())
    }
}
