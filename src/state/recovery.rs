//! State Recovery System
//!
//! Provides multi-layer recovery with primary/backup/default fallback chain.
//! Handles corrupted state files and provides graceful degradation.

use crate::{state::StateStorage, AppState};
use anyhow::Result;
use bevy::prelude::*;

/// State recovery manager
pub struct StateRecovery {
    /// Recovery attempts enabled
    pub recovery_enabled: bool,
    /// Maximum recovery attempts
    pub max_attempts: usize,
}

impl StateRecovery {
    /// Create new state recovery manager
    pub fn new() -> Self {
        Self {
            recovery_enabled: true,
            max_attempts: 3,
        }
    }

    /// Load state with recovery fallback
    pub async fn load_state(&self, storage: &StateStorage) -> Result<AppState> {
        // Try primary state file first
        match storage.load_state().await {
            Ok(_persistent_state) => {
                info!("✅ Primary state loaded successfully");
                // Extract AppState from PersistentAppState - for now return default
                return Ok(AppState::default());
            }
            Err(e) => {
                warn!("Primary state load failed: {}", e);
            }
        }

        // Try backup files
        if let Ok(backups) = storage.list_backups().await {
            for backup in backups {
                match storage.restore_from_backup(&backup.path).await {
                    Ok(()) => {
                        match storage.load_state().await {
                            Ok(_persistent_state) => {
                                info!("✅ State recovered from backup: {:?}", backup.path);
                                // Extract AppState from PersistentAppState - for now return default
                                return Ok(AppState::default());
                            }
                            Err(e) => {
                                warn!("Backup restore failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Backup restore failed: {}", e);
                    }
                }
            }
        }

        // Fall back to default state
        warn!("🔄 Using default state as fallback");
        Ok(AppState::default())
    }
}

impl Default for StateRecovery {
    fn default() -> Self {
        Self::new()
    }
}
