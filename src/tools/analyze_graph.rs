use super::McpTool;
use crate::server::SequentialThinkingServer;
use crate::types::ThoughtData;
use serde_json::{json, Value};
use std::collections::HashSet;

pub struct AnalyzeGraphTool;

impl McpTool for AnalyzeGraphTool {
    fn name(&self) -> &str {
        "analyze_graph"
    }

    fn description(&self) -> &str {
        "Query and analyze the thought graph of a thinking session without adding new thoughts. Supports querying low confidence thoughts, contradictions, unverified assumptions, dead branches, and summary statistics."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "enum": ["low_confidence", "contradictions", "unverified_assumptions", "dead_branches", "summary_stats", "quality_report"],
                    "description": "The type of analysis/query to run against the thought graph"
                },
                "confidenceThreshold": {
                    "type": "number",
                    "default": 0.5,
                    "description": "Confidence threshold to filter low confidence thoughts (default is 0.5)"
                },
                "sessionId": {
                    "type": "string",
                    "description": "The session identifier to analyze (defaults to the current active session)"
                }
            },
            "required": ["query"]
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

        let query = arguments["query"].as_str().ok_or("Missing query parameter")?;

        match query {
            "low_confidence" => {
                let threshold = arguments["confidenceThreshold"].as_f64().unwrap_or(0.5);
                let low_conf: Vec<ThoughtData> = server.thought_history.iter()
                    .filter(|t| t.confidence_score.map(|c| c <= threshold).unwrap_or(false))
                    .cloned()
                    .collect();
                Ok(json!(low_conf))
            }
            "contradictions" => {
                let mut assumed = HashSet::new();
                let mut refuted = HashSet::new();

                for t in &server.thought_history {
                    if let Some(ref ass) = t.assumptions {
                        for a in ass {
                            assumed.insert(a.trim().to_lowercase());
                        }
                    }
                    if let Some(ref ver) = t.verified_assumptions {
                        for v in ver {
                            let v_clean = v.trim().to_lowercase();
                            if v_clean.contains("refuted") || v_clean.contains("false") || v_clean.contains("falsified") {
                                let core = v_clean
                                    .replace("refuted:", "")
                                    .replace("refuted", "")
                                    .replace("false:", "")
                                    .replace("false", "")
                                    .replace("falsified:", "")
                                    .replace("falsified", "")
                                    .trim()
                                    .to_string();
                                refuted.insert(core);
                            }
                        }
                    }
                }

                let contradictions: Vec<String> = assumed.intersection(&refuted)
                    .map(|s| format!("Assumption '{}' is assumed but has been refuted/falsified.", s))
                    .collect();

                Ok(json!(contradictions))
            }
            "unverified_assumptions" => {
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

                let unverified: Vec<String> = assumed.into_iter()
                    .filter(|a| !verified.contains(a))
                    .collect();

                Ok(json!(unverified))
            }
            "dead_branches" => {
                if server.thought_history.is_empty() {
                    return Ok(json!([]));
                }

                let final_thought = &server.thought_history[server.thought_history.len() - 1];
                let mut main_chain = HashSet::new();
                let mut queue = vec![final_thought.thought_number];

                while let Some(tn) = queue.pop() {
                    if main_chain.insert(tn) {
                        if let Some(t) = server.thought_history.iter().find(|x| x.thought_number == tn) {
                            if let Some(ref parents) = t.parent_thoughts {
                                queue.extend(parents.iter().copied());
                            }
                            if let Some(branch_from) = t.branch_from_thought {
                                queue.push(branch_from);
                            }
                            if let Some(revises) = t.revises_thought {
                                queue.push(revises);
                            }
                            if t.parent_thoughts.is_none() && t.branch_from_thought.is_none() && !t.is_revision.unwrap_or(false) && t.thought_number > 1 {
                                queue.push(t.thought_number - 1);
                            }
                        }
                    }
                }

                let dead: Vec<ThoughtData> = server.thought_history.iter()
                    .filter(|t| !main_chain.contains(&t.thought_number))
                    .cloned()
                    .collect();

                Ok(json!(dead))
            }
            "summary_stats" => {
                let total_thoughts = server.thought_history.len();
                let confidences: Vec<f64> = server.thought_history.iter().filter_map(|t| t.confidence_score).collect();
                let avg_confidence = if confidences.is_empty() { 0.0 } else { confidences.iter().sum::<f64>() / confidences.len() as f64 };
                let branches_count = server.branches.len();

                let mut total_assumptions = 0;
                let mut total_verified = 0;
                for t in &server.thought_history {
                    if let Some(ref ass) = t.assumptions {
                        total_assumptions += ass.len();
                    }
                    if let Some(ref ver) = t.verified_assumptions {
                        total_verified += ver.len();
                    }
                }

                let report = crate::graph::quality::calculate_quality(&session_id, &server.thought_history);

                Ok(json!({
                    "sessionId": session_id,
                    "totalThoughts": total_thoughts,
                    "averageConfidence": avg_confidence,
                    "branchesCount": branches_count,
                    "totalAssumptions": total_assumptions,
                    "totalVerifiedAssumptions": total_verified,
                    "qualityScore": report.quality_score,
                    "grade": report.grade,
                }))
            }
            "quality_report" => {
                let report = crate::graph::quality::calculate_quality(&session_id, &server.thought_history);
                Ok(json!(report))
            }
            _ => Err(format!("Unknown query type: {}", query)),
        }
    }
}
