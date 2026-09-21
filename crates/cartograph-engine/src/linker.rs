use cartograph_ast::model::UnresolvedReference;
use cartograph_core::model::{Edge, EdgeKind, NodeKind};
use serde_json::json;

use crate::symbol_table::{NodeSummary, SymbolTable};

pub struct ReferenceResolver<'a> {
    pub symbol_table: &'a SymbolTable,
}

impl<'a> ReferenceResolver<'a> {
    pub fn new(symbol_table: &'a SymbolTable) -> Self {
        Self { symbol_table }
    }

    /// Calculate directory distance (common path segments) between caller file and target file
    fn path_proximity_score(caller_path: &str, target_path: &str) -> usize {
        let caller_parts: Vec<&str> = caller_path.split('/').collect();
        let target_parts: Vec<&str> = target_path.split('/').collect();

        let mut common = 0;
        for (c, t) in caller_parts.iter().zip(target_parts.iter()) {
            if c == t {
                common += 1;
            } else {
                break;
            }
        }
        common
    }

    /// Check if caller file is a test file
    fn is_test_file(filepath: &str) -> bool {
        filepath.contains("test_")
            || filepath.ends_with("_test.go")
            || filepath.ends_with(".spec.ts")
            || filepath.ends_with(".spec.tsx")
            || filepath.ends_with(".test.ts")
            || filepath.ends_with(".test.tsx")
            || filepath.ends_with("_spec.rb")
            || filepath.starts_with("tests/")
    }

    /// Resolve an unresolved reference into a concrete Edge
    pub fn resolve(&self, reference: &UnresolvedReference) -> Option<Edge> {
        let caller_file =
            if let Some(summary) = self.symbol_table.get_by_id(&reference.source_node_id) {
                summary.filepath.clone()
            } else if let Ok(parsed_uri) = cartograph_core::parse_uri(&reference.source_node_id) {
                parsed_uri.filepath
            } else {
                String::new()
            };
        let caller_file = caller_file.as_str();

        // Tier 1: Explicit Import Source Resolution
        if let Some(ref import_source) = reference.import_source {
            // Clean import source (e.g. "./models/user", "app.models.user")
            let clean_import = import_source
                .trim_start_matches("./")
                .trim_start_matches("../")
                .replace('\\', "/");

            for (module_path, exports) in &self.symbol_table.exports_by_module {
                if (module_path.ends_with(&clean_import) || clean_import.ends_with(module_path))
                    && let Some(target_id) = exports.get(&reference.target_symbol)
                {
                    let mut edge =
                        Edge::new(&reference.source_node_id, target_id, reference.kind.clone());
                    edge.metadata = Some(json!({
                        "fidelity": "definite",
                        "resolution": "import_source",
                        "import_path": import_source
                    }));
                    return Some(edge);
                }
            }
        }

        // Tier 2: Type-Assisted Receiver Call
        if let Some(ref type_hint) = reference.receiver_type_hint
            && let Some(target) = self
                .symbol_table
                .find_method_in_type(type_hint, &reference.target_symbol)
        {
            let mut edge = Edge::new(
                &reference.source_node_id,
                &target.id,
                reference.kind.clone(),
            );
            edge.metadata = Some(json!({
                "fidelity": "definite",
                "resolution": "type_assisted",
                "receiver_type": type_hint
            }));
            return Some(edge);
        }

        // Tier 3: Trait / Interface Implementation Resolution
        if reference.kind == EdgeKind::Implements {
            let candidates = self.symbol_table.find_by_name(&reference.target_symbol);

            for candidate in candidates {
                if matches!(candidate.kind, NodeKind::Interface | NodeKind::Other(_)) {
                    let mut edge = Edge::new(
                        &reference.source_node_id,
                        &candidate.id,
                        EdgeKind::Implements,
                    );
                    edge.metadata = Some(json!({
                        "fidelity": "definite",
                        "resolution": "trait_implementation"
                    }));
                    return Some(edge);
                }
            }
        }

        // Tier 4: Global Unique Symbol Resolution
        let candidates = self.symbol_table.find_by_name(&reference.target_symbol);
        if candidates.len() == 1 {
            let target = &candidates[0];

            // If caller is in a test file and target is non-test, link as EdgeKind::Tests
            let kind = if Self::is_test_file(caller_file) && !Self::is_test_file(&target.filepath) {
                EdgeKind::Tests
            } else {
                reference.kind.clone()
            };

            let mut edge = Edge::new(&reference.source_node_id, &target.id, kind);
            edge.metadata = Some(json!({
                "fidelity": "definite",
                "resolution": "unique_global"
            }));
            return Some(edge);
        }

        // Tier 5: Proximity Heuristics & Polymorphic Candidate Sets
        if candidates.len() > 1 {
            // Pick candidate with highest path proximity
            let mut best_candidate: Option<&NodeSummary> = None;
            let mut best_score = 0;

            for candidate in candidates {
                let score = Self::path_proximity_score(caller_file, &candidate.filepath);
                if score > best_score || best_candidate.is_none() {
                    best_score = score;
                    best_candidate = Some(candidate);
                }
            }

            if let Some(target) = best_candidate {
                let kind =
                    if Self::is_test_file(caller_file) && !Self::is_test_file(&target.filepath) {
                        EdgeKind::Tests
                    } else {
                        reference.kind.clone()
                    };

                let all_candidate_ids: Vec<String> =
                    candidates.iter().map(|c| c.id.clone()).collect();
                let conf_val = 1.0 / (candidates.len() as f64);
                let mut edge = Edge::new(&reference.source_node_id, &target.id, kind);
                edge.confidence = Some(conf_val);
                edge.metadata = Some(json!({
                    "fidelity": "dynamic_candidate",
                    "resolution": "proximity",
                    "proximity_score": best_score,
                    "candidates": all_candidate_ids,
                    "confidence": conf_val
                }));
                return Some(edge);
            }
        }

        // Tier 6: Domain-Specific Fallback (Database Tables & Models)
        if reference.kind == EdgeKind::References
            && let Some(models) = self
                .symbol_table
                .db_tables
                .get(&reference.target_symbol.to_lowercase())
            && let Some(target) = models.first()
        {
            let mut edge = Edge::new(&reference.source_node_id, &target.id, EdgeKind::References);
            edge.metadata = Some(json!({
                "fidelity": "definite",
                "resolution": "db_model_reference"
            }));
            return Some(edge);
        }

        // Tier 7: Clientside Frontend API Call to Route Linking
        if reference.kind == EdgeKind::Calls
            && (reference.receiver_var.as_deref() == Some("api_call")
                || reference.receiver_var.as_deref() == Some("fetch")
                || reference.receiver_var.as_deref() == Some("axios")
                || reference.target_symbol.starts_with('/'))
        {
            for route in &self.symbol_table.routes {
                if crate::routes::frontend_linker::matches_route_pattern(
                    &reference.target_symbol,
                    &route.name,
                ) {
                    let mut edge = Edge::new(&reference.source_node_id, &route.id, EdgeKind::Calls);
                    edge.metadata = Some(json!({
                        "fidelity": "definite",
                        "resolution": "frontend_360_api_link",
                        "client_url": reference.target_symbol,
                    }));
                    return Some(edge);
                }
            }
        }

        None
    }

    /// Resolve all references from a slice into concrete Edges
    pub fn resolve_all(&self, references: &[UnresolvedReference]) -> Vec<Edge> {
        references.iter().filter_map(|r| self.resolve(r)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_ast::model::{CallFidelity, Span};

    #[test]
    fn test_linker_import_resolution() {
        let mut table = SymbolTable::new();
        let auth_node = cartograph_core::model::Node {
            id: "cartograph://src/services/auth.rs::AuthService::verify#method".to_string(),
            name: "verify".to_string(),
            kind: NodeKind::Method,
            filepath: "src/services/auth.rs".to_string(),
            scope: "AuthService".to_string(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(10),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };
        table.register_node(&auth_node);

        let resolver = ReferenceResolver::new(&table);
        let reference = UnresolvedReference {
            source_node_id: "cartograph://src/controllers/login.rs::::login#function".to_string(),
            target_symbol: "verify".to_string(),
            import_source: Some("services/auth".to_string()),
            kind: EdgeKind::Calls,
            span: Span::new(0, 10, 1, 1, 1, 10),
            call_fidelity: CallFidelity::Definite,
            receiver_type_hint: None,
            receiver_var: None,
            arg_count: None,
        };

        let edge = resolver
            .resolve(&reference)
            .expect("Should resolve via import");
        assert_eq!(edge.target_id, auth_node.id);
        assert_eq!(edge.kind, EdgeKind::Calls);
    }

    #[test]
    fn test_linker_type_assisted_call() {
        let mut table = SymbolTable::new();
        let user_save = cartograph_core::model::Node {
            id: "cartograph://src/models/user.rs::User::save#method".to_string(),
            name: "save".to_string(),
            kind: NodeKind::Method,
            filepath: "src/models/user.rs".to_string(),
            scope: "User".to_string(),
            start_line: Some(5),
            start_col: Some(1),
            end_line: Some(15),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };
        table.register_node(&user_save);

        let resolver = ReferenceResolver::new(&table);
        let reference = UnresolvedReference {
            source_node_id: "cartograph://src/controllers/user.rs::::create#function".to_string(),
            target_symbol: "save".to_string(),
            import_source: None,
            kind: EdgeKind::Calls,
            span: Span::new(0, 10, 1, 1, 1, 10),
            call_fidelity: CallFidelity::Definite,
            receiver_type_hint: Some("User".to_string()),
            receiver_var: Some("user".to_string()),
            arg_count: None,
        };

        let edge = resolver
            .resolve(&reference)
            .expect("Should resolve via type hint");
        assert_eq!(edge.target_id, user_save.id);
    }

    #[test]
    fn test_linker_test_mapping() {
        let mut table = SymbolTable::new();
        let target_fn = cartograph_core::model::Node {
            id: "cartograph://src/math.rs::::calculate#function".to_string(),
            name: "calculate".to_string(),
            kind: NodeKind::Function,
            filepath: "src/math.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(1),
            start_col: Some(1),
            end_line: Some(5),
            end_col: Some(1),
            signature: None,
            docstring: None,
            checksum: None,
            metadata: None,
        };
        table.register_node(&target_fn);

        let resolver = ReferenceResolver::new(&table);
        let reference = UnresolvedReference {
            source_node_id: "cartograph://tests/test_math.rs::::test_calc#function".to_string(),
            target_symbol: "calculate".to_string(),
            import_source: None,
            kind: EdgeKind::Calls,
            span: Span::new(0, 10, 1, 1, 1, 10),
            call_fidelity: CallFidelity::Definite,
            receiver_type_hint: None,
            receiver_var: None,
            arg_count: None,
        };

        let edge = resolver
            .resolve(&reference)
            .expect("Should resolve test edge");
        assert_eq!(edge.target_id, target_fn.id);
        assert_eq!(edge.kind, EdgeKind::Tests); // EdgeKind upgraded to Tests!
    }
}
