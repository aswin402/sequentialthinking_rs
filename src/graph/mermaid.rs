use crate::types::ThoughtData;

/// Generates a Mermaid DAG diagram from a list of thoughts.
pub fn generate_mermaid(thoughts: &[ThoughtData]) -> String {
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

    for (i, t) in thoughts.iter().enumerate() {
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
            let prev_id = format!("T{}", thoughts[i - 1].thought_number);
            mermaid.push_str(&format!("    {} --> {}\n", prev_id, id));
        }
    }

    mermaid
}
