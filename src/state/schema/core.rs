//! Core schema definitions and versioning for application state
//!
//! This module provides the fundamental state schema structures with versioned
//! serialization support and atomic operations for the XREAL application.

use anyhow::Result;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Unused import removed

/// Schema version for state migration support
pub const STATE_SCHEMA_VERSION: &str = "1.0.0";

/// Complete application state schema for persistence
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct PersistentAppState {
    /// Schema version for migration support
    pub schema_version: String,
    /// Timestamp of last state update
    pub last_updated: u64,
    /// User preference settings
    pub user_preferences: super::preferences::UserPreferences,
    /// UI state and layout
    pub ui_state: super::ui::UiState,
    /// IMU calibration data
    pub calibration_data: super::calibration::CalibrationData,
    /// Plugin system state
    pub plugin_state: super::plugins::PluginSystemState,
    /// Performance settings and thresholds
    pub performance_settings: super::performance::PerformanceSettings,
    /// Window layout and positioning
    pub window_layout: super::window::WindowLayout,
    /// Input system configuration
    pub input_config: super::input::InputConfig,
    /// Audio system settings
    pub audio_settings: super::audio::AudioSettings,
    /// Network configuration
    pub network_config: super::network::NetworkConfig,
    /// Security settings
    pub security_settings: super::security::SecuritySettings,
}

impl Default for PersistentAppState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION.to_string(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            user_preferences: Default::default(),
            ui_state: Default::default(),
            calibration_data: Default::default(),
            plugin_state: Default::default(),
            performance_settings: Default::default(),
            window_layout: Default::default(),
            input_config: Default::default(),
            audio_settings: Default::default(),
            network_config: Default::default(),
            security_settings: Default::default(),
        }
    }
}

impl PersistentAppState {
    /// Create a new application state with current timestamp
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the last modified timestamp
    pub fn touch(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if the state schema is compatible
    pub fn is_compatible(&self) -> bool {
        self.schema_version == STATE_SCHEMA_VERSION
    }

    /// Get the age of the state in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.last_updated)
    }

    /// Validate the entire state for consistency
    pub fn validate(&self) -> Result<()> {
        // Validate schema version
        if !self.is_compatible() {
            anyhow::bail!("Incompatible schema version: {}", self.schema_version);
        }

        // Validate user preferences
        self.user_preferences.validate()?;

        // Validate UI state
        self.ui_state.validate()?;

        // Validate calibration data
        self.calibration_data.validate()?;

        // Validate plugin state
        self.plugin_state.validate()?;

        // Validate performance settings
        self.performance_settings.validate()?;

        // Validate window layout
        self.window_layout.validate()?;

        // Validate input config
        self.input_config.validate()?;

        // Validate audio settings
        self.audio_settings.validate()?;

        // Validate network config
        self.network_config.validate()?;

        // Validate security settings
        self.security_settings.validate()?;

        Ok(())
    }

    /// Reset all state to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Merge another state into this one
    pub fn merge(&mut self, other: &PersistentAppState) -> Result<()> {
        // Only merge if schema versions are compatible
        if other.schema_version != self.schema_version {
            anyhow::bail!("Cannot merge incompatible schema versions");
        }

        // Merge user preferences
        self.user_preferences.merge(&other.user_preferences)?;

        // Merge UI state
        self.ui_state.merge(&other.ui_state)?;

        // Merge calibration data (only if newer)
        if other.last_updated > self.last_updated {
            self.calibration_data = other.calibration_data.clone();
        }

        // Merge plugin state
        self.plugin_state.merge(&other.plugin_state)?;

        // Merge performance settings
        self.performance_settings.merge(&other.performance_settings)?;

        // Merge window layout
        self.window_layout.merge(&other.window_layout)?;

        // Merge input config
        self.input_config.merge(&other.input_config)?;

        // Merge audio settings
        self.audio_settings.merge(&other.audio_settings)?;

        // Merge network config
        self.network_config.merge(&other.network_config)?;

        // Merge security settings
        self.security_settings.merge(&other.security_settings)?;

        // Update timestamp
        self.touch();

        Ok(())
    }
}

/// State validation trait for all schema components
pub trait StateValidation {
    /// Validate the state component for consistency
    fn validate(&self) -> Result<()>;

    /// Merge another state component into this one
    fn merge(&mut self, other: &Self) -> Result<()>;
}

/// State migration trait for schema versioning
pub trait StateMigration {
    /// Migrate from an older version
    fn migrate_from_version(&mut self, version: &str) -> Result<()>;

    /// Get the supported migration versions
    fn supported_versions() -> Vec<&'static str>;
}

/// State serialization helper functions
pub mod serialization {
    use super::*;
    use std::path::Path;

    /// Serialize state to JSON bytes
    pub fn to_json_bytes(state: &PersistentAppState) -> Result<Vec<u8>> {
        serde_json::to_vec_pretty(state).map_err(Into::into)
    }

    /// Deserialize state from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<PersistentAppState> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }

    /// Save state to a file
    pub async fn save_to_file(state: &PersistentAppState, path: &Path) -> Result<()> {
        let bytes = to_json_bytes(state)?;
        tokio::fs::write(path, bytes).await.map_err(Into::into)
    }

    /// Load state from a file
    pub async fn load_from_file(path: &Path) -> Result<PersistentAppState> {
        let bytes = tokio::fs::read(path).await?;
        from_json_bytes(&bytes)
    }

    /// Validate and migrate state if necessary
    pub fn validate_and_migrate(mut state: PersistentAppState) -> Result<PersistentAppState> {
        // Check if migration is needed
        if state.schema_version != STATE_SCHEMA_VERSION {
            tracing::info!(
                "Migrating state from version {} to {}",
                state.schema_version,
                STATE_SCHEMA_VERSION
            );
            
            // Perform migration based on version
            match state.schema_version.as_str() {
                "0.9.0" => {
                    // Migration logic for 0.9.0 -> 1.0.0
                    state.schema_version = STATE_SCHEMA_VERSION.to_string();
                    state.touch();
                }
                _ => {
                    anyhow::bail!("Unsupported schema version: {}", state.schema_version);
                }
            }
        }

        // Validate the state
        state.validate()?;

        Ok(state)
    }
}