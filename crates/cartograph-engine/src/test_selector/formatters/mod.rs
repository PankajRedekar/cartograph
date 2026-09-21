pub mod json;
pub mod markdown;
pub mod matrix;
pub mod script;
pub mod terminal;

pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use matrix::GithubMatrixFormatter;
pub use script::ScriptFormatter;
pub use terminal::TerminalFormatter;

use cartograph_core::error::Result;
use cartograph_core::model::TestSelectionReport;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TestSelectorOutputFormat {
    #[default]
    Terminal,
    Markdown,
    Json,
    GithubMatrix,
    Script,
}

impl FromStr for TestSelectorOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "terminal" | "console" | "text" => Ok(TestSelectorOutputFormat::Terminal),
            "markdown" | "md" => Ok(TestSelectorOutputFormat::Markdown),
            "json" => Ok(TestSelectorOutputFormat::Json),
            "matrix" | "github-matrix" | "github_matrix" => {
                Ok(TestSelectorOutputFormat::GithubMatrix)
            }
            "script" | "bash" | "sh" => Ok(TestSelectorOutputFormat::Script),
            other => Err(format!(
                "Unknown format: '{other}'. Expected terminal, markdown, json, matrix, or script."
            )),
        }
    }
}

impl TestSelectorOutputFormat {
    pub fn format(
        &self,
        report: &TestSelectionReport,
        command_only: bool,
        files_only: bool,
    ) -> Result<String> {
        match self {
            TestSelectorOutputFormat::Terminal => {
                Ok(TerminalFormatter::format(report, command_only, files_only))
            }
            TestSelectorOutputFormat::Markdown => Ok(MarkdownFormatter::format(report)),
            TestSelectorOutputFormat::Json => JsonFormatter::format(report),
            TestSelectorOutputFormat::GithubMatrix => GithubMatrixFormatter::format(report),
            TestSelectorOutputFormat::Script => Ok(ScriptFormatter::format(report)),
        }
    }
}
