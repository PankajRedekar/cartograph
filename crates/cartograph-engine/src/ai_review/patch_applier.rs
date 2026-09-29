use std::fs;
use std::path::Path;
use std::process::Command;

pub struct PatchApplier;

impl PatchApplier {
    /// Safe single-step patch application with conflict pre-check
    pub fn apply_patch(repo_root: &Path, patch_path: &Path) -> Result<(), String> {
        if !patch_path.exists() {
            return Err(format!("Patch file not found: {}", patch_path.display()));
        }

        // 1. Dry run pre-check: git apply --check --unidiff-zero
        let check_output = Command::new("git")
            .current_dir(repo_root)
            .args([
                "apply",
                "--check",
                "--unidiff-zero",
                &patch_path.to_string_lossy(),
            ])
            .output()
            .map_err(|e| format!("Failed to invoke 'git apply --check': {}", e))?;

        if !check_output.status.success() {
            let stderr = String::from_utf8_lossy(&check_output.stderr);
            return Err(format!(
                "Patch pre-check failed (conflicts detected):\n{}",
                stderr.trim()
            ));
        }

        // 2. Apply patch
        let apply_output = Command::new("git")
            .current_dir(repo_root)
            .args(["apply", "--unidiff-zero", &patch_path.to_string_lossy()])
            .output()
            .map_err(|e| format!("Failed to invoke 'git apply': {}", e))?;

        if !apply_output.status.success() {
            let stderr = String::from_utf8_lossy(&apply_output.stderr);
            return Err(format!("Failed to apply patch:\n{}", stderr.trim()));
        }

        Ok(())
    }

    /// Apply unified diff string directly
    pub fn apply_patch_str(repo_root: &Path, patch_content: &str) -> Result<(), String> {
        let temp_dir = std::env::temp_dir();
        let patch_file = temp_dir.join(format!("cartograph_review_{}.patch", std::process::id()));
        fs::write(&patch_file, patch_content)
            .map_err(|e| format!("Failed to write temporary patch file: {}", e))?;

        let res = Self::apply_patch(repo_root, &patch_file);
        let _ = fs::remove_file(&patch_file);
        res
    }
}
