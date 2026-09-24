pub mod concept;
pub mod diff;
pub mod formatters;
pub mod tracer;
pub mod verbalizer;

pub use concept::ConceptTaxonomyMatcher;
pub use diff::RuleDiffEngine;
pub use formatters::{
    AnsiFormatter, GherkinFormatter, JsonFormatter, LogicOutputFormat, OutputFormat,
    TableFormatter, XmlFormatter, render_rules,
};
pub use tracer::InterProceduralTracer;
pub use verbalizer::RuleVerbalizer;

use crate::error::Result;
use cartograph_ast::AstLogicExtractor;
use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::{BusinessRule, ExtractLogicConfig, LogicStats, RuleDiff};
use std::path::{Path, PathBuf};

/// Central facade orchestrator for business logic & validation rule extraction.
pub struct LogicExtractionEngine<'a> {
    pub backend: Option<&'a dyn GraphStoreBackend>,
    pub config: &'a ExtractLogicConfig,
    pub extractor: AstLogicExtractor,
}

impl<'a> LogicExtractionEngine<'a> {
    pub fn new(backend: Option<&'a dyn GraphStoreBackend>, config: &'a ExtractLogicConfig) -> Self {
        Self {
            backend,
            config,
            extractor: AstLogicExtractor::new(),
        }
    }

    /// Extract business rules from a single source file.
    pub fn extract_file(&self, filepath: &Path) -> Result<Vec<BusinessRule>> {
        let content = std::fs::read_to_string(filepath).map_err(crate::error::EngineError::Io)?;
        let file_str = filepath.to_string_lossy();
        Ok(self
            .extractor
            .extract_rules(&file_str, &content, self.config))
    }

    /// Extract rules targeting a specific symbol name, with optional deep call-tree tracing.
    pub async fn extract_symbol(
        &self,
        symbol: &str,
        filepath: Option<&Path>,
        workspace_root: &Path,
        deep: bool,
        max_depth: usize,
    ) -> Result<Vec<BusinessRule>> {
        if deep {
            let tracer = InterProceduralTracer::new(self.backend, self.config);
            let rules = tracer
                .trace_symbol_logic(symbol, filepath, workspace_root, max_depth)
                .await;
            Ok(rules)
        } else if let Some(fp) = filepath {
            let content = std::fs::read_to_string(fp).map_err(crate::error::EngineError::Io)?;
            let file_str = fp.to_string_lossy();
            Ok(self
                .extractor
                .extract_for_symbol(&file_str, &content, symbol, self.config))
        } else if let Some(backend) = self.backend {
            let nodes = backend.get_nodes_by_name(symbol).await.unwrap_or_default();
            let mut all_rules = Vec::new();
            for node in nodes {
                let full_path = workspace_root.join(&node.filepath);
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    let file_str = full_path.to_string_lossy();
                    let rules =
                        self.extractor
                            .extract_for_symbol(&file_str, &content, symbol, self.config);
                    all_rules.extend(rules);
                }
            }
            Ok(all_rules)
        } else {
            Ok(Vec::new())
        }
    }

    /// Search and extract domain logic matching a concept taxonomy query across the workspace.
    pub fn query_concept(&self, concept: &str, workspace_root: &Path) -> Result<Vec<BusinessRule>> {
        let all_rules = self.scan_workspace(workspace_root)?;
        let matcher = ConceptTaxonomyMatcher::new(self.config);
        let matched = matcher.filter_rules(&all_rules, concept);
        Ok(matched.into_iter().cloned().collect())
    }

    /// Compare rules against a base Git branch or revision for PR review and CI quality gating.
    pub fn diff_with_git(
        &self,
        workspace_root: &Path,
        base_ref: &str,
        target_files: Option<&[&Path]>,
    ) -> Result<RuleDiff> {
        let diff_engine = RuleDiffEngine::new(self.config);

        let files: Vec<PathBuf> = if let Some(tf) = target_files {
            tf.iter().map(|p| p.to_path_buf()).collect()
        } else {
            self.collect_source_files(workspace_root)
        };

        let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();

        // 1. Extract base rules from Git
        let base_rules = diff_engine.extract_git_base_rules(workspace_root, base_ref, &file_refs);

        // 2. Extract current head rules from local workspace
        let mut head_rules = Vec::new();
        for file in &file_refs {
            if let Ok(rules) = self.extract_file(file) {
                head_rules.extend(rules);
            }
        }

        // 3. Compute diff
        Ok(diff_engine.compute_diff(base_ref, "HEAD", &base_rules, &head_rules))
    }

    /// Scan entire workspace and extract all rules across polyglot files.
    pub fn scan_workspace(&self, workspace_root: &Path) -> Result<Vec<BusinessRule>> {
        let files = self.collect_source_files(workspace_root);
        let mut rules = Vec::new();
        for f in files {
            if let Ok(r) = self.extract_file(&f) {
                rules.extend(r);
            }
        }
        Ok(rules)
    }

    /// Calculate summary statistics for extracted business rules.
    pub fn compute_stats(rules: &[BusinessRule]) -> LogicStats {
        let mut stats = LogicStats::default();
        for rule in rules {
            stats.record_rule(&rule.kind);
        }
        stats
    }

    fn collect_source_files(&self, workspace_root: &Path) -> Vec<PathBuf> {
        let walker = ignore::WalkBuilder::new(workspace_root)
            .hidden(true)
            .git_ignore(true)
            .build();

        let mut files = Vec::new();
        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
                && matches!(
                    ext.to_lowercase().as_str(),
                    "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "rb"
                )
            {
                files.push(path.to_path_buf());
            }
        }
        files
    }
}
