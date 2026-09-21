use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use sha2::{Digest, Sha256};

use crate::discovery::FileWalker;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    New,
    Modified { old_hash: String },
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub absolute_path: PathBuf,
    pub relative_path: String,
    pub hash: String,
    pub status: FileStatus,
}

#[derive(Debug, Default)]
pub struct IncrementalChangeSet {
    pub to_index: Vec<DiscoveredFile>,
    pub unchanged_count: usize,
    pub deleted_paths: Vec<String>,
}

pub struct IncrementalDetector;

impl IncrementalDetector {
    /// Compute SHA-256 hash of file content bytes
    pub fn compute_sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    /// Convert an absolute path to a normalized forward-slash relative path
    pub fn to_relative_path(path: &Path, root_path: &Path) -> String {
        if let Ok(rel) = path.strip_prefix(root_path) {
            let path_str = rel.to_string_lossy().replace('\\', "/");
            return path_str.trim_start_matches("./").to_string();
        }
        if let (Ok(canon_path), Ok(canon_root)) = (path.canonicalize(), root_path.canonicalize())
            && let Ok(rel) = canon_path.strip_prefix(&canon_root)
        {
            let path_str = rel.to_string_lossy().replace('\\', "/");
            return path_str.trim_start_matches("./").to_string();
        }
        let path_str = path.to_string_lossy().replace('\\', "/");
        path_str.trim_start_matches("./").to_string()
    }

    /// Classify discovered files on disk against pre-fetched database hashes in parallel
    pub fn compute_changeset(
        discovered: &[PathBuf],
        root_path: &Path,
        existing_hashes: &HashMap<String, String>,
        force: bool,
        mmap_threshold: usize,
    ) -> Result<IncrementalChangeSet> {
        // Parallel hash calculation across Rayon thread pool
        let processed: Vec<std::io::Result<DiscoveredFile>> = discovered
            .par_iter()
            .map(|abs_path| {
                let buf = FileWalker::read_file_bytes(abs_path, mmap_threshold)?;
                let hash = Self::compute_sha256(&buf);
                let rel_path = Self::to_relative_path(abs_path, root_path);

                let status = match existing_hashes.get(&rel_path) {
                    Some(stored) if stored == &hash && !force => FileStatus::Unchanged,
                    Some(stored) => FileStatus::Modified {
                        old_hash: stored.clone(),
                    },
                    None => FileStatus::New,
                };

                Ok(DiscoveredFile {
                    absolute_path: abs_path.clone(),
                    relative_path: rel_path,
                    hash,
                    status,
                })
            })
            .collect();

        let mut to_index = Vec::new();
        let mut unchanged_count = 0;
        let mut visited_paths = HashSet::new();

        for item_res in processed {
            let file = item_res?;
            visited_paths.insert(file.relative_path.clone());

            match file.status {
                FileStatus::Unchanged => {
                    unchanged_count += 1;
                }
                FileStatus::New | FileStatus::Modified { .. } => {
                    to_index.push(file);
                }
            }
        }

        // Detect deleted paths (in DB but not seen on disk)
        let mut deleted_paths: Vec<String> = existing_hashes
            .keys()
            .filter(|path| !visited_paths.contains(*path))
            .cloned()
            .collect();
        deleted_paths.sort();

        Ok(IncrementalChangeSet {
            to_index,
            unchanged_count,
            deleted_paths,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sha256_computation() {
        let data = b"Hello, Cartograph!";
        let hash = IncrementalDetector::compute_sha256(data);
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "b075208286d0dcd8e06bf786ac824be030439886f6e78e73bdb5c4a3d6306040"
        );
    }

    #[test]
    fn test_incremental_changeset_detection() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();

        let f1 = root.join("file1.py");
        let f2 = root.join("file2.ts");
        let f3 = root.join("file3.rs");

        fs::write(&f1, "content1").unwrap();
        fs::write(&f2, "content2").unwrap();
        fs::write(&f3, "content3").unwrap();

        let hash1 = IncrementalDetector::compute_sha256(b"content1");
        let hash2_old = IncrementalDetector::compute_sha256(b"content2_old");

        let mut existing = HashMap::new();
        existing.insert("file1.py".to_string(), hash1.clone());
        existing.insert("file2.ts".to_string(), hash2_old);
        existing.insert("file_deleted.py".to_string(), "old_hash".to_string());

        let discovered = vec![f1, f2, f3];

        let changeset =
            IncrementalDetector::compute_changeset(&discovered, root, &existing, false, 64 * 1024)
                .unwrap();

        assert_eq!(changeset.unchanged_count, 1); // file1.py unchanged
        assert_eq!(changeset.to_index.len(), 2); // file2.ts (modified), file3.rs (new)
        assert_eq!(changeset.deleted_paths, vec!["file_deleted.py".to_string()]);

        let rel_paths: Vec<String> = changeset
            .to_index
            .iter()
            .map(|f| f.relative_path.clone())
            .collect();
        assert!(rel_paths.contains(&"file2.ts".to_string()));
        assert!(rel_paths.contains(&"file3.rs".to_string()));
    }

    #[test]
    fn test_force_flag_bypasses_unchanged() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let f1 = root.join("file1.py");
        fs::write(&f1, "content1").unwrap();

        let hash1 = IncrementalDetector::compute_sha256(b"content1");
        let mut existing = HashMap::new();
        existing.insert("file1.py".to_string(), hash1);

        let changeset = IncrementalDetector::compute_changeset(
            &[f1],
            root,
            &existing,
            true, // force = true
            64 * 1024,
        )
        .unwrap();

        assert_eq!(changeset.unchanged_count, 0);
        assert_eq!(changeset.to_index.len(), 1);
    }
}
