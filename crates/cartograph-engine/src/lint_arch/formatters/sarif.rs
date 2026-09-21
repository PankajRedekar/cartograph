use serde_json::json;

use cartograph_core::config::ArchSeverity;
use cartograph_core::model::LintArchReport;

pub struct SarifFormatter;

impl SarifFormatter {
    pub fn format(report: &LintArchReport) -> String {
        let mut rules = Vec::new();
        let mut results = Vec::new();

        // 1. Boundary Violations to SARIF results
        for v in &report.violations {
            let level = match v.severity {
                ArchSeverity::Error => "error",
                ArchSeverity::Warning => "warning",
                ArchSeverity::Info => "note",
            };

            let start_line = v.line.unwrap_or(1);
            let start_col = v.column.unwrap_or(1);

            let message_text = if let Some(ref hint) = v.remediation_hint {
                format!("{}. Remediation: {}", v.message, hint)
            } else {
                v.message.clone()
            };

            results.push(json!({
                "ruleId": v.rule_name,
                "level": level,
                "message": {
                    "text": message_text
                },
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": v.source_file,
                                "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                                "startLine": start_line,
                                "startColumn": start_col
                            }
                        }
                    }
                ],
                "properties": {
                    "category": v.category,
                    "targetFile": v.target_file,
                    "fingerprint": v.fingerprint
                }
            }));
        }

        // 2. Dependency Cycles to SARIF results
        for (idx, cycle) in report.cycles.iter().enumerate() {
            let level = if cycle.is_runtime_cycle {
                "error"
            } else {
                "warning"
            };
            let first_hop = cycle.hops.first();
            let file_uri = first_hop.map(|h| h.filepath.as_str()).unwrap_or("unknown");
            let line = first_hop.and_then(|h| h.line).unwrap_or(1);

            let trace = cycle
                .hops
                .iter()
                .map(|h| h.name.as_str())
                .collect::<Vec<_>>()
                .join(" -> ");
            let text = format!(
                "Circular dependency detected (length {}): {}",
                cycle.length, trace
            );

            results.push(json!({
                "ruleId": "ARCH-CYCLE",
                "level": level,
                "message": {
                    "text": text
                },
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file_uri,
                                "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                                "startLine": line
                            }
                        }
                    }
                ],
                "properties": {
                    "cycleIndex": idx + 1,
                    "isRuntime": cycle.is_runtime_cycle,
                    "fingerprint": cycle.fingerprint
                }
            }));
        }

        rules.push(json!({
            "id": "ARCH-CYCLE",
            "name": "CircularDependency",
            "shortDescription": {
                "text": "Detects circular dependency rings between architectural components"
            }
        }));

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "cartograph-lint-arch",
                            "informationUri": "https://github.com/pankajredekar/cartograph",
                            "semanticVersion": "0.1.0",
                            "rules": rules
                        }
                    },
                    "results": results
                }
            ]
        });

        serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
    }
}
