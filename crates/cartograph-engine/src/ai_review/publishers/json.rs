use super::ReviewPublisher;
use cartograph_core::review::ReviewReport;
use std::fs;
use std::path::{Path, PathBuf};

pub struct JsonPublisher {
    pub output_path: Option<PathBuf>,
}

impl JsonPublisher {
    pub fn new(output_path: Option<PathBuf>) -> Self {
        Self { output_path }
    }
}

impl ReviewPublisher for JsonPublisher {
    fn name(&self) -> &'static str {
        "json"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json_str = serde_json::to_string_pretty(report)?;

        if let Some(ref path) = self.output_path {
            let target = if path.is_absolute() {
                path.clone()
            } else {
                repo_root.join(path)
            };

            if let Some(parent) = target.parent() {
                let _ = fs::create_dir_all(parent);
            }

            fs::write(&target, json_str)?;
        } else {
            println!("{}", json_str);
        }

        Ok(())
    }
}
