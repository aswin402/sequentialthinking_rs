use clap::Parser;
use serde_json::json;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

mod server;
mod types;
mod logging;

use server::SequentialThinkingServer;
use types::{
    CallToolParams, JsonRpcError, JsonRpcRequest, JsonRpcResponse, TextContent, ThoughtData,
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    logging::setup_logging(&args.log_format);

    let mut thinking_server = SequentialThinkingServer::new(args.disable_thought_logging);

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
                        "version": "0.3.0"
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
                let sequential_thinking_tool = json!({
                    "name": "sequentialthinking",
                    "description": "A detailed tool for dynamic and reflective problem-solving through thoughts. Supports branching, revisions, Graph of Thoughts (GoT) merging, and Clear Thought parameters.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "thought": {
                                "type": "string",
                                "description": "Your current thinking step (analysis, observations, or conclusions)"
                            },
                            "nextThoughtNeeded": {
                                "type": "boolean",
                                "description": "Whether another thought step is needed"
                            },
                            "thoughtNumber": {
                                "type": "integer",
                                "description": "Current thought number in the sequence (starts at 1)"
                            },
                            "totalThoughts": {
                                "type": "integer",
                                "description": "Estimated total thoughts needed (can be adjusted dynamically)"
                            },
                            "isRevision": {
                                "type": "boolean",
                                "description": "Whether this revises previous thinking steps"
                            },
                            "revisesThought": {
                                "type": "integer",
                                "description": "Which thought number is being reconsidered/revised"
                            },
                            "branchFromThought": {
                                "type": "integer",
                                "description": "The thought number from which this alternative branch branches out"
                            },
                            "branchId": {
                                "type": "string",
                                "description": "Identifier for the current branch"
                            },
                            "needsMoreThoughts": {
                                "type": "boolean",
                                "description": "Explicit request to add more thoughts to the estimate"
                            },
                            "parentThoughts": {
                                "type": "array",
                                "items": {
                                    "type": "integer"
                                },
                                "description": "Array of multiple parent thought numbers to merge branches (Graph of Thoughts)"
                            },
                            "assumptions": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "List of assumptions made in this thought step"
                            },
                            "verifiedAssumptions": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "Assumptions verified or refuted in this step"
                            },
                            "confidenceScore": {
                                "type": "number",
                                "description": "Confidence level in this line of reasoning (0.0 to 1.0)"
                            },
                            "criticism": {
                                "type": "string",
                                "description": "Self-criticism or evaluation of previous thoughts"
                            },
                            "hypothesis": {
                                "type": "string",
                                "description": "Hypothesis to be tested in this thought step"
                            },
                            "verificationMethod": {
                                "type": "string",
                                "description": "Method to verify or test the hypothesis"
                            },
                            "leftToBeDone": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "List of items/tasks left to be done or verified"
                            }
                        },
                        "required": ["thought", "nextThoughtNeeded", "thoughtNumber", "totalThoughts"]
                    }
                });

                let result = json!({
                    "tools": [sequential_thinking_tool]
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

                if call_params.name != "sequentialthinking" {
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

                let thought_data_res: Result<ThoughtData, serde_json::Error> =
                    serde_json::from_value(call_params.arguments);
                let thought_data = match thought_data_res {
                    Ok(td) => td,
                    Err(e) => {
                        tracing::warn!(error = %e, "Invalid tool arguments");
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req_id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: format!("Invalid tool arguments: {}", e),
                                data: None,
                            }),
                        };
                        send_response(&response);
                        continue;
                    }
                };

                match thinking_server.process_thought(thought_data) {
                    Ok(res) => {
                        tracing::info!(
                            thought_number = res.thought_number,
                            total_thoughts = res.total_thoughts,
                            "Processed thought successfully"
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
                        tracing::error!(error = %e, "Error processing thought");
                        let tool_response = ToolCallResponse {
                            content: vec![TextContent {
                                content_type: "text".to_string(),
                                text: format!("Error processing thought: {}", e),
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
