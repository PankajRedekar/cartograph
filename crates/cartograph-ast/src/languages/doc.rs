use crate::error::Result;
use crate::hash::compute_node_checksum;
use crate::line_index::LineIndex;
use crate::model::{CallFidelity, ParseContext, ParsedFile, Span, UnresolvedReference};
use crate::traits::LanguageParser;
use cartograph_core::{EdgeKind, Node, NodeKind, build_uri};

pub struct MarkdownParser;

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageParser for MarkdownParser {
    fn language_id(&self) -> &'static str {
        "markdown"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["md", "markdown"]
    }

    fn parse(&self, ctx: &ParseContext) -> Result<ParsedFile> {
        let _line_index = LineIndex::new(ctx.content);
        let mut nodes = Vec::new();
        let mut unresolved_references = Vec::new();

        let mut heading_stack: Vec<(usize, String)> = Vec::new(); // (level, name)
        let mut in_code_block = false;
        let mut code_block_start = 0;
        let mut code_block_lang = String::new();

        for (line_idx, line) in ctx.content.lines().enumerate() {
            let line_num = (line_idx + 1) as u32;
            let trimmed = line.trim();
            let byte_offset = ctx
                .content
                .lines()
                .take(line_idx)
                .map(|l| l.len() + 1)
                .sum::<usize>();

            // Detect fenced code block toggle
            if trimmed.starts_with("```") {
                if in_code_block {
                    // Closing code fence
                    let end_byte = byte_offset + line.len();
                    let span = Span::new(
                        code_block_start,
                        end_byte,
                        line_num,
                        1,
                        line_num,
                        line.len() as u32,
                    );
                    let scope = heading_stack
                        .last()
                        .map(|(_, name)| name.clone())
                        .unwrap_or_default();
                    let uri = build_uri(
                        ctx.filepath,
                        &scope,
                        &format!("code_{line_num}"),
                        &NodeKind::Other("code_block".to_string()),
                    );

                    nodes.push(Node {
                        id: uri,
                        name: format!("code_{line_num}"),
                        kind: NodeKind::Other("code_block".to_string()),
                        filepath: ctx.filepath.to_string(),
                        scope,
                        start_line: Some(line_num),
                        start_col: Some(1),
                        end_line: Some(line_num),
                        end_col: Some(line.len() as u32),
                        signature: Some(format!("```{code_block_lang}")),
                        docstring: None,
                        checksum: None,
                        metadata: Some(serde_json::json!({
                            "span": span,
                            "language": code_block_lang,
                        })),
                    });

                    in_code_block = false;
                } else {
                    // Opening code fence
                    in_code_block = true;
                    code_block_start = byte_offset;
                    code_block_lang = trimmed.trim_start_matches('`').trim().to_string();
                }
                continue;
            }

            if in_code_block {
                continue;
            }

            // Detect Headings (# Heading)
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                let heading_text = trimmed[level..].trim();

                if !heading_text.is_empty() {
                    while let Some((last_level, _)) = heading_stack.last() {
                        if *last_level >= level {
                            heading_stack.pop();
                        } else {
                            break;
                        }
                    }

                    let scope = heading_stack
                        .last()
                        .map(|(_, name)| name.clone())
                        .unwrap_or_default();
                    let end_byte = byte_offset + line.len();
                    let span = Span::new(
                        byte_offset,
                        end_byte,
                        line_num,
                        1,
                        line_num,
                        line.len() as u32,
                    );
                    let uri = build_uri(
                        ctx.filepath,
                        &scope,
                        heading_text,
                        &NodeKind::Other("doc".to_string()),
                    );

                    let checksum = if ctx.options.compute_node_checksums {
                        Some(compute_node_checksum(heading_text))
                    } else {
                        None
                    };

                    nodes.push(Node {
                        id: uri,
                        name: heading_text.to_string(),
                        kind: NodeKind::Other("doc".to_string()),
                        filepath: ctx.filepath.to_string(),
                        scope,
                        start_line: Some(line_num),
                        start_col: Some(1),
                        end_line: Some(line_num),
                        end_col: Some(line.len() as u32),
                        signature: Some(format!("{} {heading_text}", "#".repeat(level))),
                        docstring: None,
                        checksum,
                        metadata: Some(serde_json::json!({
                            "heading_level": level,
                            "span": span,
                        })),
                    });

                    heading_stack.push((level, heading_text.to_string()));
                }
            }

            // Detect cross-document markdown links: [Text](./path/to/file.md)
            if trimmed.contains("](") && trimmed.contains(".md") {
                let mut rest = trimmed;
                while let Some(open_bracket) = rest.find('[') {
                    if let Some(close_bracket) = rest[open_bracket..].find(']') {
                        let after_bracket = &rest[open_bracket + close_bracket + 1..];
                        if after_bracket.starts_with('(')
                            && let Some(close_paren) = after_bracket.find(')')
                        {
                            let target_path = &after_bracket[1..close_paren];
                            if target_path.ends_with(".md") {
                                let span = Span::new(
                                    byte_offset,
                                    byte_offset + line.len(),
                                    line_num,
                                    1,
                                    line_num,
                                    line.len() as u32,
                                );
                                let caller_id =
                                    nodes.last().map(|n| n.id.clone()).unwrap_or_default();

                                unresolved_references.push(UnresolvedReference {
                                    source_node_id: caller_id,
                                    target_symbol: target_path.to_string(),
                                    import_source: Some(target_path.to_string()),
                                    kind: EdgeKind::References,
                                    span,
                                    call_fidelity: CallFidelity::Definite,
                                    receiver_type_hint: None,
                                    receiver_var: None,
                                    arg_count: None,
                                });
                            }
                            rest = &after_bracket[close_paren + 1..];
                            continue;
                        }
                    }
                    break;
                }
            }
        }

        Ok(ParsedFile {
            filepath: ctx.filepath.to_string(),
            language: "markdown".to_string(),
            is_generated: false,
            nodes,
            edges: Vec::new(),
            unresolved_references,
            diagnostics: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_heading_hierarchy_and_links() {
        let code = r#"
# Cartograph Architecture

## Overview
This is the overview section. See [Database Guide](./docs/database.md).

## Subsystems
### Storage Engine
Details on SQLite.
"#;
        let parser = MarkdownParser::new();
        let ctx = ParseContext::new("README.md", code);
        let parsed = parser.parse(&ctx).expect("Parsing Markdown failed");

        assert_eq!(parsed.nodes.len(), 4);

        assert_eq!(parsed.nodes[0].name, "Cartograph Architecture");
        assert_eq!(parsed.nodes[0].kind, NodeKind::Other("doc".to_string()));

        assert_eq!(parsed.nodes[1].name, "Overview");
        assert_eq!(parsed.nodes[1].scope, "Cartograph Architecture");

        assert_eq!(parsed.nodes[2].name, "Subsystems");
        assert_eq!(parsed.nodes[2].scope, "Cartograph Architecture");

        assert_eq!(parsed.nodes[3].name, "Storage Engine");
        assert_eq!(parsed.nodes[3].scope, "Subsystems");

        // Check cross-doc link
        let link_ref = parsed
            .unresolved_references
            .iter()
            .find(|r| r.target_symbol == "./docs/database.md")
            .expect("Should resolve markdown link to database.md");
        assert_eq!(link_ref.kind, EdgeKind::References);
    }
}
