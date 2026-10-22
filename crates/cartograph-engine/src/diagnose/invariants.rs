use cartograph_core::config::LintArchConfig;
use cartograph_core::diagnose::TracebackFrame;
use std::collections::HashMap;

/// Invariant and Architectural Boundary Auditor.
pub struct InvariantAuditor {
    lint_config: LintArchConfig,
}

impl InvariantAuditor {
    pub fn new(lint_config: LintArchConfig) -> Self {
        Self { lint_config }
    }

    /// Evaluates which frames cross architectural layers or package encapsulation boundaries.
    pub fn audit_boundary_crossings(&self, frames: &[TracebackFrame]) -> HashMap<usize, bool> {
        let mut results = HashMap::new();

        for i in 0..frames.len().saturating_sub(1) {
            let caller = &frames[i];
            let callee = &frames[i + 1];

            if caller.is_in_project && callee.is_in_project {
                let caller_path = caller.file_path.to_string_lossy();
                let callee_path = callee.file_path.to_string_lossy();

                let is_crossing = self.is_layer_or_package_boundary(&caller_path, &callee_path);
                if is_crossing {
                    results.insert(caller.frame_index, true);
                    results.insert(callee.frame_index, true);
                }
            }
        }

        results
    }

    fn is_layer_or_package_boundary(&self, caller_path: &str, callee_path: &str) -> bool {
        // 1. Check if caller and callee belong to different configured architectural layers
        let caller_layer = self.classify_layer(caller_path);
        let callee_layer = self.classify_layer(callee_path);

        if let (Some(l1), Some(l2)) = (caller_layer, callee_layer)
            && l1 != l2
        {
            return true;
        }

        // 2. Check if caller crosses deep package encapsulation boundaries (e.g. accessing internal/)
        if callee_path.contains("/internal/") && !caller_path.contains("/internal/") {
            return true;
        }

        false
    }

    fn classify_layer(&self, file_path: &str) -> Option<String> {
        for layer in &self.lint_config.layers {
            for pattern in &layer.patterns {
                if globset::Glob::new(pattern)
                    .ok()?
                    .compile_matcher()
                    .is_match(file_path)
                {
                    return Some(layer.name.clone());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_package_boundary_crossing() {
        let auditor = InvariantAuditor::new(LintArchConfig::default());
        assert!(
            auditor.is_layer_or_package_boundary(
                "src/api/handler.rs",
                "src/billing/internal/store.rs"
            )
        );
        assert!(
            !auditor
                .is_layer_or_package_boundary("src/billing/service.rs", "src/billing/models.rs")
        );
    }
}
