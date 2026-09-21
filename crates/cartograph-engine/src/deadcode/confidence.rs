use cartograph_core::config::ProjectProfile;
use cartograph_core::model::{ConfidenceFactor, ConfidenceTier, DeadCodeCategory, DeadSymbolItem, RawDeadSymbolRecord};

/// Multi-Vector Confidence Scoring Engine (0.0 to 1.0)
pub struct ConfidenceScorer {
    profile: ProjectProfile,
}

impl ConfidenceScorer {
    pub fn new(profile: ProjectProfile) -> Self {
        Self { profile }
    }

    /// Evaluates multi-vector factors for a raw dead symbol record and returns computed confidence, tier, and factors.
    pub fn score_symbol(
        &self,
        node: &RawDeadSymbolRecord,
        category: DeadCodeCategory,
    ) -> (f64, ConfidenceTier, Vec<ConfidenceFactor>) {
        let mut factors = Vec::new();
        let mut base_score: f64 = 0.80;

        // 1. In-code suppression check
        let has_ignore_directive = node.metadata.as_ref().map(|m| {
            let m_str = m.to_string();
            m_str.contains("@deadcode ignore")
                || m_str.contains("allow_dead")
                || m_str.contains("unused_ok")
                || m_str.contains("no-dead-code")
        }).unwrap_or(false);

        if has_ignore_directive {
            factors.push(ConfidenceFactor {
                description: "Suppressed via in-code ignore directive".to_string(),
                delta: -1.0,
            });
            return (0.0, ConfidenceTier::Suppressed, factors);
        }

        // 2. Visibility evaluation
        let is_private = node.scope.contains("private")
            || node.name.starts_with('_')
            || (node.signature.as_ref().map(|s| !s.contains("pub ") && !s.contains("export ")).unwrap_or(true) && !node.filepath.ends_with("lib.rs"));

        let is_crate_private = node.signature.as_ref().map(|s| s.contains("pub(crate)")).unwrap_or(false);
        let is_public = node.signature.as_ref().map(|s| s.contains("pub ") || s.contains("export ")).unwrap_or(false);

        if is_private {
            factors.push(ConfidenceFactor {
                description: "Private / internal visibility (unreachable outside local scope)".to_string(),
                delta: 0.15,
            });
            base_score += 0.15;
        } else if is_crate_private {
            factors.push(ConfidenceFactor {
                description: "Package-private visibility (pub(crate))".to_string(),
                delta: 0.10,
            });
            base_score += 0.10;
        } else if is_public {
            match self.profile {
                ProjectProfile::Application => {
                    factors.push(ConfidenceFactor {
                        description: "Public symbol in application repository (internal to app)".to_string(),
                        delta: 0.05,
                    });
                    base_score += 0.05;
                }
                ProjectProfile::Library => {
                    factors.push(ConfidenceFactor {
                        description: "Public symbol in published library (may have external consumers)".to_string(),
                        delta: -0.45,
                    });
                    base_score -= 0.45;
                }
                ProjectProfile::Monorepo => {
                    factors.push(ConfidenceFactor {
                        description: "Public symbol in monorepo".to_string(),
                        delta: -0.15,
                    });
                    base_score -= 0.15;
                }
            }
        }

        // 3. Framework convention checks
        let name_lower = node.name.to_lowercase();
        let is_framework_convention = name_lower.starts_with("use_")
            || name_lower.starts_with("use") // React hook
            || name_lower == "componentdidmount"
            || name_lower == "componentdidupdate"
            || name_lower == "getstaticprops"
            || name_lower == "getserversideprops"
            || name_lower == "handlerequest"
            || name_lower == "ready"
            || name_lower == "setup"
            || name_lower == "teardown";

        if is_framework_convention {
            factors.push(ConfidenceFactor {
                description: "Matches common framework lifecycle convention / hook".to_string(),
                delta: -0.30,
            });
            base_score -= 0.30;
        }

        // 4. Trait or interface implementation check
        let is_trait_impl = node.scope.contains("impl ") || node.scope.contains("implements ");
        if is_trait_impl {
            factors.push(ConfidenceFactor {
                description: "Method part of trait/interface contract".to_string(),
                delta: -0.20,
            });
            base_score -= 0.20;
        }

        // 5. Serialization / ORM tag check
        let has_serialization = node.signature.as_ref().map(|s| {
            s.contains("Serialize") || s.contains("Deserialize") || s.contains("@dataclass")
        }).unwrap_or(false);

        if has_serialization {
            factors.push(ConfidenceFactor {
                description: "Struct or field tagged with serialization / dataclass attributes".to_string(),
                delta: -0.15,
            });
            base_score -= 0.15;
        }

        // 6. Category adjustments
        match category {
            DeadCodeCategory::AbandonedComponent => {
                factors.push(ConfidenceFactor {
                    description: "Component has zero JSX/TSX rendering sites".to_string(),
                    delta: 0.10,
                });
                base_score += 0.10;
            }
            DeadCodeCategory::UnreadField => {
                factors.push(ConfidenceFactor {
                    description: "Field has zero read accesses".to_string(),
                    delta: 0.10,
                });
                base_score += 0.10;
            }
            DeadCodeCategory::UnusedEnumVariant => {
                factors.push(ConfidenceFactor {
                    description: "Enum variant has zero match arms or constructor calls".to_string(),
                    delta: 0.10,
                });
                base_score += 0.10;
            }
            DeadCodeCategory::DanglingRoute => {
                factors.push(ConfidenceFactor {
                    description: "Route has zero registered callers in router".to_string(),
                    delta: 0.05,
                });
                base_score += 0.05;
            }
            _ => {}
        }

        let clamped_confidence = base_score.clamp(0.0, 1.0);
        let tier = if clamped_confidence < 0.50 {
            ConfidenceTier::Suppressed
        } else if clamped_confidence < 0.75 {
            ConfidenceTier::Low
        } else if clamped_confidence < 0.90 {
            ConfidenceTier::Medium
        } else {
            ConfidenceTier::High
        };

        (clamped_confidence, tier, factors)
    }

    /// Re-evaluates an existing DeadSymbolItem's confidence score
    pub fn apply_to_symbol(&self, item: &mut DeadSymbolItem, node: &RawDeadSymbolRecord) {
        let (score, tier, factors) = self.score_symbol(node, item.category);
        item.confidence = score;
        item.tier = tier;
        item.confidence_factors = factors;
    }
}
