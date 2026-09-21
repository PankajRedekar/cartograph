use crate::error::Result;
use crate::model::{ParseContext, ParsedFile};
use std::path::Path;

pub trait LanguageParser: Send + Sync {
    /// Unique language identifier (e.g. "python", "typescript", "ruby", "rust", "go", "sql", "markdown")
    fn language_id(&self) -> &'static str;

    /// File extensions handled by this parser (e.g. &["py", "pyi"])
    fn file_extensions(&self) -> &'static [&'static str];

    /// Special filenames handled (e.g. &["routes.rb", "Gemfile", "Dockerfile"])
    fn special_filenames(&self) -> &'static [&'static str] {
        &[]
    }

    /// Check if this parser handles a given file path
    fn can_parse(&self, filepath: &Path) -> bool {
        if let Some(filename) = filepath.file_name().and_then(|f| f.to_str())
            && self.special_filenames().contains(&filename)
        {
            return true;
        }
        if let Some(ext) = filepath.extension().and_then(|e| e.to_str()) {
            self.file_extensions()
                .contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }

    /// Full parse from scratch
    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile>;

    /// Incremental re-parse: updates existing CST in microseconds
    fn parse_incremental(
        &self,
        ctx: &ParseContext,
        old_tree: &tree_sitter::Tree,
        _edit: Option<&tree_sitter::InputEdit>,
    ) -> Result<(ParsedFile, tree_sitter::Tree)> {
        let parsed = self.parse(ctx)?;
        Ok((parsed, old_tree.clone()))
    }
}
