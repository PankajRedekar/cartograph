use cartograph_core::diagnose::FaultCategory;

/// Semantic Fault Classifier categorizing software failures into standardized archetypes.
pub struct FaultClassifier;

impl FaultClassifier {
    /// Classifies a crash into a semantic `FaultCategory`.
    pub fn classify(crash_type: &str, crash_message: &str) -> FaultCategory {
        let combined = format!("{crash_type} {crash_message}").to_lowercase();

        // 1. Null / None / Nil Pointer Dereference
        if combined.contains("nonetype")
            || combined.contains("nullpointerexception")
            || combined.contains("cannot read properties of undefined")
            || combined.contains("cannot read properties of null")
            || combined.contains("cannot read property")
            || combined.contains("nil pointer dereference")
            || combined.contains("option::unwrap")
            || combined.contains("on a `none` value")
            || (combined.contains("unwrap()") && combined.contains("none"))
            || combined.contains("undefined is not an object")
            || combined.contains("null reference")
        {
            return FaultCategory::NullDeref;
        }

        // 2. Index / Bounds Violation
        if combined.contains("indexerror")
            || combined.contains("arrayindexoutofboundsexception")
            || combined.contains("index out of range")
            || combined.contains("out of bounds")
            || combined.contains("slice bounds out of range")
            || combined.contains("range error")
        {
            return FaultCategory::OutOfBounds;
        }

        // 3. Type Drift & Contract Mismatch
        if combined.contains("typeerror")
            || combined.contains("classcastexception")
            || combined.contains("unsupported operand type")
            || combined.contains("type mismatch")
            || combined.contains("cannot cast")
        {
            return FaultCategory::TypeDrift;
        }

        // 4. Missing Guard / Missing Key
        if combined.contains("keyerror")
            || combined.contains("nosuchelementexception")
            || combined.contains("not found in map")
            || combined.contains("unhandled variant")
            || combined.contains("missing required key")
        {
            return FaultCategory::MissingGuard;
        }

        // 5. Invariant / State Breach
        if combined.contains("assertionerror")
            || combined.contains("illegalstateexception")
            || combined.contains("assertion failed")
            || combined.contains("invariant violated")
            || combined.contains("precondition failed")
        {
            return FaultCategory::StateInvariantViolation;
        }

        // 6. Concurrency & Deadlocks
        if combined.contains("deadlock")
            || combined.contains("data race")
            || combined.contains("send on closed channel")
            || combined.contains("mutex poisoned")
            || combined.contains("channel closed")
        {
            return FaultCategory::ConcurrencyPanic;
        }

        // 7. Resource & File Exhaustion
        if combined.contains("too many open files")
            || combined.contains("connectionreseterror")
            || combined.contains("broken pipe")
            || combined.contains("outofmemoryerror")
            || combined.contains("socketexception")
        {
            return FaultCategory::ResourceLeak;
        }

        FaultCategory::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_null_deref() {
        assert_eq!(
            FaultClassifier::classify("AttributeError", "'NoneType' object has no attribute 'id'"),
            FaultCategory::NullDeref
        );
        assert_eq!(
            FaultClassifier::classify("TypeError", "Cannot read properties of undefined (reading 'map')"),
            FaultCategory::NullDeref
        );
        assert_eq!(
            FaultClassifier::classify("panic", "called `Option::unwrap()` on a `None` value"),
            FaultCategory::NullDeref
        );
    }

    #[test]
    fn test_classify_bounds_and_keys() {
        assert_eq!(
            FaultClassifier::classify("IndexError", "list index out of range"),
            FaultCategory::OutOfBounds
        );
        assert_eq!(
            FaultClassifier::classify("KeyError", "'session_token'"),
            FaultCategory::MissingGuard
        );
    }
}
