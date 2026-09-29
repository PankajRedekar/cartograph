use super::ReviewPublisher;
use cartograph_core::review::{FindingSeverity, ReviewReport};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SarifPublisher {
    pub output_path: PathBuf,
}

impl SarifPublisher {
    pub fn new(output_path: Option<PathBuf>) -> Self {
        Self {
            output_path: output_path.unwrap_or_else(|| PathBuf::from("cartograph_review.sarif")),
        }
    }

    /// Generate OASIS SARIF 2.1.0 JSON representation
    pub fn generate_sarif(report: &ReviewReport) -> serde_json::Value {
        let results: Vec<serde_json::Value> = report
            .findings
            .iter()
            .map(|f| {
                let level = match f.severity {
                    FindingSeverity::Blocker => "error",
                    FindingSeverity::Warning => "warning",
                    FindingSeverity::Info => "note",
                };

                json!({
                    "ruleId": format!("cartograph/{:?}", f.category).to_lowercase(),
                    "level": level,
                    "message": {
                        "text": format!("{}: {}", f.title, f.description)
                    },
                    "locations": [
                        {
                            "physicalLocation": {
                                "artifactLocation": {
                                    "uri": f.file_path.to_string_lossy()
                                },
                                "region": {
                                    "startLine": f.line_span.start_line,
                                    "endLine": f.line_span.end_line
                                }
                            }
                        }
                    ]
                })
            })
            .collect();

        json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "cartograph-ai-review",
                            "semanticVersion": "0.2.0",
                            "informationUri": "https://github.com/pankajredekar/cartograph",
                            "rules": [
                                {
                                    "id": "cartograph/breakingcontract",
                                    "name": "BreakingContract",
                                    "shortDescription": { "text": "Public API contract break or SemVer violation" }
                                },
                                {
                                    "id": "cartograph/unhandledsideeffect",
                                    "name": "UnhandledSideEffect",
                                    "shortDescription": { "text": "Downstream caller failed to handle altered callee contract" }
                                },
                                {
                                    "id": "cartograph/testdeficit",
                                    "name": "TestDeficit",
                                    "shortDescription": { "text": "Missing test coverage for modified symbol" }
                                },
                                {
                                    "id": "cartograph/archviolation",
                                    "name": "ArchViolation",
                                    "shortDescription": { "text": "Architectural layer boundary violation" }
                                },
                                {
                                    "id": "cartograph/invariantrelaxation",
                                    "name": "InvariantRelaxation",
                                    "shortDescription": { "text": "Business rule validation relaxed or deleted" }
                                },
                                {
                                    "id": "cartograph/complexityhotspot",
                                    "name": "ComplexityHotspot",
                                    "shortDescription": { "text": "Cognitive complexity spike or hotspot regression" }
                                }
                            ]
                        }
                    },
                    "results": results
                }
            ]
        })
    }
}

impl ReviewPublisher for SarifPublisher {
    fn name(&self) -> &'static str {
        "sarif"
    }

    async fn publish(
        &self,
        report: &ReviewReport,
        repo_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sarif_json = Self::generate_sarif(report);
        let target = if self.output_path.is_absolute() {
            self.output_path.clone()
        } else {
            repo_root.join(&self.output_path)
        };

        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let formatted = serde_json::to_string_pretty(&sarif_json)?;
        fs::write(&target, formatted)?;
        Ok(())
    }
}
