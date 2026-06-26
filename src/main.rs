use clap::Parser;
use serde_json::json;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

mod server;
mod types;
mod logging;
mod persistence;
mod tools;
mod graph;

use server::SequentialThinkingServer;
use types::{
    CallToolParams, JsonRpcError, JsonRpcRequest, JsonRpcResponse, TextContent,
    ToolCallResponse,
};

/// Sequential Thinking MCP Server in Rust (with Graph of Thoughts & Clear Thought features)
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Sequential Thinking MCP Server in Rust (with Graph of Thoughts & Clear Thought features)"
)]
struct Args {
    /// Disable thought boxes print on stderr
    #[arg(short, long, env = "DISABLE_THOUGHT_LOGGING")]
    disable_thought_logging: bool,

    /// Log format for structured logs ("pretty" or "json")
    #[arg(long, default_value = "pretty", env = "LOG_FORMAT")]
    log_format: String,

    /// Storage backend ("memory" or "sqlite")
    #[arg(long, default_value = "memory", env = "STORAGE")]
    storage: String,

    /// Path to SQLite database file
    #[arg(long, env = "DB_PATH")]
    db_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    logging::setup_logging(&args.log_format);

    let store: Box<dyn persistence::ThoughtStore> = match args.storage.as_str() {
        "sqlite" => {
            let db_path = args.db_path.clone().unwrap_or_else(|| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                let dir = std::path::Path::new(&home).join(".sequentialthinking");
                let _ = std::fs::create_dir_all(&dir);
                dir.join("history.db").to_str().unwrap().to_string()
            });
            Box::new(persistence::sqlite::SqliteThoughtStore::new(&db_path).expect("Failed to initialize SQLite store"))
        }
        _ => Box::new(persistence::memory::MemoryThoughtStore::new()),
    };

    let mut thinking_server = SequentialThinkingServer::new(store, args.disable_thought_logging);

    let mut tool_registry = tools::ToolRegistry::new();
    tool_registry.register(Box::new(tools::sequentialthinking::SequentialThinkingTool));
    tool_registry.register(Box::new(tools::analyze_graph::AnalyzeGraphTool));
    tool_registry.register(Box::new(tools::export_session::ExportSessionTool));
    tool_registry.register(Box::new(tools::summarize::SummarizeReasoningTool));
    tool_registry.register(Box::new(tools::templates::TemplatesTool));

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    tracing::info!("Sequential Thinking MCP Server running on stdio");

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse JSON-RPC request");
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                send_response(&err_resp);
                continue;
            }
        };

        tracing::debug!(method = %request.method, "Parsed JSON-RPC request");

        let req_id = request.id.clone().unwrap_or(serde_json::Value::Null);

        match request.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "sequential-thinking-server",
                        "version": "0.6.0"
                    }
                });
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(result),
                    error: None,
                };
                send_response(&response);
            }
            "notifications/initialized" => {
                // Client initialization confirmation, no response required
            }
            "tools/list" => {
                let result = json!({
                    "tools": tool_registry.list()
                });

                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(result),
                    error: None,
                };
                send_response(&response);
            }
            "tools/call" => {
                let params: Result<CallToolParams, serde_json::Error> = match request.params {
                    Some(p) => serde_json::from_value(p),
                    None => {
                        tracing::warn!("Missing params for tools/call");
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req_id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Missing params".to_string(),
                                data: None,
                            }),
                        };
                        send_response(&response);
                        continue;
                    }
                };

                let call_params = match params {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!(error = %e, "Invalid params for tools/call");
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req_id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: format!("Invalid params: {}", e),
                                data: None,
                            }),
                        };
                        send_response(&response);
                        continue;
                    }
                };

                let tool = match tool_registry.get(&call_params.name) {
                    Some(t) => t,
                    None => {
                        tracing::warn!(tool = %call_params.name, "Tool not found");
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req_id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32601,
                                message: format!("Tool not found: {}", call_params.name),
                                data: None,
                            }),
                        };
                        send_response(&response);
                        continue;
                    }
                };

                match tool.execute(&mut thinking_server, call_params.arguments) {
                    Ok(res) => {
                        tracing::info!(
                            tool = %tool.name(),
                            "Processed tool call successfully"
                        );
                        let formatted_json = serde_json::to_string_pretty(&res).unwrap_or_default();
                        let text_content = TextContent {
                            content_type: "text".to_string(),
                            text: formatted_json,
                        };
                        let tool_response = ToolCallResponse {
                            content: vec![text_content],
                            is_error: None,
                        };
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req_id,
                            result: Some(serde_json::to_value(tool_response).unwrap()),
                            error: None,
                        };
                        send_response(&response);
                    }
                    Err(e) => {
                        tracing::error!(tool = %tool.name(), error = %e, "Error executing tool");
                        let tool_response = ToolCallResponse {
                            content: vec![TextContent {
                                content_type: "text".to_string(),
                                text: format!("Error executing tool: {}", e),
                            }],
                            is_error: Some(true),
                        };
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req_id,
                            result: Some(serde_json::to_value(tool_response).unwrap()),
                            error: None,
                        };
                        send_response(&response);
                    }
                }
            }
            "ping" => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: Some(json!({})),
                    error: None,
                };
                send_response(&response);
            }
            _ => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req_id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                };
                send_response(&response);
            }
        }
    }
}

fn send_response<T: serde::Serialize>(response: &T) {
    if let Ok(serialized) = serde_json::to_string(response) {
        println!("{}", serialized);
        let _ = io::stdout().flush();
    }
}
