use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

/// Representation of a single captured AST node from a Tree-sitter S-expression query
#[derive(Debug, Clone)]
pub struct CapturedNode<'a> {
    pub name: &'a str,
    pub node: Node<'a>,
    pub text: &'a str,
}

pub struct QueryEngine;

impl QueryEngine {
    /// Execute a compiled Tree-sitter query on an AST root node, collecting all matches and captures.
    pub fn matches<'a>(
        query: &'a Query,
        root_node: Node<'a>,
        source: &'a str,
    ) -> Vec<Vec<CapturedNode<'a>>> {
        let mut cursor = QueryCursor::new();
        let mut matches_iter = cursor.matches(query, root_node, source.as_bytes());
        let mut results = Vec::new();

        while let Some(m) = matches_iter.next() {
            let mut captures = Vec::new();
            for capture in m.captures {
                let name = &query.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = if node.start_byte() <= node.end_byte() && node.end_byte() <= source.len() {
                    &source[node.start_byte()..node.end_byte()]
                } else {
                    ""
                };
                captures.push(CapturedNode {
                    name,
                    node,
                    text,
                });
            }
            results.push(captures);
        }

        results
    }

    /// Flatten all captured nodes from all matches into a single list
    pub fn captures<'a>(
        query: &'a Query,
        root_node: Node<'a>,
        source: &'a str,
    ) -> Vec<CapturedNode<'a>> {
        let mut cursor = QueryCursor::new();
        let mut matches_iter = cursor.matches(query, root_node, source.as_bytes());
        let mut results = Vec::new();

        while let Some(m) = matches_iter.next() {
            for capture in m.captures {
                let name = &query.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = if node.start_byte() <= node.end_byte() && node.end_byte() <= source.len() {
                    &source[node.start_byte()..node.end_byte()]
                } else {
                    ""
                };
                results.push(CapturedNode {
                    name,
                    node,
                    text,
                });
            }
        }

        results
    }
}
