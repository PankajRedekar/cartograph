use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Command;

use cartograph_ast::AstRegistry;
use cartograph_core::config::{GitDiffMode, IndexConfig};
use ignore::WalkBuilder;

use crate::error::Result;

pub enum ZeroCopyBuffer {
    Heap(Vec<u8>),
    Mmap(memmap2::Mmap),
}

impl ZeroCopyBuffer {
    pub fn read_from_path(path: &Path, mmap_threshold: usize) -> std::io::Result<Self> {
        FileWalker::read_file_bytes(path, mmap_threshold)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.deref()
    }

    pub fn as_str(&self) -> std::result::Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.deref())
    }

    pub fn to_str_lossy(&self) -> std::borrow::Cow<'_, str> {
        match std::str::from_utf8(self.deref()) {
            Ok(s) => std::borrow::Cow::Borrowed(s),
            Err(_) => String::from_utf8_lossy(self.deref()),
        }
    }
}

impl Deref for ZeroCopyBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            ZeroCopyBuffer::Heap(v) => v.as_slice(),
            ZeroCopyBuffer::Mmap(m) => m.as_ref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub root_path: PathBuf,
    pub exclude_patterns: Vec<String>,
    pub max_file_size_bytes: usize,
    pub mmap_threshold_bytes: usize,
    pub follow_symlinks: bool,
    pub num_threads: Option<usize>,
    pub git_diff_mode: Option<GitDiffMode>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        let index_cfg = IndexConfig::default();
        Self {
            root_path: PathBuf::from("."),
            exclude_patterns: index_cfg.exclude_patterns,
            max_file_size_bytes: index_cfg.max_file_size_bytes,
            mmap_threshold_bytes: index_cfg.mmap_threshold_bytes,
            follow_symlinks: index_cfg.follow_symlinks,
            num_threads: index_cfg.num_threads,
            git_diff_mode: None,
        }
    }
}

impl From<IndexConfig> for DiscoveryConfig {
    fn from(cfg: IndexConfig) -> Self {
        Self {
            root_path: PathBuf::from("."),
            exclude_patterns: cfg.exclude_patterns,
            max_file_size_bytes: cfg.max_file_size_bytes,
            mmap_threshold_bytes: cfg.mmap_threshold_bytes,
            follow_symlinks: cfg.follow_symlinks,
            num_threads: cfg.num_threads,
            git_diff_mode: None,
        }
    }
}

pub struct FileWalker {
    config: DiscoveryConfig,
    registry: AstRegistry,
}

impl FileWalker {
    pub fn new(config: DiscoveryConfig, registry: AstRegistry) -> Self {
        Self { config, registry }
    }

    pub fn is_supported_path(&self, path: &Path) -> bool {
        self.registry.is_supported_path(path)
    }

    /// Read file content as ZeroCopyBuffer, using memory-mapping for files > threshold
    pub fn read_file_bytes(path: &Path, mmap_threshold: usize) -> std::io::Result<ZeroCopyBuffer> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let len = metadata.len() as usize;

        if len >= mmap_threshold && len > 0 {
            // SAFETY: File is opened read-only and mapped read-only for Tree-sitter parsing
            let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
            Ok(ZeroCopyBuffer::Mmap(mmap))
        } else if len == 0 {
            Ok(ZeroCopyBuffer::Heap(Vec::new()))
        } else {
            use std::io::Read;
            let mut reader = std::io::BufReader::new(file);
            let mut buffer = Vec::with_capacity(len);
            reader.read_to_end(&mut buffer)?;
            Ok(ZeroCopyBuffer::Heap(buffer))
        }
    }

    /// Discover files matching git diff mode (--staged or --branch) across single or polyrepo
    pub fn discover_git_diff_files(
        root_path: &Path,
        diff_mode: &GitDiffMode,
    ) -> std::io::Result<Vec<PathBuf>> {
        GitWorkspaceResolver::discover_changed_files(root_path, diff_mode)
    }

    /// Walk repository and collect candidate files respecting .gitignore, vendor filters, and size ceilings
    pub fn collect_files(&self) -> Result<Vec<PathBuf>> {
        // If git diff mode is set, use fast git index query
        if let Some(ref diff_mode) = self.config.git_diff_mode {
            let files = Self::discover_git_diff_files(&self.config.root_path, diff_mode)?;
            let filtered: Vec<PathBuf> = files
                .into_iter()
                .filter(|p| self.is_supported_path(p))
                .collect();
            return Ok(filtered);
        }

        let mut builder = WalkBuilder::new(&self.config.root_path);
        builder
            .hidden(true)
            .ignore(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .follow_links(self.config.follow_symlinks)
            .max_filesize(Some(self.config.max_file_size_bytes as u64));

        if let Some(threads) = self.config.num_threads {
            builder.threads(threads);
        }

        let custom_excludes: Vec<String> = self.config.exclude_patterns.clone();

        let walker = builder.build();
        let mut candidate_files = Vec::new();

        for entry_res in walker {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    tracing::debug!("Ignore walker skip: {e}");
                    continue;
                }
            };

            let path = entry.path();
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }

            // Exclude custom directories/patterns
            let path_str = path.to_string_lossy();
            let mut excluded = false;
            for pattern in &custom_excludes {
                let clean = pattern.trim_matches('/');
                if clean.contains('/') {
                    if path_str.contains(clean) {
                        excluded = true;
                        break;
                    }
                } else {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name == clean || file_name.ends_with(clean) {
                        excluded = true;
                        break;
                    }
                    for comp in path.components() {
                        if comp.as_os_str() == clean {
                            excluded = true;
                            break;
                        }
                    }
                    if excluded {
                        break;
                    }
                }
            }
            if excluded {
                continue;
            }

            // Filter by supported file extensions and special filenames
            if self.is_supported_path(path) {
                candidate_files.push(path.to_path_buf());
            }
        }

        candidate_files.sort();
        Ok(candidate_files)
    }
}

/// Information about a discovered Git repository in the workspace (single-repo or polyrepo child)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredGitRepo {
    pub name: String,
    pub root_path: PathBuf,
    pub prefix: PathBuf,
}

pub struct GitWorkspaceResolver;

impl GitWorkspaceResolver {
    /// Discover all git repositories in workspace.
    /// 1. If `workspace_root.join(".git").exists()`, returns single entry with empty prefix.
    /// 2. If not, scans immediate subdirectories for `.git` folders (polyrepo mode).
    pub fn discover_repos(workspace_root: &Path) -> Vec<DiscoveredGitRepo> {
        if workspace_root.join(".git").exists() {
            let name = workspace_root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            return vec![DiscoveredGitRepo {
                name,
                root_path: workspace_root.to_path_buf(),
                prefix: PathBuf::new(),
            }];
        }

        let mut repos = Vec::new();
        if let Ok(entries) = std::fs::read_dir(workspace_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    if folder_name.starts_with('.')
                        || folder_name == "node_modules"
                        || folder_name == "target"
                        || folder_name == "vendor"
                    {
                        continue;
                    }
                    if path.join(".git").exists() {
                        repos.push(DiscoveredGitRepo {
                            name: folder_name.clone(),
                            root_path: path.clone(),
                            prefix: PathBuf::from(folder_name),
                        });
                    }
                }
            }
        }
        repos.sort_by(|a, b| a.name.cmp(&b.name));
        repos
    }

    /// Discover modified files across single repo or all child Git repositories
    pub fn discover_changed_files(
        workspace_root: &Path,
        diff_mode: &GitDiffMode,
    ) -> std::io::Result<Vec<PathBuf>> {
        let repos = Self::discover_repos(workspace_root);
        if repos.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_files = Vec::new();
        for repo in repos {
            let mut cmd = Command::new("git");
            cmd.current_dir(&repo.root_path);

            match diff_mode {
                GitDiffMode::StagedOnly => {
                    cmd.args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"]);
                }
                GitDiffMode::BranchRange(range) => {
                    cmd.args(["diff", "--name-only", "--diff-filter=ACMR", range]);
                }
            }

            if let Ok(output) = cmd.output()
                && output.status.success()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        let rel_path = if repo.prefix.as_os_str().is_empty() {
                            PathBuf::from(trimmed)
                        } else {
                            repo.prefix.join(trimmed)
                        };
                        let full_path = workspace_root.join(&rel_path);
                        if full_path.is_file() {
                            all_files.push(full_path);
                        }
                    }
                }
            }
        }

        all_files.sort();
        all_files.dedup();
        Ok(all_files)
    }

    /// Execute git diff -U0 across single repo or all child Git repositories,
    /// rewriting paths in diff headers (a/..., b/...) so downstream diff mappers
    /// seamlessly locate files relative to workspace_root.
    pub fn execute_diff(
        workspace_root: &Path,
        branch_range: Option<&str>,
        staged: bool,
    ) -> crate::error::Result<String> {
        let repos = Self::discover_repos(workspace_root);
        if repos.is_empty() {
            return Ok(String::new());
        }

        let mut aggregated_diff = String::new();

        for repo in repos {
            let mut cmd = Command::new("git");
            cmd.current_dir(&repo.root_path);

            if staged {
                cmd.args(["diff", "-U0", "--cached"]);
            } else if let Some(range) = branch_range {
                cmd.args(["diff", "-U0", range]);
            } else {
                cmd.args(["diff", "-U0"]);
            }

            let output = match cmd.output() {
                Ok(out) => out,
                Err(_) => continue,
            };

            if output.status.success() {
                let raw = String::from_utf8_lossy(&output.stdout);
                if !raw.trim().is_empty() {
                    if repo.prefix.as_os_str().is_empty() {
                        aggregated_diff.push_str(&raw);
                        if !raw.ends_with('\n') {
                            aggregated_diff.push('\n');
                        }
                    } else {
                        let prefix_str = repo.prefix.to_string_lossy();
                        for line in raw.lines() {
                            if let Some(rest) = line.strip_prefix("--- a/") {
                                aggregated_diff.push_str(&format!("--- a/{prefix_str}/{rest}\n"));
                            } else if let Some(rest) = line.strip_prefix("+++ b/") {
                                aggregated_diff.push_str(&format!("+++ b/{prefix_str}/{rest}\n"));
                            } else if let Some(rest) = line.strip_prefix("diff --git a/") {
                                if let Some((p1, p2)) = rest.split_once(" b/") {
                                    aggregated_diff.push_str(&format!(
                                        "diff --git a/{prefix_str}/{p1} b/{prefix_str}/{p2}\n"
                                    ));
                                } else {
                                    aggregated_diff.push_str(line);
                                    aggregated_diff.push('\n');
                                }
                            } else {
                                aggregated_diff.push_str(line);
                                aggregated_diff.push('\n');
                            }
                        }
                    }
                }
            }
        }

        Ok(aggregated_diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_walker_and_extension_filtering() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();

        // Create sample files
        fs::write(root.join("main.py"), "print('hello')").unwrap();
        fs::write(root.join("index.ts"), "const x: number = 1;").unwrap();
        fs::write(root.join("lib.rs"), "pub fn add() {}").unwrap();
        fs::write(root.join("image.png"), "not source code").unwrap();
        fs::write(root.join("data.bin"), "binary").unwrap();

        // Nested directory
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/utils.go"), "package src").unwrap();

        // Ignored directory
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("node_modules/dep.js"), "module.exports = {}").unwrap();

        let config = DiscoveryConfig {
            root_path: root.to_path_buf(),
            ..Default::default()
        };
        let registry = AstRegistry::standard();
        let walker = FileWalker::new(config, registry);

        let files = walker.collect_files().unwrap();
        let file_names: Vec<String> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(file_names.contains(&"main.py".to_string()));
        assert!(file_names.contains(&"index.ts".to_string()));
        assert!(file_names.contains(&"lib.rs".to_string()));
        assert!(file_names.contains(&"utils.go".to_string()));

        // png, bin, and node_modules must be excluded!
        assert!(!file_names.contains(&"image.png".to_string()));
        assert!(!file_names.contains(&"data.bin".to_string()));
        assert!(!file_names.contains(&"dep.js".to_string()));
    }

    #[test]
    fn test_zero_copy_buffer_heap_and_mmap() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("test.py");
        let content = "def hello():\n    return 42\n";
        fs::write(&file_path, content).unwrap();

        // Heap read (threshold > size)
        let heap_buf = FileWalker::read_file_bytes(&file_path, 1024).unwrap();
        assert_eq!(heap_buf.as_str().unwrap(), content);

        // Mmap read (threshold <= size)
        let mmap_buf = FileWalker::read_file_bytes(&file_path, 5).unwrap();
        assert_eq!(mmap_buf.as_str().unwrap(), content);
    }

    #[test]
    fn test_git_workspace_resolver_single_and_polyrepo() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();

        // 1. Polyrepo detection: root has no .git, but subfolders do
        let repo_a = root.join("auth-service");
        let repo_b = root.join("billing-service");
        let ignored = root.join("node_modules");
        let regular_dir = root.join("docs");

        fs::create_dir_all(repo_a.join(".git")).unwrap();
        fs::create_dir_all(repo_b.join(".git")).unwrap();
        fs::create_dir_all(ignored.join(".git")).unwrap();
        fs::create_dir_all(&regular_dir).unwrap();

        let discovered = GitWorkspaceResolver::discover_repos(root);
        assert_eq!(discovered.len(), 2);
        assert_eq!(discovered[0].name, "auth-service");
        assert_eq!(discovered[0].prefix, PathBuf::from("auth-service"));
        assert_eq!(discovered[1].name, "billing-service");
        assert_eq!(discovered[1].prefix, PathBuf::from("billing-service"));

        // 2. Single repo detection: root has .git
        let tmp_single = tempdir().unwrap();
        let single_root = tmp_single.path();
        fs::create_dir_all(single_root.join(".git")).unwrap();

        let discovered_single = GitWorkspaceResolver::discover_repos(single_root);
        assert_eq!(discovered_single.len(), 1);
        assert_eq!(discovered_single[0].prefix, PathBuf::new());
    }

    #[test]
    fn test_git_workspace_resolver_diff_execution() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let service = root.join("my-service");
        fs::create_dir_all(&service).unwrap();

        // Check if git is available in environment
        let git_check = Command::new("git").arg("--version").output();
        if git_check.as_ref().is_ok_and(|o| !o.status.success()) || git_check.is_err() {
            return;
        }

        let run_git = |args: &[&str], dir: &Path| {
            let _ = Command::new("git").args(args).current_dir(dir).output();
        };

        run_git(&["init"], &service);
        run_git(&["config", "user.name", "TestUser"], &service);
        run_git(&["config", "user.email", "test@example.com"], &service);

        let test_file = service.join("test.txt");
        fs::write(&test_file, "line 1\n").unwrap();
        run_git(&["add", "."], &service);
        run_git(&["commit", "-m", "initial commit"], &service);

        fs::write(&test_file, "line 1\nline 2\n").unwrap();

        let diff = GitWorkspaceResolver::execute_diff(root, None, false).unwrap();
        assert!(diff.contains("a/my-service/test.txt"));
    }
}
