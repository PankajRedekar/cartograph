use std::collections::HashMap;
use std::sync::LazyLock;
use tree_sitter::{Node as TsNode, Parser, Query, Tree};

use cartograph_core::{build_uri, Edge, EdgeKind, Node, NodeKind};
use crate::error::{AstError, Result};
use crate::hash::compute_node_checksum;
use crate::line_index::LineIndex;
use crate::model::{
    AstDiagnostic, CallFidelity, DiagnosticSeverity, ParseContext, ParsedFile, Span,
    UnresolvedReference, Visibility,
};
use crate::traits::LanguageParser;
use crate::trivia::{SignatureNormalizer, TriviaBinder};

pub static PYTHON_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_python::LANGUAGE.into();
    Query::new(&language, include_str!("../../queries/python/tags.scm"))
        .expect("Failed to compile Python Tree-sitter query")
});

pub struct PythonParser;

impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonParser {
    pub fn new() -> Self {
        Self
    }

    fn init_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| AstError::ParserInit(format!("Failed to set Python language: {e:?}")))?;
        Ok(parser)
    }

    fn parse_internal(
        &self,
        ctx: &ParseContext,
        tree: &Tree,
    ) -> Result<ParsedFile> {
        let line_index = LineIndex::new(ctx.content);
        let root = tree.root_node();

        let mut extractor = PythonExtractor {
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
        };

        extractor.visit_node(root);

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: "python".to_string(),
            is_generated: false,
            nodes: extractor.nodes,
            edges: extractor.edges,
            unresolved_references: extractor.unresolved_references,
            diagnostics: extractor.diagnostics,
        })
    }
}

impl LanguageParser for PythonParser {
    fn language_id(&self) -> &'static str {
        "python"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["py", "pyi"]
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let mut parser = Self::init_parser()?;
        let tree = parser
            .parse(ctx.content, None)
            .ok_or_else(|| AstError::ParseFailure {
                filepath: ctx.filepath.to_string(),
                message: "Tree-sitter returned empty syntax tree for Python source".to_string(),
            })?;

        self.parse_internal(ctx, &tree)
    }

    fn parse_incremental(
        &self,
        ctx: &ParseContext,
        old_tree: &Tree,
        edit: Option<&tree_sitter::InputEdit>,
    ) -> Result<(ParsedFile, Tree)> {
        let mut parser = Self::init_parser()?;
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

struct PythonExtractor<'a> {
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
}

impl<'a> PythonExtractor<'a> {
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
            "class_definition" => self.visit_class(node),
            "function_definition" => self.visit_function(node, None),
            "decorated_definition" => self.visit_decorated(node),
            "call" => self.visit_call(node),
            "import_statement" => self.visit_import(node),
            "import_from_statement" => self.visit_import_from(node),
            "assignment" => self.visit_assignment(node),
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit_node(child);
                }
            }
        }
    }

    fn visit_class(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let class_name = self.node_text(name_node);
        let scope = self.current_scope();

        // Detect superclasses
        let mut superclasses = Vec::new();
        if let Some(super_node) = node.child_by_field_name("superclasses") {
            let mut cursor = super_node.walk();
            for child in super_node.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "attribute" {
                    superclasses.push(self.node_text(child).to_string());
                }
            }
        }

        // Framework heuristic: Django Model / DRF Serializer
        let mut is_django_model = false;
        let mut is_drf_serializer = false;
        for s in &superclasses {
            if s.contains("models.Model") || s == "Model" || s.contains("AbstractUser") {
                is_django_model = true;
            }
            if s.contains("Serializer") {
                is_drf_serializer = true;
            }
        }

        let kind = if is_django_model {
            NodeKind::Model
        } else {
            NodeKind::Class
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, class_name, &kind);

        // Visibility
        let visibility = self.extract_visibility(class_name);

        // Docstring
        let docstring = self.extract_body_docstring(node.child_by_field_name("body"));

        // Checksum
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        let mut metadata = serde_json::json!({
            "visibility": visibility.as_str(),
            "span": span,
        });

        if is_django_model {
            metadata["is_django_model"] = serde_json::json!(true);
        }
        if is_drf_serializer {
            metadata["is_drf_serializer"] = serde_json::json!(true);
        }
        if !superclasses.is_empty() {
            metadata["superclasses"] = serde_json::json!(superclasses);
        }

        let is_test_class = class_name.starts_with("Test") || class_name.ends_with("Test") || class_name.ends_with("Tests") || superclasses.iter().any(|s| s.contains("TestCase"));
        if is_test_class {
            metadata["is_test"] = serde_json::json!(true);
            metadata["framework"] = serde_json::json!("pytest");
        }

        self.nodes.push(Node {
            id: uri.clone(),
            name: class_name.to_string(),
            kind: kind.clone(),
            filepath: self.filepath.to_string(),
            scope: scope.clone(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("class {class_name}")),
            docstring,
            checksum,
            metadata: Some(metadata),
        });

        // Inheritance edges
        for superclass in &superclasses {
            self.unresolved_references.push(UnresolvedReference {
                source_node_id: uri.clone(),
                target_symbol: superclass.clone(),
                import_source: None,
                kind: EdgeKind::Extends,
                span,
                call_fidelity: CallFidelity::Definite,
                receiver_type_hint: None,
                receiver_var: None,
                arg_count: None,
            });
        }

        // Enter class scope
        self.scope_stack.push(class_name.to_string());

        // Visit body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                // If inside class, functions are methods
                if child.kind() == "function_definition" {
                    self.visit_function(child, Some(&uri));
                } else if child.kind() == "decorated_definition" {
                    self.visit_decorated_method(child, Some(&uri));
                } else {
                    self.visit_node(child);
                }
            }
        }

        self.scope_stack.pop();
    }

    fn visit_function(&mut self, node: TsNode, parent_class_uri: Option<&str>) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let fn_name = self.node_text(name_node);
        let scope = self.current_scope();

        let is_method = parent_class_uri.is_some() || !scope.is_empty();
        let kind = if is_method {
            NodeKind::Method
        } else {
            NodeKind::Function
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, fn_name, &kind);

        // Parameters & Type Hints
        let mut params = Vec::new();
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    params.push(self.node_text(child).to_string());
                } else if child.kind() == "typed_parameter" || child.kind() == "default_parameter" || child.kind() == "typed_default_parameter" {
                    let param_name = child.child_by_field_name("name")
                        .or_else(|| child.named_child(0));
                    if let Some(name_node) = param_name {
                        let name_str = self.node_text(name_node).to_string();
                        params.push(name_str.clone());

                        if let Some(type_node) = child.child_by_field_name("type") {
                            let type_str = self.node_text(type_node).to_string();
                            self.type_bindings.insert(name_str, type_str);
                        }
                    }
                }
            }
        }

        // Return type
        let return_type = node.child_by_field_name("return_type").map(|n| self.node_text(n).to_string());

        // Signature
        let raw_sig = if let Some(ret) = &return_type {
            format!("def {fn_name}({}) -> {ret}", params.join(", "))
        } else {
            format!("def {fn_name}({})", params.join(", "))
        };
        let signature = SignatureNormalizer::normalize(&raw_sig);

        // Visibility
        let visibility = self.extract_visibility(fn_name);

        // Docstring
        let docstring = self.extract_body_docstring(node.child_by_field_name("body"));

        // Checksum
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        // Pytest / test detection
        let is_test = fn_name.starts_with("test_") || fn_name.ends_with("_test")
            || (self.filepath.contains("test") && (fn_name.starts_with("test") || fn_name.ends_with("test")))
            || (scope.starts_with("Test") && fn_name.starts_with("test"));

        let test_type = if self.filepath.contains("e2e") {
            "e2e"
        } else if self.filepath.contains("integration") {
            "integration"
        } else {
            "unit"
        };

        let mut metadata = serde_json::json!({
            "visibility": visibility.as_str(),
            "span": span,
            "params": params,
        });

        if let Some(ret) = return_type {
            metadata["return_type"] = serde_json::json!(ret);
        }
        if is_test {
            metadata["is_test"] = serde_json::json!(true);
            metadata["framework"] = serde_json::json!("pytest");
            metadata["test_type"] = serde_json::json!(test_type);
        }

        self.nodes.push(Node {
            id: uri.clone(),
            name: fn_name.to_string(),
            kind,
            filepath: self.filepath.to_string(),
            scope: scope.clone(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(signature),
            docstring,
            checksum,
            metadata: Some(metadata),
        });

        // If parent class exists, emit `defines` edge
        if let Some(parent_uri) = parent_class_uri {
            self.edges.push(Edge::new(parent_uri.to_string(), uri.clone(), EdgeKind::Defines));
        }

        // Enter function scope for body inspection
        self.scope_stack.push(fn_name.to_string());

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }

        self.scope_stack.pop();
    }

    fn visit_decorated(&mut self, node: TsNode) {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                decorators.push(self.node_text(child).trim_start_matches('@').trim().to_string());
            } else if child.kind() == "function_definition" {
                self.visit_function(child, None);
                self.annotate_last_node_decorators(&decorators, Some(child));
            } else if child.kind() == "class_definition" {
                self.visit_class(child);
                self.annotate_last_node_decorators(&decorators, None);
            }
        }
    }

    fn visit_decorated_method(&mut self, node: TsNode, parent_class_uri: Option<&str>) {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                decorators.push(self.node_text(child).trim_start_matches('@').trim().to_string());
            } else if child.kind() == "function_definition" {
                self.visit_function(child, parent_class_uri);
                self.annotate_last_node_decorators(&decorators, Some(child));
            }
        }
    }

    fn annotate_last_node_decorators(&mut self, decorators: &[String], func_node: Option<TsNode>) {
        let (handler_id, handler_name, handler_span) = match self.nodes.last() {
            Some(n) => (n.id.clone(), n.name.clone(), Span::new(0, 0, n.start_line.unwrap_or(1), n.start_col.unwrap_or(1), n.end_line.unwrap_or(1), n.end_col.unwrap_or(1))),
            None => return,
        };

        let is_celery = decorators.iter().any(|d| d.contains("shared_task") || d.contains("task"));
        let is_pytest_mark = decorators.iter().any(|d| d.starts_with("pytest.mark") || d.starts_with("mark."));
        if let Some(last_node) = self.nodes.last_mut() {
            if let Some(meta) = &mut last_node.metadata {
                meta["decorators"] = serde_json::json!(decorators);
                if is_celery {
                    meta["is_celery_task"] = serde_json::json!(true);
                }
                if is_pytest_mark {
                    meta["is_test"] = serde_json::json!(true);
                    meta["framework"] = serde_json::json!("pytest");
                }
            }
        }

        let func_has_depends = func_node.map_or(false, |fn_node| {
            let text = self.node_text(fn_node);
            text.contains("Depends(") || text.contains("Security(")
        });

        // Extract FastAPI, Flask, and DRF route decorators
        let mut new_routes = Vec::new();
        for d in decorators {
            // 1. FastAPI: app.get("/path"), router.post("/path"), etc.
            let http_verbs = ["get", "post", "put", "delete", "patch", "options", "head"];
            for verb in &http_verbs {
                let pattern_prefix = format!(".{verb}(");
                if let Some(idx) = d.find(&pattern_prefix) {
                    let obj = &d[..idx];
                    if obj.contains("app") || obj.contains("router") || obj.contains("api") {
                        let method_upper = verb.to_uppercase();
                        let inside = &d[idx + pattern_prefix.len()..];
                        let path = self.extract_first_string_arg(inside);
                        if !path.is_empty() {
                            let has_depends = func_has_depends || d.contains("Depends") || decorators.iter().any(|dec| dec.contains("login_required") || dec.contains("jwt_required") || dec.contains("auth"));
                            new_routes.push((method_upper, path, "fastapi", has_depends));
                        }
                    }
                }
            }

            // 2. Flask: app.route("/path", methods=["GET", "POST"]), bp.route(...)
            if let Some(idx) = d.find(".route(") {
                let inside = &d[idx + 7..];
                let path = self.extract_first_string_arg(inside);
                if !path.is_empty() {
                    let mut methods = vec!["GET".to_string()];
                    if let Some(m_idx) = inside.find("methods=") {
                        let after_m = &inside[m_idx + 8..];
                        if let Some(end_m) = after_m.find(']') {
                            let m_list = &after_m[..end_m];
                            methods = m_list.split(',')
                                .map(|s| s.trim().trim_matches(['\'', '"', '[', ']']).to_uppercase())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                    }
                    let has_auth = decorators.iter().any(|dec| dec.contains("login_required") || dec.contains("jwt_required") || dec.contains("auth"));
                    for m in methods {
                        new_routes.push((m, path.clone(), "flask", has_auth));
                    }
                }
            }

            // 3. Django REST Framework: action(detail=True, methods=['post', 'delete'], url_path='reset-password')
            if d.starts_with("action(") {
                let inside = &d[7..];
                let mut methods = vec!["POST".to_string()];
                if let Some(m_idx) = inside.find("methods=") {
                    let after_m = &inside[m_idx + 8..];
                    if let Some(end_m) = after_m.find(']') {
                        let m_list = &after_m[..end_m];
                        methods = m_list.split(',')
                            .map(|s| s.trim().trim_matches(['\'', '"', '[', ']']).to_uppercase())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                let mut path = format!("/{handler_name}");
                if let Some(p_idx) = inside.find("url_path=") {
                    let after_p = &inside[p_idx + 9..];
                    let raw_p = self.extract_first_string_arg(after_p);
                    if !raw_p.is_empty() {
                        path = if raw_p.starts_with('/') { raw_p } else { format!("/{raw_p}") };
                    }
                }
                let has_auth = decorators.iter().any(|dec| dec.contains("permission_classes") || dec.contains("auth"));
                for m in methods {
                    new_routes.push((m, path.clone(), "django", has_auth));
                }
            }
        }

        for (method, path, framework, is_auth) in new_routes {
            let route_name = format!("{method} {path}");
            let route_uri = build_uri(self.filepath, "routes", &route_name, &NodeKind::Route);
            self.nodes.push(Node {
                id: route_uri.clone(),
                name: route_name.clone(),
                kind: NodeKind::Route,
                filepath: self.filepath.to_string(),
                scope: "routes".to_string(),
                start_line: Some(handler_span.start_line),
                start_col: Some(handler_span.start_col),
                end_line: Some(handler_span.end_line),
                end_col: Some(handler_span.end_col),
                signature: Some(format!("{method} {path}")),
                docstring: None,
                checksum: None,
                metadata: Some(serde_json::json!({
                    "http_method": method,
                    "raw_pattern": path,
                    "framework": framework,
                    "handler": handler_name,
                    "auth": if is_auth { "protected" } else { "public" },
                })),
            });
            self.edges.push(Edge::new(route_uri, handler_id.clone(), EdgeKind::MapsTo));
        }
    }

    fn extract_first_string_arg(&self, s: &str) -> String {
        let mut in_quote = None;
        let mut result = String::new();
        for c in s.chars() {
            if let Some(q) = in_quote {
                if c == q {
                    break;
                } else {
                    result.push(c);
                }
            } else if c == '\'' || c == '"' {
                in_quote = Some(c);
            }
        }
        result
    }

    fn visit_call(&mut self, node: TsNode) {
        let fn_node = match node.child_by_field_name("function") {
            Some(n) => n,
            None => return,
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let caller_id = self.nodes.last().map(|n| n.id.clone()).unwrap_or_default();

        let fn_text = self.node_text(fn_node);

        // Check for Django routes: path('api/v1/invoices/', views.invoice_list)
        if fn_text == "path" || fn_text == "re_path" {
            self.extract_django_route(node, span);
            return;
        }

        // Inline SQL Slicing check: cursor.execute("SELECT ...") or execute("SELECT ...")
        if self.options.extract_inline_sql && (fn_text.ends_with(".execute") || fn_text == "execute" || fn_text.ends_with(".raw") || fn_text == "raw") {
            self.extract_inline_sql(node, span, &caller_id);
        }

        // Determine if receiver call or direct call
        if fn_node.kind() == "attribute" {
            let receiver_node = fn_node.child_by_field_name("object");
            let method_node = fn_node.child_by_field_name("attribute");

            if let (Some(rec), Some(method)) = (receiver_node, method_node) {
                let rec_name = self.node_text(rec);
                let method_name = self.node_text(method);

                // Type-Assisted Call Resolution:
                let receiver_type = self.type_bindings.get(rec_name).cloned();
                let call_fidelity = if rec_name == "self" || receiver_type.is_some() {
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
                    receiver_var: Some(rec_name.to_string()),
                    arg_count: self.count_arguments(node),
                });
            }
        } else {
            // Direct standalone call (e.g. helper())
            self.unresolved_references.push(UnresolvedReference {
                source_node_id: caller_id,
                target_symbol: fn_text.to_string(),
                import_source: None,
                kind: EdgeKind::Calls,
                span,
                call_fidelity: CallFidelity::Definite,
                receiver_type_hint: None,
                receiver_var: None,
                arg_count: self.count_arguments(node),
            });
        }
    }

    fn extract_django_route(&mut self, node: TsNode, span: Span) {
        if let Some(args) = node.child_by_field_name("arguments") {
            let mut cursor = args.walk();
            let mut route_pattern = None;
            let mut target_view = None;

            for child in args.children(&mut cursor) {
                if child.kind() == "string" && route_pattern.is_none() {
                    let raw = self.node_text(child);
                    route_pattern = Some(raw.trim_matches(|c| c == '\'' || c == '"').to_string());
                } else if target_view.is_none() && (child.kind() == "identifier" || child.kind() == "attribute" || child.kind() == "call") {
                    target_view = Some(self.node_text(child).to_string());
                }
            }

            if let Some(pattern) = route_pattern {
                let uri = build_uri(self.filepath, "urls", &pattern, &NodeKind::Route);
                self.nodes.push(Node {
                    id: uri.clone(),
                    name: pattern.clone(),
                    kind: NodeKind::Route,
                    filepath: self.filepath.to_string(),
                    scope: "urls".to_string(),
                    start_line: Some(span.start_line),
                    start_col: Some(span.start_col),
                    end_line: Some(span.end_line),
                    end_col: Some(span.end_col),
                    signature: Some(format!("path('{pattern}')")),
                    docstring: None,
                    checksum: None,
                    metadata: Some(serde_json::json!({
                        "route_pattern": pattern,
                        "target_view": target_view,
                    })),
                });

                if let Some(view) = target_view {
                    self.unresolved_references.push(UnresolvedReference {
                        source_node_id: uri,
                        target_symbol: view,
                        import_source: None,
                        kind: EdgeKind::Other("maps_to".to_string()),
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

    fn extract_inline_sql(&mut self, node: TsNode, span: Span, caller_id: &str) {
        if let Some(args) = node.child_by_field_name("arguments") {
            let mut cursor = args.walk();
            for child in args.children(&mut cursor) {
                if child.kind() == "string" {
                    let raw_sql = self.node_text(child).trim_matches(|c| c == '\'' || c == '"');
                    let uppercase_sql = raw_sql.to_uppercase();

                    // Find FROM or JOIN table names
                    if let Some(idx) = uppercase_sql.find(" FROM ") {
                        let after_from = &raw_sql[idx + 6..].trim_start();
                        let table_name: String = after_from
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect();

                        if !table_name.is_empty() && !caller_id.is_empty() {
                            self.unresolved_references.push(UnresolvedReference {
                                source_node_id: caller_id.to_string(),
                                target_symbol: table_name,
                                import_source: None,
                                kind: EdgeKind::References,
                                span,
                                call_fidelity: CallFidelity::Definite,
                                receiver_type_hint: None,
                                receiver_var: None,
                                arg_count: None,
                            });
                        }
                    }
                    break;
                }
            }
        }
    }

    fn visit_import(&mut self, node: TsNode) {
        let span = self.line_index.span_for_node(&node, self.content);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "dotted_name" {
                let mod_name = self.node_text(child);
                self.unresolved_references.push(UnresolvedReference {
                    source_node_id: format!("cartograph://{}::::*#file", self.filepath),
                    target_symbol: mod_name.to_string(),
                    import_source: Some(mod_name.to_string()),
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

    fn visit_import_from(&mut self, node: TsNode) {
        let span = self.line_index.span_for_node(&node, self.content);
        let module_name = node.child_by_field_name("module_name").map(|n| self.node_text(n).to_string());

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "dotted_name" {
                let sym_name = self.node_text(child);
                if Some(sym_name) != module_name.as_deref() {
                    self.unresolved_references.push(UnresolvedReference {
                        source_node_id: format!("cartograph://{}::::*#file", self.filepath),
                        target_symbol: sym_name.to_string(),
                        import_source: module_name.clone(),
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

    fn visit_assignment(&mut self, node: TsNode) {
        if let (Some(left), Some(right)) = (node.child_by_field_name("left"), node.child_by_field_name("right")) {
            let left_text = self.node_text(left);
            let right_text = self.node_text(right);

            // Simple RHS object creation type inference: invoice = Invoice(...)
            if let Some(idx) = right_text.find('(') {
                let constructor = &right_text[..idx].trim();
                if !constructor.is_empty() && constructor.chars().next().map_or(false, |c| c.is_uppercase()) {
                    self.type_bindings.insert(left_text.to_string(), constructor.to_string());
                }
            }
        }

        // Traverse children so calls inside assignments (e.g. urlpatterns = [path(...)]) are extracted
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child);
        }
    }

    fn extract_visibility(&self, name: &str) -> Visibility {
        if name.starts_with("__") && !name.ends_with("__") {
            Visibility::Private
        } else if name.starts_with('_') && !name.ends_with('_') {
            Visibility::Private
        } else {
            Visibility::Public
        }
    }

    fn extract_body_docstring(&self, body_node: Option<TsNode>) -> Option<String> {
        let body = body_node?;
        let mut cursor = body.walk();
        if let Some(child) = body.children(&mut cursor).next() {
            if child.kind() == "expression_statement" {
                if let Some(str_node) = child.named_child(0) {
                    if str_node.kind() == "string" {
                        let raw = self.node_text(str_node);
                        let cleaned = TriviaBinder::clean_python_docstring(raw);
                        if !cleaned.is_empty() {
                            return Some(cleaned);
                        }
                    }
                }
            }
        }
        None
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_parse_classes_and_functions() {
        let code = r#"
class Invoice(models.Model):
    """Represents a customer invoice."""
    def calculate_total(self, tax: Decimal) -> Decimal:
        return self.subtotal + tax

def process_invoice(inv: Invoice):
    inv.calculate_total(10)
"#;
        let parser = PythonParser::new();
        let ctx = ParseContext::new("models.py", code);
        let parsed = parser.parse(&ctx).expect("Parsing failed");

        assert_eq!(parsed.nodes.len(), 3);

        // Check model
        let invoice_node = &parsed.nodes[0];
        assert_eq!(invoice_node.name, "Invoice");
        assert_eq!(invoice_node.kind, NodeKind::Model);
        assert_eq!(invoice_node.docstring.as_deref(), Some("Represents a customer invoice."));

        // Check method
        let calc_node = &parsed.nodes[1];
        assert_eq!(calc_node.name, "calculate_total");
        assert_eq!(calc_node.kind, NodeKind::Method);
        assert_eq!(calc_node.scope, "Invoice");

        // Check function
        let proc_node = &parsed.nodes[2];
        assert_eq!(proc_node.name, "process_invoice");
        assert_eq!(proc_node.kind, NodeKind::Function);

        // Check type-assisted call: inv is typed as Invoice
        let call_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "Invoice::calculate_total")
            .expect("Should resolve typed call");
        assert_eq!(call_ref.call_fidelity, CallFidelity::Definite);
        assert_eq!(call_ref.receiver_type_hint.as_deref(), Some("Invoice"));
    }

    #[test]
    fn test_python_inline_sql_slicing() {
        let code = r#"
def fetch_orders(cursor):
    cursor.execute("SELECT id, amount FROM orders WHERE status = 'paid'")
"#;
        let parser = PythonParser::new();
        let ctx = ParseContext::new("views.py", code);
        let parsed = parser.parse(&ctx).expect("Parsing failed");

        let sql_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "orders")
            .expect("Should extract referenced table from inline SQL");
        assert_eq!(sql_ref.kind, EdgeKind::References);
    }

    #[test]
    fn test_python_django_routes() {
        let code = r#"
urlpatterns = [
    path('api/v1/invoices/', InvoiceViewSet.as_view()),
]
"#;
        let parser = PythonParser::new();
        let ctx = ParseContext::new("urls.py", code);
        let parsed = parser.parse(&ctx).expect("Parsing failed");

        let route_node = parsed
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Route)
            .expect("Should extract route node");
        assert_eq!(route_node.name, "api/v1/invoices/");
    }

    #[test]
    fn test_python_test_extraction() {
        let code = r#"
class TestUserService:
    @pytest.mark.asyncio
    def test_create_user(self):
        assert True

def test_standalone_helper():
    pass
"#;
        let parser = PythonParser::new();
        let ctx = ParseContext::new("tests/test_users.py", code);
        let parsed = parser.parse(&ctx).expect("Parsing failed");

        let class_node = parsed
            .nodes
            .iter()
            .find(|n| n.name == "TestUserService")
            .expect("Should extract test class");
        assert_eq!(class_node.metadata.as_ref().unwrap()["is_test"], true);
        assert_eq!(class_node.metadata.as_ref().unwrap()["framework"], "pytest");

        let method_node = parsed
            .nodes
            .iter()
            .find(|n| n.name == "test_create_user")
            .expect("Should extract test method");
        assert_eq!(method_node.metadata.as_ref().unwrap()["is_test"], true);
        assert_eq!(method_node.metadata.as_ref().unwrap()["framework"], "pytest");

        let fn_node = parsed
            .nodes
            .iter()
            .find(|n| n.name == "test_standalone_helper")
            .expect("Should extract standalone test func");
        assert_eq!(fn_node.metadata.as_ref().unwrap()["is_test"], true);
    }

    #[test]
    fn test_python_fastapi_and_flask_routes() {
        let code = r#"
@app.get("/users/{user_id}")
async def get_user(user_id: int, auth: bool = Depends(get_current_user)):
    return {"id": user_id}

@app.route("/items/<int:item_id>", methods=["GET", "POST"])
def manage_item(item_id):
    return "ok"
"#;
        let parser = PythonParser::new();
        let ctx = ParseContext::new("routes.py", code);
        let parsed = parser.parse(&ctx).expect("Parsing Python routes failed");

        let fastapi_route = parsed.nodes.iter().find(|n| n.name == "GET /users/{user_id}").expect("FastAPI route not found");
        assert_eq!(fastapi_route.kind, NodeKind::Route);
        let meta = fastapi_route.metadata.as_ref().unwrap();
        assert_eq!(meta["framework"], "fastapi");
        assert_eq!(meta["auth"], "protected");

        let flask_get = parsed.nodes.iter().find(|n| n.name == "GET /items/<int:item_id>").expect("Flask GET route not found");
        assert_eq!(flask_get.kind, NodeKind::Route);
        assert_eq!(flask_get.metadata.as_ref().unwrap()["framework"], "flask");

        let flask_post = parsed.nodes.iter().find(|n| n.name == "POST /items/<int:item_id>").expect("Flask POST route not found");
        assert_eq!(flask_post.kind, NodeKind::Route);
    }
}

