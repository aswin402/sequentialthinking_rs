use super::McpTool;
use crate::server::SequentialThinkingServer;
use serde_json::{json, Value};

pub struct TemplatesTool;

impl McpTool for TemplatesTool {
    fn name(&self) -> &str {
        "reasoning_templates"
    }

    fn description(&self) -> &str {
        "Retrieve pre-structured reasoning templates to guide complex thinking processes. Includes templates for divide-and-conquer, hypothesis testing, and devil's advocate reasoning."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "enum": ["divide-and-conquer", "hypothesis-test", "devils-advocate", "all"],
                    "default": "all",
                    "description": "The reasoning template to retrieve (default is 'all')"
                }
            }
        })
    }

    fn execute(
        &self,
        _server: &mut SequentialThinkingServer,
        arguments: Value,
    ) -> Result<Value, String> {
        let template_name = arguments["template"]
            .as_str()
            .unwrap_or("all");

        let divide_and_conquer = json!({
            "name": "Divide and Conquer",
            "id": "divide-and-conquer",
            "description": "Decompose a large, complex problem into smaller, independent sub-problems. Solve each sub-problem individually, and then synthesize the results into a unified solution.",
            "recommendedSteps": [
                {
                    "step": 1,
                    "title": "Problem Scope & Boundary Analysis",
                    "description": "Clearly define the problem statement, identify inputs and outputs, and specify constraints/assumptions.",
                    "propertiesToSet": ["assumptions"]
                },
                {
                    "step": 2,
                    "title": "Decomposition Strategy",
                    "description": "Divide the main problem into smaller, non-overlapping sub-problems. Formulate a hypothesis for how they will combine.",
                    "propertiesToSet": ["hypothesis"]
                },
                {
                    "step": 3,
                    "title": "Sub-problem Exploration & Branching",
                    "description": "Spawn a branch for each sub-problem to solve them in isolation. (Use branchId and branchFromThought).",
                    "propertiesToSet": ["branchId", "branchFromThought"]
                },
                {
                    "step": 4,
                    "title": "Synthesis & Solution Merge",
                    "description": "Merge the branches and synthesize their results into the main sequence. Validate correctness of the merged solution.",
                    "propertiesToSet": ["parentThoughts", "verifiedAssumptions"]
                }
            ]
        });

        let hypothesis_test = json!({
            "name": "Hypothesis Testing",
            "id": "hypothesis-test",
            "description": "Establish a testable hypothesis, identify the underlying assumptions, design verification methods, collect evidence, and evaluate/refute the hypothesis.",
            "recommendedSteps": [
                {
                    "step": 1,
                    "title": "Hypothesis Formulation",
                    "description": "Define a clear, testable, and falsifiable hypothesis for the problem or bug root cause.",
                    "propertiesToSet": ["hypothesis", "verificationMethod"]
                },
                {
                    "step": 2,
                    "title": "Assumption Mapping",
                    "description": "List all assumptions that must hold true for the hypothesis to be valid.",
                    "propertiesToSet": ["assumptions"]
                },
                {
                    "step": 3,
                    "title": "Evidence Gathering & Verification",
                    "description": "Verify each assumption using the defined verification method. Log findings clearly.",
                    "propertiesToSet": ["verifiedAssumptions", "confidenceScore"]
                },
                {
                    "step": 4,
                    "title": "Synthesis / Backtracking",
                    "description": "Confirm or refute the hypothesis. If refuted, spawn a revision thought to form a new hypothesis.",
                    "propertiesToSet": ["isRevision", "revisesThought", "criticism"]
                }
            ]
        });

        let devils_advocate = json!({
            "name": "Devil's Advocate",
            "id": "devils-advocate",
            "description": "Identify cognitive biases, challenge dominant assumptions, actively look for edge cases and failure modes, and refine solutions to withstand criticism.",
            "recommendedSteps": [
                {
                    "step": 1,
                    "title": "Proposed Solution Formulation",
                    "description": "State the current preferred solution or consensus approach clearly.",
                    "propertiesToSet": ["thought"]
                },
                {
                    "step": 2,
                    "title": "Assumption Enumeration",
                    "description": "List every assumption supporting the proposed solution.",
                    "propertiesToSet": ["assumptions"]
                },
                {
                    "step": 3,
                    "title": "Adversarial Challenge / Criticism",
                    "description": "Actively challenge each assumption. What happens if an assumption is false? Describe worst-case edge cases and failure modes.",
                    "propertiesToSet": ["criticism"]
                },
                {
                    "step": 4,
                    "title": "Solution Hardening & Revision",
                    "description": "Revise the proposed solution to address the criticisms, add mitigations, and establish robust safety margins.",
                    "propertiesToSet": ["isRevision", "revisesThought", "leftToBeDone"]
                }
            ]
        });

        match template_name {
            "divide-and-conquer" => Ok(divide_and_conquer),
            "hypothesis-test" => Ok(hypothesis_test),
            "devils-advocate" => Ok(devils_advocate),
            "all" | _ => Ok(json!({
                "templates": [
                    divide_and_conquer,
                    hypothesis_test,
                    devils_advocate
                ]
            })),
        }
    }
}
