//! Integration tests for XREAL Bevy virtual desktop
//!
//! This module organizes all integration tests following Rust best practices
//! with clean separation between unit tests (in src/) and integration tests (in tests/).

// Test modules organized by category
pub mod plugins;
pub mod state;

// Common test utilities
pub mod test_utils;

// Test configuration and common utilities
pub mod common {
    //! Common test utilities and configuration
    
    use std::path::PathBuf;
    
    /// Get temporary directory for tests
    pub fn get_test_temp_dir() -> tempfile::TempDir {
        tempfile::TempDir::new().expect("Failed to create temporary directory for tests")
    }
    
    /// Create test data directory structure
    pub fn create_test_data_dir(base: &std::path::Path) -> anyhow::Result<()> {
        let subdirs = ["plugins", "state", "cache", "logs"];
        
        for subdir in &subdirs {
            let dir_path = base.join(subdir);
            std::fs::create_dir_all(&dir_path)?;
        }
        
        Ok(())
    }
    
    /// Test timeout configuration
    pub const DEFAULT_TEST_TIMEOUT_MS: u64 = 30_000;
    pub const INTEGRATION_TEST_TIMEOUT_MS: u64 = 60_000;
    pub const QUICK_TEST_TIMEOUT_MS: u64 = 5_000;
}