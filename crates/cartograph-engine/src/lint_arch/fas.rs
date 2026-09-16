use std::collections::HashMap;

use cartograph_core::model::{DependencyCycle, FeedbackArcCut};

/// Minimum Feedback Arc Set (FAS) advisor to identify optimal cut edge(s) breaking cycle loops
pub struct FeedbackArcAdvisor;

impl FeedbackArcAdvisor {
    /// Calculate edge participation counts and attach recommended cut edges to each cycle
    pub fn advise_cycles(cycles: &mut [DependencyCycle]) {
        if cycles.is_empty() {
            return;
        }

        // 1. Count frequency of each directed edge across all cycles: (source_name, target_name) -> count
        let mut edge_frequencies: HashMap<(String, String), usize> = HashMap::new();

        for cycle in cycles.iter() {
            let n = cycle.hops.len();
            for i in 0..n {
                let next_idx = (i + 1) % n;
                let src = cycle.hops[i].name.clone();
                let tgt = cycle.hops[next_idx].name.clone();
                *edge_frequencies.entry((src, tgt)).or_insert(0) += 1;
            }
        }

        // 2. For each cycle, pick the edge that appears in the highest number of cycles
        for cycle in cycles.iter_mut() {
            let n = cycle.hops.len();
            let mut best_cut: Option<FeedbackArcCut> = None;
            let mut highest_count = 0;

            for i in 0..n {
                let next_idx = (i + 1) % n;
                let src_hop = &cycle.hops[i];
                let tgt_hop = &cycle.hops[next_idx];

                let count = *edge_frequencies
                    .get(&(src_hop.name.clone(), tgt_hop.name.clone()))
                    .unwrap_or(&1);

                if best_cut.is_none() || count > highest_count {
                    highest_count = count;

                    let advice = format!(
                        "Invert dependency from '{}' to '{}' using an interface, trait, or event callback.",
                        tgt_hop.name, src_hop.name
                    );

                    best_cut = Some(FeedbackArcCut {
                        source_id: src_hop.node_id.clone(),
                        target_id: tgt_hop.node_id.clone(),
                        edge_kind: src_hop.outgoing_edge_kind.clone(),
                        source_file: src_hop.filepath.clone(),
                        target_file: tgt_hop.filepath.clone(),
                        cycles_broken_count: count,
                        remediation_advice: advice,
                    });
                }
            }

            cycle.recommended_cut = best_cut;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::config::{ArchSeverity, CycleGranularity};
    use cartograph_core::model::{CycleHop, EdgeKind, NodeKind};

    fn make_hop(name: &str) -> CycleHop {
        CycleHop {
            node_id: format!("{name}:id"),
            name: name.to_string(),
            filepath: format!("{name}.rs"),
            line: Some(1),
            column: Some(1),
            kind: NodeKind::File,
            outgoing_edge_kind: EdgeKind::Calls,
            is_runtime: true,
        }
    }

    #[test]
    fn test_fas_advisor_multi_cycle_shared_edge() {
        // Cycle 1: A -> B -> C -> A
        // Cycle 2: D -> B -> C -> D
        // Edge (B -> C) is shared by both cycles! Severing B -> C breaks 2 cycles.

        let cycle1 = DependencyCycle {
            fingerprint: "c1".to_string(),
            granularity: CycleGranularity::File,
            length: 3,
            is_runtime_cycle: true,
            hops: vec![make_hop("A"), make_hop("B"), make_hop("C")],
            recommended_cut: None,
            severity: ArchSeverity::Error,
        };

        let cycle2 = DependencyCycle {
            fingerprint: "c2".to_string(),
            granularity: CycleGranularity::File,
            length: 3,
            is_runtime_cycle: true,
            hops: vec![make_hop("D"), make_hop("B"), make_hop("C")],
            recommended_cut: None,
            severity: ArchSeverity::Error,
        };

        let mut cycles = vec![cycle1, cycle2];
        FeedbackArcAdvisor::advise_cycles(&mut cycles);

        // Both cycles should recommend cutting B -> C because it breaks 2 cycles
        let cut1 = cycles[0].recommended_cut.as_ref().unwrap();
        assert_eq!(cut1.source_file, "B.rs");
        assert_eq!(cut1.target_file, "C.rs");
        assert_eq!(cut1.cycles_broken_count, 2);

        let cut2 = cycles[1].recommended_cut.as_ref().unwrap();
        assert_eq!(cut2.source_file, "B.rs");
        assert_eq!(cut2.target_file, "C.rs");
        assert_eq!(cut2.cycles_broken_count, 2);
    }
}
