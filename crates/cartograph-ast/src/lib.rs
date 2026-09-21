pub mod complexity;
pub mod error;
pub mod hash;
pub mod languages;
pub mod line_index;
pub mod model;
pub mod query;
pub mod registry;
pub mod traits;
pub mod trivia;

// Re-export core types for ergonomic AST use
pub use cartograph_core::{build_uri, parse_uri, CanonicalUri, Edge, EdgeKind, Node, NodeKind};

pub use complexity::{AstComplexityAnalyzer, ClassComplexity, FileComplexityResult, SymbolComplexity};
pub use error::{AstError, Result};
pub use hash::{compute_composite_hash, compute_node_checksum};
pub use line_index::LineIndex;
pub use model::{
    AstDiagnostic, CallFidelity, DiagnosticSeverity, ParseContext, ParseOptions, ParsedFile, Span,
    UnresolvedReference, Visibility,
};
pub use query::{CapturedNode, QueryEngine};
pub use registry::AstRegistry;
pub use traits::LanguageParser;
pub use trivia::{SignatureNormalizer, TriviaBinder};

// Re-export language parsers
#[cfg(feature = "python")]
pub use languages::python::PythonParser;
#[cfg(feature = "typescript")]
pub use languages::typescript::TypeScriptParser;
#[cfg(feature = "ruby")]
pub use languages::ruby::RubyParser;
#[cfg(feature = "rust")]
pub use languages::rust::RustParser;
#[cfg(feature = "go")]
pub use languages::go::GoParser;
#[cfg(feature = "sql")]
pub use languages::sql::SqlParser;
#[cfg(feature = "doc")]
pub use languages::doc::MarkdownParser;
