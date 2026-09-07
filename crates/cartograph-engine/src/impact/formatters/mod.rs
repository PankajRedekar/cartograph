pub mod agent_xml;
pub mod json;
pub mod markdown;
pub mod terminal;

pub use agent_xml::AgentXmlFormatter;
pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use terminal::TerminalFormatter;

use cartograph_core::model::ImpactReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactOutputFormat {
    Terminal,
    Markdown,
    Json,
    AgentXml,
}

impl ImpactOutputFormat {
    pub fn format_report(&self, report: &ImpactReport) -> String {
        match self {
            ImpactOutputFormat::Terminal => TerminalFormatter::format(report),
            ImpactOutputFormat::Markdown => MarkdownFormatter::format(report),
            ImpactOutputFormat::Json => JsonFormatter::format(report),
            ImpactOutputFormat::AgentXml => AgentXmlFormatter::format(report),
        }
    }
}
