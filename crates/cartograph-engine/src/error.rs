use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] cartograph_core::CartographError),

    #[error("AST error: {0}")]
    Ast(#[from] cartograph_ast::AstError),

    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Directory walk error: {0}")]
    Walk(#[from] ignore::Error),

    #[error("File watcher error: {0}")]
    Watcher(#[from] notify::Error),

    #[error("Engine error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
