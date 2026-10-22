pub mod call_path;
pub mod classifier;
pub mod correlator;
pub mod frame_anchor;
pub mod invariants;
pub mod llm;
pub mod normalizer;
pub mod patch_synthesizer;
pub mod provenance_slicer;
pub mod publishers;
pub mod scorer;
pub mod test_scaffolder;
pub mod traceback_parser;

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::{DiagnoseConfig, LintArchConfig};
use cartograph_core::diagnose::{DiagnosisReport, DiagnosticExecutionMode, TracebackFrame};
use cartograph_core::error::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

pub use call_path::CallPathReconstructor;
pub use classifier::FaultClassifier;
pub use correlator::GitCorrelator;
pub use frame_anchor::FrameAnchor;
pub use invariants::InvariantAuditor;
pub use llm::DiagnosticLlmClient;
pub use normalizer::PathNormalizer;
pub use patch_synthesizer::PatchSynthesizer;
pub use provenance_slicer::ProvenanceSlicer;
pub use scorer::TopologicalFaultScorer;
pub use test_scaffolder::RegressionTestScaffolder;
pub use traceback_parser::TracebackParser;

/// Master Topological Root-Cause Detective Engine.
pub struct DiagnoseEngine {
    repo_root: PathBuf,
    backend: Arc<dyn GraphStoreBackend>,
    config: DiagnoseConfig,
    lint_config: LintArchConfig,
}

impl DiagnoseEngine {
    pub fn new(
        repo_root: PathBuf,
        backend: Arc<dyn GraphStoreBackend>,
        config: DiagnoseConfig,
    ) -> Self {
        Self {
            repo_root,
            backend,
            config,
            lint_config: LintArchConfig::default(),
        }
    }

    pub fn with_lint_config(mut self, lint_config: LintArchConfig) -> Self {
        self.lint_config = lint_config;
        self
    }

    /// Executes the complete 8-stage topological fault localization pipeline.
    pub async fn run_diagnose(&self, crash_input: &str) -> Result<DiagnosisReport> {
        let start_time = Instant::now();

        // 1. Polyglot Traceback Parsing
        let parsed = TracebackParser::parse(crash_input);
        let crash_type = parsed.crash_type;
        let crash_message = parsed.crash_message;

        // 2. Path & Sourcemap Normalization
        let normalizer = PathNormalizer::new(self.repo_root.clone());
        let mut frames = normalizer.normalize_frames(parsed.frames);

        // 3. AST Graph Frame Anchoring
        let anchor = FrameAnchor::new(self.backend.clone());
        anchor.anchor_frames(&mut frames).await;

        // 4. Call-Path Reconstruction
        let reconstructor = CallPathReconstructor::new(self.backend.clone());
        let (_transitions, _upstream) = reconstructor.reconstruct(&frames, self.config.deep).await;

        // 5. Semantic Fault Classification
        let category = FaultClassifier::classify(&crash_type, &crash_message);

        // 6. Def-Use Variable Provenance Slicing
        let slicer = ProvenanceSlicer::new(self.repo_root.clone());
        let provenance = slicer.slice_provenance(&frames, &crash_type, &crash_message);

        // 7. Git Blame & Branch Churn Correlation
        let correlator = GitCorrelator::new(self.repo_root.clone());
        let blame_map = if self.config.git_blame {
            correlator.correlate_frames(&frames, self.config.branch.as_deref())
        } else {
            std::collections::HashMap::new()
        };

        // 8. Invariant & Boundary Auditing
        let auditor = InvariantAuditor::new(self.lint_config.clone());
        let boundary_map = auditor.audit_boundary_crossings(&frames);

        // 9. Topological Fault Scoring
        let (candidates, root_cause, verdict) = TopologicalFaultScorer::score_candidates(
            &frames,
            provenance.as_ref(),
            &blame_map,
            &boundary_map,
            category,
        );

        // Determine primary failing coordinates
        let failing_file = frames
            .last()
            .map(|f| f.file_path.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let failing_line = frames.last().map(|f| f.line_number).unwrap_or(0);

        // 10. Remediation & Regression Test Synthesis
        let mut suggested_fix = None;
        let mut regression_test = None;
        let mut execution_mode = DiagnosticExecutionMode::OfflineDeterministic;

        if let Some(rc) = &root_cause {
            let leaf = frames.last().cloned().unwrap_or_else(|| TracebackFrame {
                frame_index: 0,
                file_path: rc.file_path.clone(),
                line_number: rc.line_number,
                column: None,
                function_name: rc.function_name.clone(),
                code_snippet: None,
                is_in_project: true,
                is_leaf_panic: true,
                ast_symbol_id: None,
                ast_node_kind: None,
                language: "unknown".to_string(),
            });

            // Scaffold regression reproducer
            regression_test = Some(RegressionTestScaffolder::scaffold(
                rc,
                &leaf,
                category,
                &crash_type,
                &crash_message,
            ));

            // Synthesize deterministic patch
            let patch_synth = PatchSynthesizer::new(self.repo_root.clone());
            suggested_fix = patch_synth.synthesize_fix(rc, &leaf, category, &crash_type);

            // If AI augmented mode requested
            if !self.config.offline {
                let llm = DiagnosticLlmClient::new(self.config.clone());
                let (ai_reasoning, ai_fix, mode) = llm
                    .synthesize_ai_diagnosis(
                        &crash_type,
                        &crash_message,
                        &frames,
                        provenance.as_ref(),
                        Some(rc),
                        category,
                    )
                    .await;

                execution_mode = mode;
                if let Some(af) = ai_fix {
                    suggested_fix = Some(af);
                } else if let (Some(reason), Some(sf)) = (ai_reasoning, &mut suggested_fix) {
                    sf.explanation = reason;
                }
            }
        }

        // Export patch to file if requested
        if let (Some(export_path), Some(fix)) = (&self.config.export_patch, &suggested_fix) {
            let _ = fs::write(export_path, &fix.unified_diff_patch);
        }

        // Apply patch if requested
        if self.config.apply_patch
            && let Some(fix) = &suggested_fix
        {
            let _ = self.apply_git_patch(&fix.unified_diff_patch);
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let report_id = format!("diag_{}", chrono::Utc::now().timestamp_millis());

        Ok(DiagnosisReport {
            id: report_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            crash_type,
            crash_message,
            failing_file,
            failing_line,
            fault_category: category,
            root_cause_candidate: root_cause,
            candidates,
            call_path: frames,
            provenance,
            suggested_fix,
            regression_test,
            verdict,
            execution_mode,
            execution_duration_ms: duration_ms,
        })
    }

    fn apply_git_patch(&self, patch: &str) -> std::result::Result<(), String> {
        let temp_patch = self.repo_root.join(".cartograph_patch.tmp");
        fs::write(&temp_patch, patch).map_err(|e| format!("Failed to write patch: {e}"))?;

        let status = Command::new("git")
            .current_dir(&self.repo_root)
            .args(["apply", ".cartograph_patch.tmp"])
            .status();

        let _ = fs::remove_file(&temp_patch);
        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(format!("git apply exited with code {:?}", s.code())),
            Err(e) => Err(format!("Failed to execute git apply: {e}")),
        }
    }
}
