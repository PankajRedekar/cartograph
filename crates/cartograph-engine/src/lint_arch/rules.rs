use chrono::{NaiveDate, Utc};
use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};

use cartograph_core::config::{ArchExemptionsConfig, ArchRuleConfig, RuleAction};
use cartograph_core::model::{BoundaryViolation, EdgeKind, RawInterFileEdgeRecord};

struct CompiledRule {
    config: ArchRuleConfig,
    source_glob: GlobSet,
    target_glob: GlobSet,
}

struct CompiledExemptions {
    ignore_globset: GlobSet,
    allowed: Vec<(GlobSet, GlobSet, Option<NaiveDate>)>,
}

/// Evaluates explicit allow/deny boundary rules and manages exemptions
pub struct RuleEvaluator {
    rules: Vec<CompiledRule>,
    exemptions: CompiledExemptions,
}

impl RuleEvaluator {
    pub fn new(rules: Vec<ArchRuleConfig>, exemptions: ArchExemptionsConfig) -> Self {
        let mut compiled_rules = Vec::with_capacity(rules.len());

        for rule in rules {
            let mut src_builder = GlobSetBuilder::new();
            let src_cleaned = rule.source.trim().replace('\\', "/");
            if let Ok(glob) = Glob::new(&src_cleaned) {
                src_builder.add(glob);
            }
            let source_glob = src_builder.build().unwrap_or_else(|_| GlobSet::empty());

            let mut tgt_builder = GlobSetBuilder::new();
            let tgt_cleaned = rule.target.trim().replace('\\', "/");
            if let Ok(glob) = Glob::new(&tgt_cleaned) {
                tgt_builder.add(glob);
            }
            let target_glob = tgt_builder.build().unwrap_or_else(|_| GlobSet::empty());

            compiled_rules.push(CompiledRule {
                config: rule,
                source_glob,
                target_glob,
            });
        }

        let mut ignore_builder = GlobSetBuilder::new();
        for pattern in &exemptions.ignore_paths {
            let cleaned = pattern.trim().replace('\\', "/");
            if let Ok(glob) = Glob::new(&cleaned) {
                ignore_builder.add(glob);
            }
        }
        let ignore_globset = ignore_builder.build().unwrap_or_else(|_| GlobSet::empty());

        let mut allowed = Vec::new();
        for entry in exemptions.allow {
            let mut s_b = GlobSetBuilder::new();
            let s_c = entry.source.trim().replace('\\', "/");
            if let Ok(g) = Glob::new(&s_c) {
                s_b.add(g);
            }

            let mut t_b = GlobSetBuilder::new();
            let t_c = entry.target.trim().replace('\\', "/");
            if let Ok(g) = Glob::new(&t_c) {
                t_b.add(g);
            }

            let expiry = entry.expires.as_deref().and_then(|exp| {
                NaiveDate::parse_from_str(exp, "%Y-%m-%d").ok()
            });

            allowed.push((
                s_b.build().unwrap_or_else(|_| GlobSet::empty()),
                t_b.build().unwrap_or_else(|_| GlobSet::empty()),
                expiry,
            ));
        }

        Self {
            rules: compiled_rules,
            exemptions: CompiledExemptions {
                ignore_globset,
                allowed,
            },
        }
    }

    /// Check if a given edge is exempt from architectural checks
    pub fn is_exempt(&self, source_file: &str, target_file: &str) -> bool {
        let s_norm = source_file.replace('\\', "/");
        let t_norm = target_file.replace('\\', "/");

        // 1. Ignore paths (e.g. tests/**, fixtures/**)
        if self.exemptions.ignore_globset.is_match(&s_norm)
            || self.exemptions.ignore_globset.is_match(&t_norm)
        {
            return true;
        }

        // 2. Explicit allowed exemptions with expiration check
        let today = Utc::now().date_naive();
        for (src_set, tgt_set, expiry) in &self.exemptions.allowed {
            if src_set.is_match(&s_norm) && tgt_set.is_match(&t_norm) {
                // If expired, exemption no longer applies
                if let Some(exp_date) = expiry {
                    if today > *exp_date {
                        continue;
                    }
                }
                return true;
            }
        }

        false
    }

    /// Evaluate custom directional rules against an edge
    pub fn evaluate_edge(&self, record: &RawInterFileEdgeRecord) -> Option<BoundaryViolation> {
        if self.is_exempt(&record.source_file, &record.target_file) {
            return None;
        }

        let s_norm = record.source_file.replace('\\', "/");
        let t_norm = record.target_file.replace('\\', "/");
        let edge_kind: EdgeKind = record.edge_kind.parse().unwrap_or(EdgeKind::Imports);

        for compiled in &self.rules {
            if compiled.source_glob.is_match(&s_norm) && compiled.target_glob.is_match(&t_norm) {
                // Check edge kinds filter if configured
                if let Some(ref kinds) = compiled.config.edge_kinds {
                    if !kinds.contains(&edge_kind) {
                        continue;
                    }
                }

                // If rule action is Deny -> emit violation!
                if compiled.config.action == RuleAction::Deny {
                    let fingerprint = Self::compute_fingerprint(
                        &compiled.config.name,
                        &record.source_file,
                        &record.target_file,
                        &record.edge_kind,
                    );

                    let msg = compiled.config.message.clone().unwrap_or_else(|| {
                        format!(
                            "Rule '{}' forbids dependency from '{}' to '{}'",
                            compiled.config.name, record.source_file, record.target_file
                        )
                    });

                    return Some(BoundaryViolation {
                        fingerprint,
                        rule_name: compiled.config.name.clone(),
                        category: "boundary_rule".to_string(),
                        severity: compiled.config.severity,
                        source_id: record.source_id.clone(),
                        source_name: record.source_name.clone(),
                        source_file: record.source_file.clone(),
                        source_layer: None,
                        target_id: record.target_id.clone(),
                        target_name: record.target_name.clone(),
                        target_file: record.target_file.clone(),
                        target_layer: None,
                        edge_kind,
                        line: record.source_line,
                        column: record.source_col,
                        snippet: None,
                        message: msg,
                        remediation_hint: compiled.config.hint.clone(),
                        is_baseline_suppressed: false,
                        is_new_in_diff: false,
                    });
                }
            }
        }

        None
    }

    fn compute_fingerprint(rule: &str, src: &str, tgt: &str, kind: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(rule.as_bytes());
        hasher.update(b":");
        hasher.update(src.as_bytes());
        hasher.update(b"->");
        hasher.update(tgt.as_bytes());
        hasher.update(b":");
        hasher.update(kind.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::config::{ArchExemptionRule, ArchSeverity};

    #[test]
    fn test_custom_deny_rule_and_exemption() {
        let rule = ArchRuleConfig {
            name: "auth-isolation".to_string(),
            description: None,
            source: "crates/auth/**".to_string(),
            target: "crates/billing/**".to_string(),
            action: RuleAction::Deny,
            edge_kinds: None,
            severity: ArchSeverity::Error,
            message: Some("Auth cannot depend on billing".to_string()),
            hint: Some("Use event bus".to_string()),
        };

        let exemptions = ArchExemptionsConfig {
            ignore_paths: vec!["tests/**".to_string()],
            allow: vec![ArchExemptionRule {
                source: "crates/auth/src/legacy.rs".to_string(),
                target: "crates/billing/**".to_string(),
                reason: Some("Tech debt waiver".to_string()),
                expires: None,
            }],
        };

        let evaluator = RuleEvaluator::new(vec![rule], exemptions);

        // Violation edge
        let bad_edge = RawInterFileEdgeRecord {
            source_id: "auth:login".to_string(),
            source_name: "login".to_string(),
            source_kind: "function".to_string(),
            source_file: "crates/auth/src/login.rs".to_string(),
            source_line: Some(1),
            source_col: Some(1),
            target_id: "billing:charge".to_string(),
            target_name: "charge".to_string(),
            target_kind: "function".to_string(),
            target_file: "crates/billing/src/charge.rs".to_string(),
            edge_kind: "calls".to_string(),
            confidence: Some(1.0),
            metadata: None,
        };

        let violation = evaluator.evaluate_edge(&bad_edge);
        assert!(violation.is_some());
        assert_eq!(violation.unwrap().rule_name, "auth-isolation");

        // Allowed exemption edge
        let exempt_edge = RawInterFileEdgeRecord {
            source_id: "auth:legacy".to_string(),
            source_name: "legacy".to_string(),
            source_kind: "function".to_string(),
            source_file: "crates/auth/src/legacy.rs".to_string(),
            source_line: Some(1),
            source_col: Some(1),
            target_id: "billing:charge".to_string(),
            target_name: "charge".to_string(),
            target_kind: "function".to_string(),
            target_file: "crates/billing/src/charge.rs".to_string(),
            edge_kind: "calls".to_string(),
            confidence: Some(1.0),
            metadata: None,
        };

        assert!(evaluator.evaluate_edge(&exempt_edge).is_none());
    }
}
