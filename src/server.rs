use crate::types::{ThoughtData, ToolResult};
use colored::Colorize;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};
use textwrap::wrap;
use tracing::instrument;

pub struct SequentialThinkingServer {
    pub thought_history: Vec<ThoughtData>,
    pub branches: HashMap<String, Vec<ThoughtData>>,
    pub disable_thought_logging: bool,
}

impl SequentialThinkingServer {
    pub fn new(disable_thought_logging: bool) -> Self {
        Self {
            thought_history: Vec::new(),
            branches: HashMap::new(),
            disable_thought_logging,
        }
    }

    fn get_terminal_width(&self) -> usize {
        if let Some((Width(w), _)) = terminal_size() {
            let width = w as usize;
            if width < 40 {
                40
            } else if width > 100 {
                100
            } else {
                width
            }
        } else {
            80
        }
    }

    #[allow(unused_assignments)]
    #[instrument(skip(self, thought_data))]
    fn format_thought(&self, thought_data: &ThoughtData) -> String {
        let thought_number = thought_data.thought_number;
        let total_thoughts = thought_data.total_thoughts;
        let thought = &thought_data.thought;

        let prefix;
        let context;
        let mut color_func: Box<dyn Fn(&str) -> colored::ColoredString> = Box::new(|s| s.blue());

        if thought_data.is_revision.unwrap_or(false) {
            prefix = "🔄 Revision".yellow().bold().to_string();
            let rev = thought_data.revises_thought.unwrap_or(0);
            context = format!(" (revising thought {})", rev);
            color_func = Box::new(|s| s.yellow());
        } else if let Some(branch_from) = thought_data.branch_from_thought {
            prefix = "🌿 Branch".green().bold().to_string();
            let branch_id = thought_data.branch_id.as_deref().unwrap_or("unknown");
            context = format!(" (from thought {}, ID: {})", branch_from, branch_id);
            color_func = Box::new(|s| s.green());
        } else {
            prefix = "💭 Thought".blue().bold().to_string();
            context = "".to_string();
        }

        let mut time_str = "".to_string();
        if let Some(ts) = thought_data.timestamp {
            time_str = format!(" @ {}", ts.format("%H:%M:%S"));
        }

        let header_plain = format!(
            "{} {}/{}{}{}",
            if thought_data.is_revision.unwrap_or(false) {
                "Revision"
            } else if thought_data.branch_from_thought.is_some() {
                "Branch"
            } else {
                "Thought"
            },
            thought_number,
            total_thoughts,
            context,
            time_str
        );

        let header_styled = format!(
            "{} {}/{}{}{}",
            prefix,
            thought_number,
            total_thoughts,
            context.cyan(),
            time_str.dimmed()
        );

        let term_width = self.get_terminal_width();
        let border_len = term_width - 4; // Subtract borders and spacing

        // Wrap thought text to fit border length
        let wrapped_lines = wrap(thought, border_len);

        let border_char = "─";
        let top_border = color_func(&format!("┌{}┐", border_char.repeat(border_len + 2)));
        let mid_border = color_func(&format!("├{}┤", border_char.repeat(border_len + 2)));
        let bot_border = color_func(&format!("└{}┘", border_char.repeat(border_len + 2)));

        let mut lines = Vec::new();
        lines.push(top_border.to_string());

        // Pad header line with color func
        let header_padding = (border_len + 2).saturating_sub(header_plain.len());
        lines.push(format!(
            "{} {}{} {}",
            color_func("│"),
            header_styled,
            " ".repeat(header_padding),
            color_func("│")
        ));

        // Add metadata if present (confidence, assumptions, hypothesis)
        let mut metadata_added = false;

        if let Some(conf) = thought_data.confidence_score {
            let stars_count = (conf * 5.0).round() as usize;
            let stars = format!("{}{}", "★".repeat(stars_count), "☆".repeat(5 - stars_count));
            let conf_str = format!("Confidence: {} ({:.0}%)", stars.yellow(), conf * 100.0);
            let plain_conf = format!(
                "Confidence: {} ({:.0}%)",
                "★".repeat(stars_count) + &"☆".repeat(5 - stars_count),
                conf * 100.0
            );
            let padding = (border_len + 2).saturating_sub(plain_conf.len());
            if !metadata_added {
                lines.push(mid_border.to_string());
                metadata_added = true;
            }
            lines.push(format!(
                "{} {}{} {}",
                color_func("│"),
                conf_str,
                " ".repeat(padding),
                color_func("│")
            ));
        }

        if let Some(ref assumptions) = thought_data.assumptions {
            if !assumptions.is_empty() {
                if !metadata_added {
                    lines.push(mid_border.to_string());
                    metadata_added = true;
                }
                for assumption in assumptions {
                    let text = format!(" 🤔 Assumption: {}", assumption);
                    let wrapped = wrap(&text, border_len);
                    for w_line in wrapped {
                        let padding = (border_len + 2).saturating_sub(w_line.len());
                        lines.push(format!(
                            "{} {}{} {}",
                            color_func("│"),
                            w_line.magenta(),
                            " ".repeat(padding),
                            color_func("│")
                        ));
                    }
                }
            }
        }

        if let Some(ref verified) = thought_data.verified_assumptions {
            if !verified.is_empty() {
                if !metadata_added {
                    lines.push(mid_border.to_string());
                    metadata_added = true;
                }
                for assumption in verified {
                    let text = format!(" ✅ Verified: {}", assumption);
                    let wrapped = wrap(&text, border_len);
                    for w_line in wrapped {
                        let padding = (border_len + 2).saturating_sub(w_line.len());
                        lines.push(format!(
                            "{} {}{} {}",
                            color_func("│"),
                            w_line.green(),
                            " ".repeat(padding),
                            color_func("│")
                        ));
                    }
                }
            }
        }

        if let Some(ref criticism) = thought_data.criticism {
            if !metadata_added {
                lines.push(mid_border.to_string());
                metadata_added = true;
            }
            let text = format!(" 🧐 Criticism: {}", criticism);
            let wrapped = wrap(&text, border_len);
            for w_line in wrapped {
                let padding = (border_len + 2).saturating_sub(w_line.len());
                lines.push(format!(
                    "{} {}{} {}",
                    color_func("│"),
                    w_line.red(),
                    " ".repeat(padding),
                    color_func("│")
                ));
            }
        }

        if let Some(ref hypothesis) = thought_data.hypothesis {
            if !metadata_added {
                lines.push(mid_border.to_string());
                metadata_added = true;
            }
            let text = format!(" 🔬 Hypothesis: {}", hypothesis);
            let wrapped = wrap(&text, border_len);
            for w_line in wrapped {
                let padding = (border_len + 2).saturating_sub(w_line.len());
                lines.push(format!(
                    "{} {}{} {}",
                    color_func("│"),
                    w_line.cyan(),
                    " ".repeat(padding),
                    color_func("│")
                ));
            }
        }

        if let Some(ref verification_method) = thought_data.verification_method {
            if !metadata_added {
                lines.push(mid_border.to_string());
                metadata_added = true;
            }
            let text = format!(" 🧪 Verification: {}", verification_method);
            let wrapped = wrap(&text, border_len);
            for w_line in wrapped {
                let padding = (border_len + 2).saturating_sub(w_line.len());
                lines.push(format!(
                    "{} {}{} {}",
                    color_func("│"),
                    w_line.cyan(),
                    " ".repeat(padding),
                    color_func("│")
                ));
            }
        }

        if let Some(ref left_to_be_done) = thought_data.left_to_be_done {
            if !left_to_be_done.is_empty() {
                if !metadata_added {
                    lines.push(mid_border.to_string());
                    metadata_added = true;
                }
                for todo in left_to_be_done {
                    let text = format!(" 📋 TODO: {}", todo);
                    let wrapped = wrap(&text, border_len);
                    for w_line in wrapped {
                        let padding = (border_len + 2).saturating_sub(w_line.len());
                        lines.push(format!(
                            "{} {}{} {}",
                            color_func("│"),
                            w_line.yellow(),
                            " ".repeat(padding),
                            color_func("│")
                        ));
                    }
                }
            }
        }

        lines.push(mid_border.to_string());

        for line in wrapped_lines {
            let padding = (border_len + 2).saturating_sub(line.len());
            lines.push(format!(
                "{} {}{} {}",
                color_func("│"),
                line,
                " ".repeat(padding),
                color_func("│")
            ));
        }

        lines.push(bot_border.to_string());
        lines.join("\n")
    }

    #[instrument(skip(self))]
    fn generate_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TD\n");
        mermaid.push_str(
            "    classDef revision fill:#fafd7c,stroke:#d4b200,stroke-width:2px,color:#000;\n",
        );
        mermaid.push_str(
            "    classDef branch fill:#a1e887,stroke:#3b7a14,stroke-width:2px,color:#000;\n",
        );
        mermaid.push_str(
            "    classDef hypothesis fill:#d1b3ff,stroke:#6a3d9a,stroke-width:2px,color:#000;\n",
        );
        mermaid.push_str(
            "    classDef standard fill:#a5ccf7,stroke:#265c96,stroke-width:2px,color:#000;\n\n",
        );

        for (i, t) in self.thought_history.iter().enumerate() {
            let id = format!("T{}", t.thought_number);

            // Clean up text preview for Mermaid label
            let mut preview = t.thought.chars().take(30).collect::<String>();
            preview = preview
                .replace('\"', "'")
                .replace('[', "(")
                .replace(']', ")");
            if t.thought.len() > 30 {
                preview.push_str("...");
            }

            let class = if t.is_revision.unwrap_or(false) {
                "revision"
            } else if t.branch_from_thought.is_some() {
                "branch"
            } else if t.hypothesis.is_some() {
                "hypothesis"
            } else {
                "standard"
            };

            mermaid.push_str(&format!(
                "    {}[\"T{}: {}\"]\n",
                id, t.thought_number, preview
            ));
            mermaid.push_str(&format!("    class {} {}\n", id, class));

            // Link tracing
            if let Some(ref parents) = t.parent_thoughts {
                if !parents.is_empty() {
                    for parent in parents {
                        mermaid.push_str(&format!("    T{} --> {}\n", parent, id));
                    }
                    continue;
                }
            }

            if let Some(branch_from) = t.branch_from_thought {
                mermaid.push_str(&format!("    T{} --> {}\n", branch_from, id));
            } else if t.is_revision.unwrap_or(false) {
                if let Some(revises) = t.revises_thought {
                    mermaid.push_str(&format!("    T{} -.->|revises| {}\n", revises, id));
                }
            } else if i > 0 {
                let prev_id = format!("T{}", self.thought_history[i - 1].thought_number);
                mermaid.push_str(&format!("    {} --> {}\n", prev_id, id));
            }
        }

        mermaid
    }

    #[instrument(skip(self))]
    pub fn process_thought(&mut self, mut input: ThoughtData) -> Result<ToolResult, String> {
        if input.thought_number > input.total_thoughts {
            input.total_thoughts = input.thought_number;
        }

        // Auto-populate timestamp if not provided
        if input.timestamp.is_none() {
            input.timestamp = Some(chrono::Utc::now());
        }

        // Add to branch map if branch parameters are specified
        if let (Some(_branch_from), Some(branch_id)) =
            (input.branch_from_thought, input.branch_id.as_ref())
        {
            self.branches
                .entry(branch_id.clone())
                .or_insert_with(Vec::new)
                .push(input.clone());
        }

        if !self.disable_thought_logging {
            let formatted = self.format_thought(&input);
            tracing::info!(target: "thought_tui", "\n{}", formatted);
        }

        let thought_number = input.thought_number;
        let total_thoughts = input.total_thoughts;
        let next_thought_needed = input.next_thought_needed;
        let left_to_be_done = input.left_to_be_done.clone().unwrap_or_default();

        self.thought_history.push(input);

        let branches = self.branches.keys().cloned().collect::<Vec<String>>();
        let confidence_history = self
            .thought_history
            .iter()
            .map(|t| t.confidence_score)
            .collect::<Vec<Option<f64>>>();
        let thought_graph_mermaid = self.generate_mermaid();

        Ok(ToolResult {
            thought_number,
            total_thoughts,
            next_thought_needed,
            branches,
            thought_history_length: self.thought_history.len(),
            thought_graph_mermaid,
            confidence_history,
            left_to_be_done,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_thought() {
        let mut server = SequentialThinkingServer::new(true);
        let input = ThoughtData {
            thought: "First thought".to_string(),
            thought_number: 1,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        let result = server.process_thought(input).unwrap();
        assert_eq!(result.thought_number, 1);
        assert_eq!(result.total_thoughts, 3);
        assert_eq!(result.next_thought_needed, true);
        assert_eq!(result.thought_history_length, 1);
    }

    #[test]
    fn test_auto_adjust_total_thoughts() {
        let mut server = SequentialThinkingServer::new(true);
        let input = ThoughtData {
            thought: "Future thought".to_string(),
            thought_number: 5,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        let result = server.process_thought(input).unwrap();
        assert_eq!(result.total_thoughts, 5);
    }

    #[test]
    fn test_branching() {
        let mut server = SequentialThinkingServer::new(true);

        let input1 = ThoughtData {
            thought: "Main line".to_string(),
            thought_number: 1,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        let input2 = ThoughtData {
            thought: "Branch line".to_string(),
            thought_number: 2,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: Some(1),
            branch_id: Some("branch-a".to_string()),
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        server.process_thought(input1).unwrap();
        let result = server.process_thought(input2).unwrap();

        assert_eq!(result.branches.len(), 1);
        assert!(result.branches.contains(&"branch-a".to_string()));
        assert!(result.thought_graph_mermaid.contains("T1 --> T2"));
    }

    #[test]
    fn test_mermaid_generation_with_got_parent() {
        let mut server = SequentialThinkingServer::new(true);

        let input1 = ThoughtData {
            thought: "Idea A".to_string(),
            thought_number: 1,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        let input2 = ThoughtData {
            thought: "Idea B".to_string(),
            thought_number: 2,
            total_thoughts: 3,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        let input3 = ThoughtData {
            thought: "Merge A and B".to_string(),
            thought_number: 3,
            total_thoughts: 3,
            next_thought_needed: false,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: Some(vec![1, 2]),
            assumptions: None,
            verified_assumptions: None,
            confidence_score: None,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
        };

        server.process_thought(input1).unwrap();
        server.process_thought(input2).unwrap();
        let result = server.process_thought(input3).unwrap();

        assert!(result.thought_graph_mermaid.contains("T1 --> T3"));
        assert!(result.thought_graph_mermaid.contains("T2 --> T3"));
    }

    #[test]
    fn test_new_fields() {
        let mut server = SequentialThinkingServer::new(true);

        let input = ThoughtData {
            thought: "Checking new fields".to_string(),
            thought_number: 1,
            total_thoughts: 1,
            next_thought_needed: false,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: Some(vec!["A1".to_string()]),
            verified_assumptions: Some(vec!["V1".to_string()]),
            confidence_score: Some(0.8),
            criticism: Some("C1".to_string()),
            hypothesis: Some("H1".to_string()),
            verification_method: Some("VM1".to_string()),
            left_to_be_done: Some(vec!["Todo1".to_string()]),
            timestamp: None,
        };

        let result = server.process_thought(input).unwrap();
        assert_eq!(result.left_to_be_done.len(), 1);
        assert_eq!(result.left_to_be_done[0], "Todo1");

        // Verify auto timestamp population
        let thought = &server.thought_history[0];
        assert!(thought.timestamp.is_some());
    }
}
