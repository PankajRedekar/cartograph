use std::collections::HashSet;
use tree_sitter::{Node as TsNode, Parser, Tree};
use cartograph_core::{ComplexityMetric, NodeKind};
use crate::error::Result;

/// Complexity extraction result for a single AST symbol (function, method, constructor)
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolComplexity {
    pub name: String,
    pub kind: NodeKind,
    pub scope: String,
    pub start_line: u32,
    pub end_line: u32,
    pub metrics: ComplexityMetric,
    pub fields_accessed: Vec<String>,
    pub methods_called: Vec<String>,
    pub is_method: bool,
    pub parent_class: Option<String>,
}

/// Aggregated complexity metrics for a class, struct, or interface
#[derive(Debug, Clone, PartialEq)]
pub struct ClassComplexity {
    pub name: String,
    pub kind: NodeKind,
    pub scope: String,
    pub start_line: u32,
    pub end_line: u32,
    /// Weighted Methods per Class: sum of member method cyclomatic complexities
    pub wmc: u32,
    pub method_count: usize,
    pub field_count: usize,
    pub loc: u32,
    pub sloc: u32,
    pub fields: Vec<String>,
    pub methods: Vec<String>,
}

/// Complete complexity result for an entire file
#[derive(Debug, Clone, PartialEq)]
pub struct FileComplexityResult {
    pub filepath: String,
    pub language: String,
    pub file_complexity: ComplexityMetric,
    pub symbols: Vec<SymbolComplexity>,
    pub classes: Vec<ClassComplexity>,
}

/// Polyglot Tree-Sitter AST complexity visitor
#[derive(Default, Clone)]
pub struct AstComplexityAnalyzer;

impl AstComplexityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Detect language from filepath extension
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
                "sql" => "sql",
                "md" | "markdown" => "markdown",
                _ => "unknown",
            }
        } else {
            "unknown"
        }
    }

    /// Analyze complexity of source code given a file path
    pub fn analyze(&self, filepath: &str, content: &str) -> Result<FileComplexityResult> {
        let lang = Self::detect_language(filepath);
        self.analyze_language(filepath, lang, content)
    }

    /// Analyze complexity of source code given an explicit language tag
    pub fn analyze_language(&self, filepath: &str, language: &str, content: &str) -> Result<FileComplexityResult> {
        let (loc, sloc, cloc, comment_ratio) = compute_loc_metrics(content, 1, count_lines(content) as u32);

        let tree = parse_cst(language, content);
        if let Some(tree) = tree {
            let mut visitor = ComplexityVisitor::new(content, language);
            visitor.visit_root(tree.root_node());

            let mut file_metric = ComplexityMetric {
                cyclomatic: visitor.file_cyclomatic.max(1),
                cognitive: visitor.file_cognitive,
                loc,
                sloc,
                cloc,
                comment_ratio,
                max_nesting_depth: visitor.file_max_nesting,
                parameter_count: 0,
            };

            // If file has functions, sum or take max appropriately
            if !visitor.symbols.is_empty() {
                let sum_cc: u32 = visitor.symbols.iter().map(|s| s.metrics.cyclomatic).sum();
                let sum_cog: u32 = visitor.symbols.iter().map(|s| s.metrics.cognitive).sum();
                let max_nest: u32 = visitor.symbols.iter().map(|s| s.metrics.max_nesting_depth).max().unwrap_or(0);
                file_metric.cyclomatic = sum_cc.max(1);
                file_metric.cognitive = sum_cog;
                file_metric.max_nesting_depth = max_nest.max(visitor.file_max_nesting);
            }

            // Merge duplicate classes (e.g. Rust struct_item + impl_item)
            let mut merged_classes: Vec<ClassComplexity> = Vec::new();
            let mut class_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

            for cls in visitor.classes {
                if let Some(&idx) = class_map.get(&cls.name) {
                    let existing = &mut merged_classes[idx];
                    existing.start_line = existing.start_line.min(cls.start_line);
                    existing.end_line = existing.end_line.max(cls.end_line);
                    existing.wmc += cls.wmc;
                    for f in cls.fields {
                        if !existing.fields.contains(&f) {
                            existing.fields.push(f);
                        }
                    }
                    for m in cls.methods {
                        if !existing.methods.contains(&m) {
                            existing.methods.push(m);
                        }
                    }
                    existing.method_count = existing.methods.len();
                    existing.field_count = existing.fields.len();
                    let (loc, sloc, _, _) = compute_loc_metrics(content, existing.start_line, existing.end_line);
                    existing.loc = loc;
                    existing.sloc = sloc;
                } else {
                    class_map.insert(cls.name.clone(), merged_classes.len());
                    merged_classes.push(cls);
                }
            }

            Ok(FileComplexityResult {
                filepath: filepath.to_string(),
                language: language.to_string(),
                file_complexity: file_metric,
                symbols: visitor.symbols,
                classes: merged_classes,
            })
        } else {
            // Fallback for languages without Tree-Sitter grammar or parse errors
            let file_metric = ComplexityMetric {
                cyclomatic: 1,
                cognitive: 0,
                loc,
                sloc,
                cloc,
                comment_ratio,
                max_nesting_depth: 0,
                parameter_count: 0,
            };
            Ok(FileComplexityResult {
                filepath: filepath.to_string(),
                language: language.to_string(),
                file_complexity: file_metric,
                symbols: Vec::new(),
                classes: Vec::new(),
            })
        }
    }
}

/// Helper to parse source code into a Tree-sitter Tree
fn parse_cst(language: &str, content: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let lang_res = match language {
        #[cfg(feature = "rust")]
        "rust" => parser.set_language(&tree_sitter_rust::LANGUAGE.into()),
        #[cfg(feature = "typescript")]
        "typescript" | "javascript" => parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
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

fn count_lines(content: &str) -> usize {
    content.lines().count().max(1)
}

/// Computes (LOC, SLOC, CLOC, CommentRatio) for a line range (1-indexed)
pub fn compute_loc_metrics(content: &str, start_line: u32, end_line: u32) -> (u32, u32, u32, f64) {
    let mut loc = 0u32;
    let mut sloc = 0u32;
    let mut cloc = 0u32;

    let lines: Vec<&str> = content.lines().collect();
    let start_idx = (start_line.saturating_sub(1)) as usize;
    let end_idx = (end_line as usize).min(lines.len());

    let mut in_block_comment = false;

    for i in start_idx..end_idx {
        if i >= lines.len() {
            break;
        }
        loc += 1;
        let line = lines[i].trim();

        if line.is_empty() {
            continue;
        }

        if in_block_comment {
            cloc += 1;
            if line.contains("*/") || line.contains("\"\"\"") || line.contains("'''") {
                in_block_comment = false;
            }
            continue;
        }

        if line.starts_with("//") || line.starts_with('#') || line.starts_with("--") {
            cloc += 1;
            continue;
        }

        if line.starts_with("/*") {
            cloc += 1;
            if !line.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if line.starts_with("\"\"\"") || line.starts_with("'''") {
            cloc += 1;
            // Check if it closes on the same line
            if line.len() > 3 && (line[3..].contains("\"\"\"") || line[3..].contains("'''")) {
                // Closed on same line
            } else {
                in_block_comment = true;
            }
            continue;
        }

        sloc += 1;
    }

    let comment_ratio = if sloc + cloc > 0 {
        cloc as f64 / (sloc + cloc) as f64
    } else {
        0.0
    };

    (loc.max(1), sloc, cloc, comment_ratio)
}

/// Visitor that walks the CST to extract complexity
struct ComplexityVisitor<'a> {
    content: &'a str,
    language: &'a str,
    symbols: Vec<SymbolComplexity>,
    classes: Vec<ClassComplexity>,
    current_class: Option<ClassContext>,
    file_cyclomatic: u32,
    file_cognitive: u32,
    file_max_nesting: u32,
}

struct ClassContext {
    name: String,
    kind: NodeKind,
    scope: String,
    start_line: u32,
    end_line: u32,
    fields: Vec<String>,
    methods: Vec<String>,
    wmc: u32,
}

impl<'a> ComplexityVisitor<'a> {
    fn new(content: &'a str, language: &'a str) -> Self {
        Self {
            content,
            language,
            symbols: Vec::new(),
            classes: Vec::new(),
            current_class: None,
            file_cyclomatic: 1,
            file_cognitive: 0,
            file_max_nesting: 0,
        }
    }

    fn visit_root(&mut self, root: TsNode) {
        self.walk_node(root, "", 0);
    }

    fn walk_node(&mut self, node: TsNode, current_scope: &str, nesting: u32) {
        let kind = node.kind();

        // 1. Check for Class / Struct / Interface declaration
        if self.is_class_node(kind) {
            let class_name = self.extract_name(node).unwrap_or_else(|| "AnonymousClass".to_string());
            let node_kind = self.classify_class_kind(kind);
            let start_line = (node.start_position().row + 1) as u32;
            let end_line = (node.end_position().row + 1) as u32;

            let prev_class = self.current_class.take();
            self.current_class = Some(ClassContext {
                name: class_name.clone(),
                kind: node_kind,
                scope: current_scope.to_string(),
                start_line,
                end_line,
                fields: self.extract_class_fields(node),
                methods: Vec::new(),
                wmc: 0,
            });

            let new_scope = if current_scope.is_empty() {
                class_name.clone()
            } else {
                format!("{current_scope}::{class_name}")
            };

            // Walk children inside class
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.walk_node(child, &new_scope, nesting);
            }

            if let Some(ctx) = self.current_class.take() {
                let (loc, sloc, _, _) = compute_loc_metrics(self.content, ctx.start_line, ctx.end_line);
                self.classes.push(ClassComplexity {
                    name: ctx.name,
                    kind: ctx.kind,
                    scope: ctx.scope,
                    start_line: ctx.start_line,
                    end_line: ctx.end_line,
                    wmc: ctx.wmc,
                    method_count: ctx.methods.len(),
                    field_count: ctx.fields.len(),
                    loc,
                    sloc,
                    fields: ctx.fields,
                    methods: ctx.methods,
                });
            }

            self.current_class = prev_class;
            return;
        }

        // 2. Check for Function / Method declaration
        if self.is_function_node(kind) {
            let fn_name = self.extract_name(node).unwrap_or_else(|| "anonymous_fn".to_string());
            let start_line = (node.start_position().row + 1) as u32;
            let end_line = (node.end_position().row + 1) as u32;

            let is_method = self.current_class.is_some() || self.is_method_kind(kind, node);
            let parent_class_name = self.current_class.as_ref().map(|c| c.name.clone());

            let param_count = self.extract_param_count(node);
            let (loc, sloc, cloc, comment_ratio) = compute_loc_metrics(self.content, start_line, end_line);

            // Compute cyclomatic & cognitive complexity for this function
            let mut fn_analyzer = FunctionComplexityAnalyzer::new(self.content, self.language);
            fn_analyzer.analyze_function(node);

            let metric = ComplexityMetric {
                cyclomatic: fn_analyzer.cyclomatic.max(1),
                cognitive: fn_analyzer.cognitive,
                loc,
                sloc,
                cloc,
                comment_ratio,
                max_nesting_depth: fn_analyzer.max_nesting,
                parameter_count: param_count,
            };

            if let Some(cls) = self.current_class.as_mut() {
                cls.methods.push(fn_name.clone());
                cls.wmc += metric.cyclomatic;
            }

            self.symbols.push(SymbolComplexity {
                name: fn_name,
                kind: if is_method { NodeKind::Method } else { NodeKind::Function },
                scope: current_scope.to_string(),
                start_line,
                end_line,
                metrics: metric,
                fields_accessed: fn_analyzer.fields_accessed.into_iter().collect(),
                methods_called: fn_analyzer.methods_called.into_iter().collect(),
                is_method,
                parent_class: parent_class_name,
            });

            // Do not recurse into function body as child functions are analyzed independently
            return;
        }

        // 3. Fallthrough: walk children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_node(child, current_scope, nesting);
        }
    }

    fn is_class_node(&self, kind: &str) -> bool {
        match self.language {
            "rust" => matches!(kind, "struct_item" | "enum_item" | "trait_item" | "impl_item"),
            "typescript" | "tsx" | "javascript" => matches!(kind, "class_declaration" | "class" | "interface_declaration"),
            "python" => kind == "class_definition",
            "go" => kind == "type_declaration" || kind == "type_spec",
            "ruby" => matches!(kind, "class" | "module"),
            _ => false,
        }
    }

    fn classify_class_kind(&self, kind: &str) -> NodeKind {
        match kind {
            "interface_declaration" | "trait_item" => NodeKind::Interface,
            "struct_item" | "type_spec" => NodeKind::Struct,
            "enum_item" => NodeKind::Enum,
            "module" => NodeKind::Module,
            _ => NodeKind::Class,
        }
    }

    fn is_function_node(&self, kind: &str) -> bool {
        match self.language {
            "rust" => kind == "function_item",
            "typescript" | "tsx" | "javascript" => {
                matches!(kind, "function_declaration" | "method_definition" | "arrow_function" | "function_expression")
            }
            "python" => kind == "function_definition",
            "go" => matches!(kind, "function_declaration" | "method_declaration"),
            "ruby" => matches!(kind, "method" | "singleton_method"),
            _ => false,
        }
    }

    fn is_method_kind(&self, kind: &str, node: TsNode) -> bool {
        match self.language {
            "go" => kind == "method_declaration" || node.child_by_field_name("receiver").is_some(),
            "typescript" | "tsx" | "javascript" => kind == "method_definition",
            "ruby" => kind == "method",
            _ => false,
        }
    }

    fn extract_name(&self, node: TsNode) -> Option<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            let bytes = self.content.as_bytes();
            let text = &bytes[name_node.start_byte()..name_node.end_byte()];
            return String::from_utf8(text.to_vec()).ok();
        }

        // Check for Go type_spec name or Rust impl_item type
        if self.language == "rust" && node.kind() == "impl_item" {
            if let Some(type_node) = node.child_by_field_name("type") {
                let bytes = self.content.as_bytes();
                return String::from_utf8(bytes[type_node.start_byte()..type_node.end_byte()].to_vec()).ok();
            }
        }

        None
    }

    fn extract_param_count(&self, node: TsNode) -> u32 {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut count = 0u32;
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                let k = child.kind();
                // Filter out commas, parentheses, etc.
                if k != "(" && k != ")" && k != "," && k != "comment" {
                    // In Python, ignore 'self' and 'cls'
                    if self.language == "python" {
                        let text = self.get_node_text(child);
                        if text == "self" || text == "cls" {
                            continue;
                        }
                    }
                    count += 1;
                }
            }
            count
        } else {
            0
        }
    }

    fn extract_class_fields(&self, node: TsNode) -> Vec<String> {
        let mut fields = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match self.language {
                "rust" => {
                    if child.kind() == "field_declaration_list" {
                        let mut inner_cursor = child.walk();
                        for field in child.children(&mut inner_cursor) {
                            if field.kind() == "field_declaration" {
                                if let Some(name_node) = field.child_by_field_name("name") {
                                    fields.push(self.get_node_text(name_node));
                                }
                            }
                        }
                    }
                }
                "typescript" | "tsx" | "javascript" => {
                    if child.kind() == "class_body" || child.kind() == "interface_body" {
                        let mut inner_cursor = child.walk();
                        for member in child.children(&mut inner_cursor) {
                            if matches!(member.kind(), "public_field_definition" | "field_definition" | "property_signature") {
                                if let Some(name_node) = member.child_by_field_name("name") {
                                    fields.push(self.get_node_text(name_node));
                                }
                            }
                        }
                    }
                }
                "go" => {
                    if child.kind() == "type_spec" {
                        if let Some(type_node) = child.child_by_field_name("type") {
                            if type_node.kind() == "struct_type" {
                                if let Some(field_list) = type_node.child_by_field_name("fields") {
                                    let mut inner_cursor = field_list.walk();
                                    for field in field_list.children(&mut inner_cursor) {
                                        if field.kind() == "field_declaration" {
                                            if let Some(name_node) = field.child_by_field_name("name") {
                                                fields.push(self.get_node_text(name_node));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        fields
    }

    fn get_node_text(&self, node: TsNode) -> String {
        let bytes = self.content.as_bytes();
        String::from_utf8_lossy(&bytes[node.start_byte()..node.end_byte()]).to_string()
    }
}

/// Analyzer for a single function or method's body
struct FunctionComplexityAnalyzer<'a> {
    content: &'a str,
    language: &'a str,
    cyclomatic: u32,
    cognitive: u32,
    max_nesting: u32,
    fields_accessed: HashSet<String>,
    methods_called: HashSet<String>,
}

impl<'a> FunctionComplexityAnalyzer<'a> {
    fn new(content: &'a str, language: &'a str) -> Self {
        Self {
            content,
            language,
            cyclomatic: 1, // McCabe baseline v(G) = 1
            cognitive: 0,
            max_nesting: 0,
            fields_accessed: HashSet::new(),
            methods_called: HashSet::new(),
        }
    }

    fn analyze_function(&mut self, fn_node: TsNode) {
        let mut cursor = fn_node.walk();
        for child in fn_node.children(&mut cursor) {
            // Find function body or block
            let k = child.kind();
            if k == "body" || k == "block" || k == "statement_block" || k == "body_statement" {
                self.walk_body(child, 0);
            } else if k != "parameters" && k != "name" && k != "return_type" {
                self.walk_body(child, 0);
            }
        }
    }

    fn walk_body(&mut self, node: TsNode, current_nesting: u32) {
        let kind = node.kind();
        let mut next_nesting = current_nesting;

        // Check for McCabe Cyclomatic and Sonar Cognitive increments
        match self.language {
            "rust" => self.handle_rust_node(node, kind, current_nesting, &mut next_nesting),
            "typescript" | "tsx" | "javascript" => {
                self.handle_ts_node(node, kind, current_nesting, &mut next_nesting)
            }
            "python" => self.handle_python_node(node, kind, current_nesting, &mut next_nesting),
            "go" => self.handle_go_node(node, kind, current_nesting, &mut next_nesting),
            "ruby" => self.handle_ruby_node(node, kind, current_nesting, &mut next_nesting),
            _ => {}
        }

        self.max_nesting = self.max_nesting.max(next_nesting);

        // Check for field access and method call patterns
        self.detect_field_and_method_access(node, kind);

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_body(child, next_nesting);
        }
    }

    fn handle_rust_node(&mut self, node: TsNode, kind: &str, current_nesting: u32, next_nesting: &mut u32) {
        match kind {
            "if_expression" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
                if node.child_by_field_name("alternative").is_some() {
                    self.cognitive += 1; // flat +1 for else
                }
            }
            "while_expression" | "for_expression" | "loop_expression" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "match_expression" => {
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "match_arm" => {
                // Ignore wildcard _ =>
                let is_wildcard = node
                    .child_by_field_name("pattern")
                    .map(|p| p.kind() == "_")
                    .unwrap_or(false);
                if !is_wildcard {
                    self.cyclomatic += 1;
                }
            }
            "try_expression" => {
                self.cyclomatic += 1; // '?' operator
            }
            "binary_expression" => {
                if let Some(op) = node.child_by_field_name("operator") {
                    let op_str = self.get_node_text(op);
                    if op_str == "&&" || op_str == "||" {
                        self.cyclomatic += 1;
                        self.cognitive += 1;
                    }
                }
            }
            "closure_expression" => {
                *next_nesting += 1;
            }
            _ => {}
        }
    }

    fn handle_ts_node(&mut self, node: TsNode, kind: &str, current_nesting: u32, next_nesting: &mut u32) {
        match kind {
            "if_statement" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
                if node.child_by_field_name("alternative").is_some() {
                    self.cognitive += 1;
                }
            }
            "while_statement" | "do_statement" | "for_statement" | "for_in_statement" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "switch_statement" => {
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "switch_case" => {
                self.cyclomatic += 1;
            }
            "catch_clause" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "ternary_expression" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
            }
            "binary_expression" => {
                if let Some(op) = node.child_by_field_name("operator") {
                    let op_str = self.get_node_text(op);
                    if op_str == "&&" || op_str == "||" || op_str == "??" {
                        self.cyclomatic += 1;
                        self.cognitive += 1;
                    }
                }
            }
            "arrow_function" | "function_expression" => {
                *next_nesting += 1;
            }
            _ => {}
        }
    }

    fn handle_python_node(&mut self, node: TsNode, kind: &str, current_nesting: u32, next_nesting: &mut u32) {
        match kind {
            "if_statement" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
                if node.child_by_field_name("alternative").is_some() {
                    self.cognitive += 1;
                }
            }
            "elif_clause" => {
                self.cyclomatic += 1;
                self.cognitive += 1;
            }
            "for_statement" | "while_statement" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "except_clause" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "conditional_expression" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
            }
            "match_statement" => {
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "case_clause" => {
                self.cyclomatic += 1;
            }
            "boolean_operator" => {
                self.cyclomatic += 1;
                self.cognitive += 1;
            }
            "lambda" => {
                *next_nesting += 1;
            }
            _ => {}
        }
    }

    fn handle_go_node(&mut self, node: TsNode, kind: &str, current_nesting: u32, next_nesting: &mut u32) {
        match kind {
            "if_statement" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
                if node.child_by_field_name("alternative").is_some() {
                    self.cognitive += 1;
                }
            }
            "for_statement" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "expression_switch_statement" | "type_switch_statement" | "select_statement" => {
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "expression_case_clause" | "type_case_clause" | "communication_case" => {
                self.cyclomatic += 1;
            }
            "binary_expression" => {
                if let Some(op) = node.child_by_field_name("operator") {
                    let op_str = self.get_node_text(op);
                    if op_str == "&&" || op_str == "||" {
                        self.cyclomatic += 1;
                        self.cognitive += 1;
                    }
                }
            }
            "func_literal" => {
                *next_nesting += 1;
            }
            _ => {}
        }
    }

    fn handle_ruby_node(&mut self, _node: TsNode, kind: &str, current_nesting: u32, next_nesting: &mut u32) {
        match kind {
            "if" | "unless" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "elsif" => {
                self.cyclomatic += 1;
                self.cognitive += 1;
            }
            "else" => {
                self.cognitive += 1;
            }
            "while" | "until" | "for" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "rescue" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "case" => {
                self.cognitive += 1 + current_nesting;
                *next_nesting += 1;
            }
            "when" => {
                self.cyclomatic += 1;
            }
            "conditional" => {
                self.cyclomatic += 1;
                self.cognitive += 1 + current_nesting;
            }
            "binary" => {
                self.cyclomatic += 1;
                self.cognitive += 1;
            }
            "block" => {
                *next_nesting += 1;
            }
            _ => {}
        }
    }

    fn detect_field_and_method_access(&mut self, node: TsNode, kind: &str) {
        match self.language {
            "rust" => {
                // self.foo or self.bar()
                if kind == "field_expression" {
                    if let Some(val) = node.child_by_field_name("value") {
                        if self.get_node_text(val) == "self" {
                            if let Some(field) = node.child_by_field_name("field") {
                                let field_name = self.get_node_text(field);
                                if let Some(parent) = node.parent() {
                                    if parent.kind() == "call_expression" && parent.child_by_field_name("function") == Some(node) {
                                        self.methods_called.insert(field_name);
                                        return;
                                    }
                                }
                                self.fields_accessed.insert(field_name);
                            }
                        }
                    }
                }
            }
            "typescript" | "tsx" | "javascript" => {
                // this.foo or this.bar()
                if kind == "member_expression" {
                    if let Some(obj) = node.child_by_field_name("object") {
                        if self.get_node_text(obj) == "this" {
                            if let Some(prop) = node.child_by_field_name("property") {
                                let prop_name = self.get_node_text(prop);
                                if let Some(parent) = node.parent() {
                                    if parent.kind() == "call_expression" && parent.child_by_field_name("function") == Some(node) {
                                        self.methods_called.insert(prop_name);
                                        return;
                                    }
                                }
                                self.fields_accessed.insert(prop_name);
                            }
                        }
                    }
                }
            }
            "python" => {
                // self.foo or self.bar()
                if kind == "attribute" {
                    if let Some(obj) = node.child_by_field_name("object") {
                        if self.get_node_text(obj) == "self" {
                            if let Some(attr) = node.child_by_field_name("attribute") {
                                let attr_name = self.get_node_text(attr);
                                if let Some(parent) = node.parent() {
                                    if parent.kind() == "call" && parent.child_by_field_name("function") == Some(node) {
                                        self.methods_called.insert(attr_name);
                                        return;
                                    }
                                }
                                self.fields_accessed.insert(attr_name);
                            }
                        }
                    }
                }
            }
            "ruby" => {
                // @foo instance variable
                if kind == "instance_variable" {
                    let text = self.get_node_text(node);
                    let clean = text.trim_start_matches('@').to_string();
                    self.fields_accessed.insert(clean);
                }
            }
            _ => {}
        }
    }

    fn get_node_text(&self, node: TsNode) -> String {
        let bytes = self.content.as_bytes();
        String::from_utf8_lossy(&bytes[node.start_byte()..node.end_byte()]).to_string()
    }
}
