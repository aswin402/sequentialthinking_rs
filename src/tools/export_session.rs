use super::McpTool;
use crate::server::SequentialThinkingServer;
use serde_json::{json, Value};

pub struct ExportSessionTool;

impl McpTool for ExportSessionTool {
    fn name(&self) -> &str {
        "export_session"
    }

    fn description(&self) -> &str {
        "Export the reasoning session in various formats: mermaid graph, JSON Graph, markdown report, or Graphviz DOT format."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["mermaid", "json", "markdown", "dot"],
                    "description": "The target export format"
                },
                "sessionId": {
                    "type": "string",
                    "description": "The session identifier to export (defaults to the current active session)"
                }
            },
            "required": ["format"]
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

        let format = arguments["format"].as_str().ok_or("Missing format parameter")?;

        match format {
            "mermaid" => {
                let mermaid_graph = server.generate_mermaid();
                Ok(json!({ "format": "mermaid", "sessionId": session_id, "data": mermaid_graph }))
            }
            "json" => {
                let mut nodes = Vec::new();
                let mut edges = Vec::new();
                for (i, t) in server.thought_history.iter().enumerate() {
                    nodes.push(json!({
                        "id": format!("T{}", t.thought_number),
                        "thoughtNumber": t.thought_number,
                        "thought": t.thought,
                        "confidenceScore": t.confidence_score,
                        "timestamp": t.timestamp,
                    }));

                    if let Some(ref parents) = t.parent_thoughts {
                        for parent in parents {
                            edges.push(json!({ "source": format!("T{}", parent), "target": format!("T{}", t.thought_number), "type": "parent" }));
                        }
                        continue;
                    }
                    if let Some(branch_from) = t.branch_from_thought {
                        edges.push(json!({ "source": format!("T{}", branch_from), "target": format!("T{}", t.thought_number), "type": "branch" }));
                    } else if t.is_revision.unwrap_or(false) {
                        if let Some(revises) = t.revises_thought {
                            edges.push(json!({ "source": format!("T{}", revises), "target": format!("T{}", t.thought_number), "type": "revision" }));
                        }
                    } else if i > 0 {
                        let prev = server.thought_history[i - 1].thought_number;
                        edges.push(json!({ "source": format!("T{}", prev), "target": format!("T{}", t.thought_number), "type": "standard" }));
                    }
                }
                let json_graph = json!({ "nodes": nodes, "edges": edges });
                Ok(json!({ "format": "json", "sessionId": session_id, "data": json_graph }))
            }
            "markdown" => {
                let mut md = String::new();
                md.push_str(&format!("# Reasoning Session History - Session `{}`\n\n", session_id));
                for t in &server.thought_history {
                    let kind = if t.is_revision.unwrap_or(false) { "Revision" } else if t.branch_from_thought.is_some() { "Branch" } else { "Thought" };
                    md.push_str(&format!("## {} {}\n", kind, t.thought_number));
                    if let Some(ts) = t.timestamp {
                        md.push_str(&format!("*Timestamp: {}*\n\n", ts.format("%Y-%m-%d %H:%M:%S UTC")));
                    }
                    md.push_str(&format!("{}\n\n", t.thought));
                    if let Some(ref assumptions) = t.assumptions {
                        if !assumptions.is_empty() {
                            md.push_str("### Assumptions\n");
                            for a in assumptions {
                                md.push_str(&format!("- 🤔 {}\n", a));
                            }
                            md.push_str("\n");
                        }
                    }
                    if let Some(ref verified) = t.verified_assumptions {
                        if !verified.is_empty() {
                            md.push_str("### Verified Assumptions\n");
                            for v in verified {
                                md.push_str(&format!("- ✅ {}\n", v));
                            }
                            md.push_str("\n");
                        }
                    }
                    if let Some(conf) = t.confidence_score {
                        md.push_str(&format!("*Confidence Score: {}/5 ({:.0}%)*\n\n", (conf * 5.0).round(), conf * 100.0));
                    }
                    if let Some(ref criticism) = t.criticism {
                        md.push_str(&format!("> **🧐 Self-Criticism:** {}\n\n", criticism));
                    }
                    if let Some(ref hypothesis) = t.hypothesis {
                        md.push_str(&format!("> **🔬 Hypothesis:** {}\n\n", hypothesis));
                    }
                    if let Some(ref verification) = t.verification_method {
                        md.push_str(&format!("> **🧪 Verification:** {}\n\n", verification));
                    }
                    md.push_str("---\n\n");
                }
                Ok(json!({ "format": "markdown", "sessionId": session_id, "data": md }))
            }
            "dot" => {
                let mut dot = String::from("digraph G {\n");
                dot.push_str("  node [shape=box, style=filled, fontname=\"Arial\"];\n");
                for (i, t) in server.thought_history.iter().enumerate() {
                    let id = format!("T{}", t.thought_number);
                    let clean_thought = t.thought.replace('\"', "\\\"");
                    let preview = clean_thought.chars().take(20).collect::<String>();
                    let label = format!("T{}: {}\\n...", t.thought_number, preview);

                    let color = if t.is_revision.unwrap_or(false) {
                        "\"#fafd7c\""
                    } else if t.branch_from_thought.is_some() {
                        "\"#a1e887\""
                    } else {
                        "\"#a5ccf7\""
                    };
                    dot.push_str(&format!("  {} [label=\"{}\", fillcolor={}];\n", id, label, color));

                    if let Some(ref parents) = t.parent_thoughts {
                        for parent in parents {
                            dot.push_str(&format!("  T{} -> {};\n", parent, id));
                        }
                        continue;
                    }
                    if let Some(branch_from) = t.branch_from_thought {
                        dot.push_str(&format!("  T{} -> {};\n", branch_from, id));
                    } else if t.is_revision.unwrap_or(false) {
                        if let Some(revises) = t.revises_thought {
                            dot.push_str(&format!("  T{} -> {} [style=dotted, label=\"revises\"];\n", revises, id));
                        }
                    } else if i > 0 {
                        let prev = server.thought_history[i - 1].thought_number;
                        dot.push_str(&format!("  T{} -> {};\n", prev, id));
                    }
                }
                dot.push_str("}\n");
                Ok(json!({ "format": "dot", "sessionId": session_id, "data": dot }))
            }
            _ => Err(format!("Unknown format: {}", format)),
        }
    }
}
