use super::McpTool;
use crate::server::SequentialThinkingServer;
use crate::types::ThoughtData;
use serde_json::json;

pub struct SequentialThinkingTool;

impl McpTool for SequentialThinkingTool {
    fn name(&self) -> &str {
        "sequentialthinking"
    }

    fn description(&self) -> &str {
        "A detailed tool for dynamic and reflective problem-solving through thoughts. Supports branching, revisions, Graph of Thoughts (GoT) merging, and Clear Thought parameters."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
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
                },
                "sessionId": {
                    "type": "string",
                    "description": "Unique identifier for the current thinking session"
                }
            },
            "required": ["thought", "nextThoughtNeeded", "thoughtNumber", "totalThoughts"]
        })
    }

    fn execute(
        &self,
        server: &mut SequentialThinkingServer,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let thought_data: ThoughtData = serde_json::from_value(arguments)
            .map_err(|e| format!("Invalid arguments: {}", e))?;

        let result = server.process_thought(thought_data)?;
        Ok(serde_json::to_value(result).unwrap_or(serde_json::Value::Null))
    }
}
