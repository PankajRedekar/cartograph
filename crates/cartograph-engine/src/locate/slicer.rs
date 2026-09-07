use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use cartograph_core::model::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSlice {
    pub node_id: String,
    pub filepath: String,
    pub start_line: u32,
    pub end_line: u32,
    pub context_start_line: u32,
    pub context_end_line: u32,
    pub content: String,
    pub estimated_tokens: usize,
    pub is_truncated: bool,
}

pub struct CodeSlicer {
    pub repo_root: PathBuf,
    pub context_lines: usize,
    file_cache: HashMap<String, Vec<String>>,
}

impl CodeSlicer {
    pub fn new(repo_root: PathBuf, context_lines: usize) -> Self {
        Self {
            repo_root,
            context_lines,
            file_cache: HashMap::new(),
        }
    }

    /// Extract syntax-padded source code slice for an AST node
    pub fn slice_node(&mut self, node: &Node) -> std::io::Result<CodeSlice> {
        let abs_path = if Path::new(&node.filepath).is_absolute() {
            PathBuf::from(&node.filepath)
        } else {
            self.repo_root.join(&node.filepath)
        };

        if !self.file_cache.contains_key(&node.filepath) {
            let content = fs::read_to_string(&abs_path)?;
            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            self.file_cache.insert(node.filepath.clone(), lines);
        }

        let lines = self.file_cache.get(&node.filepath).unwrap();
        let total_lines = lines.len() as u32;

        let start_line = node.start_line.unwrap_or(1).max(1);
        let end_line = node.end_line.unwrap_or(start_line).min(total_lines).max(start_line);

        let context_start_line = start_line.saturating_sub(self.context_lines as u32).max(1);
        let context_end_line = (end_line + self.context_lines as u32).min(total_lines);

        let mut formatted = String::new();
        for line_num in context_start_line..=context_end_line {
            let idx = (line_num - 1) as usize;
            if idx < lines.len() {
                let is_symbol_line = line_num >= start_line && line_num <= end_line;
                let marker = if is_symbol_line { ">" } else { " " };
                formatted.push_str(&format!("{marker} {:4} | {}\n", line_num, lines[idx]));
            }
        }

        let estimated_tokens = (formatted.len() as f64 / 3.8).ceil() as usize;

        Ok(CodeSlice {
            node_id: node.id.clone(),
            filepath: node.filepath.clone(),
            start_line,
            end_line,
            context_start_line,
            context_end_line,
            content: formatted,
            estimated_tokens,
            is_truncated: false,
        })
    }

    /// Slices signature and docstring only without disk read
    pub fn slice_signature_only(node: &Node) -> CodeSlice {
        let mut content = String::new();
        if let Some(ref doc) = node.docstring {
            for line in doc.lines() {
                content.push_str(&format!("/// {line}\n"));
            }
        }
        if let Some(ref sig) = node.signature {
            content.push_str(sig);
        } else {
            content.push_str(&format!("{} {}", node.kind.as_str(), node.name));
        }

        let start_line = node.start_line.unwrap_or(1);
        let end_line = node.end_line.unwrap_or(start_line);
        let estimated_tokens = (content.len() as f64 / 3.8).ceil() as usize;

        CodeSlice {
            node_id: node.id.clone(),
            filepath: node.filepath.clone(),
            start_line,
            end_line,
            context_start_line: start_line,
            context_end_line: end_line,
            content,
            estimated_tokens,
            is_truncated: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartograph_core::model::NodeKind;
    use tempfile::tempdir;

    #[test]
    fn test_slice_node_with_context() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("service.rs");
        let source_code = r#"// Line 1: header
// Line 2: comment
pub fn target_function() {
    let a = 1;
    let b = 2;
}
// Line 7: footer
// Line 8: end
"#;
        fs::write(&file_path, source_code).unwrap();

        let node = Node {
            id: "cartograph://service.rs::::target_function#function".to_string(),
            name: "target_function".to_string(),
            kind: NodeKind::Function,
            filepath: "service.rs".to_string(),
            scope: "".to_string(),
            start_line: Some(3),
            start_col: Some(1),
            end_line: Some(6),
            end_col: Some(2),
            signature: Some("pub fn target_function()".to_string()),
            docstring: None,
            checksum: None,
            metadata: None,
        };

        let mut slicer = CodeSlicer::new(dir.path().to_path_buf(), 2);
        let slice = slicer.slice_node(&node).unwrap();

        assert_eq!(slice.start_line, 3);
        assert_eq!(slice.end_line, 6);
        assert_eq!(slice.context_start_line, 1);
        assert_eq!(slice.context_end_line, 8);
        assert!(slice.content.contains(">    3 | pub fn target_function()"));
        assert!(slice.content.contains("     1 | // Line 1: header"));
        assert!(slice.content.contains("     8 | // Line 8: end"));
    }
}
