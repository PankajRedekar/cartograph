pub mod adapters;
pub mod git_notes;
pub mod json;
pub mod markdown;
pub mod patch;
pub mod sarif;
pub mod terminal;
pub mod webhook;

use cartograph_core::review::ReviewReport;
use std::path::Path;

pub use git_notes::GitNotesPublisher;
pub use json::JsonPublisher;
pub use markdown::MarkdownPublisher;
pub use patch::PatchPublisher;
pub use sarif::SarifPublisher;
pub use terminal::TerminalPublisher;
pub use webhook::WebhookPublisher;

/// Universal review publisher trait
pub trait ReviewPublisher: Send + Sync {
    /// Friendly name of publisher
    fn name(&self) -> &'static str;

    /// Publish review report to target medium
    fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send;
}
