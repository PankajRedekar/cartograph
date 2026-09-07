use cartograph_core::{Edge, EdgeKind, Node};
use serde::{Deserialize, Serialize};

/// Exact source code coordinates including byte offsets for zero-copy slicing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub fn new(
        start_byte: usize,
        end_byte: usize,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> Self {
        Self {
            start_byte,
            end_byte,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        if self.start_byte <= self.end_byte && self.end_byte <= source.len() {
            &source[self.start_byte..self.end_byte]
        } else {
            ""
        }
    }
}

/// Symbol access control visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Protected,
    Private,
    PackagePrivate,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Protected => "protected",
            Visibility::Private => "private",
            Visibility::PackagePrivate => "package_private",
        }
    }
}

/// Call resolution fidelity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallFidelity {
    /// Definite call: same-file symbol, explicit import, or verified receiver type
    Definite,
    /// Dynamic candidate call: receiver invocation where type is unverified at single-file scope
    DynamicCandidate,
}

/// Unresolved external reference for Phase 1.3 Linker
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedReference {
    pub source_node_id: String,
    pub target_symbol: String,
    pub import_source: Option<String>,
    pub kind: EdgeKind,
    pub span: Span,
    pub call_fidelity: CallFidelity,
    pub receiver_type_hint: Option<String>,
    pub receiver_var: Option<String>,
    pub arg_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ParseContext<'a> {
    pub filepath: &'a str,
    pub content: &'a str,
    pub options: ParseOptions,
}

impl<'a> ParseContext<'a> {
    pub fn new(filepath: &'a str, content: &'a str) -> Self {
        Self {
            filepath,
            content,
            options: ParseOptions::default(),
        }
    }

    pub fn with_options(filepath: &'a str, content: &'a str, options: ParseOptions) -> Self {
        Self {
            filepath,
            content,
            options,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub extract_docstrings: bool,
    pub extract_signatures: bool,
    pub extract_call_sites: bool,
    pub extract_inline_sql: bool,
    pub compute_node_checksums: bool,
    pub skip_generated: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            extract_docstrings: true,
            extract_signatures: true,
            extract_call_sites: true,
            extract_inline_sql: true,
            compute_node_checksums: true,
            skip_generated: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedFile {
    pub filepath: String,
    pub language: String,
    pub is_generated: bool,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub unresolved_references: Vec<UnresolvedReference>,
    pub diagnostics: Vec<AstDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstDiagnostic {
    pub span: Span,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}
