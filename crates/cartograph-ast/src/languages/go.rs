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

pub static GO_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_go::LANGUAGE.into();
    Query::new(&language, include_str!("../../queries/go/tags.scm"))
        .expect("Failed to compile Go Tree-sitter query")
});

pub struct GoParser;

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GoParser {
    pub fn new() -> Self {
        Self
    }

    fn init_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_go::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| AstError::ParserInit(format!("Failed to set Go language: {e:?}")))?;
        Ok(parser)
    }

    fn parse_internal(&self, ctx: &ParseContext, tree: &Tree) -> Result<ParsedFile> {
        let line_index = LineIndex::new(ctx.content);
        let root = tree.root_node();

        let mut extractor = GoExtractor {
            filepath: ctx.filepath,
            content: ctx.content,
            line_index: &line_index,
            options: &ctx.options,
            nodes: Vec::new(),
            edges: Vec::new(),
            unresolved_references: Vec::new(),
            diagnostics: Vec::new(),
        };

        extractor.visit_node(root);

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: "go".to_string(),
            is_generated: false,
            nodes: extractor.nodes,
            edges: extractor.edges,
            unresolved_references: extractor.unresolved_references,
            diagnostics: extractor.diagnostics,
        })
    }
}

impl LanguageParser for GoParser {
    fn language_id(&self) -> &'static str {
        "go"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["go"]
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let mut parser = Self::init_parser()?;
        let tree = parser
            .parse(ctx.content, None)
            .ok_or_else(|| AstError::ParseFailure {
                filepath: ctx.filepath.to_string(),
                message: "Tree-sitter returned empty syntax tree for Go source".to_string(),
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
            message: "Incremental re-parse returned empty syntax tree for Go".to_string(),
        })?;

        let parsed = self.parse_internal(ctx, &new_tree)?;
        Ok((parsed, new_tree))
    }
}

struct GoExtractor<'a> {
    filepath: &'a str,
    content: &'a str,
    line_index: &'a LineIndex,
    options: &'a crate::model::ParseOptions,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    unresolved_references: Vec<UnresolvedReference>,
    diagnostics: Vec<AstDiagnostic>,
}

impl<'a> GoExtractor<'a> {
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
            "type_spec" => self.visit_type_spec(node),
            "function_declaration" => self.visit_function(node),
            "method_declaration" => self.visit_method(node),
            "call_expression" => self.visit_call(node),
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit_node(child);
                }
            }
        }
    }

    fn visit_type_spec(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let type_name = self.node_text(name_node);
        let type_kind_node = node.child_by_field_name("type");

        let kind = if let Some(tk) = type_kind_node {
            if tk.kind() == "interface_type" {
                NodeKind::Interface
            } else {
                NodeKind::Struct
            }
        } else {
            NodeKind::Struct
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, "", type_name, &kind);

        let visibility = self.extract_go_visibility(type_name);
        let docstring = self.find_leading_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: type_name.to_string(),
            kind: kind.clone(),
            filepath: self.filepath.to_string(),
            scope: String::new(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("type {type_name} {}", kind.as_str())),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });
    }

    fn visit_function(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let fn_name = self.node_text(name_node);
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, "", fn_name, &NodeKind::Function);

        let visibility = self.extract_go_visibility(fn_name);
        let docstring = self.find_leading_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: fn_name.to_string(),
            kind: NodeKind::Function,
            filepath: self.filepath.to_string(),
            scope: String::new(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(SignatureNormalizer::normalize(&format!("func {fn_name}()"))),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
    }

    fn visit_method(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let method_name = self.node_text(name_node);

        // Extract receiver type: func (s *Server) Start()
        let receiver_type = self.extract_receiver_type(node).unwrap_or_default();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &receiver_type, method_name, &NodeKind::Method);

        let visibility = self.extract_go_visibility(method_name);
        let docstring = self.find_leading_comments(node);
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
            scope: receiver_type.clone(),
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(SignatureNormalizer::normalize(&format!("func ({receiver_type}) {method_name}()"))),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });

        if !receiver_type.is_empty() {
            let parent_uri = build_uri(self.filepath, "", &receiver_type, &NodeKind::Struct);
            self.edges.push(Edge::new(parent_uri, uri, EdgeKind::Defines));
        }

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
    }

    fn visit_call(&mut self, node: TsNode) {
        let span = self.line_index.span_for_node(&node, self.content);
        let caller_id = self.nodes.last().map(|n| n.id.clone()).unwrap_or_default();

        if let Some(fn_node) = node.child_by_field_name("function") {
            let fn_text = self.node_text(fn_node);
            self.unresolved_references.push(UnresolvedReference {
                source_node_id: caller_id,
                target_symbol: fn_text.to_string(),
                import_source: None,
                kind: EdgeKind::Calls,
                span,
                call_fidelity: CallFidelity::Definite,
                receiver_type_hint: None,
                receiver_var: None,
                arg_count: None,
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child);
        }
    }

    fn extract_receiver_type(&self, method_node: TsNode) -> Option<String> {
        let receiver_list = method_node.child_by_field_name("receiver")?;
        let mut cursor = receiver_list.walk();
        for child in receiver_list.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                if let Some(type_node) = child.child_by_field_name("type") {
                    let text = self.node_text(type_node);
                    return Some(text.trim_start_matches('*').to_string());
                }
            }
        }
        None
    }

    fn extract_go_visibility(&self, name: &str) -> Visibility {
        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
            Visibility::Public
        } else {
            Visibility::PackagePrivate
        }
    }

    fn find_leading_comments(&self, node: TsNode) -> Option<String> {
        let target = if node.kind() == "type_spec" {
            node.parent().unwrap_or(node)
        } else {
            node
        };
        let mut prev = target.prev_sibling();
        let mut comments = Vec::new();

        while let Some(sibling) = prev {
            if sibling.kind() == "comment" {
                let text = self.node_text(sibling);
                comments.push(text);
                prev = sibling.prev_sibling();
            } else {
                break;
            }
        }

        if !comments.is_empty() {
            comments.reverse();
            let raw = comments.join("\n");
            let cleaned = TriviaBinder::clean_line_comments(&raw, "//");
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_struct_interface_and_methods() {
        let code = r#"
package server

// Server manages connections
type Server struct {
    port int
}

type Runner interface {
    Run() error
}

func (s *Server) Start() error {
    return nil
}

func NewServer() *Server {
    return &Server{port: 8080}
}
"#;
        let parser = GoParser::new();
        let ctx = ParseContext::new("server.go", code);
        let parsed = parser.parse(&ctx).expect("Parsing Go failed");

        assert_eq!(parsed.nodes.len(), 4);

        let struct_node = &parsed.nodes[0];
        assert_eq!(struct_node.name, "Server");
        assert_eq!(struct_node.kind, NodeKind::Struct);
        assert_eq!(struct_node.docstring.as_deref(), Some("Server manages connections"));

        let iface_node = &parsed.nodes[1];
        assert_eq!(iface_node.name, "Runner");
        assert_eq!(iface_node.kind, NodeKind::Interface);

        let method_node = &parsed.nodes[2];
        assert_eq!(method_node.name, "Start");
        assert_eq!(method_node.kind, NodeKind::Method);
        assert_eq!(method_node.scope, "Server");

        let fn_node = &parsed.nodes[3];
        assert_eq!(fn_node.name, "NewServer");
        assert_eq!(fn_node.kind, NodeKind::Function);
    }
}
