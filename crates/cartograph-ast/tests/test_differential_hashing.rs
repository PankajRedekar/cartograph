use cartograph_ast::traits::LanguageParser;
use cartograph_ast::{ParseContext, PythonParser};

#[test]
fn test_differential_hashing_preserves_sibling_checksums() {
    let original_code = r#"
class AccountService:
    def deposit(self, amount: int) -> int:
        return amount

    def withdraw(self, amount: int) -> int:
        return amount

    def check_balance(self) -> int:
        return 100
"#;

    // Developer modifies only `withdraw`
    let modified_code = r#"
class AccountService:
    def deposit(self, amount: int) -> int:
        return amount

    def withdraw(self, amount: int) -> int:
        # Added security validation
        if amount < 0:
            raise ValueError("negative")
        return amount

    def check_balance(self) -> int:
        return 100
"#;

    let parser = PythonParser::new();
    let ctx1 = ParseContext::new("account.py", original_code);
    let parsed1 = parser.parse(&ctx1).expect("Original parse failed");

    let ctx2 = ParseContext::new("account.py", modified_code);
    let parsed2 = parser.parse(&ctx2).expect("Modified parse failed");

    let orig_deposit = parsed1.nodes.iter().find(|n| n.name == "deposit").unwrap();
    let mod_deposit = parsed2.nodes.iter().find(|n| n.name == "deposit").unwrap();

    let orig_balance = parsed1
        .nodes
        .iter()
        .find(|n| n.name == "check_balance")
        .unwrap();
    let mod_balance = parsed2
        .nodes
        .iter()
        .find(|n| n.name == "check_balance")
        .unwrap();

    let orig_withdraw = parsed1.nodes.iter().find(|n| n.name == "withdraw").unwrap();
    let mod_withdraw = parsed2.nodes.iter().find(|n| n.name == "withdraw").unwrap();

    // Sibling functions deposit and check_balance have IDENTICAL checksums
    assert_eq!(orig_deposit.checksum, mod_deposit.checksum);
    assert_eq!(orig_balance.checksum, mod_balance.checksum);

    // Modified function withdraw has a DIFFERENT checksum
    assert_ne!(orig_withdraw.checksum, mod_withdraw.checksum);
}
