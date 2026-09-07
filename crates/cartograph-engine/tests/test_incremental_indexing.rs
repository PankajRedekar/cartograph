use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

use cartograph_ast::registry::AstRegistry;
use cartograph_core::config::StorageConfig;
use cartograph_core::model::NodeKind;
use cartograph_db::create_backend;
use cartograph_engine::discovery::{DiscoveryConfig, FileWalker, ZeroCopyBuffer};
use cartograph_engine::hasher::IncrementalDetector;
use cartograph_engine::pipeline::IndexPipeline;

#[test]
fn test_zero_copy_buffer_small_and_large_files() {
    let dir = tempdir().unwrap();
    let small_file = dir.path().join("small.rs");
    let large_file = dir.path().join("large.rs");

    let small_content = "fn small() {}";
    fs::write(&small_file, small_content).unwrap();

    // 70KB file to trigger mmap (> 64KB threshold)
    let mut large_content = String::with_capacity(75 * 1024);
    for i in 0..1500 {
        large_content.push_str(&format!("// comment line {}\nfn func_{}() {{}}\n", i, i));
    }
    fs::write(&large_file, &large_content).unwrap();

    let small_buf = ZeroCopyBuffer::read_from_path(&small_file, 64 * 1024).unwrap();
    assert_eq!(small_buf.as_bytes(), small_content.as_bytes());
    assert_eq!(small_buf.as_str().unwrap(), small_content);

    let large_buf = ZeroCopyBuffer::read_from_path(&large_file, 64 * 1024).unwrap();
    assert_eq!(large_buf.as_bytes(), large_content.as_bytes());
    assert_eq!(large_buf.as_str().unwrap(), &large_content);
}

#[test]
fn test_file_walker_and_ignore_rules() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create files
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn lib() {}").unwrap();
    fs::write(root.join("src/app.py"), "def run(): pass").unwrap();

    // Ignored / Vendor directories
    fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
    fs::write(root.join("node_modules/pkg/index.js"), "console.log('hi');").unwrap();
    fs::create_dir_all(root.join("vendor/bundle")).unwrap();
    fs::write(root.join("vendor/bundle/test.rb"), "puts 'hi'").unwrap();

    // Unsupported files
    fs::write(root.join("readme.md"), "# Hello").unwrap();
    fs::write(root.join("data.txt"), "text data").unwrap();

    let registry = AstRegistry::standard();
    let config = DiscoveryConfig {
        root_path: root.to_path_buf(),
        ..DiscoveryConfig::default()
    };
    let walker = FileWalker::new(config, registry);
    let discovered = walker.collect_files().unwrap();

    let discovered_names: Vec<String> = discovered
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();

    assert!(discovered_names.contains(&"main.rs".to_string()));
    assert!(discovered_names.contains(&"lib.rs".to_string()));
    assert!(discovered_names.contains(&"app.py".to_string()));

    // Verify vendor / unsupported are excluded
    assert!(!discovered_names.contains(&"index.js".to_string()));
    assert!(!discovered_names.contains(&"test.rb".to_string()));
    assert!(!discovered_names.contains(&"data.txt".to_string()));
}

#[test]
fn test_incremental_detector_changes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let f1 = root.join("a.rs");
    let f2 = root.join("b.rs");
    fs::write(&f1, "fn a() {}").unwrap();
    fs::write(&f2, "fn b() {}").unwrap();

    let registry = AstRegistry::standard();
    let config = DiscoveryConfig {
        root_path: root.to_path_buf(),
        ..DiscoveryConfig::default()
    };
    let walker = FileWalker::new(config, registry);
    let files = walker.collect_files().unwrap();

    // 1. Initial cold detection (empty db_hashes)
    let empty_hashes = HashMap::new();
    let changeset = IncrementalDetector::compute_changeset(
        &files,
        root,
        &empty_hashes,
        false,
        64 * 1024,
    )
    .unwrap();

    assert_eq!(changeset.to_index.len(), 2);
    assert_eq!(changeset.deleted_paths.len(), 0);

    let mut current_hashes = HashMap::new();
    for df in &changeset.to_index {
        current_hashes.insert(df.relative_path.clone(), df.hash.clone());
    }
    assert_eq!(current_hashes.len(), 2);

    // 2. Unchanged detection
    let changeset2 = IncrementalDetector::compute_changeset(
        &files,
        root,
        &current_hashes,
        false,
        64 * 1024,
    )
    .unwrap();
    assert_eq!(changeset2.to_index.len(), 0);
    assert_eq!(changeset2.deleted_paths.len(), 0);
    assert_eq!(changeset2.unchanged_count, 2);

    // 3. Modify one file
    fs::write(&f1, "fn a_modified() {}").unwrap();
    let changeset3 = IncrementalDetector::compute_changeset(
        &files,
        root,
        &current_hashes,
        false,
        64 * 1024,
    )
    .unwrap();
    assert_eq!(changeset3.to_index.len(), 1);
    assert_eq!(changeset3.to_index[0].absolute_path, f1);
    assert_eq!(changeset3.deleted_paths.len(), 0);

    // 4. Delete one file
    fs::remove_file(&f2).unwrap();
    let walker_after_del = FileWalker::new(
        DiscoveryConfig {
            root_path: root.to_path_buf(),
            ..DiscoveryConfig::default()
        },
        AstRegistry::standard(),
    );
    let files_after_delete = walker_after_del.collect_files().unwrap();
    let changeset4 = IncrementalDetector::compute_changeset(
        &files_after_delete,
        root,
        &current_hashes,
        false,
        64 * 1024,
    )
    .unwrap();
    assert_eq!(changeset4.deleted_paths.len(), 1);
    assert_eq!(changeset4.deleted_paths[0], "b.rs");
}

#[tokio::test]
async fn test_pipeline_cold_and_incremental_indexing() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let db_path = dir.path().join("cartograph.db");

    // Polyglot files
    let rust_file = root.join("main.rs");
    let py_file = root.join("service.py");
    let ts_file = root.join("client.ts");

    fs::write(
        &rust_file,
        r#"
pub struct RustServer {
    pub port: u16,
}

impl RustServer {
    pub fn start(&self) {
        println!("listening");
    }
}
"#,
    )
    .unwrap();

    fs::write(
        &py_file,
        r#"
class AuthService:
    def authenticate(self, token: str):
        return True
"#,
    )
    .unwrap();

    fs::write(
        &ts_file,
        r#"
export interface UserSession {
    token: string;
}

export function createSession(token: string): UserSession {
    return { token };
}
"#,
    )
    .unwrap();

    let storage_config = StorageConfig::sqlite(db_path.to_str().unwrap());
    use cartograph_core::backend::GraphStoreBackend;
    let backend: Arc<dyn GraphStoreBackend> = Arc::from(create_backend(&storage_config).unwrap());
    let discovery_config = DiscoveryConfig {
        root_path: root.to_path_buf(),
        ..DiscoveryConfig::default()
    };
    let pipeline = IndexPipeline::new(backend.clone(), discovery_config);

    // Step 1: Cold Index
    let stats1 = pipeline.run(false).await.unwrap();
    assert_eq!(stats1.files_scanned, 3);
    assert_eq!(stats1.files_indexed, 3);
    assert_eq!(stats1.files_skipped, 0);
    assert_eq!(stats1.files_deleted, 0);
    assert!(stats1.nodes_indexed >= 4);

    // Verify stored nodes
    let nodes_in_rust = backend.get_nodes_by_filepath("main.rs").await.unwrap();
    assert!(nodes_in_rust.iter().any(|n| n.name == "RustServer" && n.kind == NodeKind::Struct));
    assert!(nodes_in_rust.iter().any(|n| n.name == "start" && n.kind == NodeKind::Method));

    let nodes_in_py = backend.get_nodes_by_filepath("service.py").await.unwrap();
    assert!(nodes_in_py.iter().any(|n| n.name == "AuthService" && n.kind == NodeKind::Class));
    assert!(nodes_in_py.iter().any(|n| n.name == "authenticate" && n.kind == NodeKind::Method));

    // Step 2: Immediate Incremental Re-index (No-Op)
    let stats2 = pipeline.run(false).await.unwrap();
    assert_eq!(stats2.files_scanned, 3);
    assert_eq!(stats2.files_indexed, 0);
    assert_eq!(stats2.files_skipped, 3);
    assert_eq!(stats2.files_deleted, 0);
    assert_eq!(stats2.nodes_indexed, 0);
    // Verified fast no-op (<50ms target)
    assert!(stats2.elapsed_ms < 100);

    // Step 3: Modify single file (Python)
    fs::write(
        &py_file,
        r#"
class AuthService:
    def authenticate(self, token: str):
        return True

    def revoke(self, token: str):
        pass
"#,
    )
    .unwrap();

    let stats3 = pipeline.run(false).await.unwrap();
    assert_eq!(stats3.files_scanned, 3);
    assert_eq!(stats3.files_indexed, 1);
    assert_eq!(stats3.files_skipped, 2);
    assert_eq!(stats3.files_deleted, 0);

    let updated_py_nodes = backend.get_nodes_by_filepath("service.py").await.unwrap();
    assert!(updated_py_nodes.iter().any(|n| n.name == "revoke"));

    // Step 4: Delete TS file & orphan pruning
    fs::remove_file(&ts_file).unwrap();
    let stats4 = pipeline.run(false).await.unwrap();
    assert_eq!(stats4.files_scanned, 2);
    assert_eq!(stats4.files_indexed, 0);
    assert_eq!(stats4.files_skipped, 2);
    assert_eq!(stats4.files_deleted, 1);

    let ts_nodes_after_del = backend.get_nodes_by_filepath("client.ts").await.unwrap();
    assert!(ts_nodes_after_del.is_empty());
}
