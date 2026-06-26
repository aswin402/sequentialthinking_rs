use crate::server::SequentialThinkingServer;
use crate::tools::ToolRegistry;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod stdio;
pub mod http;

/// Trait representing an MCP transport layer (Stdio or HTTP/SSE).
pub trait Transport {
    async fn run(
        self,
        server: Arc<Mutex<SequentialThinkingServer>>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Result<(), String>;
}

/// Helper function to process a JSON-RPC request string and return the response value if needed.
pub fn handle_message(
    line: &str,
    tool_registry: &ToolRegistry,
    server: &mut SequentialThinkingServer,
) -> Option<serde_json::Value> {
    use crate::types::{CallToolParams, JsonRpcError, JsonRpcRequest, JsonRpcResponse, ToolCallResponse, TextContent};
    use serde_json::json;

    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(req) => req,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse JSON-RPC request");
            return Some(json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": format!("Parse error: {}", e)
                }
            }));
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
                    "version": "1.0.0"
                }
            });
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: Some(result),
                error: None,
            };
            serde_json::to_value(&response).ok()
        }
        "notifications/initialized" => {
            // Client initialization confirmation, no response required
            None
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
            serde_json::to_value(&response).ok()
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
                            message: "Missing parameters".to_string(),
                            data: None,
                        }),
                    };
                    return serde_json::to_value(&response).ok();
                }
            };

            match params {
                Ok(call_params) => {
                    tracing::info!(tool = %call_params.name, "Calling tool");
                    if let Some(tool) = tool_registry.get(&call_params.name) {
                        match tool.execute(server, call_params.arguments) {
                            Ok(result) => {
                                let formatted_json = serde_json::to_string_pretty(&result).unwrap_or_default();
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
                                serde_json::to_value(&response).ok()
                            }
                            Err(err_msg) => {
                                tracing::error!(error = %err_msg, tool = %call_params.name, "Tool execution failed");
                                let tool_response = ToolCallResponse {
                                    content: vec![TextContent {
                                        content_type: "text".to_string(),
                                        text: format!("Error executing tool: {}", err_msg),
                                    }],
                                    is_error: Some(true),
                                };
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: req_id,
                                    result: Some(serde_json::to_value(tool_response).unwrap()),
                                    error: None,
                                };
                                serde_json::to_value(&response).ok()
                            }
                        }
                    } else {
                        tracing::warn!(tool = %call_params.name, "Requested tool not found");
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
                        serde_json::to_value(&response).ok()
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Invalid parameters for tools/call");
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req_id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Invalid parameters: {}", e),
                            data: None,
                        }),
                    };
                    serde_json::to_value(&response).ok()
                }
            }
        }
        other => {
            tracing::warn!(method = %other, "Unsupported JSON-RPC method");
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", other),
                    data: None,
                }),
            };
            serde_json::to_value(&response).ok()
        }
    }
}
