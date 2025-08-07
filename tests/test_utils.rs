//! Common test utilities and helpers
//!
//! This module provides shared testing utilities used across multiple test files
//! to reduce code duplication and ensure consistent testing patterns.

use std::path::PathBuf;
use tempfile::TempDir;

/// Creates a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Creates a temporary file path within a given directory
pub fn create_temp_file_path(dir: &TempDir, filename: &str) -> PathBuf {
    dir.path().join(filename)
}

/// Test configuration constants
pub mod constants {
    pub const TEST_PLUGIN_ID: &str = "com.test.example";
    pub const TEST_PLUGIN_NAME: &str = "Test Plugin";
    pub const TEST_PLUGIN_VERSION: &str = "1.0.0";
    pub const TEST_PLUGIN_AUTHOR: &str = "Test Author";
    pub const TEST_PLUGIN_DESCRIPTION: &str = "A test plugin for unit testing";

    pub const TEST_ENCRYPTION_KEY: &str = "test-key-32-bytes-long-for-aes256";
    pub const TEST_SCHEMA_VERSION: &str = "1.0.0";
}

/// Helper macros for common test patterns
#[macro_export]
macro_rules! assert_plugin_valid {
    ($plugin:expr) => {
        assert!(
            $plugin.validate().is_ok(),
            "Plugin should be valid: {:?}",
            $plugin.validate().err()
        );
    };
}

#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $expected:expr) => {
        match $result {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => assert!(
                e.to_string().contains($expected),
                "Error '{}' should contain '{}'",
                e,
                $expected
            ),
        }
    };
}

/// Creates a default test state for testing  
pub fn create_test_state() -> xreal_virtual_desktop::AppState {
    xreal_virtual_desktop::AppState::default()
}

/// Creates a test plugin system configuration
pub fn create_test_plugin_config() -> xreal_virtual_desktop::plugins::PluginSystemConfig {
    use xreal_virtual_desktop::plugins::PluginSystemConfig;

    PluginSystemConfig {
        plugin_directories: vec!["/tmp/test_plugins".into()],
        max_plugins: 10,
        enable_hot_reload: false,
    }
}

/// Async test helper for state operations
pub async fn test_state_roundtrip(
    storage: &xreal_virtual_desktop::state::storage::StateStorage,
    state: &xreal_virtual_desktop::AppState,
) -> Result<xreal_virtual_desktop::AppState, Box<dyn std::error::Error>> {
    storage.save_state(state).await?;
    let loaded_state = storage.load_state().await?;
    Ok(loaded_state)
}

/// Helper for testing serialization roundtrips
pub fn test_serialization_roundtrip(
    serializer: &xreal_virtual_desktop::state::serialization::StateSerializer,
    state: &xreal_virtual_desktop::state::AppState,
) -> Result<xreal_virtual_desktop::state::AppState, Box<dyn std::error::Error>> {
    let json_string = serializer.serialize_to_string(state)?;
    let deserialized_state = serializer.deserialize_from_string(&json_string)?;
    Ok(deserialized_state)
}

/// Performance testing utilities
pub mod performance {
    use std::time::{Duration, Instant};

    /// Times the execution of a function
    pub fn time_function<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Times an async function
    pub async fn time_async_function<F, Fut, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Asserts that an operation completes within a time limit
    pub fn assert_within_time<F, R>(f: F, max_duration: Duration) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = time_function(f);
        assert!(
            duration <= max_duration,
            "Operation took {:?}, expected <= {:?}",
            duration,
            max_duration
        );
        result
    }
}
