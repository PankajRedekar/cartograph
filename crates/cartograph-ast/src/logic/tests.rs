use crate::logic::AstLogicExtractor;
use cartograph_core::{ExtractLogicConfig, ProvenanceSource, RuleKind};

#[test]
fn test_extract_rust_guard_and_calculation() {
    let code = r#"
pub fn checkout(cart: &Cart, user: &User) -> Result<Order, CheckoutError> {
    if cart.items.len() == 0 {
        return Err(CheckoutError::EmptyCart("Cart cannot be empty"));
    }

    if user.age < 18 {
        return Err(CheckoutError::Underage("Must be 18 or older"));
    }

    let discount = cart.total * 0.15;
    Ok(Order::new(cart.total - discount))
}
"#;
    let config = ExtractLogicConfig::default();
    let extractor = AstLogicExtractor::new();
    let rules = extractor.extract_rules("src/checkout.rs", code, &config);

    assert_eq!(rules.len(), 3, "Expected 3 rules, found {}", rules.len());

    let empty_cart_rule = rules
        .iter()
        .find(|r| r.title.contains("Cart cannot be empty"))
        .unwrap();
    assert_eq!(empty_cart_rule.kind, RuleKind::Validation);
    assert!(empty_cart_rule.confidence >= 0.60);

    let underage_rule = rules
        .iter()
        .find(|r| r.title.contains("Must be 18 or older"))
        .unwrap();
    assert_eq!(underage_rule.kind, RuleKind::Validation);

    let calc_rule = rules
        .iter()
        .find(|r| r.kind == RuleKind::Calculation)
        .unwrap();
    assert_eq!(calc_rule.title, "Calculate discount");
}

#[test]
fn test_extract_typescript_validation() {
    let code = r#"
export function validateBooking(booking: Booking, user: User) {
    if (booking.durationDays <= 0) {
        throw new ValidationError("Duration must be positive");
    }

    if (user.status !== "Active") {
        throw new AuthError("User account must be active");
    }

    const fee = booking.price * 0.05;
    return fee;
}
"#;
    let config = ExtractLogicConfig::default();
    let extractor = AstLogicExtractor::new();
    let rules = extractor.extract_rules("src/booking.ts", code, &config);

    assert_eq!(rules.len(), 3, "Expected 3 rules, found {}", rules.len());

    let duration_rule = rules
        .iter()
        .find(|r| r.title.contains("Duration must be positive"))
        .unwrap();
    assert_eq!(duration_rule.kind, RuleKind::Validation);

    let status_rule = rules
        .iter()
        .find(|r| r.title.contains("User account must be active"))
        .unwrap();
    assert_eq!(status_rule.kind, RuleKind::StateTransition);
}

#[test]
fn test_extract_python_assert_and_exception() {
    let code = r#"
def process_withdrawal(account, amount):
    assert amount > 0, "Withdrawal amount must be strictly positive"

    if account.balance < amount:
        raise ValueError("Insufficient balance for withdrawal")

    fee = amount * 0.02
    return fee
"#;
    let config = ExtractLogicConfig::default();
    let extractor = AstLogicExtractor::new();
    let rules = extractor.extract_rules("app/banking.py", code, &config);

    assert_eq!(rules.len(), 3, "Expected 3 rules, found {}", rules.len());

    let assert_rule = rules
        .iter()
        .find(|r| r.kind == RuleKind::Invariant)
        .unwrap();
    assert!(assert_rule.title.contains("strictly positive"));

    let balance_rule = rules
        .iter()
        .find(|r| r.title.contains("Insufficient balance"))
        .unwrap();
    assert_eq!(balance_rule.kind, RuleKind::Validation);

    let calc_rule = rules
        .iter()
        .find(|r| r.kind == RuleKind::Calculation)
        .unwrap();
    assert_eq!(calc_rule.title, "Calculate fee");
}

#[test]
fn test_extract_go_guards_and_struct_tags() {
    let code = r#"
type PaymentRequest struct {
    Amount float64 `validate:"required,min=1"`
}

func ProcessPayment(amount float64) (string, error) {
    if amount <= 0 {
        return "", fmt.Errorf("amount must be greater than zero")
    }

    fee := amount * 0.03
    return fmt.Sprintf("Fee: %f", fee), nil
}
"#;
    let config = ExtractLogicConfig::default();
    let extractor = AstLogicExtractor::new();
    let rules = extractor.extract_rules("pkg/payment.go", code, &config);

    assert!(
        rules.len() >= 2,
        "Expected at least 2 rules, found {}",
        rules.len()
    );

    let guard_rule = rules
        .iter()
        .find(|r| r.title.contains("amount must be greater than zero"))
        .unwrap();
    assert_eq!(guard_rule.kind, RuleKind::Validation);

    let tag_rule = rules
        .iter()
        .find(|r| r.title.contains("Struct tag validation"))
        .unwrap();
    assert_eq!(tag_rule.kind, RuleKind::Invariant);
}

#[test]
fn test_extract_ruby_validations_and_guards() {
    let code = r#"
class Product
  validates :price, presence: true, numericality: { greater_than: 0 }

  def apply_discount(rate)
    raise ArgumentError, "Invalid discount rate" if rate < 0 || rate > 1.0

    discount = price * rate
    discount
  end
end
"#;
    let config = ExtractLogicConfig::default();
    let extractor = AstLogicExtractor::new();
    let rules = extractor.extract_rules("app/models/product.rb", code, &config);

    assert!(
        rules.len() >= 2,
        "Expected at least 2 rules, found {}",
        rules.len()
    );

    let val_rule = rules
        .iter()
        .find(|r| r.title.contains("ActiveRecord validation"))
        .unwrap();
    assert_eq!(val_rule.kind, RuleKind::Invariant);

    let guard_rule = rules
        .iter()
        .find(|r| r.title.contains("Invalid discount rate"))
        .unwrap();
    assert_eq!(guard_rule.kind, RuleKind::Validation);
}

#[test]
fn test_provenance_tracing_parameters_and_locals() {
    let code = r#"
fn calculate_shipping(weight: f64, destination: String) -> f64 {
    let base_rate = 5.0;
    if weight > 10.0 {
        panic!("Overweight parcel");
    }
    base_rate + weight * 0.5
}
"#;
    let config = ExtractLogicConfig::default();
    let extractor = AstLogicExtractor::new();
    let rules = extractor.extract_rules("src/shipping.rs", code, &config);

    let rule = rules
        .iter()
        .find(|r| r.title.contains("Overweight parcel"))
        .unwrap();
    assert_eq!(rule.variable_provenance.len(), 1);

    let weight_prov = &rule.variable_provenance[0];
    assert_eq!(weight_prov.variable, "weight");
    assert!(matches!(
        weight_prov.source,
        ProvenanceSource::Parameter { index: 0, .. }
    ));
}
