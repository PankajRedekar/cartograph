use cartograph_core::diagnose::DiagnosisReport;
use serde_json::json;

pub struct SarifPublisher;

impl SarifPublisher {
    pub fn render(report: &DiagnosisReport) -> String {
        let thread_flow_locations: Vec<serde_json::Value> = report
            .call_path
            .iter()
            .map(|frame| {
                json!({
                    "location": {
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": frame.file_path.to_string_lossy()
                            },
                            "region": {
                                "startLine": frame.line_number,
                                "startColumn": frame.column.unwrap_or(1)
                            }
                        },
                        "message": {
                            "text": format!("Frame: {}", frame.function_name)
                        }
                    }
                })
            })
            .collect();

        let root_cause_loc = report
            .root_cause_candidate
            .as_ref()
            .map(|rc| {
                json!([{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": rc.file_path.to_string_lossy()
                        },
                        "region": {
                            "startLine": rc.line_number,
                            "startColumn": 1
                        }
                    },
                    "message": {
                        "text": format!("Root Cause: {}", rc.function_name)
                    }
                }])
            })
            .unwrap_or_else(|| {
                json!([{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": report.failing_file
                        },
                        "region": {
                            "startLine": report.failing_line,
                            "startColumn": 1
                        }
                    },
                    "message": {
                        "text": format!("Panic Site: {}:{}", report.failing_file, report.failing_line)
                    }
                }])
            });

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "Cartograph Root-Cause Detective",
                            "semanticVersion": "3.3.0",
                            "informationUri": "https://github.com/PankajRedekar/cartograph",
                            "rules": [
                                {
                                    "id": format!("cartograph/{}", report.fault_category.to_string().to_lowercase()),
                                    "name": report.fault_category.to_string(),
                                    "shortDescription": {
                                        "text": format!("Root cause diagnosis for {}", report.crash_type)
                                    }
                                }
                            ]
                        }
                    },
                    "results": [
                        {
                            "ruleId": format!("cartograph/{}", report.fault_category.to_string().to_lowercase()),
                            "level": "error",
                            "message": {
                                "text": format!("{}: {}", report.crash_type, report.crash_message)
                            },
                            "locations": root_cause_loc,
                            "codeFlows": [
                                {
                                    "threadFlows": [
                                        {
                                            "locations": thread_flow_locations
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
    }
}
