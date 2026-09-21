use cartograph_core::model::{ComplexityReport, HotspotQuadrant};
use serde_json::json;

pub struct SarifFormatter;

impl SarifFormatter {
    /// Format complexity report as SARIF 2.1.0 for GitHub Code Scanning
    pub fn format(report: &ComplexityReport) -> String {
        let mut results = Vec::new();

        for r in &report.top_hotspots {
            if r.quadrant == HotspotQuadrant::CriticalHotspot {
                results.push(json!({
                    "ruleId": "complexity/critical-hotspot",
                    "level": "error",
                    "message": {
                        "text": format!(
                            "Critical Hotspot Q1: `{}` has high complexity (CC: {}, Cog: {}) and high churn velocity ({:.2}). Refactoring prioritized.",
                            r.name, r.complexity.cyclomatic, r.complexity.cognitive, r.churn.churn_velocity_score
                        )
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": r.filepath,
                                "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                                "startLine": r.start_line,
                                "endLine": r.end_line
                            }
                        }
                    }]
                }));
            }

            if r.test_deficit.is_danger_zone {
                results.push(json!({
                    "ruleId": "complexity/danger-zone",
                    "level": "warning",
                    "message": {
                        "text": format!(
                            "Danger Zone: `{}` has high complexity (CC: {}) operating with 0 tests in the test graph.",
                            r.name, r.complexity.cyclomatic
                        )
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": r.filepath,
                                "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                                "startLine": r.start_line,
                                "endLine": r.end_line
                            }
                        }
                    }]
                }));
            }

            if let Some(gc) = &r.god_class
                && gc.is_god_class
            {
                results.push(json!({
                        "ruleId": "complexity/god-class",
                        "level": "error",
                        "message": {
                            "text": format!(
                                "God-Class Anti-pattern: `{}` violates cohesion boundaries (GCI: {:.1}, LCOM4: {}). Extractions recommended.",
                                r.name, gc.god_class_index, r.class_cohesion.as_ref().map(|c| c.lcom4).unwrap_or(1)
                            )
                        },
                        "locations": [{
                            "physicalLocation": {
                                "artifactLocation": {
                                "uri": r.filepath,
                                "uriBaseId": "%SRCROOT%"
                            },
                            "region": {
                                "startLine": r.start_line,
                                "endLine": r.end_line
                            }
                        }
                    }]
                }));
            }
        }

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "cartograph-complexity",
                        "version": "0.1.0",
                        "informationUri": "https://github.com/PankajRedekar/cartograph",
                        "rules": [
                            {
                                "id": "complexity/critical-hotspot",
                                "shortDescription": { "text": "High complexity and high git churn hotspot" }
                            },
                            {
                                "id": "complexity/danger-zone",
                                "shortDescription": { "text": "High complexity symbol with zero test coverage" }
                            },
                            {
                                "id": "complexity/god-class",
                                "shortDescription": { "text": "God-Class anti-pattern violating cohesion boundaries" }
                            }
                        ]
                    }
                },
                "results": results
            }]
        });

        serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
    }
}
