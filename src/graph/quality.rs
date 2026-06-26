use std::collections::{HashMap, HashSet};
use crate::types::ThoughtData;

#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityReport {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "totalThoughts")]
    pub total_thoughts: usize,
    #[serde(rename = "averageConfidence")]
    pub average_confidence: f64,
    #[serde(rename = "assumptionsCount")]
    pub assumptions_count: usize,
    #[serde(rename = "verifiedAssumptionsCount")]
    pub verified_assumptions_count: usize,
    #[serde(rename = "verifiedAssumptionsRatio")]
    pub verified_assumptions_ratio: f64,
    #[serde(rename = "contradictionsCount")]
    pub contradictions_count: usize,
    pub contradictions: Vec<String>,
    #[serde(rename = "loopDetected")]
    pub loop_detected: bool,
    #[serde(rename = "loopPath")]
    pub loop_path: Option<Vec<usize>>,
    #[serde(rename = "qualityScore")]
    pub quality_score: f64,
    pub grade: String,
}

/// Detects cycles in the thought graph dependencies.
/// Returns the first detected cycle path of thought numbers.
pub fn detect_cycle(thoughts: &[ThoughtData]) -> Option<Vec<usize>> {
    let mut adj = HashMap::new();
    for t in thoughts {
        let mut deps = Vec::new();
        if let Some(ref parents) = t.parent_thoughts {
            deps.extend(parents.iter().copied());
        }
        if let Some(branch_from) = t.branch_from_thought {
            deps.push(branch_from);
        }
        if let Some(revises) = t.revises_thought {
            deps.push(revises);
        }
        adj.insert(t.thought_number, deps);
    }

    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for &node in adj.keys() {
        if !visited.contains(&node) {
            if let Some(cycle) = dfs_find_cycle(node, &adj, &mut visited, &mut rec_stack, &mut path) {
                return Some(cycle);
            }
        }
    }
    None
}

fn dfs_find_cycle(
    node: usize,
    adj: &HashMap<usize, Vec<usize>>,
    visited: &mut HashSet<usize>,
    rec_stack: &mut HashSet<usize>,
    path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Some(neighbors) = adj.get(&node) {
        for &neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if let Some(cycle) = dfs_find_cycle(neighbor, adj, visited, rec_stack, path) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(&neighbor) {
                if let Some(pos) = path.iter().position(|&x| x == neighbor) {
                    let mut cycle_path = path[pos..].to_vec();
                    cycle_path.push(neighbor); // Close the cycle loop
                    return Some(cycle_path);
                }
            }
        }
    }

    rec_stack.remove(&node);
    path.pop();
    None
}

/// Evaluates the thought graph and generates a detailed quality report.
pub fn calculate_quality(session_id: &str, thoughts: &[ThoughtData]) -> QualityReport {
    let total_thoughts = thoughts.len();

    // 1. Average confidence calculation
    let confidences: Vec<f64> = thoughts.iter().filter_map(|t| t.confidence_score).collect();
    let average_confidence = if confidences.is_empty() {
        0.75 // Default reasonable confidence if not specified
    } else {
        confidences.iter().sum::<f64>() / confidences.len() as f64
    };

    // 2. Assumptions counting and verification matching
    let mut assumed = HashSet::new();
    let mut verified = HashSet::new();
    let mut refuted = HashSet::new();

    for t in thoughts {
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
                } else {
                    let core = v_clean
                        .replace("verified:", "")
                        .replace("verified", "")
                        .replace("true:", "")
                        .replace("true", "")
                        .trim()
                        .to_string();
                    verified.insert(core);
                    // Also consider the raw string as verified
                    verified.insert(v_clean);
                }
            }
        }
    }

    let assumptions_count = assumed.len();
    let mut verified_assumptions_count = 0;
    for a in &assumed {
        if verified.contains(a) || refuted.contains(a) {
            verified_assumptions_count += 1;
        }
    }

    let verified_assumptions_ratio = if assumptions_count == 0 {
        1.0
    } else {
        verified_assumptions_count as f64 / assumptions_count as f64
    };

    // 3. Contradiction identification
    let contradictions: Vec<String> = assumed
        .intersection(&refuted)
        .map(|s| format!("Assumption '{}' is declared but refuted/falsified.", s))
        .collect();
    let contradictions_count = contradictions.len();

    // 4. Loop/Cycle detection
    let loop_path = detect_cycle(thoughts);
    let loop_detected = loop_path.is_some();

    // 5. Quality Score Calculation
    // Base score components (maximum 100):
    // - Confidence Score (up to 30 points)
    // - Verified Assumptions Ratio (up to 30 points)
    // - Structure Penalty (revisions/branching sanity)
    // - Contradictions Penalty (deduct 20 points per contradiction, max penalty 40)
    // - Loop Penalty (deduct 30 points if a dependency loop is detected)
    let mut score = 0.0;

    // Confidence component
    score += average_confidence * 40.0;

    // Assumptions component
    score += verified_assumptions_ratio * 40.0;

    // Structure adjustments: penalize if there are 0 thoughts
    if total_thoughts > 0 {
        score += 20.0; // Bonus for actual progress
    }

    // Contradiction penalties
    let contradiction_penalty = (contradictions_count as f64 * 20.0).min(40.0);
    score -= contradiction_penalty;

    // Loop penalty
    if loop_detected {
        score -= 30.0;
    }

    // Bound the final score to [0.0, 100.0]
    let quality_score = score.clamp(0.0, 100.0);

    // Grade assignment
    let grade = if quality_score >= 90.0 {
        "A".to_string()
    } else if quality_score >= 80.0 {
        "B".to_string()
    } else if quality_score >= 70.0 {
        "C".to_string()
    } else if quality_score >= 60.0 {
        "D".to_string()
    } else {
        "F".to_string()
    };

    QualityReport {
        session_id: session_id.to_string(),
        total_thoughts,
        average_confidence,
        assumptions_count,
        verified_assumptions_count,
        verified_assumptions_ratio,
        contradictions_count,
        contradictions,
        loop_detected,
        loop_path,
        quality_score,
        grade,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_thought(num: usize, confidence: Option<f64>) -> ThoughtData {
        ThoughtData {
            thought: format!("Thought {}", num),
            thought_number: num,
            total_thoughts: 10,
            next_thought_needed: true,
            is_revision: None,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: None,
            parent_thoughts: None,
            assumptions: None,
            verified_assumptions: None,
            confidence_score: confidence,
            criticism: None,
            hypothesis: None,
            verification_method: None,
            left_to_be_done: None,
            timestamp: None,
            session_id: Some("test-session".to_string()),
        }
    }

    #[test]
    fn test_detect_cycle_none() {
        let t1 = create_dummy_thought(1, Some(0.8));
        let mut t2 = create_dummy_thought(2, Some(0.9));
        t2.parent_thoughts = Some(vec![1]);
        
        let thoughts = vec![t1, t2];
        assert!(detect_cycle(&thoughts).is_none());
    }

    #[test]
    fn test_detect_cycle_exists() {
        let mut t1 = create_dummy_thought(1, Some(0.8));
        t1.parent_thoughts = Some(vec![2]); // T1 depends on T2
        let mut t2 = create_dummy_thought(2, Some(0.9));
        t2.parent_thoughts = Some(vec![1]); // T2 depends on T1
        
        let thoughts = vec![t1, t2];
        let cycle = detect_cycle(&thoughts);
        assert!(cycle.is_some());
        let path = cycle.unwrap();
        assert!(path.contains(&1));
        assert!(path.contains(&2));
    }

    #[test]
    fn test_contradiction_detection() {
        let mut t1 = create_dummy_thought(1, Some(0.8));
        t1.assumptions = Some(vec!["Gravity is constant".to_string()]);
        
        let mut t2 = create_dummy_thought(2, Some(0.7));
        t2.verified_assumptions = Some(vec!["refuted: Gravity is constant".to_string()]);
        
        let thoughts = vec![t1, t2];
        let report = calculate_quality("test-session", &thoughts);
        assert_eq!(report.contradictions_count, 1);
        assert!(report.quality_score < 100.0);
    }
}
