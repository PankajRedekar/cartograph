pub mod agent_xml;
pub mod json;
pub mod markdown;
pub mod sarif;
pub mod terminal;

pub use agent_xml::AgentXmlFormatter;
pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use sarif::SarifFormatter;
pub use terminal::TerminalFormatter;

use cartograph_core::model::LintArchReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchOutputFormat {
    Terminal,
    Markdown,
    Sarif,
    Json,
    AgentXml,
}

impl ArchOutputFormat {
    pub fn format_report(&self, report: &LintArchReport) -> String {
        match self {
            ArchOutputFormat::Terminal => TerminalFormatter::format(report),
            ArchOutputFormat::Markdown => MarkdownFormatter::format(report),
            ArchOutputFormat::Sarif => SarifFormatter::format(report),
            ArchOutputFormat::Json => JsonFormatter::format(report),
            ArchOutputFormat::AgentXml => AgentXmlFormatter::format(report),
        }
    }
}
