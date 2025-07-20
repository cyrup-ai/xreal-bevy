//! Serialization Layer for State Persistence
//! 
//! Handles conversion between AppState and JSON format with comprehensive
//! error handling and no unwrap/expect usage. Supports versioned serialization
//! for schema migration.

use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;
use zstd;

use crate::state::{AppState, StateError};
use bevy::prelude::{info, warn, error};

/// Serialization manager for state persistence
pub struct StateSerializer {
    /// Pretty-print JSON for debugging
    pretty_print: bool,
    /// Compression enabled
    compression_enabled: bool,
    /// Backup original format during serialization
    create_backup: bool,
}

impl StateSerializer {
    /// Create new state serializer
    pub fn new() -> Self {
        Self {
            pretty_print: cfg!(debug_assertions),
            compression_enabled: false,
            create_backup: true,
        }
    }
    
    /// Create state serializer with custom configuration
    pub fn with_config(pretty_print: bool, compression_enabled: bool, create_backup: bool) -> Self {
        Self {
            pretty_print,
            compression_enabled,
            create_backup,
        }
    }
    
    /// Serialize AppState to JSON string
    pub fn serialize_to_string(&self, state: &AppState) -> Result<String> {
        let json_result = if self.pretty_print {
            serde_json::to_string_pretty(state)
        } else {
            serde_json::to_string(state)
        };
        
        match json_result {
            Ok(json_string) => {
                if self.compression_enabled {
                    self.compress_json(&json_string)
                } else {
                    Ok(json_string)
                }
            }
            Err(e) => Err(StateError::SerializationError(e).into()),
        }
    }
    
    /// Deserialize AppState from JSON string
    pub fn deserialize_from_string(&self, json_string: &str) -> Result<AppState> {
        let decompressed_json = if self.compression_enabled {
            self.decompress_json(json_string)?
        } else {
            json_string.to_string()
        };
        
        match serde_json::from_str::<AppState>(&decompressed_json) {
            Ok(state) => Ok(state),
            Err(e) => Err(StateError::SerializationError(e).into()),
        }
    }
    
    /// Serialize AppState to file
    pub fn serialize_to_file(&self, state: &AppState, file_path: &Path) -> Result<()> {
        // Create backup if requested
        if self.create_backup && file_path.exists() {
            let backup_path = file_path.with_extension("backup.json");
            if let Err(e) = fs::copy(file_path, &backup_path) {
                warn!("Failed to create backup file: {}", e);
                // Continue with serialization - backup failure shouldn't prevent saving
            }
        }
        
        // Serialize to string
        let json_string = self.serialize_to_string(state)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| StateError::StorageError(e))?;
            }
        }
        
        // Write to temporary file first for atomic operation
        let temp_path = file_path.with_extension("tmp");
        match fs::write(&temp_path, json_string) {
            Ok(()) => {
                // Atomic rename to final location
                match fs::rename(&temp_path, file_path) {
                    Ok(()) => {
                        info!("✅ State serialized successfully to {:?}", file_path);
                        Ok(())
                    }
                    Err(e) => {
                        // Cleanup temp file on failure
                        let _ = fs::remove_file(&temp_path);
                        Err(StateError::StorageError(e).into())
                    }
                }
            }
            Err(e) => {
                // Cleanup temp file on failure
                let _ = fs::remove_file(&temp_path);
                Err(StateError::StorageError(e).into())
            }
        }
    }
    
    /// Deserialize AppState from file
    pub fn deserialize_from_file(&self, file_path: &Path) -> Result<AppState> {
        // Check if file exists
        if !file_path.exists() {
            return Err(StateError::StorageError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("State file not found: {:?}", file_path)
                )
            ).into());
        }
        
        // Read file contents
        let json_string = fs::read_to_string(file_path)
            .map_err(|e| StateError::StorageError(e))?;
        
        // Deserialize from string
        match self.deserialize_from_string(&json_string) {
            Ok(state) => {
                info!("✅ State deserialized successfully from {:?}", file_path);
                Ok(state)
            }
            Err(e) => {
                error!("❌ Failed to deserialize state from {:?}: {}", file_path, e);
                Err(e)
            }
        }
    }
    
    /// Validate JSON format without full deserialization
    pub fn validate_json(&self, json_string: &str) -> Result<()> {
        // First check if it's valid JSON
        let json_value: serde_json::Value = serde_json::from_str(json_string)
            .map_err(|e| StateError::SerializationError(e))?;
        
        // Check for required fields
        if !json_value.is_object() {
            return Err(StateError::CorruptedState("Root must be an object".to_string()).into());
        }
        
        let obj = json_value.as_object().ok_or_else(|| {
            StateError::CorruptedState("Invalid JSON object".to_string())
        })?;
        
        // Check for schema version
        if !obj.contains_key("schema_version") {
            return Err(StateError::CorruptedState("Missing schema_version field".to_string()).into());
        }
        
        // Check for required top-level fields
        let required_fields = [
            "schema_version",
            "last_updated",
            "user_preferences",
            "ui_state",
            "calibration_data",
            "plugin_state",
            "performance_settings",
            "window_layout",
        ];
        
        for field in &required_fields {
            if !obj.contains_key(*field) {
                return Err(StateError::CorruptedState(
                    format!("Missing required field: {}", field)
                ).into());
            }
        }
        
        info!("✅ JSON validation passed");
        Ok(())
    }
    
    /// Migrate state from older schema version
    pub fn migrate_state(&self, json_string: &str) -> Result<AppState> {
        // Parse as generic JSON value first
        let mut json_value: serde_json::Value = serde_json::from_str(json_string)
            .map_err(|e| StateError::SerializationError(e))?;
        
        // Get schema version
        let schema_version = json_value.get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");
        
        info!("🔄 Migrating state from schema version {}", schema_version);
        
        // Apply migrations based on version
        match schema_version {
            "0.0.0" => {
                // Migration from initial version
                self.migrate_from_v0_0_0(&mut json_value)?;
            }
            "0.1.0" => {
                // Migration from v0.1.0
                self.migrate_from_v0_1_0(&mut json_value)?;
            }
            crate::state::schema::STATE_SCHEMA_VERSION => {
                // Already current version
                info!("✅ State already at current schema version");
            }
            _ => {
                warn!("⚠️  Unknown schema version: {}, attempting to load as-is", schema_version);
            }
        }
        
        // Update schema version to current
        if let Some(obj) = json_value.as_object_mut() {
            obj.insert("schema_version".to_string(), 
                      serde_json::Value::String(crate::state::schema::STATE_SCHEMA_VERSION.to_string()));
        }
        
        // Convert back to string and deserialize
        let migrated_json = serde_json::to_string(&json_value)
            .map_err(|e| StateError::SerializationError(e))?;
        
        self.deserialize_from_string(&migrated_json)
    }
    
    /// Compress JSON string using zstd compression algorithm
    /// Uses level 3 for optimal balance of compression ratio and speed
    fn compress_json(&self, json_string: &str) -> Result<String> {
        if !self.compression_enabled {
            return Ok(json_string.to_string());
        }

        let json_bytes = json_string.as_bytes();
        
        // Use compression level 3 for optimal performance/ratio balance
        // Level 3 provides ~70% compression with minimal CPU overhead
        let compressed_bytes = zstd::encode_all(json_bytes, 3)
            .map_err(|e| anyhow::anyhow!("Compression failed: {}", e))?;
        
        // Encode compressed bytes as base64 for safe string storage
        use base64::Engine;
        let base64_string = base64::engine::general_purpose::STANDARD.encode(&compressed_bytes);
        
        info!("Compressed {} bytes to {} bytes ({:.1}% reduction)", 
              json_bytes.len(), 
              compressed_bytes.len(),
              100.0 * (1.0 - compressed_bytes.len() as f64 / json_bytes.len() as f64));
        
        Ok(base64_string)
    }
    
    /// Decompress JSON string using zstd decompression algorithm
    fn decompress_json(&self, compressed_string: &str) -> Result<String> {
        if !self.compression_enabled {
            return Ok(compressed_string.to_string());
        }

        // Decode base64 string to compressed bytes
        use base64::Engine;
        let compressed_bytes = base64::engine::general_purpose::STANDARD
            .decode(compressed_string)
            .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;
        
        // Decompress using zstd
        let decompressed_bytes = zstd::decode_all(&compressed_bytes[..])
            .map_err(|e| anyhow::anyhow!("Decompression failed: {}", e))?;
        
        // Convert bytes back to UTF-8 string
        let json_string = String::from_utf8(decompressed_bytes)
            .map_err(|e| anyhow::anyhow!("UTF-8 conversion failed: {}", e))?;
        
        Ok(json_string)
    }
    
    /// Migrate from schema version 0.0.0
    fn migrate_from_v0_0_0(&self, _json_value: &mut serde_json::Value) -> Result<()> {
        // Add any fields that were missing in v0.0.0
        info!("🔄 Applying v0.0.0 migration");
        
        // Migration logic would go here
        // For now, just log the migration
        Ok(())
    }
    
    /// Migrate from schema version 0.1.0
    fn migrate_from_v0_1_0(&self, _json_value: &mut serde_json::Value) -> Result<()> {
        // Add any fields that were missing in v0.1.0
        info!("🔄 Applying v0.1.0 migration");
        
        // Migration logic would go here
        // For now, just log the migration
        Ok(())
    }
}

impl Default for StateSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch serialization operations for multiple state files
pub struct BatchStateSerializer {
    /// Individual serializer
    serializer: StateSerializer,
    /// Operations to perform
    operations: Vec<BatchOperation>,
}

/// Batch operation types
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Save { state: AppState, file_path: std::path::PathBuf },
    Load { file_path: std::path::PathBuf },
    Validate { file_path: std::path::PathBuf },
    Migrate { file_path: std::path::PathBuf },
}

impl BatchStateSerializer {
    /// Create new batch serializer
    pub fn new() -> Self {
        Self {
            serializer: StateSerializer::new(),
            operations: Vec::new(),
        }
    }
    
    /// Add save operation to batch
    pub fn add_save_operation(&mut self, state: AppState, file_path: std::path::PathBuf) {
        self.operations.push(BatchOperation::Save { state, file_path });
    }
    
    /// Add load operation to batch
    pub fn add_load_operation(&mut self, file_path: std::path::PathBuf) {
        self.operations.push(BatchOperation::Load { file_path });
    }
    
    /// Add validation operation to batch
    pub fn add_validation_operation(&mut self, file_path: std::path::PathBuf) {
        self.operations.push(BatchOperation::Validate { file_path });
    }
    
    /// Add migration operation to batch
    pub fn add_migration_operation(&mut self, file_path: std::path::PathBuf) {
        self.operations.push(BatchOperation::Migrate { file_path });
    }
    
    /// Execute all batch operations
    pub fn execute(&self) -> Result<Vec<BatchResult>> {
        let mut results = Vec::new();
        
        for operation in &self.operations {
            let result = match operation {
                BatchOperation::Save { state, file_path } => {
                    match self.serializer.serialize_to_file(state, file_path) {
                        Ok(()) => BatchResult::SaveSuccess { file_path: file_path.clone() },
                        Err(e) => BatchResult::SaveError { file_path: file_path.clone(), error: e.to_string() },
                    }
                }
                BatchOperation::Load { file_path } => {
                    match self.serializer.deserialize_from_file(file_path) {
                        Ok(state) => BatchResult::LoadSuccess { file_path: file_path.clone(), state },
                        Err(e) => BatchResult::LoadError { file_path: file_path.clone(), error: e.to_string() },
                    }
                }
                BatchOperation::Validate { file_path } => {
                    match fs::read_to_string(file_path) {
                        Ok(json_string) => {
                            match self.serializer.validate_json(&json_string) {
                                Ok(()) => BatchResult::ValidateSuccess { file_path: file_path.clone() },
                                Err(e) => BatchResult::ValidateError { file_path: file_path.clone(), error: e.to_string() },
                            }
                        }
                        Err(e) => BatchResult::ValidateError { file_path: file_path.clone(), error: e.to_string() },
                    }
                }
                BatchOperation::Migrate { file_path } => {
                    match fs::read_to_string(file_path) {
                        Ok(json_string) => {
                            match self.serializer.migrate_state(&json_string) {
                                Ok(state) => BatchResult::MigrateSuccess { file_path: file_path.clone(), state },
                                Err(e) => BatchResult::MigrateError { file_path: file_path.clone(), error: e.to_string() },
                            }
                        }
                        Err(e) => BatchResult::MigrateError { file_path: file_path.clone(), error: e.to_string() },
                    }
                }
            };
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
    }
    
    /// Get number of operations
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }
}

impl Default for BatchStateSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of batch operations
#[derive(Debug)]
pub enum BatchResult {
    SaveSuccess { file_path: std::path::PathBuf },
    SaveError { file_path: std::path::PathBuf, error: String },
    LoadSuccess { file_path: std::path::PathBuf, state: AppState },
    LoadError { file_path: std::path::PathBuf, error: String },
    ValidateSuccess { file_path: std::path::PathBuf },
    ValidateError { file_path: std::path::PathBuf, error: String },
    MigrateSuccess { file_path: std::path::PathBuf, state: AppState },
    MigrateError { file_path: std::path::PathBuf, error: String },
}

impl BatchResult {
    /// Check if result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, 
            BatchResult::SaveSuccess { .. } |
            BatchResult::LoadSuccess { .. } |
            BatchResult::ValidateSuccess { .. } |
            BatchResult::MigrateSuccess { .. }
        )
    }
    
    /// Get error message if result is an error
    pub fn get_error(&self) -> Option<&str> {
        match self {
            BatchResult::SaveError { error, .. } |
            BatchResult::LoadError { error, .. } |
            BatchResult::ValidateError { error, .. } |
            BatchResult::MigrateError { error, .. } => Some(error),
            _ => None,
        }
    }
    
    /// Get file path from result
    pub fn get_file_path(&self) -> &std::path::PathBuf {
        match self {
            BatchResult::SaveSuccess { file_path } |
            BatchResult::SaveError { file_path, .. } |
            BatchResult::LoadSuccess { file_path, .. } |
            BatchResult::LoadError { file_path, .. } |
            BatchResult::ValidateSuccess { file_path } |
            BatchResult::ValidateError { file_path, .. } |
            BatchResult::MigrateSuccess { file_path, .. } |
            BatchResult::MigrateError { file_path, .. } => file_path,
        }
    }
}

/// Utility functions for state serialization
pub mod utils {
    use super::*;
    
    /// Check if file contains valid state JSON
    pub fn is_valid_state_file(file_path: &Path) -> bool {
        let serializer = StateSerializer::new();
        
        match fs::read_to_string(file_path) {
            Ok(json_string) => {
                serializer.validate_json(&json_string).is_ok()
            }
            Err(_) => false,
        }
    }
    
    /// Get state file size in bytes
    pub fn get_state_file_size(file_path: &Path) -> Result<u64> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| StateError::StorageError(e))?;
        Ok(metadata.len())
    }
    
    /// Get state file modification time
    pub fn get_state_file_modified_time(file_path: &Path) -> Result<std::time::SystemTime> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| StateError::StorageError(e))?;
        metadata.modified()
            .map_err(|e| StateError::StorageError(e))
    }
    
    /// Create state file backup
    pub fn create_state_backup(file_path: &Path) -> Result<std::path::PathBuf> {
        let backup_path = file_path.with_extension("backup.json");
        fs::copy(file_path, &backup_path)
            .map_err(|e| StateError::StorageError(e))?;
        Ok(backup_path)
    }
    
    /// Restore state from backup
    pub fn restore_from_backup(file_path: &Path) -> Result<()> {
        let backup_path = file_path.with_extension("backup.json");
        
        if !backup_path.exists() {
            return Err(StateError::StorageError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Backup file not found"
                )
            ).into());
        }
        
        fs::copy(&backup_path, file_path)
            .map_err(|e| StateError::StorageError(e))?;
        
        info!("✅ State restored from backup: {:?}", backup_path);
        Ok(())
    }
    
    /// Clean up old backup files
    pub fn cleanup_old_backups(directory: &Path, max_backups: usize) -> Result<()> {
        let mut backup_files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("backup.json") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                backup_files.push((path, modified));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove excess backups
        if backup_files.len() > max_backups {
            for (path, _) in backup_files.iter().skip(max_backups) {
                if let Err(e) = fs::remove_file(path) {
                    warn!("Failed to remove old backup file {:?}: {}", path, e);
                }
            }
            
            let removed_count = backup_files.len() - max_backups;
            info!("🧹 Cleaned up {} old backup files", removed_count);
        }
        
        Ok(())
    }
}

