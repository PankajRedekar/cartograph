pub mod ansi;
pub mod gherkin;
pub mod json;
pub mod table;
pub mod xml;

pub use ansi::AnsiFormatter;
pub use gherkin::GherkinFormatter;
pub use json::JsonFormatter;
pub use table::TableFormatter;
pub use xml::XmlFormatter;

use cartograph_core::BusinessRule;

/// Format selector enum for logic extraction output generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOutputFormat {
    Terminal,
    Gherkin,
    Table,
    Xml,
    Json,
}

pub type OutputFormat = LogicOutputFormat;

impl LogicOutputFormat {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "terminal" | "ansi" | "text" => Some(LogicOutputFormat::Terminal),
            "gherkin" | "bdd" | "feature" => Some(LogicOutputFormat::Gherkin),
            "table" | "md" | "markdown" => Some(LogicOutputFormat::Table),
            "xml" | "prompt" => Some(LogicOutputFormat::Xml),
            "json" => Some(LogicOutputFormat::Json),
            _ => None,
        }
    }
}

/// Render a slice of business rules using the chosen format.
pub fn render_rules(format: OutputFormat, rules: &[BusinessRule], title: Option<&str>) -> String {
    match format {
        OutputFormat::Terminal => AnsiFormatter::format(rules, title),
        OutputFormat::Gherkin => GherkinFormatter::format(rules, title),
        OutputFormat::Table => TableFormatter::format(rules, title),
        OutputFormat::Xml => XmlFormatter::format(rules),
        OutputFormat::Json => JsonFormatter::format(rules),
    }
}
