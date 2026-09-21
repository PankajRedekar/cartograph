use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use tokio::sync::{broadcast, oneshot};

use cartograph_core::backend::GraphStoreBackend;
use cartograph_core::config::StorageConfig;
use cartograph_db::create_backend;
use cartograph_engine::discovery::DiscoveryConfig;
use cartograph_engine::pipeline::IndexPipeline;
use cartograph_engine::watcher::{FileWatcher, WatchEvent, WatcherSettings};

#[tokio::test]
async fn test_watcher_handle_file_save_and_noop() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let db_path = dir.path().join("watcher_save.db");

    let file_path = root.join("service.rs");
    fs::write(&file_path, "pub fn run() {}").unwrap();

    let storage_config = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = Arc::from(create_backend(&storage_config).unwrap());

    let settings = WatcherSettings {
        root_path: root.to_path_buf(),
        debounce_duration: Duration::from_millis(25),
        cst_cache_capacity: 100,
        track_blast_radius: true,
        lint_arch: None,
        deadcode: None,
    };

    let watcher = FileWatcher::new(backend.clone(), settings);

    // Initial save
    let event1 = watcher.handle_file_save(&file_path).await.unwrap();
    assert!(event1.is_some());
    if let Some(WatchEvent::FileIndexed { filepath, nodes, .. }) = event1 {
        assert_eq!(filepath, "service.rs");
        assert_eq!(nodes, 1);
    } else {
        panic!("Expected FileIndexed event");
    }

    // Unchanged file save should be a no-op (returns None)
    let event2 = watcher.handle_file_save(&file_path).await.unwrap();
    assert!(event2.is_none());

    // Modify file
    fs::write(&file_path, "pub fn run() {}\npub fn stop() {}").unwrap();
    let event3 = watcher.handle_file_save(&file_path).await.unwrap();
    assert!(event3.is_some());
    if let Some(WatchEvent::FileIndexed { filepath, nodes, .. }) = event3 {
        assert_eq!(filepath, "service.rs");
        assert_eq!(nodes, 2);
    } else {
        panic!("Expected FileIndexed event on modify");
    }
}

#[tokio::test]
async fn test_watcher_reverse_blast_radius_tracking() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let db_path = dir.path().join("watcher_blast.db");

    let core_rs = root.join("core.rs");
    let api_rs = root.join("api.rs");

    fs::write(&core_rs, "pub fn compute() {}").unwrap();
    fs::write(
        &api_rs,
        r#"
fn handle() {
    compute();
}
"#,
    )
    .unwrap();

    let storage_config = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = Arc::from(create_backend(&storage_config).unwrap());

    // 1. Run pipeline index to link api.rs -> core.rs
    let discovery_config = DiscoveryConfig {
        root_path: root.to_path_buf(),
        ..DiscoveryConfig::default()
    };
    let pipeline = IndexPipeline::new(backend.clone(), discovery_config);
    let stats = pipeline.run(false).await.unwrap();
    assert_eq!(stats.files_indexed, 2);
    assert!(stats.edges_indexed >= 1);

    let settings = WatcherSettings {
        root_path: root.to_path_buf(),
        debounce_duration: Duration::from_millis(25),
        cst_cache_capacity: 100,
        track_blast_radius: true,
        lint_arch: None,
        deadcode: None,
    };
    let watcher = FileWatcher::new(backend.clone(), settings);

    // 2. Modify core.rs and verify api.rs is flagged in affected_dependents
    fs::write(&core_rs, "pub fn compute() { println!(\"updated\"); }").unwrap();

    let event = watcher.handle_file_save(&core_rs).await.unwrap();
    assert!(event.is_some());
    if let Some(WatchEvent::FileIndexed { filepath, affected_dependents, .. }) = event {
        assert_eq!(filepath, "core.rs");
        assert!(
            affected_dependents.contains(&"api.rs".to_string()),
            "Blast radius must contain api.rs, got: {:?}",
            affected_dependents
        );
    } else {
        panic!("Expected FileIndexed event");
    }

    // 3. Delete core.rs and verify blast radius before deletion
    let del_event = watcher.handle_file_delete(&core_rs).await.unwrap();
    assert!(del_event.is_some());
    if let Some(WatchEvent::FileDeleted { filepath, affected_dependents, .. }) = del_event {
        assert_eq!(filepath, "core.rs");
        assert!(
            affected_dependents.contains(&"api.rs".to_string()),
            "Blast radius on delete must contain api.rs, got: {:?}",
            affected_dependents
        );
    } else {
        panic!("Expected FileDeleted event");
    }
}

#[tokio::test]
async fn test_watcher_background_loop_and_channel_dispatch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let db_path = dir.path().join("watcher_loop.db");

    let test_file = root.join("worker.rs");
    fs::write(&test_file, "fn work() {}").unwrap();

    let storage_config = StorageConfig::sqlite(db_path.to_str().unwrap());
    let backend: Arc<dyn GraphStoreBackend> = Arc::from(create_backend(&storage_config).unwrap());

    let settings = WatcherSettings {
        root_path: root.to_path_buf(),
        debounce_duration: Duration::from_millis(30),
        cst_cache_capacity: 50,
        track_blast_radius: false,
        lint_arch: None,
        deadcode: None,
    };
    let watcher = Arc::new(FileWatcher::new(backend.clone(), settings));

    let (event_tx, mut event_rx) = broadcast::channel(32);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let watcher_clone = watcher.clone();
    let handle = tokio::spawn(async move {
        watcher_clone.start(event_tx, shutdown_rx).await
    });

    // Give watcher time to initialize in background
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Trigger filesystem event by modifying worker.rs
    fs::write(&test_file, "fn work() { println!(\"working\"); }").unwrap();

    // Await event with 3-second timeout
    let received = tokio::time::timeout(Duration::from_secs(3), async {
        while let Ok(ev) = event_rx.recv().await {
            match ev {
                WatchEvent::FileIndexed { filepath, .. } if filepath == "worker.rs" => {
                    return true;
                }
                _ => {}
            }
        }
        false
    })
    .await;

    // Graceful shutdown
    let _ = shutdown_tx.send(());
    let join_res = handle.await.unwrap();
    assert!(join_res.is_ok());

    assert!(received.is_ok(), "Timed out waiting for file watch event");
    assert!(received.unwrap(), "Expected FileIndexed event for worker.rs");
}
