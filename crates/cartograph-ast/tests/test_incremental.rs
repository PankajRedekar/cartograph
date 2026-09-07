use cartograph_ast::languages::python::PythonParser;
use cartograph_ast::traits::LanguageParser;
use cartograph_ast::ParseContext;
use std::time::Instant;

#[test]
fn test_incremental_reparsing_speed_and_correctness() {
    let original_code = r#"
class Calculator:
    def add(self, a: int, b: int) -> int:
        return a + b

    def multiply(self, a: int, b: int) -> int:
        return a * b
"#;

    let modified_code = r#"
class Calculator:
    def add(self, a: int, b: int) -> int:
        return a + b + 1

    def multiply(self, a: int, b: int) -> int:
        return a * b
"#;

    let parser = PythonParser::new();
    let mut ts_parser = tree_sitter::Parser::new();
    let lang = tree_sitter_python::LANGUAGE.into();
    ts_parser.set_language(&lang).unwrap();
    let old_tree = ts_parser.parse(original_code, None).unwrap();

    let ctx = ParseContext::new("calc.py", modified_code);

    // 1. Test with InputEdit
    let edit_offset = original_code.find("return a + b").unwrap() + "return a + b".len();
    let edit = tree_sitter::InputEdit {
        start_byte: edit_offset,
        old_end_byte: edit_offset,
        new_end_byte: edit_offset + 4, // " + 1"
        start_position: tree_sitter::Point { row: 3, column: 20 },
        old_end_position: tree_sitter::Point { row: 3, column: 20 },
        new_end_position: tree_sitter::Point { row: 3, column: 24 },
    };

    let start = Instant::now();
    let (parsed_with_edit, _new_tree) = parser
        .parse_incremental(&ctx, &old_tree, Some(&edit))
        .expect("Incremental parse with edit failed");
    let elapsed = start.elapsed();

    assert_eq!(parsed_with_edit.nodes.len(), 3);
    assert_eq!(parsed_with_edit.nodes[0].name, "Calculator");
    assert_eq!(parsed_with_edit.nodes[1].name, "add");
    assert_eq!(parsed_with_edit.nodes[2].name, "multiply");
    assert!(elapsed.as_millis() < 50, "Incremental parse took too long: {:?}", elapsed);

    // 2. Test fallback without edit
    let (parsed_fallback, _) = parser
        .parse_incremental(&ctx, &old_tree, None)
        .expect("Incremental fallback failed");
    assert_eq!(parsed_fallback.nodes.len(), 3);
    assert_eq!(parsed_fallback.nodes[1].name, "add");
    assert_eq!(parsed_fallback.nodes[2].name, "multiply");
}
