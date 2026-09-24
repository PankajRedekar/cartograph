pub mod filter;
pub mod languages;
pub mod slicer;
pub mod visitor;

pub use filter::NoiseFilter;
pub use languages::{
    ExtractedRuleCandidate, GoLogicExtractor, PythonLogicExtractor, RubyLogicExtractor,
    RustLogicExtractor, TypeScriptLogicExtractor, generate_rule_id,
};
pub use slicer::{ParameterInfo, SymbolicProvenanceSlicer};
pub use visitor::AstLogicVisitor;

use cartograph_core::{BusinessRule, ExtractLogicConfig};

/// High-level facade for polyglot AST business rule extraction
#[derive(Default, Clone)]
pub struct AstLogicExtractor;

impl AstLogicExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract all business rules from source code content
    pub fn extract_rules(
        &self,
        filepath: &str,
        content: &str,
        config: &ExtractLogicConfig,
    ) -> Vec<BusinessRule> {
        let visitor = AstLogicVisitor::new(content, filepath, config);
        visitor.extract_all()
    }

    /// Extract business rules belonging to or referencing a specific symbol
    pub fn extract_for_symbol(
        &self,
        filepath: &str,
        content: &str,
        symbol_name: &str,
        config: &ExtractLogicConfig,
    ) -> Vec<BusinessRule> {
        let visitor = AstLogicVisitor::new(content, filepath, config);
        visitor.extract_symbol(symbol_name)
    }
}

#[cfg(test)]
mod tests;
