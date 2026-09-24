use cartograph_ast::AstLogicExtractor;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::model::EdgeKind;
use cartograph_core::{BusinessRule, ExtractLogicConfig};
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Inter-procedural call-tree BFS traversal engine with depth bounding and cycle prevention.
pub struct InterProceduralTracer<'a> {
    pub backend: Option<&'a dyn GraphStoreBackend>,
    pub extractor: AstLogicExtractor,
    pub config: &'a ExtractLogicConfig,
}

impl<'a> InterProceduralTracer<'a> {
    pub fn new(backend: Option<&'a dyn GraphStoreBackend>, config: &'a ExtractLogicConfig) -> Self {
        Self {
            backend,
            extractor: AstLogicExtractor::new(),
            config,
        }
    }

    /// Perform inter-procedural BFS trace starting from a root symbol or node.
    /// Traverses outgoing `Calls` edges up to `max_depth` (defaulting to config.max_call_depth).
    pub async fn trace_symbol_logic(
        &self,
        root_symbol: &str,
        root_filepath: Option<&Path>,
        workspace_root: &Path,
        max_depth: usize,
    ) -> Vec<BusinessRule> {
        let mut visited_symbols = HashSet::new();
        let mut queue: VecDeque<(String, Option<PathBuf>, usize, Vec<String>)> = VecDeque::new();
        let mut all_rules = Vec::new();

        queue.push_back((
            root_symbol.to_string(),
            root_filepath.map(|p| p.to_path_buf()),
            0,
            vec![root_symbol.to_string()],
        ));
        visited_symbols.insert(root_symbol.to_string());

        while let Some((symbol, filepath_opt, depth, call_path)) = queue.pop_front() {
            // 1. Extract rules for the current symbol
            let mut current_rules = if let Some(ref fp) = filepath_opt {
                self.extract_from_file_for_symbol(fp, &symbol)
            } else if let Some(backend) = self.backend {
                if let Ok(nodes) = backend.get_nodes_by_name(&symbol).await {
                    if let Some(target_node) = nodes.first() {
                        let full_path = workspace_root.join(&target_node.filepath);
                        self.extract_from_file_for_symbol(&full_path, &symbol)
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // Annotate rules with call tree provenance
            for rule in &mut current_rules {
                rule.call_depth = depth;
                rule.call_path = call_path.clone();
            }
            all_rules.extend(current_rules);

            // 2. Stop descending if reached max_depth
            if depth >= max_depth {
                continue;
            }

            // 3. Discover outgoing calls
            let callees = self
                .discover_callees(&symbol, filepath_opt.as_deref(), workspace_root)
                .await;

            for (callee_name, callee_path) in callees {
                if !visited_symbols.contains(&callee_name) {
                    visited_symbols.insert(callee_name.clone());
                    let mut next_path = call_path.clone();
                    next_path.push(callee_name.clone());
                    queue.push_back((callee_name, callee_path, depth + 1, next_path));
                }
            }
        }

        all_rules
    }

    fn extract_from_file_for_symbol(&self, filepath: &Path, symbol: &str) -> Vec<BusinessRule> {
        let content = match std::fs::read_to_string(filepath) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let file_str = filepath.to_string_lossy();
        self.extractor
            .extract_for_symbol(&file_str, &content, symbol, self.config)
    }

    async fn discover_callees(
        &self,
        symbol: &str,
        filepath_opt: Option<&Path>,
        workspace_root: &Path,
    ) -> Vec<(String, Option<PathBuf>)> {
        let mut callees = Vec::new();

        // Mode A: Query Graph Backend for outgoing Calls edges
        if let Some(backend) = self.backend
            && let Ok(nodes) = backend.get_nodes_by_name(symbol).await
        {
            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            if let Ok(edges) = backend.get_edges_for_nodes(&node_ids, 0.5).await {
                for edge in edges {
                    if edge.kind == EdgeKind::Calls
                        && node_ids.contains(&edge.source_id)
                        && let Ok(Some(callee_node)) = backend.get_node(&edge.target_id).await
                    {
                        let callee_path = workspace_root.join(&callee_node.filepath);
                        callees.push((callee_node.name.clone(), Some(callee_path)));
                    }
                }
            }
        }

        // Mode B: If backend returned no callees or is absent, inspect caller function in file
        if callees.is_empty()
            && let Some(fp) = filepath_opt
            && let Ok(content) = std::fs::read_to_string(fp)
        {
            // Find potential callee function calls in the caller's text
            let calls = harvest_callee_names_from_text(&content, symbol);
            for c in calls {
                callees.push((c, Some(fp.to_path_buf())));
            }
        }

        callees
    }
}

/// Fallback heuristic scanner to discover invoked function names in symbol scope
fn harvest_callee_names_from_text(content: &str, caller_name: &str) -> Vec<String> {
    let mut names = Vec::new();
    let caller_clean = caller_name.rsplit("::").next().unwrap_or(caller_name);

    // Locate function block
    if let Some(start) = content.find(caller_clean) {
        let block_slice = &content[start..];
        let end = block_slice
            .find("\n}\n")
            .unwrap_or(block_slice.len().min(2000));
        let body = &block_slice[..end];

        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }
            if let Some(open_paren) = trimmed.find('(') {
                let pre = &trimmed[..open_paren];
                let ident = pre
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .rfind(|s| !s.is_empty());

                if let Some(name) = ident
                    && name != caller_clean
                    && name.len() >= 3
                    && !matches!(
                        name,
                        "if" | "while"
                            | "for"
                            | "match"
                            | "let"
                            | "fn"
                            | "pub"
                            | "return"
                            | "print"
                            | "println"
                            | "format"
                            | "panic"
                            | "Some"
                            | "Ok"
                            | "Err"
                    )
                    && !names.contains(&name.to_string())
                {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_inter_procedural_tracer_offline() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("service.rs");

        let code = r#"
fn checkout(cart_total: f64) {
    validate_cart(cart_total);
    if cart_total <= 0.0 {
        panic!("Empty cart");
    }
}

fn validate_cart(total: f64) {
    if total > 50000.0 {
        panic!("Exceeds maximum allowable order total");
    }
}
"#;
        std::fs::write(&file, code).unwrap();

        let config = ExtractLogicConfig::default();
        let tracer = InterProceduralTracer::new(None, &config);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let rules = rt.block_on(tracer.trace_symbol_logic("checkout", Some(&file), tmp.path(), 2));

        assert!(
            rules.len() >= 2,
            "Expected rules from both checkout and validate_cart"
        );
        let root_rule = rules
            .iter()
            .find(|r| r.enclosing_symbol.contains("checkout"))
            .unwrap();
        assert_eq!(root_rule.call_depth, 0);

        let callee_rule = rules
            .iter()
            .find(|r| r.enclosing_symbol.contains("validate_cart"))
            .unwrap();
        assert_eq!(callee_rule.call_depth, 1);
        assert_eq!(callee_rule.call_path, vec!["checkout", "validate_cart"]);
    }
}
