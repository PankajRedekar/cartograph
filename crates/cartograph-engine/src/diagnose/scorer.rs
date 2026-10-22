use super::correlator::GitBlameInfo;
use cartograph_core::diagnose::{
    DiagnosisVerdict, FaultCandidate, FaultCategory, FaultRole, ProvenanceSlice, TracebackFrame,
};
use std::collections::HashMap;

/// Topological Fault Scorer computing normalized suspicion scores and ranking fault candidates.
pub struct TopologicalFaultScorer;

impl TopologicalFaultScorer {
    /// Evaluates and ranks all frames in the traceback, computing suspicion scores and roles.
    pub fn score_candidates(
        frames: &[TracebackFrame],
        provenance: Option<&ProvenanceSlice>,
        blame_map: &HashMap<usize, GitBlameInfo>,
        boundary_crossings: &HashMap<usize, bool>,
        _category: FaultCategory,
    ) -> (
        Vec<FaultCandidate>,
        Option<FaultCandidate>,
        DiagnosisVerdict,
    ) {
        let mut candidates = Vec::new();

        let project_frames: Vec<&TracebackFrame> =
            frames.iter().filter(|f| f.is_in_project).collect();
        let total_proj = project_frames.len();

        for frame in frames {
            if !frame.is_in_project {
                continue;
            }

            let mut score: f64 = 0.35; // Baseline prior
            let mut reasons = Vec::new();

            // 1. Provenance origin match
            if let Some(prov) = provenance
                && frame.file_path == prov.origin_file
                && (frame.line_number == prov.origin_line
                    || (frame.line_number.abs_diff(prov.origin_line) <= 5))
            {
                score += 0.45;
                reasons.push(format!(
                    "Origin site of tainted variable `{}` (def-use backward slice)",
                    prov.variable_name
                ));
            }

            // 2. Git Blame & Churn recency
            let blame_recency = blame_map.get(&frame.frame_index);
            if let Some(blame) = blame_recency {
                if blame.is_modified_in_branch {
                    score += 0.30;
                    reasons.push("Modified in current branch/PR diff".to_string());
                } else if blame.age_days <= 7 {
                    score += 0.25;
                    reasons.push(format!(
                        "Recently committed ({} days ago by {})",
                        blame.age_days, blame.author
                    ));
                } else if blame.age_days <= 30 {
                    score += 0.12;
                    reasons.push(format!("Modified {} days ago", blame.age_days));
                }
            }

            // 3. Boundary crossing
            let is_boundary = boundary_crossings
                .get(&frame.frame_index)
                .copied()
                .unwrap_or(false);
            if is_boundary {
                score += 0.15;
                reasons.push(
                    "Crosses architectural layer or package encapsulation boundary".to_string(),
                );
            }

            // 4. Position damping: leaf panic site is usually a consequence, not the origin
            if frame.is_leaf_panic && total_proj > 1 {
                score -= 0.10;
                reasons.push("Leaf panic/exception site (terminal crash symptom)".to_string());
            }

            // Clamp score between 0.05 and 0.99
            let clamped_score = score.clamp(0.05, 0.99);

            let primary_reason = if reasons.is_empty() {
                "Call path intermediate frame".to_string()
            } else {
                reasons.join("; ")
            };

            candidates.push(FaultCandidate {
                frame_index: frame.frame_index,
                file_path: frame.file_path.clone(),
                line_number: frame.line_number,
                function_name: frame.function_name.clone(),
                role: FaultRole::InnocentCaller, // Will be assigned after ranking
                suspicion_score: (clamped_score * 100.0).round() / 100.0,
                reason: primary_reason,
                blame_recency_days: blame_recency.map(|b| b.age_days),
                cognitive_complexity: None,
                is_boundary_crossing: is_boundary,
                variable_taint: provenance.map(|p| p.variable_name.clone()),
            });
        }

        // Sort descending by suspicion score
        candidates.sort_by(|a, b| b.suspicion_score.partial_cmp(&a.suspicion_score).unwrap());

        // Assign roles
        let max_score = candidates.first().map(|c| c.suspicion_score).unwrap_or(0.0);
        for candidate in candidates.iter_mut() {
            let is_leaf = frames
                .get(candidate.frame_index)
                .map(|f| f.is_leaf_panic)
                .unwrap_or(false);

            if is_leaf {
                candidate.role = FaultRole::PanicSite;
            } else if candidate.suspicion_score == max_score && max_score >= 0.60 {
                candidate.role = FaultRole::RootCause;
            } else if candidate.suspicion_score >= 0.45 {
                candidate.role = FaultRole::Propagator;
            } else {
                candidate.role = FaultRole::InnocentCaller;
            }
        }

        // If highest candidate is marked PanicSite because it was single frame or sole project frame
        if let Some(top) = candidates.first_mut()
            && top.suspicion_score >= 0.60
            && top.role != FaultRole::RootCause
        {
            top.role = FaultRole::RootCause;
        }

        let root_cause = candidates.first().cloned();
        let verdict = if max_score >= 0.75 {
            DiagnosisVerdict::DefiniteRootCause
        } else if max_score >= 0.50 {
            DiagnosisVerdict::ProbableRootCause
        } else {
            DiagnosisVerdict::Inconclusive
        };

        (candidates, root_cause, verdict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_topological_fault_scorer() {
        let frame1 = TracebackFrame {
            frame_index: 0,
            file_path: PathBuf::from("src/main.py"),
            line_number: 10,
            column: None,
            function_name: "main".to_string(),
            code_snippet: None,
            is_in_project: true,
            is_leaf_panic: false,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let frame2 = TracebackFrame {
            frame_index: 1,
            file_path: PathBuf::from("src/services/repo.py"),
            line_number: 22,
            column: None,
            function_name: "get_user".to_string(),
            code_snippet: None,
            is_in_project: true,
            is_leaf_panic: false,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let frame3 = TracebackFrame {
            frame_index: 2,
            file_path: PathBuf::from("src/services/order.py"),
            line_number: 45,
            column: None,
            function_name: "process".to_string(),
            code_snippet: None,
            is_in_project: true,
            is_leaf_panic: true,
            ast_symbol_id: None,
            ast_node_kind: None,
            language: "python".to_string(),
        };

        let frames = vec![frame1, frame2, frame3];
        let provenance = ProvenanceSlice {
            variable_name: "user".to_string(),
            origin_file: PathBuf::from("src/services/repo.py"),
            origin_line: 22,
            origin_expression: "db.query()".to_string(),
            propagation_steps: vec![],
        };

        let blame_map = HashMap::new();
        let boundary_crossings = HashMap::new();

        let (candidates, root_cause, verdict) = TopologicalFaultScorer::score_candidates(
            &frames,
            Some(&provenance),
            &blame_map,
            &boundary_crossings,
            FaultCategory::NullDeref,
        );

        assert_eq!(candidates.len(), 3);
        assert!(root_cause.is_some());
        let top = root_cause.unwrap();
        assert_eq!(top.file_path, PathBuf::from("src/services/repo.py"));
        assert_eq!(top.role, FaultRole::RootCause);
        assert!(top.suspicion_score >= 0.75);
        assert_eq!(verdict, DiagnosisVerdict::DefiniteRootCause);
    }
}
