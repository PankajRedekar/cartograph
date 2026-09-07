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
        };

        extractor.visit_node(root);

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
        self.edges.push(Edge {
            source_id: parent_class_uri.to_string(),
            target_id: uri,
            kind: EdgeKind::Defines,
            metadata: None,
        });

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

        let span = self.line_index.span_for_node(&node, self.content);
        let caller_id = self.nodes.last().map(|n| n.id.clone()).unwrap_or_default();

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
}
