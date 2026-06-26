use super::McpTool;
use crate::server::SequentialThinkingServer;
use serde_json::{json, Value};
use std::collections::HashSet;

pub struct SummarizeReasoningTool;

impl McpTool for SummarizeReasoningTool {
    fn name(&self) -> &str {
        "summarize_reasoning"
    }

    fn description(&self) -> &str {
        "Retrieve a structured summary and timeline of the reasoning chain for the specified session."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "sessionId": {
                    "type": "string",
                    "description": "The session identifier to summarize (defaults to the current active session)"
                }
            }
        })
    }

    fn execute(
        &self,
        server: &mut SequentialThinkingServer,
        arguments: Value,
    ) -> Result<Value, String> {
        let session_id = arguments["sessionId"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| server.current_session_id.clone());

        if session_id.is_empty() {
            return Err("No active session and no sessionId provided".to_string());
        }

        server.load_session(&session_id)?;

        let total_thoughts = server.thought_history.len();
        let total_branches = server.branches.len();

        // Calculate average confidence
        let confidences: Vec<f64> = server.thought_history.iter()
            .filter_map(|t| t.confidence_score)
            .collect();
        let average_confidence = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f64>() / confidences.len() as f64
        };

        // Identify merge points
        let mut merge_points = Vec::new();
        for t in &server.thought_history {
            if let Some(ref parents) = t.parent_thoughts {
                if parents.len() > 1 {
                    merge_points.push(t.thought_number);
                }
            }
        }

        // Get unverified assumptions
        let mut assumed = HashSet::new();
        let mut verified = HashSet::new();
        for t in &server.thought_history {
            if let Some(ref ass) = t.assumptions {
                for a in ass {
                    assumed.insert(a.clone());
                }
            }
            if let Some(ref ver) = t.verified_assumptions {
                for v in ver {
                    let v_clean = v.replace("verified:", "")
                        .replace("refuted:", "")
                        .replace("false:", "")
                        .trim()
                        .to_string();
                    verified.insert(v_clean);
                    verified.insert(v.clone());
                }
            }
        }
        let unverified_assumptions: Vec<String> = assumed.into_iter()
            .filter(|a| !verified.contains(a))
            .collect();

        // Open todos from the final thought (if any)
        let open_todos = if let Some(last_thought) = server.thought_history.last() {
            last_thought.left_to_be_done.clone().unwrap_or_default()
        } else {
            Vec::new()
        };

        // Construct timeline
        let mut parts = Vec::new();
        for t in &server.thought_history {
            let mut part = format!("T{}", t.thought_number);
            if let Some(branch_from) = t.branch_from_thought {
                let b_id = t.branch_id.as_deref().unwrap_or("unknown");
                part = format!("{}(branch:{}, from:T{})", part, b_id, branch_from);
            } else if let Some(ref parents) = t.parent_thoughts {
                if parents.len() > 1 {
                    let p_str = parents.iter().map(|p| format!("T{}", p)).collect::<Vec<String>>().join("+");
                    part = format!("{}(merge:{})", part, p_str);
                }
            } else if t.is_revision.unwrap_or(false) {
                let rev = t.revises_thought.unwrap_or(0);
                part = format!("{}(revises:T{})", part, rev);
            }
            parts.push(part);
        }
        let timeline = parts.join(" → ");

        Ok(json!({
            "sessionId": session_id,
            "totalThoughts": total_thoughts,
            "totalBranches": total_branches,
            "mergePoints": merge_points,
            "averageConfidence": average_confidence,
            "unverifiedAssumptions": unverified_assumptions,
            "openTodos": open_todos,
            "timeline": timeline,
        }))
    }
}
