pub mod sequentialthinking;
pub mod analyze_graph;
pub mod export_session;
pub mod summarize;
pub mod templates;

use crate::server::SequentialThinkingServer;
use std::collections::HashMap;

pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn execute(
        &self,
        server: &mut SequentialThinkingServer,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn McpTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn McpTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn McpTool>> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<serde_json::Value> {
        let mut list = self.tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "inputSchema": t.input_schema(),
                })
            })
            .collect::<Vec<serde_json::Value>>();
        
        // Sort by tool name to keep list order deterministic
        list.sort_by(|a, b| {
            a["name"].as_str().unwrap_or("").cmp(b["name"].as_str().unwrap_or(""))
        });
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::memory::MemoryThoughtStore;
    use crate::types::ThoughtData;

    fn setup_server_with_data() -> SequentialThinkingServer {
        let store = Box::new(MemoryThoughtStore::new());
        let mut server = SequentialThinkingServer::new(store, true);

        // Process a few thoughts
        // T1: Standard thought with assumption A1
        server.process_thought(ThoughtData {
            thought: "Initial thought".to_string(),
            thought_number: 1,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: Some(vec!["A1".to_string()]),
            verified_assumptions: None,
            confidence_score: Some(0.8),
            criticism: None,
            hypothesis: Some("H1".to_string()),
            verification_method: Some("V1".to_string()),
            left_to_be_done: Some(vec!["Todo1".to_string()]),
            timestamp: None,
            session_id: Some("test-session".to_string()),
        }).unwrap();

        // T2: Low confidence thought branching from T1
        server.process_thought(ThoughtData {
            thought: "Branching thought".to_string(),
            thought_number: 2,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: Some(1),
            branch_id: Some("branch-a".to_string()),
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: Some(vec!["A2".to_string()]),
            verified_assumptions: Some(vec!["refuted: A1".to_string()]), // refutes A1
            confidence_score: Some(0.3), // low confidence
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
            session_id: Some("test-session".to_string()),
        }).unwrap();

        // T3: Revision of T1
        server.process_thought(ThoughtData {
            thought: "Revising first thought".to_string(),
            thought_number: 3,
            total_thoughts: 3,
            next_thought_needed: false,
            is_revision: Some(true),
            revises_thought: Some(1),
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: Some(0.9),
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
            session_id: Some("test-session".to_string()),
        }).unwrap();

        server
    }

    #[test]
    fn test_sequential_thinking_tool() {
        let store = Box::new(MemoryThoughtStore::new());
        let mut server = SequentialThinkingServer::new(store, true);
        let tool = sequentialthinking::SequentialThinkingTool;

        let args = serde_json::json!({
            "thought": "Direct tool thought",
            "thoughtNumber": 1,
            "totalThoughts": 1,
            "nextThoughtNeeded": false,
            "sessionId": "direct-session"
        });

        let res = tool.execute(&mut server, args).unwrap();
        assert_eq!(res["thoughtNumber"], 1);
        assert_eq!(res["sessionId"], "direct-session");
    }

    #[test]
    fn test_analyze_graph_tool() {
        let mut server = setup_server_with_data();
        let tool = analyze_graph::AnalyzeGraphTool;

        // Test low confidence
        let res = tool.execute(&mut server, serde_json::json!({
            "query": "low_confidence",
            "confidenceThreshold": 0.4,
            "sessionId": "test-session"
        })).unwrap();
        let list: Vec<serde_json::Value> = serde_json::from_value(res).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["thoughtNumber"], 2);

        // Test contradictions
        let res = tool.execute(&mut server, serde_json::json!({
            "query": "contradictions",
            "sessionId": "test-session"
        })).unwrap();
        let list: Vec<String> = serde_json::from_value(res).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].contains("a1"));

        // Test unverified assumptions
        let res = tool.execute(&mut server, serde_json::json!({
            "query": "unverified_assumptions",
            "sessionId": "test-session"
        })).unwrap();
        let list: Vec<String> = serde_json::from_value(res).unwrap();
        assert!(list.contains(&"A2".to_string()));
        assert!(!list.contains(&"A1".to_string()));

        // Test dead branches
        let res = tool.execute(&mut server, serde_json::json!({
            "query": "dead_branches",
            "sessionId": "test-session"
        })).unwrap();
        let list: Vec<serde_json::Value> = serde_json::from_value(res).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["thoughtNumber"], 2);

        // Test summary stats
        let res = tool.execute(&mut server, serde_json::json!({
            "query": "summary_stats",
            "sessionId": "test-session"
        })).unwrap();
        assert_eq!(res["totalThoughts"], 3);
        assert_eq!(res["branchesCount"], 1);
        assert!(res["qualityScore"].is_number());
        assert!(res["grade"].is_string());

        // Test quality report
        let res = tool.execute(&mut server, serde_json::json!({
            "query": "quality_report",
            "sessionId": "test-session"
        })).unwrap();
        assert_eq!(res["totalThoughts"], 3);
        assert!(res["qualityScore"].is_number());
        assert!(res["grade"].is_string());
    }

    #[test]
    fn test_export_session_tool() {
        let mut server = setup_server_with_data();
        let tool = export_session::ExportSessionTool;

        // Mermaid
        let res = tool.execute(&mut server, serde_json::json!({
            "format": "mermaid",
            "sessionId": "test-session"
        })).unwrap();
        assert!(res["data"].as_str().unwrap().contains("graph TD"));

        // Markdown
        let res = tool.execute(&mut server, serde_json::json!({
            "format": "markdown",
            "sessionId": "test-session"
        })).unwrap();
        assert!(res["data"].as_str().unwrap().contains("# Reasoning Session History"));

        // JSON Graph
        let res = tool.execute(&mut server, serde_json::json!({
            "format": "json",
            "sessionId": "test-session"
        })).unwrap();
        assert!(res["data"]["nodes"].is_array());
        assert!(res["data"]["edges"].is_array());

        // DOT
        let res = tool.execute(&mut server, serde_json::json!({
            "format": "dot",
            "sessionId": "test-session"
        })).unwrap();
        assert!(res["data"].as_str().unwrap().contains("digraph G"));
    }

    #[test]
    fn test_summarize_reasoning_tool() {
        let mut server = setup_server_with_data();
        let tool = summarize::SummarizeReasoningTool;

        let res = tool.execute(&mut server, serde_json::json!({
            "sessionId": "test-session"
        })).unwrap();
        assert_eq!(res["totalThoughts"], 3);
        assert_eq!(res["totalBranches"], 1);
        assert!(res["timeline"].as_str().unwrap().contains("T1 → T2"));
    }

    #[test]
    fn test_templates_tool() {
        let mut server = setup_server_with_data();
        let tool = templates::TemplatesTool;

        let res = tool.execute(&mut server, serde_json::json!({
            "template": "all"
        })).unwrap();
        assert!(res["templates"].is_array());
        assert_eq!(res["templates"].as_array().unwrap().len(), 3);

        let res = tool.execute(&mut server, serde_json::json!({
            "template": "hypothesis-test"
        })).unwrap();
        assert_eq!(res["id"], "hypothesis-test");
        assert!(res["recommendedSteps"].is_array());
    }
}
