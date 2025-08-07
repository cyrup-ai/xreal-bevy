//! Tests for state storage system
//!
//! Extracted from src/state/storage.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use tempfile::TempDir;
use xreal_virtual_desktop::state::storage::*;

#[tokio::test]
async fn test_storage_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = StorageConfig {
        base_directory: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = StateStorage::with_config(config).expect("Failed to create storage");
    let state = AppState::default();

    // Test save
    storage.save_state(&state).await.expect("Save failed");

    // Test load
    let loaded_state = storage.load_state().await.expect("Load failed");
    assert_eq!(loaded_state.schema_version, state.schema_version);

    // Test validation
    storage
        .validate_state_file()
        .await
        .expect("Validation failed");
}

#[tokio::test]
async fn test_backup_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = StorageConfig {
        base_directory: temp_dir.path().to_path_buf(),
        backup_config: BackupConfig {
            enabled: true,
            max_backups: 3,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = StateStorage::with_config(config).expect("Failed to create storage");
    let state = AppState::default();

    // Save state multiple times to create backups
    for i in 0..5 {
        let mut modified_state = state.clone();
        modified_state.last_updated = i;
        storage
            .save_state(&modified_state)
            .await
            .expect("Save failed");

        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Check backup count
    let backups = storage.list_backups().await.expect("List backups failed");
    assert!(backups.len() <= 5); // Should have some backups

    // Test cleanup
    storage.cleanup_old_backups().await.expect("Cleanup failed");

    let backups_after_cleanup = storage.list_backups().await.expect("List backups failed");
    assert!(backups_after_cleanup.len() <= 3); // Should respect max_backups
}

#[tokio::test]
async fn test_atomic_writes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = StorageConfig {
        base_directory: temp_dir.path().to_path_buf(),
        atomic_writes: true,
        ..Default::default()
    };

    let storage = StateStorage::with_config(config).expect("Failed to create storage");
    let file_path = storage.get_primary_state_file_path();

    // Test atomic write
    let data = r#"{"test": "data"}"#;
    storage
        .atomic_write(&file_path, data)
        .await
        .expect("Atomic write failed");

    // Verify file exists and has correct content
    assert!(file_path.exists());
    let contents = fs::read_to_string(&file_path).await.expect("Read failed");
    assert_eq!(contents, data);

    // Verify no temp file remains
    let temp_path = file_path.with_extension("tmp");
    assert!(!temp_path.exists());
}

#[tokio::test]
async fn test_async_storage_wrapper() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = StorageConfig {
        base_directory: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = StateStorage::with_config(config).expect("Failed to create storage");
    let async_storage = AsyncStateStorage::new(storage);

    let state = AppState::default();

    // Test async save
    async_storage
        .save_state_async(state.clone())
        .await
        .expect("Async save failed");

    // Test async load
    let loaded_state = async_storage
        .load_state_async()
        .await
        .expect("Async load failed");
    assert_eq!(loaded_state.schema_version, state.schema_version);
}

#[tokio::test]
async fn test_storage_statistics() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = StorageConfig {
        base_directory: temp_dir.path().to_path_buf(),
        backup_config: BackupConfig {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let storage = StateStorage::with_config(config).expect("Failed to create storage");
    let state = AppState::default();

    // Save state to create primary file
    storage.save_state(&state).await.expect("Save failed");

    // Get statistics
    let stats = storage.get_statistics().await.expect("Statistics failed");

    assert!(stats.primary_file_size > 0);
    assert!(stats.primary_file_modified.is_some());
    assert_eq!(stats.backup_count, 0); // No backups yet
    assert_eq!(stats.total_storage_size, stats.primary_file_size);

    // Test size formatting
    assert!(stats.formatted_primary_size().contains("B"));
    assert!(stats.formatted_total_size().contains("B"));
}
