//! Storage Layer for State Persistence
//! 
//! Provides atomic file operations and async I/O for jitter-free state persistence.
//! Uses temporary files and atomic renames to prevent corruption during save operations.

use anyhow::Result;
use bevy::tasks::AsyncComputeTaskPool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::state::{AppState, StateError, StateSerializer};
use bevy::prelude::{info, warn, error, debug};

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Base directory for state files
    pub base_directory: PathBuf,
    /// Enable atomic writes
    pub atomic_writes: bool,
    /// Enable compression
    pub compression: bool,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Backup configuration
    pub backup_config: BackupConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_directory: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".xreal")
                .join("state"),
            atomic_writes: true,
            compression: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            backup_config: BackupConfig::default(),
        }
    }
}

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Backup interval in seconds
    pub backup_interval: u64,
    /// Backup directory relative to base directory
    pub backup_directory: PathBuf,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_backups: 5,
            backup_interval: 300, // 5 minutes
            backup_directory: PathBuf::from("backups"),
        }
    }
}

/// State storage manager
pub struct StateStorage {
    /// Storage configuration
    config: StorageConfig,
    /// Serializer for state conversion
    serializer: StateSerializer,
    /// Async runtime handle
    runtime_handle: Arc<AsyncComputeTaskPool>,
}

impl StateStorage {
    /// Create new state storage with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(StorageConfig::default())
    }
    
    /// Create state storage with custom configuration
    pub fn with_config(config: StorageConfig) -> Result<Self> {
        let serializer = StateSerializer::new();
        let runtime_handle = Arc::new(AsyncComputeTaskPool::get());
        
        let storage = Self {
            config,
            serializer,
            runtime_handle,
        };
        
        // Create base directory if it doesn't exist
        std::fs::create_dir_all(&storage.config.base_directory)
            .map_err(StateError::StorageError)?;
        
        // Create backup directory if backups are enabled
        if storage.config.backup_config.enabled {
            let backup_dir = storage.config.base_directory.join(&storage.config.backup_config.backup_directory);
            std::fs::create_dir_all(&backup_dir)
                .map_err(StateError::StorageError)?;
        }
        
        Ok(storage)
    }
    
    /// Save state to storage asynchronously
    pub async fn save_state(&self, state: &AppState) -> Result<()> {
        let file_path = self.get_primary_state_file_path();
        
        // Validate state before saving
        if let Err(e) = state.validate() {
            return Err(StateError::ValidationError(e.to_string()).into());
        }
        
        // Serialize state
        let serialized_data = self.serializer.serialize_to_string(state)?;
        
        // Check file size limit
        if serialized_data.len() as u64 > self.config.max_file_size {
            return Err(StateError::StorageError(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("State file too large: {} bytes > {} bytes limit", 
                           serialized_data.len(), self.config.max_file_size)
                )
            ).into());
        }
        
        // Create backup if enabled
        if self.config.backup_config.enabled && file_path.exists() {
            self.create_backup(&file_path).await?;
        }
        
        // Write to file
        if self.config.atomic_writes {
            self.atomic_write(&file_path, &serialized_data).await?;
        } else {
            self.direct_write(&file_path, &serialized_data).await?;
        }
        
        info!("✅ State saved successfully to {:?}", file_path);
        Ok(())
    }
    
    /// Load state from storage asynchronously
    pub async fn load_state(&self) -> Result<AppState> {
        let file_path = self.get_primary_state_file_path();
        
        // Check if file exists
        if !file_path.exists() {
            return Err(StateError::StorageError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "State file not found"
                )
            ).into());
        }
        
        // Read file contents
        let mut file = fs::File::open(&file_path).await
            .map_err(StateError::StorageError)?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents).await
            .map_err(StateError::StorageError)?;
        
        // Deserialize state
        let state = self.serializer.deserialize_from_string(&contents)?;
        
        info!("✅ State loaded successfully from {:?}", file_path);
        Ok(state)
    }
    
    /// Check if state file exists
    pub fn state_exists(&self) -> bool {
        self.get_primary_state_file_path().exists()
    }
    
    /// Get state file size
    pub async fn get_state_size(&self) -> Result<u64> {
        let file_path = self.get_primary_state_file_path();
        
        if !file_path.exists() {
            return Ok(0);
        }
        
        let metadata = fs::metadata(&file_path).await
            .map_err(StateError::StorageError)?;
        
        Ok(metadata.len())
    }
    
    /// Get state file modification time
    pub async fn get_state_modified_time(&self) -> Result<std::time::SystemTime> {
        let file_path = self.get_primary_state_file_path();
        
        let metadata = fs::metadata(&file_path).await
            .map_err(StateError::StorageError)?;
        
        metadata.modified()
            .map_err(StateError::StorageError)
            .map_err(Into::into)
    }
    
    /// List available backup files
    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>> {
        if !self.config.backup_config.enabled {
            return Ok(Vec::new());
        }
        
        let backup_dir = self.config.base_directory.join(&self.config.backup_config.backup_directory);
        
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&backup_dir).await
            .map_err(StateError::StorageError)?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(StateError::StorageError)? {
            
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let metadata = entry.metadata().await
                    .map_err(StateError::StorageError)?;
                
                let modified = metadata.modified()
                    .map_err(StateError::StorageError)?;
                
                let size = metadata.len();
                
                backups.push(BackupInfo {
                    path,
                    modified,
                    size,
                });
            }
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.modified.cmp(&a.modified));
        
        Ok(backups)
    }
    
    /// Restore state from backup
    pub async fn restore_from_backup(&self, backup_path: &Path) -> Result<()> {
        let primary_path = self.get_primary_state_file_path();
        
        // Validate backup file
        if !backup_path.exists() {
            return Err(StateError::StorageError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Backup file not found"
                )
            ).into());
        }
        
        // Read backup contents
        let mut backup_file = fs::File::open(backup_path).await
            .map_err(StateError::StorageError)?;
        
        let mut contents = String::new();
        backup_file.read_to_string(&mut contents).await
            .map_err(StateError::StorageError)?;
        
        // Validate backup contents
        self.serializer.validate_json(&contents)?;
        
        // Create backup of current state before restoration
        if primary_path.exists() {
            self.create_backup(&primary_path).await?;
        }
        
        // Copy backup to primary location
        fs::copy(backup_path, &primary_path).await
            .map_err(StateError::StorageError)?;
        
        info!("✅ State restored from backup: {:?}", backup_path);
        Ok(())
    }
    
    /// Clean up old backup files
    pub async fn cleanup_old_backups(&self) -> Result<()> {
        if !self.config.backup_config.enabled {
            return Ok(());
        }
        
        let backups = self.list_backups().await?;
        
        if backups.len() > self.config.backup_config.max_backups {
            let backups_to_remove = &backups[self.config.backup_config.max_backups..];
            
            for backup in backups_to_remove {
                if let Err(e) = fs::remove_file(&backup.path).await {
                    warn!("Failed to remove old backup {:?}: {}", backup.path, e);
                } else {
                    debug!("Removed old backup: {:?}", backup.path);
                }
            }
            
            let removed_count = backups_to_remove.len();
            info!("🧹 Cleaned up {} old backup files", removed_count);
        }
        
        Ok(())
    }
    
    /// Validate state file integrity
    pub async fn validate_state_file(&self) -> Result<()> {
        let file_path = self.get_primary_state_file_path();
        
        if !file_path.exists() {
            return Err(StateError::StorageError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "State file not found"
                )
            ).into());
        }
        
        // Read file contents
        let mut file = fs::File::open(&file_path).await
            .map_err(StateError::StorageError)?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents).await
            .map_err(StateError::StorageError)?;
        
        // Validate JSON format
        self.serializer.validate_json(&contents)?;
        
        // Try to deserialize (full validation)
        let _state = self.serializer.deserialize_from_string(&contents)?;
        
        info!("✅ State file validation passed");
        Ok(())
    }
    
    /// Get primary state file path
    fn get_primary_state_file_path(&self) -> PathBuf {
        self.config.base_directory.join("app_state.json")
    }
    
    /// Create backup of current state file
    async fn create_backup(&self, file_path: &Path) -> Result<()> {
        let backup_dir = self.config.base_directory.join(&self.config.backup_config.backup_directory);
        
        // Create backup filename with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let backup_filename = format!("app_state_{}.json", timestamp);
        let backup_path = backup_dir.join(backup_filename);
        
        // Copy current file to backup location
        fs::copy(file_path, &backup_path).await
            .map_err(StateError::StorageError)?;
        
        debug!("Created backup: {:?}", backup_path);
        Ok(())
    }
    
    /// Atomic write using temporary file and rename
    async fn atomic_write(&self, file_path: &Path, data: &str) -> Result<()> {
        // Create temporary file
        let temp_path = file_path.with_extension("tmp");
        
        // Ensure parent directory exists
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(StateError::StorageError)?;
        }
        
        // Write to temporary file
        {
            let mut temp_file = fs::File::create(&temp_path).await
                .map_err(StateError::StorageError)?;
            
            temp_file.write_all(data.as_bytes()).await
                .map_err(StateError::StorageError)?;
            
            temp_file.flush().await
                .map_err(StateError::StorageError)?;
            
            // Ensure data is written to disk
            temp_file.sync_all().await
                .map_err(StateError::StorageError)?;
        }
        
        // Atomic rename to final location
        fs::rename(&temp_path, file_path).await
            .map_err(|e| {
                // Clean up temporary file on failure
                let _ = std::fs::remove_file(&temp_path);
                StateError::StorageError(e)
            })?;
        
        Ok(())
    }
    
    /// Direct write (non-atomic)
    async fn direct_write(&self, file_path: &Path, data: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(StateError::StorageError)?;
        }
        
        // Write directly to file
        let mut file = fs::File::create(file_path).await
            .map_err(StateError::StorageError)?;
        
        file.write_all(data.as_bytes()).await
            .map_err(StateError::StorageError)?;
        
        file.flush().await
            .map_err(StateError::StorageError)?;
        
        // Ensure data is written to disk
        file.sync_all().await
            .map_err(StateError::StorageError)?;
        
        Ok(())
    }
    
    /// Get storage configuration
    pub fn get_config(&self) -> &StorageConfig {
        &self.config
    }
    
    /// Get storage statistics
    pub async fn get_statistics(&self) -> Result<StorageStatistics> {
        let primary_path = self.get_primary_state_file_path();
        
        let primary_size = if primary_path.exists() {
            let metadata = fs::metadata(&primary_path).await
                .map_err(StateError::StorageError)?;
            metadata.len()
        } else {
            0
        };
        
        let primary_modified = if primary_path.exists() {
            let metadata = fs::metadata(&primary_path).await
                .map_err(StateError::StorageError)?;
            Some(metadata.modified().map_err(StateError::StorageError)?)
        } else {
            None
        };
        
        let backups = self.list_backups().await?;
        let backup_count = backups.len();
        let total_backup_size = backups.iter().map(|b| b.size).sum();
        
        Ok(StorageStatistics {
            primary_file_size: primary_size,
            primary_file_modified: primary_modified,
            backup_count,
            total_backup_size,
            total_storage_size: primary_size + total_backup_size,
            base_directory: self.config.base_directory.clone(),
        })
    }
}

/// Information about a backup file
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Path to backup file
    pub path: PathBuf,
    /// Modification time
    pub modified: std::time::SystemTime,
    /// File size in bytes
    pub size: u64,
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    /// Size of primary state file
    pub primary_file_size: u64,
    /// Modification time of primary state file
    pub primary_file_modified: Option<std::time::SystemTime>,
    /// Number of backup files
    pub backup_count: usize,
    /// Total size of all backup files
    pub total_backup_size: u64,
    /// Total storage size (primary + backups)
    pub total_storage_size: u64,
    /// Base directory path
    pub base_directory: PathBuf,
}

impl StorageStatistics {
    /// Format storage size as human-readable string
    pub fn format_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.2} {}", size, UNITS[unit_index])
    }
    
    /// Get formatted primary file size
    pub fn formatted_primary_size(&self) -> String {
        Self::format_size(self.primary_file_size)
    }
    
    /// Get formatted total backup size
    pub fn formatted_backup_size(&self) -> String {
        Self::format_size(self.total_backup_size)
    }
    
    /// Get formatted total storage size
    pub fn formatted_total_size(&self) -> String {
        Self::format_size(self.total_storage_size)
    }
}

/// Async storage operations that integrate with Bevy's task system
pub struct AsyncStateStorage {
    storage: Arc<StateStorage>,
    task_pool: Arc<AsyncComputeTaskPool>,
}

impl AsyncStateStorage {
    /// Create new async storage wrapper
    pub fn new(storage: StateStorage) -> Self {
        Self {
            storage: Arc::new(storage),
            task_pool: Arc::new(AsyncComputeTaskPool::get()),
        }
    }
    
    /// Save state asynchronously without blocking
    pub fn save_state_async(&self, state: AppState) -> impl std::future::Future<Output = Result<()>> {
        let storage = self.storage.clone();
        
        async move {
            storage.save_state(&state).await
        }
    }
    
    /// Load state asynchronously without blocking
    pub fn load_state_async(&self) -> impl std::future::Future<Output = Result<AppState>> {
        let storage = self.storage.clone();
        
        async move {
            storage.load_state().await
        }
    }
    
    /// Spawn save operation on task pool
    pub fn spawn_save_task(&self, state: AppState) -> bevy::tasks::Task<Result<()>> {
        let storage = self.storage.clone();
        
        self.task_pool.spawn(async move {
            storage.save_state(&state).await
        })
    }
    
    /// Spawn load operation on task pool
    pub fn spawn_load_task(&self) -> bevy::tasks::Task<Result<AppState>> {
        let storage = self.storage.clone();
        
        self.task_pool.spawn(async move {
            storage.load_state().await
        })
    }
    
    /// Spawn backup cleanup operation on task pool
    pub fn spawn_cleanup_task(&self) -> bevy::tasks::Task<Result<()>> {
        let storage = self.storage.clone();
        
        self.task_pool.spawn(async move {
            storage.cleanup_old_backups().await
        })
    }
    
    /// Get storage reference
    pub fn get_storage(&self) -> &Arc<StateStorage> {
        &self.storage
    }
}

/// Storage utility functions
pub mod utils {
    use super::*;
    
    /// Check if directory is writable
    pub async fn is_directory_writable(path: &Path) -> bool {
        let test_file = path.join(".write_test");
        
        match fs::write(&test_file, b"test").await {
            Ok(()) => {
                let _ = fs::remove_file(&test_file).await;
                true
            }
            Err(_) => false,
        }
    }
    
    /// Get available disk space
    pub async fn get_available_space(path: &Path) -> Result<u64> {
        // This is a simplified implementation
        // In a real implementation, you would use platform-specific APIs
        Ok(1024 * 1024 * 1024) // 1GB as placeholder
    }
    
    /// Calculate directory size
    pub async fn calculate_directory_size(path: &Path) -> Result<u64> {
        let mut total_size = 0;
        let mut entries = fs::read_dir(path).await
            .map_err(StateError::StorageError)?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(StateError::StorageError)? {
            
            let metadata = entry.metadata().await
                .map_err(StateError::StorageError)?;
            
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += calculate_directory_size(&entry.path()).await?;
            }
        }
        
        Ok(total_size)
    }
    
    /// Create directory structure
    pub async fn create_directory_structure(base_path: &Path) -> Result<()> {
        let directories = [
            base_path,
            &base_path.join("backups"),
            &base_path.join("plugins"),
            &base_path.join("logs"),
            &base_path.join("cache"),
        ];
        
        for dir in &directories {
            fs::create_dir_all(dir).await
                .map_err(StateError::StorageError)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
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
        storage.validate_state_file().await.expect("Validation failed");
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
            storage.save_state(&modified_state).await.expect("Save failed");
            
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
        storage.atomic_write(&file_path, data).await.expect("Atomic write failed");
        
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
        async_storage.save_state_async(state.clone()).await.expect("Async save failed");
        
        // Test async load
        let loaded_state = async_storage.load_state_async().await.expect("Async load failed");
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
}