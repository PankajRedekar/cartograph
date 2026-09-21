use crate::model::Span;

/// High-speed line and column coordinate index based on byte offset tables.
#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    total_bytes: usize,
}

impl LineIndex {
    /// Build line index from source string
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (byte_offset, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(byte_offset + 1);
            }
        }

        Self {
            line_starts,
            total_bytes: source.len(),
        }
    }

    /// Resolve 1-indexed (line, col) for a given byte offset.
    /// Column counts UTF-8 code points within that line.
    pub fn line_col(&self, byte_offset: usize, source: &str) -> (u32, u32) {
        let offset = byte_offset.min(self.total_bytes);

        // Binary search for the line start
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert_idx) => insert_idx - 1,
        };

        let line_start_byte = self.line_starts[line_idx];
        let line_number = (line_idx + 1) as u32;

        // Calculate 1-indexed column counting UTF-8 chars in line
        let col = if offset >= line_start_byte && offset <= source.len() {
            let line_slice = &source[line_start_byte..offset];
            (line_slice.chars().count() + 1) as u32
        } else {
            1
        };

        (line_number, col)
    }

    /// Create a `Span` from start and end byte offsets.
    pub fn create_span(&self, start_byte: usize, end_byte: usize, source: &str) -> Span {
        let (start_line, start_col) = self.line_col(start_byte, source);
        let (end_line, end_col) = self.line_col(end_byte, source);

        Span::new(
            start_byte, end_byte, start_line, start_col, end_line, end_col,
        )
    }

    /// Convert a Tree-Sitter node directly into a `Span`.
    pub fn span_for_node(&self, node: &tree_sitter::Node, source: &str) -> Span {
        self.create_span(node.start_byte(), node.end_byte(), source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_index_single_line() {
        let source = "hello world";
        let index = LineIndex::new(source);
        assert_eq!(index.line_col(0, source), (1, 1));
        assert_eq!(index.line_col(6, source), (1, 7));
    }

    #[test]
    fn test_line_index_multi_line() {
        let source = "line 1\nline 2\nline 3";
        let index = LineIndex::new(source);
        assert_eq!(index.line_col(0, source), (1, 1));
        assert_eq!(index.line_col(7, source), (2, 1)); // start of line 2
        assert_eq!(index.line_col(14, source), (3, 1)); // start of line 3
        assert_eq!(index.line_col(18, source), (3, 5)); // 'e' in line 3
    }

    #[test]
    fn test_line_index_unicode() {
        let source = "🦀 = crab\n🎉 party";
        let index = LineIndex::new(source);
        // Crab emoji is 4 bytes in UTF-8: bytes 0..4 is crab, byte 4 is space
        assert_eq!(index.line_col(0, source), (1, 1));
        assert_eq!(index.line_col(4, source), (1, 2)); // space after crab
        // "🦀 = crab\n" is 4 + 1 + 1 + 1 + 4 + 1 = 12 bytes. Byte 12 is start of line 2 ("🎉")
        assert_eq!(index.line_col(12, source), (2, 1)); // party line
    }

    #[test]
    fn test_span_creation() {
        let source = "def foo():\n    return 42\n";
        let index = LineIndex::new(source);
        let span = index.create_span(4, 7, source); // "foo"
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_col, 8);
        assert_eq!(span.slice(source), "foo");
    }
}
