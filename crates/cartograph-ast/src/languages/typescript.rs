use std::collections::HashMap;
use std::sync::LazyLock;
use tree_sitter::{Node as TsNode, Parser, Query, Tree};

use cartograph_core::{build_uri, Edge, EdgeKind, Node, NodeKind};
use crate::error::{AstError, Result};
use crate::hash::compute_node_checksum;
use crate::line_index::LineIndex;
use crate::model::{
    AstDiagnostic, CallFidelity, DiagnosticSeverity, ParseContext, ParsedFile,
    UnresolvedReference, Visibility,
};
use crate::traits::LanguageParser;
use crate::trivia::{SignatureNormalizer, TriviaBinder};

pub static TS_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    Query::new(&language, include_str!("../../queries/typescript/tags.scm"))
        .expect("Failed to compile TypeScript Tree-sitter query")
});

pub static TSX_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_typescript::LANGUAGE_TSX.into();
    Query::new(&language, include_str!("../../queries/typescript/tags.scm"))
        .expect("Failed to compile TSX Tree-sitter query")
});

pub struct TypeScriptParser {
    is_tsx: bool,
}

impl TypeScriptParser {
    pub fn typescript() -> Self {
        Self { is_tsx: false }
    }

    pub fn tsx() -> Self {
        Self { is_tsx: true }
    }

    fn init_parser(&self) -> Result<Parser> {
        let mut parser = Parser::new();
        let language = if self.is_tsx {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        };
        parser
            .set_language(&language)
            .map_err(|e| AstError::ParserInit(format!("Failed to set TypeScript language: {e:?}")))?;
        Ok(parser)
    }

    fn parse_internal(&self, ctx: &ParseContext, tree: &Tree) -> Result<ParsedFile> {
        let line_index = LineIndex::new(ctx.content);
        let root = tree.root_node();

        let mut extractor = TypeScriptExtractor {
            filepath: ctx.filepath,
            content: ctx.content,
            line_index: &line_index,
            options: &ctx.options,
            scope_stack: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            unresolved_references: Vec::new(),
            diagnostics: Vec::new(),
            type_bindings: HashMap::new(),
            current_controller_prefix: None,
            current_controller_has_guard: false,
        };

        extractor.visit_node(root);
        extractor.detect_nextjs_routes();

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: if self.is_tsx { "tsx".to_string() } else { "typescript".to_string() },
            is_generated: false,
            nodes: extractor.nodes,
            edges: extractor.edges,
            unresolved_references: extractor.unresolved_references,
            diagnostics: extractor.diagnostics,
        })
    }
}

impl LanguageParser for TypeScriptParser {
    fn language_id(&self) -> &'static str {
        if self.is_tsx { "tsx" } else { "typescript" }
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        if self.is_tsx {
            &["tsx", "jsx"]
        } else {
            &["ts", "js", "mjs", "cjs"]
        }
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let mut parser = self.init_parser()?;
        let tree = parser
            .parse(ctx.content, None)
            .ok_or_else(|| AstError::ParseFailure {
                filepath: ctx.filepath.to_string(),
                message: "Tree-sitter returned empty syntax tree for TypeScript/TSX".to_string(),
            })?;

        self.parse_internal(ctx, &tree)
    }

    fn parse_incremental(
        &self,
        ctx: &ParseContext,
        old_tree: &Tree,
        edit: Option<&tree_sitter::InputEdit>,
    ) -> Result<(ParsedFile, Tree)> {
        let mut parser = self.init_parser()?;
        let new_tree = if let Some(e) = edit {
            let mut edited = old_tree.clone();
            edited.edit(e);
            parser.parse(ctx.content, Some(&edited))
        } else {
            parser.parse(ctx.content, None)
        }
        .ok_or_else(|| AstError::ParseFailure {
            filepath: ctx.filepath.to_string(),
            message: "Incremental re-parse returned empty syntax tree".to_string(),
        })?;

        let parsed = self.parse_internal(ctx, &new_tree)?;
        Ok((parsed, new_tree))
    }
}

struct TypeScriptExtractor<'a> {
    filepath: &'a str,
    content: &'a str,
    line_index: &'a LineIndex,
    options: &'a crate::model::ParseOptions,
    scope_stack: Vec<String>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    unresolved_references: Vec<UnresolvedReference>,
    diagnostics: Vec<AstDiagnostic>,
    type_bindings: HashMap<String, String>, // var_name -> type_name
    current_controller_prefix: Option<String>,
    current_controller_has_guard: bool,
}

impl<'a> TypeScriptExtractor<'a> {
    fn current_scope(&self) -> String {
        self.scope_stack.join("::")
    }

    fn node_text(&self, node: TsNode) -> &'a str {
        if node.start_byte() <= node.end_byte() && node.end_byte() <= self.content.len() {
            &self.content[node.start_byte()..node.end_byte()]
        } else {
            ""
        }
    }

    fn visit_node(&mut self, node: TsNode) {
        if node.is_error() {
            let span = self.line_index.span_for_node(&node, self.content);
            self.diagnostics.push(AstDiagnostic {
                span,
                message: format!("Syntax error near '{}'", self.node_text(node)),
                severity: DiagnosticSeverity::Warning,
            });
            return;
        }

        match node.kind() {
            "interface_declaration" => self.visit_interface(node),
            "type_alias_declaration" => self.visit_type_alias(node),
            "class_declaration" => self.visit_class(node),
            "function_declaration" => self.visit_function(node, None),
            "lexical_declaration" | "variable_declaration" => self.visit_variable_declaration(node),
            "export_statement" => self.visit_export(node),
            "import_statement" => self.visit_import(node),
            "call_expression" => self.visit_call(node),
            "jsx_opening_element" | "jsx_self_closing_element" => self.visit_jsx_element(node),
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit_node(child);
                }
            }
        }
    }

    fn visit_interface(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, name, &NodeKind::Interface);

        let docstring = self.find_leading_jsdoc(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: name.to_string(),
            kind: NodeKind::Interface,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("interface {name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": "public",
                "span": span,
            })),
        });
    }

    fn visit_type_alias(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, name, &NodeKind::Struct);

        let docstring = self.find_leading_jsdoc(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: name.to_string(),
            kind: NodeKind::Struct,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("type {name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": "public",
                "span": span,
            })),
        });
    }

    fn visit_class(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let class_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, class_name, &NodeKind::Class);

        let docstring = self.find_leading_jsdoc(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri.clone(),
            name: class_name.to_string(),
            kind: NodeKind::Class,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("class {class_name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": "public",
                "span": span,
            })),
        });

        // NestJS Controller decorator detection
        let decorators = self.extract_decorators(node);
        let mut controller_prefix = None;
        let mut has_guard = false;
        for d in &decorators {
            if d.starts_with("Controller") {
                if let Some(start) = d.find('(') {
                    let inside = d[start + 1..].trim_end_matches(')').trim().trim_matches(['\'', '"']);
                    controller_prefix = Some(if inside.starts_with('/') {
                        inside.to_string()
                    } else if inside.is_empty() {
                        "/".to_string()
                    } else {
                        format!("/{inside}")
                    });
                } else {
                    controller_prefix = Some("/".to_string());
                }
            }
            if d.starts_with("UseGuards") {
                has_guard = true;
            }
        }
        let old_prefix = self.current_controller_prefix.take();
        let old_guard = self.current_controller_has_guard;
        if controller_prefix.is_some() {
            self.current_controller_prefix = controller_prefix;
            self.current_controller_has_guard = has_guard;
        }

        // Enter class scope
        self.scope_stack.push(class_name.to_string());

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "method_definition" {
                    self.visit_method(child, &uri);
                } else {
                    self.visit_node(child);
                }
            }
        }

        self.scope_stack.pop();
        self.current_controller_prefix = old_prefix;
        self.current_controller_has_guard = old_guard;
    }

    fn visit_method(&mut self, node: TsNode, parent_class_uri: &str) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let method_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, method_name, &NodeKind::Method);

        // Visibility
        let visibility = self.extract_ts_visibility(node);

        let docstring = self.find_leading_jsdoc(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri.clone(),
            name: method_name.to_string(),
            kind: NodeKind::Method,
            filepath: self.filepath.to_string(),
            scope: scope.clone(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("{method_name}()")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });

        // defines edge
        self.edges.push(Edge::new(parent_class_uri.to_string(), uri.clone(), EdgeKind::Defines));

        // NestJS Route decorator extraction
        let decorators = self.extract_decorators(node);
        let mut method_guard = false;
        for d in &decorators {
            if d.starts_with("UseGuards") {
                method_guard = true;
            }
        }

        if let Some(ref prefix) = self.current_controller_prefix {
            for d in &decorators {
                for http_m in &["Get", "Post", "Put", "Delete", "Patch", "Options", "Head", "All"] {
                    if d.starts_with(http_m) {
                        let mut subpath = "";
                        if let Some(start) = d.find('(') {
                            subpath = d[start + 1..].trim_end_matches(')').trim().trim_matches(['\'', '"']);
                        }
                        let full_path = if subpath.is_empty() {
                            prefix.clone()
                        } else if subpath.starts_with('/') {
                            format!("{}{subpath}", prefix.trim_end_matches('/'))
                        } else {
                            format!("{}/{}", prefix.trim_end_matches('/'), subpath)
                        };
                        let method_upper = http_m.to_uppercase();
                        let route_name = format!("{method_upper} {full_path}");
                        let route_uri = build_uri(self.filepath, "routes", &route_name, &NodeKind::Route);
                        let is_protected = self.current_controller_has_guard || method_guard;

                        self.nodes.push(Node {
                            id: route_uri.clone(),
                            name: route_name.clone(),
                            kind: NodeKind::Route,
                            filepath: self.filepath.to_string(),
                            scope: "routes".to_string(),
                            start_line: Some(span.start_line),
                            start_col: Some(span.start_col),
                            end_line: Some(span.end_line),
                            end_col: Some(span.end_col),
                            signature: Some(format!("{method_upper} {full_path}")),
                            docstring: None,
                            checksum: None,
                            metadata: Some(serde_json::json!({
                                "http_method": method_upper,
                                "raw_pattern": full_path,
                                "framework": "nestjs",
                                "handler": method_name,
                                "auth": if is_protected { "protected" } else { "public" },
                                "span": span,
                            })),
                        });
                        self.edges.push(Edge::new(route_uri, uri.clone(), EdgeKind::MapsTo));
                    }
                }
            }
        }

        // Visit body
        self.scope_stack.push(method_name.to_string());
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
        self.scope_stack.pop();
    }

    fn visit_function(&mut self, node: TsNode, export_visibility: Option<Visibility>) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let fn_name = self.node_text(name_node);
        let scope = self.current_scope();

        // Parameter type inspection for type-assisted calls
        self.extract_parameter_types(node);

        // Detect React component vs regular function vs hook
        let is_hook = fn_name.starts_with("use") && fn_name.len() > 3 && fn_name.chars().nth(3).map_or(false, |c| c.is_uppercase());
        let returns_jsx = self.has_jsx_return(node);
        let is_component = returns_jsx || (fn_name.chars().next().map_or(false, |c| c.is_uppercase()) && self.contains_jsx(node));

        let kind = if is_component {
            NodeKind::Component
        } else {
            NodeKind::Function
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, fn_name, &kind);
        let visibility = export_visibility.unwrap_or(Visibility::Private);

        let docstring = self.find_leading_jsdoc(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        let mut metadata = serde_json::json!({
            "visibility": visibility.as_str(),
            "span": span,
        });

        if is_component {
            metadata["is_react_component"] = serde_json::json!(true);
        }
        if is_hook {
            metadata["is_react_hook"] = serde_json::json!(true);
        }

        self.nodes.push(Node {
            id: uri,
            name: fn_name.to_string(),
            kind,
            filepath: self.filepath.to_string(),
            scope: scope.clone(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(SignatureNormalizer::normalize(&format!("function {fn_name}()"))),
            docstring,
            checksum,
            metadata: Some(metadata),
        });

        // Visit body
        self.scope_stack.push(fn_name.to_string());
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
        self.scope_stack.pop();
    }

    fn visit_variable_declaration(&mut self, node: TsNode) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let (Some(name_node), Some(value_node)) = (child.child_by_field_name("name"), child.child_by_field_name("value")) {
                    let var_name = self.node_text(name_node);

                    // Check arrow function / component: const Header = () => <header/>
                    if value_node.kind() == "arrow_function" || value_node.kind() == "function_expression" {
                        let is_hook = var_name.starts_with("use") && var_name.len() > 3 && var_name.chars().nth(3).map_or(false, |c| c.is_uppercase());
                        let returns_jsx = self.has_jsx_return(value_node);
                        let is_component = returns_jsx || (var_name.chars().next().map_or(false, |c| c.is_uppercase()) && self.contains_jsx(value_node));

                        let kind = if is_component {
                            NodeKind::Component
                        } else {
                            NodeKind::Function
                        };

                        let scope = self.current_scope();
                        let span = self.line_index.span_for_node(&child, self.content);
                        let uri = build_uri(self.filepath, &scope, var_name, &kind);

                        let docstring = self.find_leading_jsdoc(node);
                        let checksum = if self.options.compute_node_checksums {
                            Some(compute_node_checksum(self.node_text(child)))
                        } else {
                            None
                        };

                        let mut metadata = serde_json::json!({
                            "visibility": "public",
                            "span": span,
                        });
                        if is_component {
                            metadata["is_react_component"] = serde_json::json!(true);
                        }
                        if is_hook {
                            metadata["is_react_hook"] = serde_json::json!(true);
                        }

                        self.nodes.push(Node {
                            id: uri,
                            name: var_name.to_string(),
                            kind,
                            filepath: self.filepath.to_string(),
                            scope,
                            start_line: Some(span.start_line),
                            start_col: Some(span.start_col),
                            end_line: Some(span.end_line),
                            end_col: Some(span.end_col),
                            signature: Some(format!("const {var_name} = () => ...")),
                            docstring,
                            checksum,
                            metadata: Some(metadata),
                        });
                    }
                }
            }
            self.visit_node(child);
        }
    }

    fn visit_export(&mut self, node: TsNode) {
        if let Some(decl) = node.child_by_field_name("declaration") {
            if decl.kind() == "function_declaration" {
                self.visit_function(decl, Some(Visibility::Public));
                return;
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child);
        }
    }

    fn visit_import(&mut self, node: TsNode) {
        let span = self.line_index.span_for_node(&node, self.content);
        let source_path = node.child_by_field_name("source").map(|s| {
            self.node_text(s).trim_matches(|c| c == '\'' || c == '"').to_string()
        });

        // Scan import clause for imported specifiers
        if let Some(clause) = node.child_by_field_name("import_clause") {
            let mut cursor = clause.walk();
            for child in clause.children(&mut cursor) {
                if child.kind() == "identifier" {
                    let sym = self.node_text(child);
                    self.unresolved_references.push(UnresolvedReference {
                        source_node_id: format!("cartograph://{}::::*#file", self.filepath),
                        target_symbol: sym.to_string(),
                        import_source: source_path.clone(),
                        kind: EdgeKind::Imports,
                        span,
                        call_fidelity: CallFidelity::Definite,
                        receiver_type_hint: None,
                        receiver_var: None,
                        arg_count: None,
                    });
                } else if child.kind() == "named_imports" {
                    let mut inner = child.walk();
                    for spec in child.children(&mut inner) {
                        if spec.kind() == "import_specifier" {
                            if let Some(name) = spec.child_by_field_name("name") {
                                let sym = self.node_text(name);
                                self.unresolved_references.push(UnresolvedReference {
                                    source_node_id: format!("cartograph://{}::::*#file", self.filepath),
                                    target_symbol: sym.to_string(),
                                    import_source: source_path.clone(),
                                    kind: EdgeKind::Imports,
                                    span,
                                    call_fidelity: CallFidelity::Definite,
                                    receiver_type_hint: None,
                                    receiver_var: None,
                                    arg_count: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn visit_call(&mut self, node: TsNode) {
        let fn_node = match node.child_by_field_name("function") {
            Some(n) => n,
            None => return,
        };

        let mut base_node = fn_node;
        while base_node.kind() == "call_expression" {
            if let Some(inner) = base_node.child_by_field_name("function") {
                base_node = inner;
            } else {
                break;
            }
        }
        let fn_base_name = if base_node.kind() == "member_expression" {
            base_node
                .child_by_field_name("object")
                .map(|o| self.node_text(o))
                .unwrap_or_else(|| self.node_text(base_node))
        } else {
            self.node_text(base_node)
        };

        // Check if this is a test suite: describe("...", () => { ... }) or suite("...", ...)
        if fn_base_name == "describe" || fn_base_name == "suite" {
            let mut suite_name = String::new();
            if let Some(args) = node.child_by_field_name("arguments") {
                if let Some(first_arg) = args.named_child(0) {
                    suite_name = self.node_text(first_arg).trim_matches(['\'', '"', '`']).to_string();
                }
            }

            if !suite_name.is_empty() {
                self.scope_stack.push(suite_name.clone());
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.id() != fn_node.id() {
                    self.visit_node(child);
                }
            }

            if !suite_name.is_empty() {
                self.scope_stack.pop();
            }
            return;
        }

        // Check if this is a test case: it("...", () => { ... }) or test("...", () => { ... })
        if fn_base_name == "it" || fn_base_name == "test" {
            let mut test_name = String::new();
            if let Some(args) = node.child_by_field_name("arguments") {
                if let Some(first_arg) = args.named_child(0) {
                    test_name = self.node_text(first_arg).trim_matches(['\'', '"', '`']).to_string();
                }
            }

            if !test_name.is_empty() {
                let scope = self.current_scope();
                let span = self.line_index.span_for_node(&node, self.content);
                let uri = build_uri(self.filepath, &scope, &test_name, &NodeKind::Function);

                let framework = if self.filepath.contains("vitest") {
                    "vitest"
                } else {
                    "jest"
                };

                let test_type = if self.filepath.contains("e2e") {
                    "e2e"
                } else if self.filepath.contains("integration") {
                    "integration"
                } else {
                    "unit"
                };

                self.nodes.push(Node {
                    id: uri,
                    name: test_name,
                    kind: NodeKind::Function,
                    filepath: self.filepath.to_string(),
                    scope,
                    start_line: Some(span.start_line),
                    start_col: Some(span.start_col),
                    end_line: Some(span.end_line),
                    end_col: Some(span.end_col),
                    signature: Some(format!("test {fn_base_name}()")),
                    docstring: None,
                    checksum: None,
                    metadata: Some(serde_json::json!({
                        "is_test": true,
                        "framework": framework,
                        "test_type": test_type,
                        "span": span,
                    })),
                });

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.id() != fn_node.id() {
                        self.visit_node(child);
                    }
                }
                return;
            }
        }

        let span = self.line_index.span_for_node(&node, self.content);
        let caller_id = self.nodes.last().map(|n| n.id.clone()).unwrap_or_default();

        // 1. Clientside fetch calls: fetch('/api/users', { method: 'POST' })
        if fn_base_name == "fetch" || fn_base_name == "window.fetch" {
            if let Some(args) = node.child_by_field_name("arguments") {
                if let Some(first_arg) = args.named_child(0) {
                    if let Some(url) = self.extract_string_content(first_arg) {
                        let mut method = "GET".to_string();
                        if let Some(second_arg) = args.named_child(1) {
                            let text = self.node_text(second_arg).to_uppercase();
                            for m in &["POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] {
                                if text.contains(&format!("\"{m}\"")) || text.contains(&format!("'{m}'")) || text.contains(&format!("METHOD: {m}")) {
                                    method = m.to_string();
                                    break;
                                }
                            }
                        }
                        self.record_frontend_api_call(node, "fetch", &method, &url);
                    }
                }
            }
        }

        // 2. Clientside axios / HTTP client calls: axios.get('/api/users'), axios.post('/api/users', body)
        if fn_node.kind() == "member_expression" {
            if let (Some(obj), Some(prop)) = (fn_node.child_by_field_name("object"), fn_node.child_by_field_name("property")) {
                let obj_name = self.node_text(obj);
                let method_name = self.node_text(prop).to_lowercase();
                let http_methods = ["get", "post", "put", "patch", "delete", "options", "head"];
                if (obj_name == "axios" || obj_name.ends_with("Client") || obj_name == "api" || obj_name == "http")
                    && http_methods.contains(&method_name.as_str())
                {
                    if let Some(args) = node.child_by_field_name("arguments") {
                        if let Some(first_arg) = args.named_child(0) {
                            if let Some(url) = self.extract_string_content(first_arg) {
                                self.record_frontend_api_call(node, "axios", &method_name.to_uppercase(), &url);
                            }
                        }
                    }
                }
            }
        }

        // 3. Express / router route registration: app.get('/users', handler), router.post('/users', auth, handler)
        if fn_node.kind() == "member_expression" {
            if let (Some(obj), Some(prop)) = (fn_node.child_by_field_name("object"), fn_node.child_by_field_name("property")) {
                let obj_name = self.node_text(obj);
                let method_name = self.node_text(prop).to_lowercase();
                let http_methods = ["get", "post", "put", "patch", "delete", "options", "head", "all"];
                if http_methods.contains(&method_name.as_str()) {
                    if let Some(args) = node.child_by_field_name("arguments") {
                        if let Some(first_arg) = args.named_child(0) {
                            if let Some(route_path) = self.extract_string_content(first_arg) {
                                if route_path.starts_with('/') || route_path.starts_with('*') || obj_name.contains("app") || obj_name.contains("router") || obj_name.contains("server") {
                                    let http_method = method_name.to_uppercase();
                                    let span = self.line_index.span_for_node(&node, self.content);
                                    let route_name = format!("{http_method} {route_path}");
                                    let uri = build_uri(self.filepath, "routes", &route_name, &NodeKind::Route);

                                    let mut middlewares = Vec::new();
                                    let mut handler_name = None;
                                    let mut cursor = args.walk();
                                    for (i, child) in args.children(&mut cursor).filter(|c| c.is_named()).enumerate() {
                                        if i == 0 { continue; }
                                        let text = self.node_text(child).trim();
                                        if child.kind() == "identifier" || child.kind() == "member_expression" {
                                            middlewares.push(text.to_string());
                                        }
                                    }
                                    if !middlewares.is_empty() {
                                        handler_name = middlewares.pop();
                                    }

                                    let is_protected = middlewares.iter().any(|m| {
                                        let l = m.to_lowercase();
                                        l.contains("auth") || l.contains("guard") || l.contains("jwt") || l.contains("session") || l.contains("protect")
                                    });

                                    self.nodes.push(Node {
                                        id: uri.clone(),
                                        name: route_name.clone(),
                                        kind: NodeKind::Route,
                                        filepath: self.filepath.to_string(),
                                        scope: "routes".to_string(),
                                        start_line: Some(span.start_line),
                                        start_col: Some(span.start_col),
                                        end_line: Some(span.end_line),
                                        end_col: Some(span.end_col),
                                        signature: Some(format!("{http_method} {route_path}")),
                                        docstring: None,
                                        checksum: None,
                                        metadata: Some(serde_json::json!({
                                            "http_method": http_method,
                                            "raw_pattern": route_path,
                                            "framework": "express",
                                            "middlewares": middlewares,
                                            "handler": handler_name,
                                            "auth": if is_protected { "protected" } else { "public" },
                                            "span": span,
                                        })),
                                    });

                                    if let Some(ref handler) = handler_name {
                                        self.unresolved_references.push(UnresolvedReference {
                                            source_node_id: uri,
                                            target_symbol: handler.clone(),
                                            import_source: None,
                                            kind: EdgeKind::MapsTo,
                                            span,
                                            call_fidelity: CallFidelity::Definite,
                                            receiver_type_hint: None,
                                            receiver_var: None,
                                            arg_count: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if fn_node.kind() == "member_expression" {
            if let (Some(obj), Some(prop)) = (fn_node.child_by_field_name("object"), fn_node.child_by_field_name("property")) {
                let obj_name = self.node_text(obj);
                let method_name = self.node_text(prop);

                let receiver_type = self.type_bindings.get(obj_name).cloned();
                let call_fidelity = if obj_name == "this" || receiver_type.is_some() {
                    CallFidelity::Definite
                } else {
                    CallFidelity::DynamicCandidate
                };

                let target_symbol = if let Some(ref t) = receiver_type {
                    format!("{t}::{method_name}")
                } else {
                    method_name.to_string()
                };

                self.unresolved_references.push(UnresolvedReference {
                    source_node_id: caller_id,
                    target_symbol,
                    import_source: None,
                    kind: EdgeKind::Calls,
                    span,
                    call_fidelity,
                    receiver_type_hint: receiver_type,
                    receiver_var: Some(obj_name.to_string()),
                    arg_count: self.count_arguments(node),
                });
            }
        } else {
            let fn_name = self.node_text(fn_node);
            self.unresolved_references.push(UnresolvedReference {
                source_node_id: caller_id,
                target_symbol: fn_name.to_string(),
                import_source: None,
                kind: EdgeKind::Calls,
                span,
                call_fidelity: CallFidelity::Definite,
                receiver_type_hint: None,
                receiver_var: None,
                arg_count: self.count_arguments(node),
            });
        }

        // Recursively visit call expression arguments to discover nested calls/callbacks
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.id() != fn_node.id() {
                self.visit_node(child);
            }
        }
    }

    fn visit_jsx_element(&mut self, node: TsNode) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let tag_name = self.node_text(name_node);
            // Custom React components start with capital letter (<Header />, <UserProfile />)
            if tag_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                let span = self.line_index.span_for_node(&node, self.content);
                let caller_id = self.nodes.last().map(|n| n.id.clone()).unwrap_or_default();

                self.unresolved_references.push(UnresolvedReference {
                    source_node_id: caller_id,
                    target_symbol: tag_name.to_string(),
                    import_source: None,
                    kind: EdgeKind::Calls,
                    span,
                    call_fidelity: CallFidelity::Definite,
                    receiver_type_hint: None,
                    receiver_var: None,
                    arg_count: None,
                });
            }
        }
    }

    fn extract_parameter_types(&mut self, fn_node: TsNode) {
        if let Some(params) = fn_node.child_by_field_name("parameters") {
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                if child.kind() == "required_parameter" || child.kind() == "optional_parameter" {
                    if let (Some(name_node), Some(type_annot)) = (child.child_by_field_name("pattern"), child.child_by_field_name("type")) {
                        let param_name = self.node_text(name_node);
                        let type_str = self.node_text(type_annot).trim_start_matches(':').trim();
                        self.type_bindings.insert(param_name.to_string(), type_str.to_string());
                    }
                }
            }
        }
    }

    fn has_jsx_return(&self, fn_node: TsNode) -> bool {
        let mut has_jsx = false;
        self.find_node_recursively(fn_node, &mut |n| {
            if n.kind() == "jsx_element" || n.kind() == "jsx_self_closing_element" || n.kind() == "jsx_fragment" {
                has_jsx = true;
            }
        });
        has_jsx
    }

    fn contains_jsx(&self, node: TsNode) -> bool {
        let mut found = false;
        self.find_node_recursively(node, &mut |n| {
            if n.kind().starts_with("jsx_") {
                found = true;
            }
        });
        found
    }

    fn find_node_recursively<F>(&self, node: TsNode, predicate: &mut F)
    where
        F: FnMut(TsNode),
    {
        predicate(node);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_node_recursively(child, predicate);
        }
    }

    fn find_leading_jsdoc(&self, node: TsNode) -> Option<String> {
        if let Some(prev) = node.prev_sibling() {
            if prev.kind() == "comment" {
                let text = self.node_text(prev);
                let cleaned = TriviaBinder::clean_jsdoc(text);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
        None
    }

    fn extract_ts_visibility(&self, method_node: TsNode) -> Visibility {
        let mut cursor = method_node.walk();
        for child in method_node.children(&mut cursor) {
            if child.kind() == "accessibility_modifier" {
                match self.node_text(child) {
                    "private" => return Visibility::Private,
                    "protected" => return Visibility::Protected,
                    "public" => return Visibility::Public,
                    _ => {}
                }
            }
        }
        Visibility::Public
    }

    fn count_arguments(&self, call_node: TsNode) -> Option<usize> {
        call_node.child_by_field_name("arguments").map(|args| {
            let mut count = 0;
            let mut cursor = args.walk();
            for child in args.children(&mut cursor) {
                if child.is_named() {
                    count += 1;
                }
            }
            count
        })
    }

    fn extract_string_content(&self, node: TsNode) -> Option<String> {
        match node.kind() {
            "string" => {
                let text = self.node_text(node);
                Some(text.trim_matches(|c| c == '\'' || c == '"').to_string())
            }
            "template_string" => {
                let text = self.node_text(node);
                Some(text.trim_matches('`').to_string())
            }
            "binary_expression" => {
                let left = node.child_by_field_name("left")?;
                self.extract_string_content(left)
            }
            _ => None,
        }
    }

    fn extract_decorators(&self, node: TsNode) -> Vec<String> {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                decorators.push(self.node_text(child).trim_start_matches('@').trim().to_string());
            }
        }
        let mut prev = node.prev_sibling();
        while let Some(s) = prev {
            if s.kind() == "decorator" {
                decorators.push(self.node_text(s).trim_start_matches('@').trim().to_string());
                prev = s.prev_sibling();
            } else {
                break;
            }
        }
        if let Some(parent) = node.parent() {
            if parent.kind() == "export_statement" {
                let mut p_cursor = parent.walk();
                for child in parent.children(&mut p_cursor) {
                    if child.kind() == "decorator" {
                        decorators.push(self.node_text(child).trim_start_matches('@').trim().to_string());
                    }
                }
                let mut p_prev = parent.prev_sibling();
                while let Some(s) = p_prev {
                    if s.kind() == "decorator" {
                        decorators.push(self.node_text(s).trim_start_matches('@').trim().to_string());
                        p_prev = s.prev_sibling();
                    } else {
                        break;
                    }
                }
            }
        }
        decorators
    }

    fn record_frontend_api_call(&mut self, call_node: TsNode, client: &str, method: &str, url: &str) {
        let span = self.line_index.span_for_node(&call_node, self.content);
        // Prefer enclosing Component if inside a component tree, otherwise enclosing Function
        let comp_idx = self.nodes.iter().rposition(|n| n.kind == NodeKind::Component)
            .or_else(|| self.nodes.iter().rposition(|n| n.kind == NodeKind::Function));

        let target_id = comp_idx.map(|idx| self.nodes[idx].id.clone())
            .unwrap_or_else(|| format!("cartograph://{}::::*#file", self.filepath));

        if let Some(idx) = comp_idx {
            if let Some(ref mut meta) = self.nodes[idx].metadata {
                if let Some(arr) = meta.get_mut("api_call_sites").and_then(|v| v.as_array_mut()) {
                    arr.push(serde_json::json!({
                        "client_kind": client,
                        "method": method,
                        "url_pattern": url,
                        "line": span.start_line,
                    }));
                } else {
                    meta["api_call_sites"] = serde_json::json!([{
                        "client_kind": client,
                        "method": method,
                        "url_pattern": url,
                        "line": span.start_line,
                    }]);
                }
            }
        }

        self.unresolved_references.push(UnresolvedReference {
            source_node_id: target_id,
            target_symbol: url.to_string(),
            import_source: None,
            kind: EdgeKind::Calls,
            span,
            call_fidelity: CallFidelity::Definite,
            receiver_type_hint: Some(method.to_string()),
            receiver_var: Some(client.to_string()),
            arg_count: None,
        });
    }

    fn detect_nextjs_routes(&mut self) {
        let filepath = self.filepath;
        if filepath.contains("/app/") || filepath.starts_with("app/") {
            if filepath.ends_with("/route.ts") || filepath.ends_with("/route.js") || filepath.ends_with("/route.tsx") || filepath.ends_with("/route.jsx") {
                let idx = filepath.find("app/").unwrap();
                let route_part = &filepath[idx + 4..];
                let trimmed = route_part
                    .trim_end_matches("/route.ts")
                    .trim_end_matches("/route.js")
                    .trim_end_matches("/route.tsx")
                    .trim_end_matches("/route.jsx");
                let raw_pattern = if trimmed.is_empty() {
                    "/".to_string()
                } else if trimmed.starts_with('/') {
                    trimmed.to_string()
                } else {
                    format!("/{trimmed}")
                };

                let http_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
                let matched_funcs: Vec<(String, String)> = self.nodes.iter()
                    .filter(|n| n.kind == NodeKind::Function && http_methods.contains(&n.name.as_str()))
                    .map(|n| (n.name.clone(), n.id.clone()))
                    .collect();

                for (fn_name, fn_id) in matched_funcs {
                    let route_name = format!("{fn_name} {raw_pattern}");
                    let uri = build_uri(self.filepath, "routes", &route_name, &NodeKind::Route);
                    self.nodes.push(Node {
                        id: uri.clone(),
                        name: route_name.clone(),
                        kind: NodeKind::Route,
                        filepath: self.filepath.to_string(),
                        scope: "routes".to_string(),
                        start_line: Some(1),
                        start_col: Some(1),
                        end_line: Some(1),
                        end_col: Some(1),
                        signature: Some(format!("{fn_name} {raw_pattern}")),
                        docstring: None,
                        checksum: None,
                        metadata: Some(serde_json::json!({
                            "http_method": fn_name,
                            "raw_pattern": raw_pattern,
                            "framework": "nextjs",
                            "handler": fn_name,
                            "auth": "unknown",
                        })),
                    });
                    self.edges.push(Edge::new(uri, fn_id, EdgeKind::MapsTo));
                }
            }
        } else if filepath.contains("/pages/api/") || filepath.starts_with("pages/api/") {
            let idx = filepath.find("pages/api/").unwrap();
            let route_part = &filepath[idx + 9..];
            let raw_pattern = format!("/api/{}", route_part.trim_end_matches(".ts").trim_end_matches(".js").trim_end_matches(".tsx").trim_end_matches(".jsx"));
            let uri = build_uri(self.filepath, "routes", &format!("ALL {raw_pattern}"), &NodeKind::Route);
            self.nodes.push(Node {
                id: uri.clone(),
                name: format!("ALL {raw_pattern}"),
                kind: NodeKind::Route,
                filepath: self.filepath.to_string(),
                scope: "routes".to_string(),
                start_line: Some(1),
                start_col: Some(1),
                end_line: Some(1),
                end_col: Some(1),
                signature: Some(format!("ALL {raw_pattern}")),
                docstring: None,
                checksum: None,
                metadata: Some(serde_json::json!({
                    "http_method": "ALL",
                    "raw_pattern": raw_pattern,
                    "framework": "nextjs",
                    "auth": "unknown",
                })),
            });
            if let Some(h) = self.nodes.iter().find(|n| n.kind == NodeKind::Function && (n.name == "default" || n.name == "handler")) {
                self.edges.push(Edge::new(uri, h.id.clone(), EdgeKind::MapsTo));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_interfaces_and_classes() {
        let code = r#"
interface UserProfile {
    id: string;
    email: string;
}

export class AuthService {
    private token: string;

    public login(creds: Credentials): boolean {
        return true;
    }
}
"#;
        let parser = TypeScriptParser::typescript();
        let ctx = ParseContext::new("auth.ts", code);
        let parsed = parser.parse(&ctx).expect("Parsing TS failed");

        assert_eq!(parsed.nodes.len(), 3);
        assert_eq!(parsed.nodes[0].name, "UserProfile");
        assert_eq!(parsed.nodes[0].kind, NodeKind::Interface);

        assert_eq!(parsed.nodes[1].name, "AuthService");
        assert_eq!(parsed.nodes[1].kind, NodeKind::Class);

        assert_eq!(parsed.nodes[2].name, "login");
        assert_eq!(parsed.nodes[2].kind, NodeKind::Method);
        assert_eq!(parsed.nodes[2].scope, "AuthService");
    }

    #[test]
    fn test_tsx_react_component_and_hooks() {
        let code = r#"
import { Button } from './Button';

export function Header(props: HeaderProps) {
    const [open, setOpen] = useState(false);
    return (
        <header>
            <Button label="Click Me" />
        </header>
    );
}
"#;
        let parser = TypeScriptParser::tsx();
        let ctx = ParseContext::new("Header.tsx", code);
        let parsed = parser.parse(&ctx).expect("Parsing TSX failed");

        let component_node = parsed
            .nodes
            .iter()
            .find(|n| n.name == "Header")
            .expect("Should find Header component");
        assert_eq!(component_node.kind, NodeKind::Component);

        // Check JSX child element call
        let button_call = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "Button")
            .expect("Should extract Button JSX element call");
        assert_eq!(button_call.kind, EdgeKind::Calls);
    }

    #[test]
    fn test_typescript_test_extraction() {
        let code = r#"
describe("AuthModule", () => {
    it("should authenticate valid users", () => {
        const result = login("user", "pass");
        expect(result).toBe(true);
    });

    test.skip("should reject expired token", () => {
        expect(true).toBe(true);
    });
});
"#;
        let parser = TypeScriptParser::typescript();
        let ctx = ParseContext::new("tests/auth.test.ts", code);
        let parsed = parser.parse(&ctx).expect("Parsing test TS failed");

        let test_node = parsed
            .nodes
            .iter()
            .find(|n| n.name == "should authenticate valid users")
            .expect("Should extract it test case");

        assert_eq!(test_node.kind, NodeKind::Function);
        assert_eq!(test_node.scope, "AuthModule");
        let meta = test_node.metadata.as_ref().unwrap();
        assert_eq!(meta["is_test"], true);
        assert_eq!(meta["framework"], "jest");

        let second_test = parsed
            .nodes
            .iter()
            .find(|n| n.name == "should reject expired token")
            .expect("Should extract test.skip case");
        assert_eq!(second_test.scope, "AuthModule");
        assert_eq!(second_test.metadata.as_ref().unwrap()["is_test"], true);
    }

    #[test]
    fn test_express_routes_and_middlewares() {
        let code = r#"
app.get('/api/users/:id', requireAuth, getUserHandler);
router.post('/api/users', validateBody, createUserHandler);
"#;
        let parser = TypeScriptParser::typescript();
        let ctx = ParseContext::new("src/routes.ts", code);
        let parsed = parser.parse(&ctx).expect("Parsing Express routes failed");

        let route_get = parsed.nodes.iter().find(|n| n.name == "GET /api/users/:id").expect("GET route not found");
        assert_eq!(route_get.kind, NodeKind::Route);
        let meta_get = route_get.metadata.as_ref().unwrap();
        assert_eq!(meta_get["http_method"], "GET");
        assert_eq!(meta_get["framework"], "express");
        assert_eq!(meta_get["auth"], "protected");
        assert_eq!(meta_get["handler"], "getUserHandler");

        let maps_to_edge = parsed.unresolved_references.iter().find(|r| r.source_node_id == route_get.id && r.target_symbol == "getUserHandler");
        assert!(maps_to_edge.is_some(), "Should have MapsTo edge to handler");
    }

    #[test]
    fn test_nextjs_app_router() {
        let code = r#"
export async function GET(request: Request) {
    return Response.json({ users: [] });
}

export async function POST(request: Request) {
    return Response.json({ status: "created" });
}
"#;
        let parser = TypeScriptParser::typescript();
        let ctx = ParseContext::new("src/app/api/users/[id]/route.ts", code);
        let parsed = parser.parse(&ctx).expect("Parsing Next.js route failed");

        let get_route = parsed.nodes.iter().find(|n| n.name == "GET /api/users/[id]").expect("GET Next.js route not found");
        assert_eq!(get_route.kind, NodeKind::Route);
        assert_eq!(get_route.metadata.as_ref().unwrap()["framework"], "nextjs");

        let post_route = parsed.nodes.iter().find(|n| n.name == "POST /api/users/[id]").expect("POST Next.js route not found");
        assert_eq!(post_route.kind, NodeKind::Route);
    }

    #[test]
    fn test_nestjs_controller_and_guards() {
        let code = r#"
@Controller('users')
@UseGuards(AuthGuard)
export class UsersController {
    @Get(':id')
    public getUser(): User {
        return new User();
    }

    @Post()
    public createUser(): User {
        return new User();
    }
}
"#;
        let parser = TypeScriptParser::typescript();
        let ctx = ParseContext::new("src/users.controller.ts", code);
        let parsed = parser.parse(&ctx).expect("Parsing NestJS failed");

        let get_route = parsed.nodes.iter().find(|n| n.name == "GET /users/:id").expect("GET NestJS route not found");
        assert_eq!(get_route.kind, NodeKind::Route);
        let meta = get_route.metadata.as_ref().unwrap();
        assert_eq!(meta["auth"], "protected");
        assert_eq!(meta["framework"], "nestjs");

        let post_route = parsed.nodes.iter().find(|n| n.name == "POST /users").expect("POST NestJS route not found");
        assert_eq!(post_route.kind, NodeKind::Route);
    }

    #[test]
    fn test_frontend_api_call_extraction() {
        let code = r#"
export function CheckoutButton() {
    const handleCheckout = async () => {
        const res = await fetch('/api/checkout', { method: 'POST' });
        const data = await axios.get('/api/cart');
    };
    return <button onClick={handleCheckout}>Checkout</button>;
}
"#;
        let parser = TypeScriptParser::tsx();
        let ctx = ParseContext::new("src/components/CheckoutButton.tsx", code);
        let parsed = parser.parse(&ctx).expect("Parsing TSX component failed");

        let button_comp = parsed.nodes.iter().find(|n| n.name == "CheckoutButton").expect("CheckoutButton component not found");
        let meta = button_comp.metadata.as_ref().unwrap();
        let call_sites = meta["api_call_sites"].as_array().expect("api_call_sites missing");
        assert_eq!(call_sites.len(), 2);
        assert_eq!(call_sites[0]["url_pattern"], "/api/checkout");
        assert_eq!(call_sites[0]["method"], "POST");
        assert_eq!(call_sites[1]["url_pattern"], "/api/cart");
        assert_eq!(call_sites[1]["method"], "GET");
    }
}

