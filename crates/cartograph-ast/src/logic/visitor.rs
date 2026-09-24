use super::filter::NoiseFilter;
use super::languages::{
    GoLogicExtractor, PythonLogicExtractor, RubyLogicExtractor, RustLogicExtractor,
    TypeScriptLogicExtractor, generate_rule_id,
};
use super::slicer::{ParameterInfo, SymbolicProvenanceSlicer};
use cartograph_core::{BusinessRule, ExtractLogicConfig};
use std::path::PathBuf;
use tree_sitter::{Node as TsNode, Parser, Tree};

pub struct AstLogicVisitor<'a> {
    pub content: &'a str,
    pub filepath: &'a str,
    pub config: &'a ExtractLogicConfig,
}

impl<'a> AstLogicVisitor<'a> {
    pub fn new(content: &'a str, filepath: &'a str, config: &'a ExtractLogicConfig) -> Self {
        Self {
            content,
            filepath,
            config,
        }
    }

    /// Extract all business rules from the file
    pub fn extract_all(&self) -> Vec<BusinessRule> {
        let language = Self::detect_language(self.filepath);
        let tree = match parse_cst(language, self.content) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let filter = NoiseFilter::new(
            self.config.min_confidence,
            self.config.include_technical_guards,
            self.config.domain_keywords.clone(),
        );

        let mut rules = Vec::new();
        self.traverse_scope(tree.root_node(), "", language, &filter, &mut rules);
        rules
    }

    /// Extract rules targeting a specific symbol name
    pub fn extract_symbol(&self, symbol_name: &str) -> Vec<BusinessRule> {
        let all_rules = self.extract_all();
        all_rules
            .into_iter()
            .filter(|r| r.enclosing_symbol.contains(symbol_name) || r.title.contains(symbol_name))
            .collect()
    }

    fn traverse_scope(
        &self,
        node: TsNode,
        current_scope: &str,
        language: &str,
        filter: &NoiseFilter,
        out: &mut Vec<BusinessRule>,
    ) {
        let kind = node.kind();
        let (is_func, func_name) = match (language, kind) {
            ("rust", "function_item") => (true, self.get_node_name(node, "name")),
            (
                "typescript" | "javascript" | "tsx",
                "function_declaration" | "method_definition" | "arrow_function",
            ) => (true, self.get_node_name(node, "name")),
            ("python", "function_definition") => (true, self.get_node_name(node, "name")),
            ("go", "function_declaration" | "method_declaration") => {
                (true, self.get_node_name(node, "name"))
            }
            ("ruby", "method" | "singleton_method") => (true, self.get_node_name(node, "name")),
            _ => (false, None),
        };

        if is_func {
            let symbol_name = if let Some(name) = func_name {
                if current_scope.is_empty() {
                    name
                } else {
                    format!("{current_scope}::{name}")
                }
            } else {
                format!("{current_scope}::anonymous")
            };

            // Check if symbol matches ignore globs
            if self.is_ignored_symbol(&symbol_name) {
                return;
            }

            self.extract_function_rules(node, &symbol_name, language, filter, out);

            // Traverse children with updated scope
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.traverse_scope(child, &symbol_name, language, filter, out);
            }
            return;
        }

        // Class / Struct / Impl scope updates
        let class_name = match (language, kind) {
            ("rust", "struct_item" | "impl_item") => self
                .get_node_name(node, "name")
                .or_else(|| self.get_node_name(node, "type")),
            (
                "typescript" | "javascript" | "tsx",
                "class_declaration" | "interface_declaration",
            ) => self.get_node_name(node, "name"),
            ("python", "class_definition") => self.get_node_name(node, "name"),
            ("go", "type_spec") => self.get_node_name(node, "name"),
            ("ruby", "class" | "module") => self.get_node_name(node, "name"),
            _ => None,
        };

        let next_scope = if let Some(cls) = class_name {
            if current_scope.is_empty() {
                cls
            } else {
                format!("{current_scope}::{cls}")
            }
        } else {
            current_scope.to_string()
        };

        // Extract standalone / declaration-level rules (struct tags, ActiveRecord validations)
        let standalone_candidate = match (language, kind) {
            ("go", "field_declaration") => {
                let extractor = GoLogicExtractor::new(self.content, &next_scope);
                extractor.extract_struct_tag(node)
            }
            ("ruby", "call") => {
                let extractor = RubyLogicExtractor::new(self.content, &next_scope);
                extractor.extract_validation_call(node)
            }
            _ => None,
        };

        if let Some(candidate) = standalone_candidate {
            let confidence = filter.score_confidence(
                &candidate.kind,
                &candidate.predicate,
                candidate.error_message.as_deref(),
                &candidate.raw_snippet,
            );
            if filter.should_retain(confidence, false) {
                let domain_concepts =
                    self.infer_domain_concepts(&candidate.raw_snippet, &next_scope);
                let id = generate_rule_id(&next_scope, &candidate.span, &candidate.raw_snippet);
                out.push(BusinessRule {
                    id,
                    kind: candidate.kind,
                    title: candidate.title,
                    domain_concepts,
                    enclosing_symbol: next_scope.clone(),
                    file_path: PathBuf::from(self.filepath),
                    span: candidate.span,
                    predicate: candidate.predicate,
                    consequence: candidate.consequence,
                    invariant: candidate.invariant,
                    confidence,
                    call_depth: 0,
                    call_path: vec![next_scope.clone()],
                    variable_provenance: Vec::new(),
                    raw_snippet: candidate.raw_snippet,
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_scope(child, &next_scope, language, filter, out);
        }
    }

    fn extract_function_rules(
        &self,
        func_node: TsNode,
        symbol_name: &str,
        language: &str,
        filter: &NoiseFilter,
        out: &mut Vec<BusinessRule>,
    ) {
        let parameters = self.extract_parameters(func_node, language);
        let func_start_line = func_node.start_position().row + 1;

        let candidates = match language {
            "rust" => {
                let extractor = RustLogicExtractor::new(self.content, symbol_name);
                extractor.extract(func_node)
            }
            "typescript" | "javascript" | "tsx" => {
                let extractor = TypeScriptLogicExtractor::new(self.content, symbol_name);
                extractor.extract(func_node)
            }
            "python" => {
                let extractor = PythonLogicExtractor::new(self.content, symbol_name);
                extractor.extract(func_node)
            }
            "go" => {
                let extractor = GoLogicExtractor::new(self.content, symbol_name);
                extractor.extract(func_node)
            }
            "ruby" => {
                let extractor = RubyLogicExtractor::new(self.content, symbol_name);
                extractor.extract(func_node)
            }
            _ => Vec::new(),
        };

        for candidate in candidates {
            let confidence = filter.score_confidence(
                &candidate.kind,
                &candidate.predicate,
                candidate.error_message.as_deref(),
                &candidate.raw_snippet,
            );

            let is_tech =
                filter.is_technical_null_check(&candidate.predicate, &candidate.raw_snippet);
            if !filter.should_retain(confidence, is_tech) {
                continue;
            }

            let provenance = SymbolicProvenanceSlicer::slice_provenance(
                &candidate.predicate,
                candidate.span.start_line,
                func_start_line,
                &parameters,
                self.content,
            );

            let domain_concepts = self.infer_domain_concepts(&candidate.raw_snippet, symbol_name);
            let id = generate_rule_id(symbol_name, &candidate.span, &candidate.raw_snippet);

            out.push(BusinessRule {
                id,
                kind: candidate.kind,
                title: candidate.title,
                domain_concepts,
                enclosing_symbol: symbol_name.to_string(),
                file_path: PathBuf::from(self.filepath),
                span: candidate.span,
                predicate: candidate.predicate,
                consequence: candidate.consequence,
                invariant: candidate.invariant,
                confidence,
                call_depth: 0,
                call_path: vec![symbol_name.to_string()],
                variable_provenance: provenance,
                raw_snippet: candidate.raw_snippet,
            });
        }
    }

    fn extract_parameters(&self, func_node: TsNode, _language: &str) -> Vec<ParameterInfo> {
        let mut params = Vec::new();
        let param_node = func_node
            .child_by_field_name("parameters")
            .or_else(|| func_node.child_by_field_name("params"));

        if let Some(pn) = param_node {
            let mut cursor = pn.walk();
            let mut idx = 0;
            for child in pn.children(&mut cursor) {
                let p_text = self.get_node_text(child).trim();
                if p_text == "(" || p_text == ")" || p_text == "," {
                    continue;
                }
                let clean_p = p_text.trim_start_matches('&').trim_start_matches("mut ");
                let parts: Vec<&str> = clean_p.splitn(2, ':').collect();
                let name = parts[0].trim().to_string();
                let type_hint = parts.get(1).map(|t| t.trim().to_string());

                if !name.is_empty() && name != "self" && name != "&self" && name != "&mut self" {
                    params.push(ParameterInfo {
                        name,
                        type_hint,
                        default_value: None,
                        index: idx,
                    });
                    idx += 1;
                }
            }
        }
        params
    }

    fn infer_domain_concepts(&self, snippet: &str, symbol: &str) -> Vec<String> {
        let combined = format!("{snippet} {symbol}").to_lowercase();
        let mut concepts = Vec::new();

        for (concept, aliases) in &self.config.concept_aliases {
            if combined.contains(concept.as_str()) {
                concepts.push(concept.clone());
                continue;
            }
            for alias in aliases {
                if combined.contains(alias.as_str()) {
                    concepts.push(concept.clone());
                    break;
                }
            }
        }

        // Standard defaults if aliases didn't catch
        for default_kw in &[
            "discount", "tax", "fee", "price", "credit", "balance", "kyc", "auth",
        ] {
            if combined.contains(default_kw) && !concepts.iter().any(|c| c == *default_kw) {
                concepts.push(default_kw.to_string());
            }
        }

        concepts.sort();
        concepts.dedup();
        concepts
    }

    fn is_ignored_symbol(&self, symbol: &str) -> bool {
        for pattern in &self.config.ignore_symbols {
            if pattern.starts_with('*') && symbol.ends_with(pattern.trim_start_matches('*')) {
                return true;
            }
            if pattern.ends_with('*') && symbol.starts_with(pattern.trim_end_matches('*')) {
                return true;
            }
            if symbol.contains(pattern.as_str()) {
                return true;
            }
        }
        false
    }

    fn get_node_name(&self, node: TsNode, field_name: &str) -> Option<String> {
        node.child_by_field_name(field_name)
            .map(|n| self.get_node_text(n).to_string())
    }

    fn get_node_text(&self, node: TsNode) -> &str {
        &self.content[node.start_byte()..node.end_byte()]
    }

    pub fn detect_language(filepath: &str) -> &'static str {
        let path = std::path::Path::new(filepath);
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "ts" | "mts" | "cts" => "typescript",
                "tsx" | "jsx" => "tsx",
                "js" | "mjs" | "cjs" => "javascript",
                "py" | "pyi" => "python",
                "go" => "go",
                "rb" | "rake" => "ruby",
                _ => "unknown",
            }
        } else {
            "unknown"
        }
    }
}

fn parse_cst(language: &str, content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let lang_res = match language {
        #[cfg(feature = "rust")]
        "rust" => parser.set_language(&tree_sitter_rust::LANGUAGE.into()),
        #[cfg(feature = "typescript")]
        "typescript" | "javascript" => {
            parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        }
        #[cfg(feature = "typescript")]
        "tsx" => parser.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into()),
        #[cfg(feature = "python")]
        "python" => parser.set_language(&tree_sitter_python::LANGUAGE.into()),
        #[cfg(feature = "go")]
        "go" => parser.set_language(&tree_sitter_go::LANGUAGE.into()),
        #[cfg(feature = "ruby")]
        "ruby" => parser.set_language(&tree_sitter_ruby::LANGUAGE.into()),
        _ => return None,
    };

    if lang_res.is_err() {
        return None;
    }

    parser.parse(content, None)
}
