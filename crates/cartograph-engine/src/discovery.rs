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

    /// Discover files matching git diff mode (--staged or --branch)
    pub fn discover_git_diff_files(
        root_path: &Path,
        diff_mode: &GitDiffMode,
    ) -> std::io::Result<Vec<PathBuf>> {
        let mut cmd = Command::new("git");
        cmd.current_dir(root_path);

        match diff_mode {
            GitDiffMode::StagedOnly => {
                cmd.args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"]);
            }
            GitDiffMode::BranchRange(range) => {
                cmd.args(["diff", "--name-only", "--diff-filter=ACMR", range]);
            }
        }

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "git diff failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let full_path = root_path.join(trimmed);
                if full_path.is_file() {
                    files.push(full_path);
                }
            }
        }
        Ok(files)
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
                if path_str.contains(pattern.as_str()) {
                    excluded = true;
                    break;
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
}
