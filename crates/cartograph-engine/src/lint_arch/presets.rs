use cartograph_core::config::{
    ArchLayerConfig, ArchSeverity, CycleGranularity, LintArchConfig, PackageEncapsulationConfig,
};

pub struct ArchPresets;

impl ArchPresets {
    /// Apply a named preset onto a LintArchConfig
    pub fn apply_preset(config: &mut LintArchConfig, preset_name: &str) {
        match preset_name.to_lowercase().replace('_', "-").as_str() {
            "clean" | "clean-architecture" => {
                config.layers = Self::clean_architecture_layers();
            }
            "mvc" => {
                config.layers = Self::mvc_layers();
            }
            "hexagonal" | "ports-and-adapters" => {
                config.layers = Self::hexagonal_layers();
            }
            "monorepo" => {
                config.package_encapsulation = PackageEncapsulationConfig {
                    enabled: true,
                    ..Default::default()
                };
                config.cycle_granularity = CycleGranularity::Package;
            }
            _ => {}
        }
    }

    /// Clean Architecture preset: Domain -> Application -> Adapters -> Infrastructure
    pub fn clean_architecture_layers() -> Vec<ArchLayerConfig> {
        vec![
            ArchLayerConfig {
                name: "domain".to_string(),
                description: Some("Core domain entities and business rules".to_string()),
                patterns: vec![
                    "**/domain/**".to_string(),
                    "**/entities/**".to_string(),
                    "**/core/**".to_string(),
                ],
                forbidden: vec![
                    "application".to_string(),
                    "adapters".to_string(),
                    "infrastructure".to_string(),
                    "presentation".to_string(),
                ],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "application".to_string(),
                description: Some("Use cases and application services".to_string()),
                patterns: vec![
                    "**/application/**".to_string(),
                    "**/usecases/**".to_string(),
                    "**/services/**".to_string(),
                ],
                forbidden: vec![
                    "adapters".to_string(),
                    "infrastructure".to_string(),
                    "presentation".to_string(),
                ],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "adapters".to_string(),
                description: Some("Interface adapters, gateways, and repositories".to_string()),
                patterns: vec![
                    "**/adapters/**".to_string(),
                    "**/gateways/**".to_string(),
                    "**/repositories/**".to_string(),
                ],
                forbidden: vec!["infrastructure".to_string(), "presentation".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "infrastructure".to_string(),
                description: Some("Frameworks, drivers, DB, and presentation".to_string()),
                patterns: vec![
                    "**/infrastructure/**".to_string(),
                    "**/infra/**".to_string(),
                    "**/db/**".to_string(),
                    "**/web/**".to_string(),
                    "**/controllers/**".to_string(),
                ],
                forbidden: vec![],
                allowed: None,
                severity: ArchSeverity::Error,
            },
        ]
    }

    /// MVC Preset: Model -> Controller -> View
    pub fn mvc_layers() -> Vec<ArchLayerConfig> {
        vec![
            ArchLayerConfig {
                name: "model".to_string(),
                description: Some("Data models and business entities".to_string()),
                patterns: vec![
                    "**/models/**".to_string(),
                    "**/model/**".to_string(),
                    "**/entities/**".to_string(),
                ],
                forbidden: vec!["controller".to_string(), "view".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "controller".to_string(),
                description: Some("Request handlers and controllers".to_string()),
                patterns: vec![
                    "**/controllers/**".to_string(),
                    "**/controller/**".to_string(),
                    "**/handlers/**".to_string(),
                ],
                forbidden: vec!["view".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "view".to_string(),
                description: Some("Templates, UI components, and views".to_string()),
                patterns: vec![
                    "**/views/**".to_string(),
                    "**/view/**".to_string(),
                    "**/templates/**".to_string(),
                    "**/components/**".to_string(),
                ],
                forbidden: vec![],
                allowed: None,
                severity: ArchSeverity::Error,
            },
        ]
    }

    /// Hexagonal Architecture Preset: Domain -> Ports -> Adapters
    pub fn hexagonal_layers() -> Vec<ArchLayerConfig> {
        vec![
            ArchLayerConfig {
                name: "domain".to_string(),
                description: Some("Core domain logic".to_string()),
                patterns: vec!["**/domain/**".to_string()],
                forbidden: vec!["adapters".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "ports".to_string(),
                description: Some("Inbound and outbound interfaces".to_string()),
                patterns: vec!["**/ports/**".to_string()],
                forbidden: vec!["adapters".to_string()],
                allowed: None,
                severity: ArchSeverity::Error,
            },
            ArchLayerConfig {
                name: "adapters".to_string(),
                description: Some("Primary and secondary adapters".to_string()),
                patterns: vec!["**/adapters/**".to_string()],
                forbidden: vec![],
                allowed: None,
                severity: ArchSeverity::Error,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_application() {
        let mut cfg = LintArchConfig::default();
        ArchPresets::apply_preset(&mut cfg, "clean-architecture");
        assert_eq!(cfg.layers.len(), 4);
        assert_eq!(cfg.layers[0].name, "domain");
        assert_eq!(cfg.layers[3].name, "infrastructure");

        let mut mvc_cfg = LintArchConfig::default();
        ArchPresets::apply_preset(&mut mvc_cfg, "mvc");
        assert_eq!(mvc_cfg.layers.len(), 3);
        assert_eq!(mvc_cfg.layers[0].name, "model");
        assert_eq!(mvc_cfg.layers[1].name, "controller");
        assert_eq!(mvc_cfg.layers[2].name, "view");
    }
}
