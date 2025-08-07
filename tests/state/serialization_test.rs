//! Tests for state serialization system
//!
//! Extracted from src/state/serialization.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use std::path::PathBuf;
use tempfile::TempDir;
use xreal_virtual_desktop::state::serialization::*;
use xreal_virtual_desktop::AppState;

#[test]
fn test_serialize_deserialize_roundtrip() {
    let serializer = StateSerializer::new();
    let original_state = AppState::default();

    // Serialize to string
    let json_string = serializer
        .serialize_to_string(&original_state)
        .expect("Serialization failed");

    // Deserialize back
    let deserialized_state = serializer
        .deserialize_from_string(&json_string)
        .expect("Deserialization failed");

    // Verify schema version matches
    assert_eq!(
        deserialized_state.schema_version,
        original_state.schema_version
    );
}

#[test]
fn test_file_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_state.json");

    let serializer = StateSerializer::new();
    let original_state = AppState::default();

    // Serialize to file
    serializer
        .serialize_to_file(&original_state, &file_path)
        .expect("File serialization failed");

    // Verify file exists
    assert!(file_path.exists());

    // Deserialize from file
    let deserialized_state = serializer
        .deserialize_from_file(&file_path)
        .expect("File deserialization failed");

    // Verify schema version matches
    assert_eq!(
        deserialized_state.schema_version,
        original_state.schema_version
    );
}

#[test]
fn test_json_validation() {
    let serializer = StateSerializer::new();
    let state = AppState::default();
    let json_string = serializer
        .serialize_to_string(&state)
        .expect("Serialization failed");

    // Valid JSON should pass
    assert!(serializer.validate_json(&json_string).is_ok());

    // Invalid JSON should fail
    assert!(serializer.validate_json("invalid json").is_err());

    // Missing required field should fail
    assert!(serializer.validate_json("{}").is_err());
}

#[test]
fn test_batch_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path1 = temp_dir.path().join("state1.json");
    let file_path2 = temp_dir.path().join("state2.json");

    let mut batch_serializer = BatchStateSerializer::new();
    let state1 = AppState::default();
    let state2 = AppState::default();

    // Add operations
    batch_serializer.add_save_operation(state1, file_path1.clone());
    batch_serializer.add_save_operation(state2, file_path2.clone());
    batch_serializer.add_load_operation(file_path1);
    batch_serializer.add_validation_operation(file_path2);

    // Execute batch
    let results = batch_serializer.execute().expect("Batch execution failed");

    // Verify results
    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| r.is_success()));
}

#[test]
fn test_utility_functions() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_state.json");

    let serializer = StateSerializer::new();
    let state = AppState::default();

    // Create test file
    serializer
        .serialize_to_file(&state, &file_path)
        .expect("File creation failed");

    // Test utility functions
    assert!(utils::is_valid_state_file(&file_path));
    assert!(utils::get_state_file_size(&file_path).expect("Size check failed") > 0);
    assert!(utils::get_state_file_modified_time(&file_path).is_ok());

    // Test backup creation
    let backup_path = utils::create_state_backup(&file_path).expect("Backup creation failed");
    assert!(backup_path.exists());

    // Test backup restoration
    fs::remove_file(&file_path).expect("File removal failed");
    utils::restore_from_backup(&file_path).expect("Backup restoration failed");
    assert!(file_path.exists());
}
