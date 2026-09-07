use cartograph_ast::AstRegistry;

#[test]
fn test_zero_panic_on_broken_syntax() {
    let registry = AstRegistry::standard();

    // Incomplete Python syntax (unclosed paren, missing colon)
    let broken_python = r#"
class Broken:
    def unfinished(self, x
    def valid_helper(self):
        return 42
"#;
    let py_res = registry.parse_file("broken.py", broken_python, None);
    assert!(py_res.is_ok(), "Parser should not panic on broken Python syntax");
    let res = py_res.unwrap();
    // Valid surrounding helper should still be extracted!
    assert!(res.nodes.iter().any(|n| n.name == "valid_helper" || n.name == "Broken"));

    // Incomplete TypeScript syntax
    let broken_ts = r#"
export function broken(a: string, {
    const x = 1;
"#;
    let ts_res = registry.parse_file("broken.ts", broken_ts, None);
    assert!(ts_res.is_ok(), "Parser should not panic on broken TS syntax");

    // Incomplete Ruby syntax
    let broken_ruby = r#"
class BrokenRuby
    def foo(
end
"#;
    let rb_res = registry.parse_file("broken.rb", broken_ruby, None);
    assert!(rb_res.is_ok(), "Parser should not panic on broken Ruby syntax");

    // Empty file
    let empty_res = registry.parse_file("empty.rs", "", None);
    assert!(empty_res.is_ok(), "Parser should not panic on empty file");
}
