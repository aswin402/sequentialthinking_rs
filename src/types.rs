use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: serde_json::Value,
    #[serde(rename = "clientInfo")]
    pub client_info: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtData {
    pub thought: String,
    #[serde(rename = "thoughtNumber")]
    pub thought_number: usize,
    #[serde(rename = "totalThoughts")]
    pub total_thoughts: usize,
    #[serde(rename = "nextThoughtNeeded")]
    pub next_thought_needed: bool,

    // Original optional fields
    #[serde(rename = "isRevision")]
    pub is_revision: Option<bool>,
    #[serde(rename = "revisesThought")]
    pub revises_thought: Option<usize>,
    #[serde(rename = "branchFromThought")]
    pub branch_from_thought: Option<usize>,
    #[serde(rename = "branchId")]
    pub branch_id: Option<String>,
    #[serde(rename = "needsMoreThoughts")]
    pub needs_more_thoughts: Option<bool>,

    // Enhanced GoT / Clear Thought fields
    #[serde(rename = "parentThoughts")]
    pub parent_thoughts: Option<Vec<usize>>,
    pub assumptions: Option<Vec<String>>,
    #[serde(rename = "verifiedAssumptions")]
    pub verified_assumptions: Option<Vec<String>>,
    #[serde(rename = "confidenceScore")]
    pub confidence_score: Option<f64>,
    pub criticism: Option<String>,
    pub hypothesis: Option<String>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Option<String>,
    #[serde(rename = "leftToBeDone")]
    pub left_to_be_done: Option<Vec<String>>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResponse {
    pub content: Vec<TextContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    #[serde(rename = "thoughtNumber")]
    pub thought_number: usize,
    #[serde(rename = "totalThoughts")]
    pub total_thoughts: usize,
    #[serde(rename = "nextThoughtNeeded")]
    pub next_thought_needed: bool,
    pub branches: Vec<String>,
    #[serde(rename = "thoughtHistoryLength")]
    pub thought_history_length: usize,
    #[serde(rename = "thoughtGraphMermaid")]
    pub thought_graph_mermaid: String,
    #[serde(rename = "confidenceHistory")]
    pub confidence_history: Vec<Option<f64>>,
    #[serde(rename = "leftToBeDone")]
    pub left_to_be_done: Vec<String>,
}
