use cartograph_ast::complexity::AstComplexityAnalyzer;
use cartograph_core::NodeKind;

#[test]
fn test_rust_complexity_extraction() {
    let code = r#"
// A sample complex Rust function
pub fn process_order(id: u32, is_vip: bool, discount: Option<f64>) -> Result<f64, String> {
    let mut total = 100.0;

    // Decision 1
    if is_vip {
        total *= 0.8;
        // Nested decision 2
        if let Some(d) = discount {
            total -= d;
        }
    } else {
        total *= 0.95;
    }

    // Decision 3: match expression
    match id {
        1..=10 => total -= 5.0,
        11..=50 => total -= 2.0,
        _ => {}
    }

    // Decision 4: loop with break
    for i in 0..3 {
        if i == 1 {
            break;
        }
    }

    Ok(total)
}

pub struct BillingManager {
    pub balance: f64,
}

impl BillingManager {
    pub fn new() -> Self {
        Self { balance: 0.0 }
    }

    pub fn deposit(&mut self, amount: f64) {
        if amount > 0.0 {
            self.balance += amount;
        }
    }

    pub fn execute(&mut self) {
        self.deposit(50.0);
    }
}
"#;

    let analyzer = AstComplexityAnalyzer::new();
    let result = analyzer
        .analyze("src/orders.rs", code)
        .expect("Analysis failed");

    assert_eq!(result.language, "rust");
    assert!(!result.symbols.is_empty(), "Should extract symbols");

    // Check process_order
    let order_fn = result
        .symbols
        .iter()
        .find(|s| s.name == "process_order")
        .expect("process_order not found");
    assert_eq!(order_fn.kind, NodeKind::Function);
    assert_eq!(order_fn.metrics.parameter_count, 3);
    assert!(
        order_fn.metrics.cyclomatic >= 6,
        "Expected CC >= 6, got {}",
        order_fn.metrics.cyclomatic
    );
    assert!(
        order_fn.metrics.cognitive >= 4,
        "Expected Cognitive >= 4, got {}",
        order_fn.metrics.cognitive
    );
    assert!(
        order_fn.metrics.max_nesting_depth >= 2,
        "Expected nesting >= 2, got {}",
        order_fn.metrics.max_nesting_depth
    );
    assert!(order_fn.metrics.sloc > 0);

    // Check BillingManager struct
    let billing = result
        .classes
        .iter()
        .find(|c| c.name == "BillingManager")
        .expect("BillingManager not found");
    assert_eq!(billing.kind, NodeKind::Struct);
    assert!(billing.fields.contains(&"balance".to_string()));

    // Check deposit method field access
    let deposit_m = result
        .symbols
        .iter()
        .find(|s| s.name == "deposit")
        .expect("deposit not found");
    assert!(
        deposit_m.fields_accessed.contains(&"balance".to_string()),
        "deposit should access self.balance"
    );

    // Check execute method call
    let execute_m = result
        .symbols
        .iter()
        .find(|s| s.name == "execute")
        .expect("execute not found");
    assert!(
        execute_m.methods_called.contains(&"deposit".to_string()),
        "execute should call self.deposit"
    );
}

#[test]
fn test_typescript_complexity_extraction() {
    let code = r#"
export class OrderService {
    private cache: Map<string, any>;
    private retryCount: number;

    constructor() {
        this.cache = new Map();
        this.retryCount = 3;
    }

    public calculateTax(amount: number, state: string, isExempt?: boolean): number {
        if (isExempt ?? false) {
            return 0;
        }

        let rate = 0.05;
        if (state === "CA") {
            rate = 0.09;
        } else if (state === "NY") {
            rate = 0.085;
        }

        switch (state) {
            case "OR":
            case "NH":
                rate = 0.0;
                break;
            default:
                break;
        }

        return amount * rate;
    }

    public getFromCache(key: string): any {
        return this.cache.get(key);
    }
}
"#;

    let analyzer = AstComplexityAnalyzer::new();
    let result = analyzer
        .analyze("src/order_service.ts", code)
        .expect("Analysis failed");

    assert_eq!(result.language, "typescript");
    let tax_fn = result
        .symbols
        .iter()
        .find(|s| s.name == "calculateTax")
        .expect("calculateTax not found");
    assert_eq!(tax_fn.kind, NodeKind::Method);
    assert_eq!(tax_fn.metrics.parameter_count, 3);
    assert!(
        tax_fn.metrics.cyclomatic >= 6,
        "Expected CC >= 6, got {}",
        tax_fn.metrics.cyclomatic
    );
    assert!(
        tax_fn.metrics.cognitive >= 3,
        "Expected Cog >= 3, got {}",
        tax_fn.metrics.cognitive
    );

    let get_cache = result
        .symbols
        .iter()
        .find(|s| s.name == "getFromCache")
        .expect("getFromCache not found");
    assert!(get_cache.fields_accessed.contains(&"cache".to_string()));
}

#[test]
fn test_python_complexity_extraction() {
    let code = r#"
class PaymentProcessor:
    def __init__(self, api_key: str):
        self.api_key = api_key
        self.connected = False

    def authenticate(self, retries: int = 3) -> bool:
        if not self.api_key:
            return False
        
        while retries > 0:
            try:
                if self.connect():
                    self.connected = True
                    return True
            except ConnectionError:
                retries -= 1
        return False

    def connect(self) -> bool:
        return True
"#;

    let analyzer = AstComplexityAnalyzer::new();
    let result = analyzer
        .analyze("src/payments.py", code)
        .expect("Analysis failed");

    assert_eq!(result.language, "python");
    let auth_fn = result
        .symbols
        .iter()
        .find(|s| s.name == "authenticate")
        .expect("authenticate not found");
    assert_eq!(auth_fn.kind, NodeKind::Method);
    // retries (self ignored in param count)
    assert_eq!(auth_fn.metrics.parameter_count, 1);
    assert!(
        auth_fn.metrics.cyclomatic >= 4,
        "Expected CC >= 4, got {}",
        auth_fn.metrics.cyclomatic
    );
    assert!(
        auth_fn.metrics.cognitive >= 3,
        "Expected Cog >= 3, got {}",
        auth_fn.metrics.cognitive
    );
    assert!(
        auth_fn.methods_called.contains(&"connect".to_string()),
        "authenticate should call connect"
    );
    assert!(
        auth_fn.fields_accessed.contains(&"api_key".to_string())
            || auth_fn.fields_accessed.contains(&"connected".to_string())
    );
}

#[test]
fn test_go_complexity_extraction() {
    let code = r#"
package main

type Server struct {
    port int
    running bool
}

func (s *Server) Start(port int) error {
    s.port = port
    if s.running {
        return nil
    }

    for i := 0; i < 3; i++ {
        if i == 2 {
            s.running = true
            break
        }
    }

    return nil
}
"#;

    let analyzer = AstComplexityAnalyzer::new();
    let result = analyzer
        .analyze("server.go", code)
        .expect("Analysis failed");

    assert_eq!(result.language, "go");
    let start_fn = result
        .symbols
        .iter()
        .find(|s| s.name == "Start")
        .expect("Start not found");
    assert_eq!(start_fn.kind, NodeKind::Method);
    assert!(
        start_fn.metrics.cyclomatic >= 4,
        "Expected CC >= 4, got {}",
        start_fn.metrics.cyclomatic
    );
}

#[test]
fn test_ruby_complexity_extraction() {
    let code = r#"
class Account
  def initialize(balance)
    @balance = balance
  end

  def withdraw(amount)
    if amount > 0 && @balance >= amount
      @balance -= amount
      true
    else
      false
    end
  end
end
"#;

    let analyzer = AstComplexityAnalyzer::new();
    let result = analyzer
        .analyze("account.rb", code)
        .expect("Analysis failed");

    assert_eq!(result.language, "ruby");
    let withdraw_fn = result
        .symbols
        .iter()
        .find(|s| s.name == "withdraw")
        .expect("withdraw not found");
    assert_eq!(withdraw_fn.kind, NodeKind::Method);
    assert!(
        withdraw_fn.metrics.cyclomatic >= 3,
        "Expected CC >= 3, got {}",
        withdraw_fn.metrics.cyclomatic
    );
    assert!(withdraw_fn.fields_accessed.contains(&"balance".to_string()));
}
