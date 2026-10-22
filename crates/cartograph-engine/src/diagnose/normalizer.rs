use super::traceback_parser::RawFrame;
use cartograph_core::diagnose::TracebackFrame;
use std::fs;
use std::path::{Path, PathBuf};

/// Normalizes file paths, resolves container prefixes, detects languages, and extracts source snippets.
pub struct PathNormalizer {
    repo_root: PathBuf,
}

impl PathNormalizer {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Normalizes a collection of raw parsed frames into project-anchored `TracebackFrame`s.
    pub fn normalize_frames(&self, raw_frames: Vec<RawFrame>) -> Vec<TracebackFrame> {
        let total = raw_frames.len();
        raw_frames
            .into_iter()
            .enumerate()
            .map(|(idx, raw)| {
                let (norm_path, in_project) = self.normalize_path(&raw.file_path);
                let language = self.detect_language(&norm_path);
                let snippet = raw.code_snippet.or_else(|| {
                    if in_project {
                        self.extract_source_snippet(&norm_path, raw.line_number)
                    } else {
                        None
                    }
                });

                TracebackFrame {
                    frame_index: idx,
                    file_path: norm_path,
                    line_number: raw.line_number,
                    column: raw.column,
                    function_name: raw.function_name,
                    code_snippet: snippet,
                    is_in_project: in_project,
                    is_leaf_panic: idx == total.saturating_sub(1),
                    ast_symbol_id: None,
                    ast_node_kind: None,
                    language,
                }
            })
            .collect()
    }

    /// Strips container prefixes and verifies presence within the repository.
    pub fn normalize_path(&self, raw_path: &str) -> (PathBuf, bool) {
        let cleaned = Self::strip_container_prefixes(raw_path);
        let path_obj = Path::new(&cleaned);

        // Check if path is standard library or third-party dependency
        if Self::is_vendor_or_stdlib(&cleaned) {
            return (path_obj.to_path_buf(), false);
        }

        // 1. Direct match relative to repo root
        let direct = self.repo_root.join(path_obj);
        if direct.is_file() {
            return (path_obj.to_path_buf(), true);
        }

        // 2. If cleaned is absolute and starts with repo_root
        if let Ok(rel) = direct.strip_prefix(&self.repo_root)
            && self.repo_root.join(rel).is_file()
        {
            return (rel.to_path_buf(), true);
        }

        // 3. Fallback: match by filename if found anywhere in repo
        if let Some(file_name) = path_obj.file_name().and_then(|n| n.to_str())
            && let Some(found_rel) = self.find_file_in_repo(file_name)
        {
            return (found_rel, true);
        }

        // If not found in repo, keep normalized relative path and mark out-of-project
        (path_obj.to_path_buf(), false)
    }

    /// Strip container and common runtime directory prefixes.
    pub fn strip_container_prefixes(path: &str) -> String {
        let prefixes = [
            "/app/",
            "/workspace/",
            "/var/task/",
            "/root/",
            "/home/node/app/",
            "/usr/src/app/",
            "C:\\app\\",
            "C:\\workspace\\",
        ];

        let mut res = path.trim().replace('\\', "/");
        for p in &prefixes {
            let p_norm = p.replace('\\', "/");
            if res.starts_with(&p_norm) {
                res = res[p_norm.len()..].to_string();
                break;
            }
        }

        // Strip leading "./" or "/"
        while res.starts_with("./") {
            res = res[2..].to_string();
        }
        if res.starts_with('/') {
            res = res[1..].to_string();
        }

        res
    }

    /// Returns true if the path belongs to third-party packages or stdlib runtimes.
    pub fn is_vendor_or_stdlib(path: &str) -> bool {
        let vendors = [
            "site-packages/",
            "dist-packages/",
            "node_modules/",
            ".cargo/registry/",
            ".rustup/toolchains/",
            "go/pkg/mod/",
            "/usr/lib/",
            "/usr/local/lib/",
            "<built-in>",
            "<stdin>",
        ];
        vendors.iter().any(|v| path.contains(v))
    }

    /// Detect programming language from path extension.
    pub fn detect_language(&self, path: &Path) -> String {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "py" => "python".to_string(),
            "rs" => "rust".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "js" | "jsx" | "mjs" | "cjs" => "javascript".to_string(),
            "go" => "go".to_string(),
            "java" => "java".to_string(),
            "rb" => "ruby".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "cc" | "hpp" => "cpp".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Extracts lines `[line - 2, line + 2]` around the target line.
    pub fn extract_source_snippet(&self, rel_path: &Path, line: usize) -> Option<String> {
        let full_path = self.repo_root.join(rel_path);
        let content = fs::read_to_string(&full_path).ok()?;
        let lines: Vec<&str> = content.lines().collect();

        if line == 0 || line > lines.len() {
            return None;
        }

        let start = line.saturating_sub(2).max(1);
        let end = (line + 2).min(lines.len());

        let mut snippet = String::new();
        for i in start..=end {
            let prefix = if i == line { ">" } else { " " };
            snippet.push_str(&format!("{prefix} {:4} | {}\n", i, lines[i - 1]));
        }

        Some(snippet.trim_end().to_string())
    }

    fn find_file_in_repo(&self, target_filename: &str) -> Option<PathBuf> {
        for entry in ignore::WalkBuilder::new(&self.repo_root)
            .max_depth(Some(8))
            .build()
            .flatten()
        {
            if entry.file_type().is_some_and(|f| f.is_file())
                && entry.file_name().to_string_lossy() == target_filename
                && let Ok(rel) = entry.path().strip_prefix(&self.repo_root)
            {
                return Some(rel.to_path_buf());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_container_prefixes() {
        assert_eq!(
            PathNormalizer::strip_container_prefixes("/app/src/services/billing.ts"),
            "src/services/billing.ts"
        );
        assert_eq!(
            PathNormalizer::strip_container_prefixes(
                "/workspace/crates/cartograph-core/src/lib.rs"
            ),
            "crates/cartograph-core/src/lib.rs"
        );
        assert_eq!(
            PathNormalizer::strip_container_prefixes("./src/main.py"),
            "src/main.py"
        );
    }

    #[test]
    fn test_is_vendor_or_stdlib() {
        assert!(PathNormalizer::is_vendor_or_stdlib(
            "/usr/local/lib/python3.11/site-packages/pydantic/main.py"
        ));
        assert!(PathNormalizer::is_vendor_or_stdlib(
            "/app/node_modules/express/lib/router.js"
        ));
        assert!(!PathNormalizer::is_vendor_or_stdlib(
            "src/controllers/order.py"
        ));
    }

    #[test]
    fn test_detect_language() {
        let norm = PathNormalizer::new(PathBuf::from("."));
        assert_eq!(norm.detect_language(Path::new("app/test.py")), "python");
        assert_eq!(norm.detect_language(Path::new("src/main.rs")), "rust");
        assert_eq!(
            norm.detect_language(Path::new("web/index.ts")),
            "typescript"
        );
        assert_eq!(norm.detect_language(Path::new("pkg/api.go")), "go");
    }
}
