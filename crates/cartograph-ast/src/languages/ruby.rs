use std::sync::LazyLock;
use tree_sitter::{Node as TsNode, Parser, Query, Tree};

use crate::error::{AstError, Result};
use crate::hash::compute_node_checksum;
use crate::line_index::LineIndex;
use crate::model::{
    AstDiagnostic, CallFidelity, DiagnosticSeverity, ParseContext, ParsedFile, UnresolvedReference,
    Visibility,
};
use crate::traits::LanguageParser;
use crate::trivia::{SignatureNormalizer, TriviaBinder};
use cartograph_core::{Edge, EdgeKind, Node, NodeKind, build_uri};

pub static RUBY_TAGS_QUERY: LazyLock<Query> = LazyLock::new(|| {
    let language = tree_sitter_ruby::LANGUAGE.into();
    Query::new(&language, include_str!("../../queries/ruby/tags.scm"))
        .expect("Failed to compile Ruby Tree-sitter query")
});

pub struct RubyParser;

impl Default for RubyParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RubyParser {
    pub fn new() -> Self {
        Self
    }

    fn init_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_ruby::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| AstError::ParserInit(format!("Failed to set Ruby language: {e:?}")))?;
        Ok(parser)
    }

    fn parse_internal(&self, ctx: &ParseContext, tree: &Tree) -> Result<ParsedFile> {
        let line_index = LineIndex::new(ctx.content);
        let root = tree.root_node();

        let mut extractor = RubyExtractor {
            filepath: ctx.filepath,
            content: ctx.content,
            line_index: &line_index,
            options: &ctx.options,
            scope_stack: Vec::new(),
            namespace_stack: Vec::new(),
            current_visibility: Visibility::Public,
            nodes: Vec::new(),
            edges: Vec::new(),
            unresolved_references: Vec::new(),
            diagnostics: Vec::new(),
        };

        extractor.visit_node(root);

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: "ruby".to_string(),
            is_generated: false,
            nodes: extractor.nodes,
            edges: extractor.edges,
            unresolved_references: extractor.unresolved_references,
            diagnostics: extractor.diagnostics,
        })
    }
}

impl LanguageParser for RubyParser {
    fn language_id(&self) -> &'static str {
        "ruby"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["rb", "rake", "gemspec"]
    }

    fn special_filenames(&self) -> &'static [&'static str] {
        &["routes.rb", "Gemfile", "Rakefile"]
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let mut parser = Self::init_parser()?;
        let tree = parser
            .parse(ctx.content, None)
            .ok_or_else(|| AstError::ParseFailure {
                filepath: ctx.filepath.to_string(),
                message: "Tree-sitter returned empty syntax tree for Ruby source".to_string(),
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
            message: "Incremental re-parse returned empty syntax tree for Ruby".to_string(),
        })?;

        let parsed = self.parse_internal(ctx, &new_tree)?;
        Ok((parsed, new_tree))
    }
}

struct RubyExtractor<'a> {
    filepath: &'a str,
    content: &'a str,
    line_index: &'a LineIndex,
    options: &'a crate::model::ParseOptions,
    scope_stack: Vec<String>,
    namespace_stack: Vec<String>,
    current_visibility: Visibility,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    unresolved_references: Vec<UnresolvedReference>,
    diagnostics: Vec<AstDiagnostic>,
}

impl<'a> RubyExtractor<'a> {
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
            "class" => self.visit_class(node),
            "module" => self.visit_module(node),
            "method" => self.visit_method(node, false),
            "singleton_method" => self.visit_method(node, true),
            "command" => self.visit_command(node),
            "call" => self.visit_call(node),
            "identifier" => {
                let text = self.node_text(node);
                if text == "private" {
                    self.current_visibility = Visibility::Private;
                } else if text == "protected" {
                    self.current_visibility = Visibility::Protected;
                } else if text == "public" {
                    self.current_visibility = Visibility::Public;
                }
            }
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

        let mut superclass = None;
        if let Some(super_node) = node.child_by_field_name("superclass") {
            let text = self.node_text(super_node).trim_start_matches('<').trim();
            if !text.is_empty() {
                superclass = Some(text.to_string());
            }
        }

        let is_rails_model = superclass
            .as_deref()
            .is_some_and(|s| s == "ApplicationRecord" || s == "ActiveRecord::Base");
        let is_rails_controller = superclass
            .as_deref()
            .is_some_and(|s| s == "ApplicationController" || s.ends_with("Controller"));

        let kind = if is_rails_model {
            NodeKind::Model
        } else {
            NodeKind::Class
        };

        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, class_name, &kind);

        let docstring = self.find_leading_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        let mut metadata = serde_json::json!({
            "visibility": "public",
            "span": span,
        });
        if is_rails_model {
            metadata["is_rails_model"] = serde_json::json!(true);
        }
        if is_rails_controller {
            metadata["is_rails_controller"] = serde_json::json!(true);
        }
        if let Some(ref sc) = superclass {
            metadata["superclass"] = serde_json::json!(sc);
        }

        self.nodes.push(Node {
            id: uri.clone(),
            name: class_name.to_string(),
            kind,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("class {class_name}")),
            docstring,
            checksum,
            metadata: Some(metadata),
        });

        if let Some(sc) = superclass {
            self.unresolved_references.push(UnresolvedReference {
                source_node_id: uri.clone(),
                target_symbol: sc,
                import_source: None,
                kind: EdgeKind::Extends,
                span,
                call_fidelity: CallFidelity::Definite,
                receiver_type_hint: None,
                receiver_var: None,
                arg_count: None,
            });
        }

        let prev_visibility = self.current_visibility;
        self.current_visibility = Visibility::Public;
        self.scope_stack.push(class_name.to_string());

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }

        self.scope_stack.pop();
        self.current_visibility = prev_visibility;
    }

    fn visit_module(&mut self, node: TsNode) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let mod_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, mod_name, &NodeKind::Module);

        let docstring = self.find_leading_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        self.nodes.push(Node {
            id: uri,
            name: mod_name.to_string(),
            kind: NodeKind::Module,
            filepath: self.filepath.to_string(),
            scope,
            start_line: Some(span.start_line),
            start_col: Some(span.start_col),
            end_line: Some(span.end_line),
            end_col: Some(span.end_col),
            signature: Some(format!("module {mod_name}")),
            docstring,
            checksum,
            metadata: Some(serde_json::json!({ "span": span })),
        });

        self.scope_stack.push(mod_name.to_string());
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
        self.scope_stack.pop();
    }

    fn visit_method(&mut self, node: TsNode, is_singleton: bool) {
        let name_node = match node.child_by_field_name("name") {
            Some(n) => n,
            None => return,
        };
        let method_name = self.node_text(name_node);
        let scope = self.current_scope();
        let span = self.line_index.span_for_node(&node, self.content);
        let uri = build_uri(self.filepath, &scope, method_name, &NodeKind::Method);

        let docstring = self.find_leading_comments(node);
        let checksum = if self.options.compute_node_checksums {
            Some(compute_node_checksum(self.node_text(node)))
        } else {
            None
        };

        let mut metadata = serde_json::json!({
            "visibility": self.current_visibility.as_str(),
            "span": span,
        });
        if is_singleton {
            metadata["is_singleton"] = serde_json::json!(true);
        }

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
            signature: Some(SignatureNormalizer::normalize(&format!(
                "def {method_name}"
            ))),
            docstring,
            checksum,
            metadata: Some(metadata),
        });

        // defines edge if inside class/module
        if let Some(parent_name) = self.scope_stack.last() {
            let parent_uri = build_uri(self.filepath, "", parent_name, &NodeKind::Class);
            self.edges
                .push(Edge::new(parent_uri, uri, EdgeKind::Defines));
        }

        self.scope_stack.push(method_name.to_string());
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.visit_node(child);
            }
        }
        self.scope_stack.pop();
    }

    fn visit_command(&mut self, node: TsNode) {
        self.handle_invocation(node);
    }

    fn visit_call(&mut self, node: TsNode) {
        self.handle_invocation(node);
    }

    fn handle_invocation(&mut self, node: TsNode) {
        let method_node = match node
            .child_by_field_name("method")
            .or_else(|| node.child_by_field_name("name"))
            .or_else(|| node.named_child(0))
        {
            Some(n) => n,
            None => return,
        };
        let method_name = self.node_text(method_node);
        let span = self.line_index.span_for_node(&node, self.content);
        let caller_id = self.nodes.last().map(|n| n.id.clone()).unwrap_or_default();

        // ActiveRecord associations: has_many :line_items, belongs_to :user
        if method_name == "has_many" || method_name == "belongs_to" || method_name == "has_one" {
            if let Some(args) = node.child_by_field_name("arguments")
                && let Some(first_arg) = args.named_child(0)
            {
                let rel_name = self.node_text(first_arg).trim_start_matches(':');
                let model_name = Self::snake_to_pascal(rel_name);
                self.unresolved_references.push(UnresolvedReference {
                    source_node_id: caller_id.clone(),
                    target_symbol: model_name,
                    import_source: None,
                    kind: EdgeKind::References,
                    span,
                    call_fidelity: CallFidelity::Definite,
                    receiver_type_hint: None,
                    receiver_var: None,
                    arg_count: None,
                });
            }
        } else if method_name == "namespace" {
            // Rails route namespace nesting: namespace :api do ... end
            let mut ns_name = String::new();
            if let Some(args) = node.child_by_field_name("arguments")
                && let Some(first_arg) = args.named_child(0)
            {
                ns_name = self
                    .node_text(first_arg)
                    .trim_matches([':', '\'', '"'])
                    .to_string();
            }
            if !ns_name.is_empty() {
                self.namespace_stack.push(ns_name);
            }
            if let Some(block) = node.child_by_field_name("block") {
                let mut cursor = block.walk();
                for child in block.children(&mut cursor) {
                    self.visit_node(child);
                }
            }
            if !self.namespace_stack.is_empty() {
                self.namespace_stack.pop();
            }
            return;
        } else if method_name == "resources" || method_name == "resource" {
            // Rails routes: resources :invoices
            if let Some(args) = node.child_by_field_name("arguments")
                && let Some(first_arg) = args.named_child(0)
            {
                let resource_name = self
                    .node_text(first_arg)
                    .trim_start_matches(':')
                    .trim_matches(['\'', '"']);
                let ns_prefix = if self.namespace_stack.is_empty() {
                    "".to_string()
                } else {
                    format!("/{}", self.namespace_stack.join("/"))
                };
                let base_path = format!("{ns_prefix}/{resource_name}");

                // Master resource node (for backwards compatibility with existing tests)
                let uri = build_uri(self.filepath, "routes", resource_name, &NodeKind::Route);
                self.nodes.push(Node {
                    id: uri,
                    name: resource_name.to_string(),
                    kind: NodeKind::Route,
                    filepath: self.filepath.to_string(),
                    scope: "routes".to_string(),
                    start_line: Some(span.start_line),
                    start_col: Some(span.start_col),
                    end_line: Some(span.end_line),
                    end_col: Some(span.end_col),
                    signature: Some(format!("resources :{resource_name}")),
                    docstring: None,
                    checksum: None,
                    metadata: Some(serde_json::json!({
                        "route_dsl": "resources",
                        "resource": resource_name,
                        "framework": "rails",
                    })),
                });

                // Expand 7 RESTful actions
                let rest_actions = if method_name == "resources" {
                    vec![
                        ("GET", base_path.clone(), "index"),
                        ("POST", base_path.clone(), "create"),
                        ("GET", format!("{base_path}/new"), "new"),
                        ("GET", format!("{base_path}/:id"), "show"),
                        ("GET", format!("{base_path}/:id/edit"), "edit"),
                        ("PATCH", format!("{base_path}/:id"), "update"),
                        ("DELETE", format!("{base_path}/:id"), "destroy"),
                    ]
                } else {
                    vec![
                        ("GET", base_path.clone(), "show"),
                        ("POST", base_path.clone(), "create"),
                        ("GET", format!("{base_path}/new"), "new"),
                        ("GET", format!("{base_path}/edit"), "edit"),
                        ("PATCH", base_path.clone(), "update"),
                        ("DELETE", base_path.clone(), "destroy"),
                    ]
                };

                for (m, path, action) in rest_actions {
                    let route_name = format!("{m} {path}");
                    let route_uri =
                        build_uri(self.filepath, "routes", &route_name, &NodeKind::Route);
                    let controller_target = format!("{}#{}", resource_name, action);

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
                        signature: Some(format!("{m} {path}")),
                        docstring: None,
                        checksum: None,
                        metadata: Some(serde_json::json!({
                            "http_method": m,
                            "raw_pattern": path,
                            "framework": "rails",
                            "handler": controller_target,
                            "auth": "unknown",
                        })),
                    });

                    self.unresolved_references.push(UnresolvedReference {
                        source_node_id: route_uri,
                        target_symbol: controller_target,
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
        } else if ["get", "post", "put", "patch", "delete"].contains(&method_name) {
            // Explicit Rails routes: get '/users/:id', to: 'users#show'
            if let Some(args) = node.child_by_field_name("arguments")
                && let Some(first_arg) = args.named_child(0)
            {
                let raw_path = self.node_text(first_arg).trim_matches(['\'', '"']);
                if raw_path.starts_with('/') {
                    let ns_prefix = if self.namespace_stack.is_empty() {
                        "".to_string()
                    } else {
                        format!("/{}", self.namespace_stack.join("/"))
                    };
                    let full_path = format!("{}{raw_path}", ns_prefix);
                    let method_upper = method_name.to_uppercase();
                    let route_name = format!("{method_upper} {full_path}");
                    let uri = build_uri(self.filepath, "routes", &route_name, &NodeKind::Route);

                    // Find `to: 'controller#action'`
                    let full_text = self.node_text(args);
                    let mut handler = None;
                    if let Some(to_idx) = full_text.find("to:") {
                        let after_to = &full_text[to_idx + 3..];
                        let h = after_to
                            .trim()
                            .trim_matches(['\'', '"', ','])
                            .chars()
                            .take_while(|c| *c != ',' && *c != ' ' && *c != '\n')
                            .collect::<String>();
                        if !h.is_empty() {
                            handler = Some(h.trim_matches(['\'', '"']).to_string());
                        }
                    }

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
                        signature: Some(format!("{method_upper} {full_path}")),
                        docstring: None,
                        checksum: None,
                        metadata: Some(serde_json::json!({
                            "http_method": method_upper,
                            "raw_pattern": full_path,
                            "framework": "rails",
                            "handler": handler,
                            "auth": "unknown",
                        })),
                    });

                    if let Some(h) = handler {
                        self.unresolved_references.push(UnresolvedReference {
                            source_node_id: uri,
                            target_symbol: h,
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
        } else if method_name == "find_by_sql" || method_name == "execute" {
            // Inline SQL slicing in Ruby
            if self.options.extract_inline_sql
                && let Some(args) = node.child_by_field_name("arguments")
                && let Some(arg) = args.named_child(0)
            {
                let raw_sql = self.node_text(arg).trim_matches(|c| c == '\'' || c == '"');
                let uppercase = raw_sql.to_uppercase();
                if let Some(idx) = uppercase.find(" FROM ") {
                    let table: String = raw_sql[idx + 6..]
                        .trim_start()
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !table.is_empty() && !caller_id.is_empty() {
                        self.unresolved_references.push(UnresolvedReference {
                            source_node_id: caller_id.clone(),
                            target_symbol: table,
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
            }
        } else {
            // General method call
            let receiver_name = node
                .child_by_field_name("receiver")
                .map(|r| self.node_text(r).to_string());
            let call_fidelity = if receiver_name.as_deref() == Some("self") {
                CallFidelity::Definite
            } else {
                CallFidelity::DynamicCandidate
            };

            self.unresolved_references.push(UnresolvedReference {
                source_node_id: caller_id,
                target_symbol: method_name.to_string(),
                import_source: None,
                kind: EdgeKind::Calls,
                span,
                call_fidelity,
                receiver_type_hint: None,
                receiver_var: receiver_name,
                arg_count: None,
            });
        }

        // Traverse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child);
        }
    }

    fn find_leading_comments(&self, node: TsNode) -> Option<String> {
        let mut prev = node.prev_sibling();
        let mut comments = Vec::new();

        while let Some(sibling) = prev {
            if sibling.kind() == "comment" {
                comments.push(self.node_text(sibling));
                prev = sibling.prev_sibling();
            } else {
                break;
            }
        }

        if !comments.is_empty() {
            comments.reverse();
            let raw = comments.join("\n");
            let cleaned = TriviaBinder::clean_line_comments(&raw, "#");
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }

        None
    }

    fn snake_to_pascal(s: &str) -> String {
        let singular = if let Some(stripped) = s.strip_suffix('s') {
            stripped
        } else {
            s
        };

        singular
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_model_associations() {
        let code = r#"
# Customer Invoice Model
class Invoice < ApplicationRecord
  belongs_to :customer
  has_many :line_items

  def calculate_total
    total = 0
  end

  private

  def validate_terms
    true
  end
end
"#;
        let parser = RubyParser::new();
        let ctx = ParseContext::new("app/models/invoice.rb", code);
        let parsed = parser.parse(&ctx).expect("Parsing Ruby failed");

        assert_eq!(parsed.nodes.len(), 3);

        let model_node = &parsed.nodes[0];
        assert_eq!(model_node.name, "Invoice");
        assert_eq!(model_node.kind, NodeKind::Model);
        assert_eq!(
            model_node.docstring.as_deref(),
            Some("Customer Invoice Model")
        );

        let calc_node = &parsed.nodes[1];
        assert_eq!(calc_node.name, "calculate_total");
        assert_eq!(calc_node.kind, NodeKind::Method);
        let calc_meta = calc_node.metadata.as_ref().unwrap();
        assert_eq!(calc_meta["visibility"], "public");

        let priv_node = &parsed.nodes[2];
        assert_eq!(priv_node.name, "validate_terms");
        let priv_meta = priv_node.metadata.as_ref().unwrap();
        assert_eq!(priv_meta["visibility"], "private");

        // Check association references
        let customer_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "Customer")
            .expect("Should resolve customer association");
        assert_eq!(customer_ref.kind, EdgeKind::References);

        let line_item_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "LineItem")
            .expect("Should resolve line_items association");
        assert_eq!(line_item_ref.kind, EdgeKind::References);
    }

    #[test]
    fn test_ruby_routes_dsl() {
        let code = r#"
Rails.application.routes.draw do
  resources :invoices
end
"#;
        let parser = RubyParser::new();
        let ctx = ParseContext::new("config/routes.rb", code);
        let parsed = parser.parse(&ctx).expect("Parsing routes.rb failed");

        let route_node = parsed
            .nodes
            .iter()
            .find(|n| n.kind == NodeKind::Route)
            .expect("Should find route node");
        assert_eq!(route_node.name, "invoices");
    }

    #[test]
    fn test_ruby_rails_namespace_and_restful_routes() {
        let code = r#"
Rails.application.routes.draw do
  namespace :api do
    namespace :v1 do
      resources :users
      get '/healthz', to: 'health#check'
    end
  end
end
"#;
        let parser = RubyParser::new();
        let ctx = ParseContext::new("config/routes.rb", code);
        let parsed = parser.parse(&ctx).expect("Parsing nested routes failed");

        let get_users = parsed
            .nodes
            .iter()
            .find(|n| n.name == "GET /api/v1/users")
            .expect("GET /api/v1/users not found");
        assert_eq!(get_users.kind, NodeKind::Route);
        let meta = get_users.metadata.as_ref().unwrap();
        assert_eq!(meta["framework"], "rails");
        assert_eq!(meta["handler"], "users#index");

        let post_users = parsed
            .nodes
            .iter()
            .find(|n| n.name == "POST /api/v1/users")
            .expect("POST /api/v1/users not found");
        assert_eq!(post_users.kind, NodeKind::Route);

        let delete_user = parsed
            .nodes
            .iter()
            .find(|n| n.name == "DELETE /api/v1/users/:id")
            .expect("DELETE /api/v1/users/:id not found");
        assert_eq!(delete_user.kind, NodeKind::Route);

        let health_route = parsed
            .nodes
            .iter()
            .find(|n| n.name == "GET /api/v1/healthz")
            .expect("GET /api/v1/healthz not found");
        assert_eq!(health_route.kind, NodeKind::Route);
        assert_eq!(
            health_route.metadata.as_ref().unwrap()["handler"],
            "health#check"
        );
    }
}
