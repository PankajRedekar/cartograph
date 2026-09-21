use thiserror::Error;

#[derive(Error, Debug)]
pub enum AstError {
    #[error("Language '{0}' is unsupported or not enabled")]
    UnsupportedLanguage(String),

    #[error("Tree-sitter parser initialization failed: {0}")]
    ParserInit(String),

    #[error("Query compilation error: {0}")]
    QueryCompilation(String),

    #[error("Parsing error in file '{filepath}': {message}")]
    ParseFailure { filepath: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] cartograph_core::CartographError),
}

pub type Result<T> = std::result::Result<T, AstError>;
