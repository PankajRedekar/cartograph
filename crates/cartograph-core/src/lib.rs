pub mod backend;
pub mod config;
pub mod error;
pub mod model;
pub mod uri;

pub use backend::GraphStoreBackend;
pub use config::{
    ArchExemptionRule, ArchExemptionsConfig, ArchLayerConfig, ArchRuleConfig, ArchSeverity,
    ComplexityConfig, CouplingFilter, CycleGranularity, DeadCodeCategoryFilter, DeadCodeConfig,
    GitDiffMode, IndexConfig, LintArchConfig, MemoryConfig, PackageEncapsulationConfig,
    ProfileConfig, ProjectProfile, RuleAction, SqliteConfig, StorageConfig, WatchConfig,
};
pub use error::{CartographError, Result};
pub use model::{
    AiContextFrictionMetric, ArchBaseline, BaselineCycleEntry, BaselineDeadItemEntry,
    BaselineHotspotEntry, BaselineViolationEntry, BoundaryViolation, ChurnMetric,
    ClassCohesionMetric, ComplexityBaseline, ComplexityDiffDelta, ComplexityGranularity,
    ComplexityMetric, ComplexityRecord, ComplexityReport, ComplexityTrendMetric, ConfidenceFactor,
    ConfidenceTier, CouplingMetric, CycleHop, DeadCodeBaseline, DeadCodeCategory,
    DeadCodeImpactMetrics, DeadCodePRDelta, DeadCodeReport, DeadIsland, DeadSymbolItem,
    DecompositionCluster, DependencyCycle, DeprecatedSymbolStatus, Edge, EdgeKind, FeedbackArcCut,
    GhostDependencyReport, GodClassEvaluation, GodClassSeverity, GraphStats, HotspotQuadrant,
    LintArchReport, LintArchSummary, MaintainabilityMetric, Node, NodeKind, RawClassMemberRecord,
    RawDeadSymbolRecord, RawFrontendCallerRecord, RawImpactRecord, RawInterFileEdgeRecord,
    RawRouteModelRecord, RawRouteTraceRecord, RawSymbolCouplingRecord, RawTestRecord, SearchResult,
    TestDeficitMetric, TrendDirection, ZombieFileReport,
};
pub use uri::{build_uri, parse_uri, CanonicalUri, URI_SCHEME};

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_canonical_uri_roundtrip() {
        let uri = build_uri(
            "src/services/auth.rs",
            "AuthService",
            "verify_token",
            &NodeKind::Method,
        );
        assert_eq!(
            uri,
            "cartograph://src/services/auth.rs::AuthService::verify_token#method"
        );

        let parsed = parse_uri(&uri).expect("URI parsing failed");
        assert_eq!(parsed.filepath, "src/services/auth.rs");
        assert_eq!(parsed.scope, "AuthService");
        assert_eq!(parsed.name, "verify_token");
        assert_eq!(parsed.kind, NodeKind::Method);
        assert_eq!(parsed.to_uri(), uri);
    }

    #[test]
    fn test_canonical_uri_empty_scope() {
        let uri = build_uri("main.rs", "", "main", &NodeKind::Function);
        assert_eq!(uri, "cartograph://main.rs::::main#function");

        let parsed = parse_uri(&uri).expect("URI parsing should succeed with empty scope");
        assert_eq!(parsed.filepath, "main.rs");
        assert_eq!(parsed.scope, "");
        assert_eq!(parsed.name, "main");
        assert_eq!(parsed.kind, NodeKind::Function);
    }

    #[test]
    fn test_canonical_uri_invalid() {
        assert!(parse_uri("http://foo::bar::baz#fn").is_err());
        assert!(parse_uri("cartograph://foo::bar").is_err());
        assert!(parse_uri("cartograph://foo::bar#").is_err());
        assert!(parse_uri("cartograph://::scope::name#function").is_err());
        assert!(parse_uri("cartograph://path::scope::#function").is_err());
    }

    #[test]
    fn test_node_serialization() {
        let node = Node {
            id: "cartograph://src/lib.rs::Auth::login#function".to_string(),
            name: "login".to_string(),
            kind: NodeKind::Function,
            filepath: "src/lib.rs".to_string(),
            scope: "Auth".to_string(),
            start_line: Some(10),
            start_col: Some(1),
            end_line: Some(25),
            end_col: Some(2),
            signature: Some("pub fn login(user: &str) -> bool".to_string()),
            docstring: Some("Authenticate user".to_string()),
            checksum: Some("abc123hash".to_string()),
            metadata: Some(serde_json::json!({ "visibility": "public" })),
        };

        let json = serde_json::to_string(&node).expect("Failed to serialize Node");
        let deserialized: Node = serde_json::from_str(&json).expect("Failed to deserialize Node");
        assert_eq!(node, deserialized);
    }

    #[test]
    fn test_edge_serialization() {
        let edge = Edge {
            source_id: "cartograph://src/a.rs::::foo#function".to_string(),
            target_id: "cartograph://src/b.rs::::bar#function".to_string(),
            kind: EdgeKind::Calls,
            metadata: Some(serde_json::json!({ "call_count": 3 })),
            confidence: Some(0.95),
            weight: Some(1.0),
        };

        let json = serde_json::to_string(&edge).expect("Failed to serialize Edge");
        let deserialized: Edge = serde_json::from_str(&json).expect("Failed to deserialize Edge");
        assert_eq!(edge, deserialized);
    }

    #[test]
    fn test_kinds_from_str() {
        assert_eq!(NodeKind::from_str("function").unwrap(), NodeKind::Function);
        assert_eq!(NodeKind::from_str("CLASS").unwrap(), NodeKind::Class);
        assert_eq!(
            NodeKind::from_str("custom_kind").unwrap(),
            NodeKind::Other("custom_kind".to_string())
        );

        assert_eq!(EdgeKind::from_str("calls").unwrap(), EdgeKind::Calls);
        assert_eq!(EdgeKind::from_str("IMPORTS").unwrap(), EdgeKind::Imports);
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::default();
        assert_eq!(config.backend, "sqlite");
        assert!(config.sqlite.is_some());
        let sqlite = config.sqlite.unwrap();
        assert_eq!(sqlite.journal_mode, "WAL");
        assert_eq!(sqlite.mmap_size, 268_435_456);

        let mem_config = StorageConfig::memory();
        assert_eq!(mem_config.backend, "memory");
        assert!(mem_config.memory.is_some());
    }

    #[test]
    fn test_profile_config() {
        let mut profiles = std::collections::HashMap::new();
        profiles.insert("test".to_string(), StorageConfig::memory());
        profiles.insert("prod".to_string(), StorageConfig::sqlite("prod.db"));

        let profile_config = ProfileConfig {
            active_profile: "test".to_string(),
            profiles,
        };

        let active = profile_config.active_storage();
        assert_eq!(active.backend, "memory");
    }
}
