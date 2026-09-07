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

pub static RUST_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_rust::LANGUAGE.into();
    Query::new(&language, include_str!("../../queries/rust/tags.scm"))
        .expect("Failed to compile Rust Tree-sitter query")
});

pub struct RustParser;

impl Default for RustParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RustParser {
    pub fn new() -> Self {
        Self
    }

    fn init_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| AstError::ParserInit(format!("Failed to set Rust language: {e:?}")))?;
        Ok(parser)
    }

    fn parse_internal(&self, ctx: &ParseContext, tree: &Tree) -> Result<ParsedFile> {
        let line_index = LineIndex::new(ctx.content);
        let root = tree.root_node();

        let mut extractor = RustExtractor {
            filepath: ctx.filepath,
            content: ctx.content,
            line_index: &line_index,
            options: &ctx.options,
            scope_stack: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            unresolved_references: Vec::new(),
            diagnostics: Vec::new(),
        };

        extractor.visit_node(root);

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: "rust".to_string(),
            is_generated: false,
            nodes: extractor.nodes,
            edges: extractor.edges,
            unresolved_references: extractor.unresolved_references,
            diagnostics: extractor.diagnostics,
        })
    }
}

impl LanguageParser for RustParser {
    fn language_id(&self) -> &'static str {
        "rust"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let mut parser = Self::init_parser()?;
        let tree = parser
            .parse(ctx.content, None)
            .ok_or_else(|| AstError::ParseFailure {
                filepath: ctx.filepath.to_string(),
                message: "Tree-sitter returned empty syntax tree for Rust source".to_string(),
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
            message: "Incremental re-parse returned empty syntax tree for Rust".to_string(),
        })?;

        let parsed = self.parse_internal(ctx, &new_tree)?;
        Ok((parsed, new_tree))
    }
}

struct RustExtractor<'a> {
    filepath: &'a str,
    content: &'a str,
    line_index: &'a LineIndex,
    options: &'a crate::model::ParseOptions,
    scope_stack: Vec<String>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    unresolved_references: Vec<UnresolvedReference>,
    diagnostics: Vec<AstDiagnostic>,
}

impl<'a> RustExtractor<'a> {
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
            "struct_item" => self.visit_struct(node),
            "enum_item" => self.visit_enum(node),
            "trait_item" => self.visit_trait(node),
            "impl_item" => self.visit_impl(node),
            "function_item" => self.visit_function(node),
            "call_expression" => self.visit_call(node),
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.visit_node(child);
                }
            }
        }
    }

    fn visit_struct(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let struct_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, struct_name, &NodeKind::Struct);

        let visibility = self.extract_rust_visibility(node);
        let docstring = self.find_leading_doc_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: struct_name.to_string(),
            kind: NodeKind::Struct,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("struct {struct_name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });
    }

    fn visit_enum(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let enum_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, enum_name, &NodeKind::Enum);

        let visibility = self.extract_rust_visibility(node);
        let docstring = self.find_leading_doc_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: enum_name.to_string(),
            kind: NodeKind::Enum,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("enum {enum_name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });
    }

    fn visit_trait(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let trait_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, trait_name, &NodeKind::Interface);

        let visibility = self.extract_rust_visibility(node);
        let docstring = self.find_leading_doc_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: trait_name.to_string(),
            kind: NodeKind::Interface,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("trait {trait_name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });
    }

    fn visit_impl(&mut self, node: TsNode) {
        let type_node = match node.child_by_field_name("type") {
            Some(n) => n,
            None => return,
        };
        let type_name = self.node_text(type_node);
        let trait_name = node.child_by_field_name("trait").map(|t| self.node_text(t));

        let span = self.line_index.span_for_node(&node, self.content);

        // If implementing a trait (impl Trait for Struct), create Implements edge
        if let Some(tr) = trait_name {
            let source_uri = build_uri(self.filepath, "", type_name, &NodeKind::Struct);
            self.unresolved_references.push(UnresolvedReference {
                source_node_id: source_uri,
                target_symbol: tr.to_string(),
                import_source: None,
                kind: EdgeKind::Implements,
                span,
                call_fidelity: CallFidelity::Definite,
                receiver_type_hint: None,
                receiver_var: None,
                arg_count: None,
            });
        }

        self.scope_stack.push(type_name.to_string());
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
        self.scope_stack.pop();
    }

    fn visit_function(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let fn_name = self.node_text(name_node);
        let scope = self.current_scope();
        let is_method = !scope.is_empty();

        let kind = if is_method {
            NodeKind::Method
        } else {
            NodeKind::Function
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, fn_name, &kind);

        let visibility = self.extract_rust_visibility(node);
        let docstring = self.find_leading_doc_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

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
            signature: Some(SignatureNormalizer::normalize(&format!("fn {fn_name}()"))),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({
                "visibility": visibility.as_str(),
                "span": span,
            })),
        });

        // defines edge if inside an impl block
        if let Some(parent) = self.scope_stack.last() {
            let parent_uri = build_uri(self.filepath, "", parent, &NodeKind::Struct);
            self.edges.push(Edge {
                source_id: parent_uri,
                target_id: uri,
                kind: EdgeKind::Defines,
                metadata: None,
            });
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

    fn extract_rust_visibility(&self, node: TsNode) -> Visibility {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let text = self.node_text(child);
                if text.contains("crate") {
                    return Visibility::PackagePrivate;
                } else if text.starts_with("pub") {
                    return Visibility::Public;
                }
            }
        }
        Visibility::Private
    }

    fn find_leading_doc_comments(&self, node: TsNode) -> Option<String> {
        let mut prev = node.prev_sibling();
        let mut comments = Vec::new();

        while let Some(sibling) = prev {
            if sibling.kind() == "line_comment" {
                let text = self.node_text(sibling);
                if text.starts_with("///") || text.starts_with("//!") {
                    comments.push(text);
                }
                prev = sibling.prev_sibling();
            } else {
                break;
            }
        }

        if !comments.is_empty() {
            comments.reverse();
            let raw = comments.join("\n");
            let cleaned = TriviaBinder::clean_line_comments(&raw, "///");
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
    fn test_rust_struct_trait_and_impl() {
        let code = r#"
/// Represents a user
pub struct User {
    pub name: String,
}

pub trait Authenticatable {
    fn authenticate(&self) -> bool;
}

impl Authenticatable for User {
    fn authenticate(&self) -> bool {
        true
    }
}
"#;
        let parser = RustParser::new();
        let ctx = ParseContext::new("src/user.rs", code);
        let parsed = parser.parse(&ctx).expect("Parsing Rust failed");

        assert_eq!(parsed.nodes.len(), 3);
        assert_eq!(parsed.nodes[0].name, "User");
        assert_eq!(parsed.nodes[0].kind, NodeKind::Struct);
        assert_eq!(parsed.nodes[0].docstring.as_deref(), Some("Represents a user"));

        assert_eq!(parsed.nodes[1].name, "Authenticatable");
        assert_eq!(parsed.nodes[1].kind, NodeKind::Interface);

        assert_eq!(parsed.nodes[2].name, "authenticate");
        assert_eq!(parsed.nodes[2].kind, NodeKind::Method);
        assert_eq!(parsed.nodes[2].scope, "User");

        // Check Implements edge
        let impl_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "Authenticatable")
            .expect("Should resolve Authenticatable trait implementation");
        assert_eq!(impl_ref.kind, EdgeKind::Implements);
    }
}
