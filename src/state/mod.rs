//! State Persistence System for XREAL Virtual Desktop
//! 
//! Provides comprehensive state persistence across application restarts with:
//! - Atomic file operations to prevent corruption
//! - Multi-layer recovery system (primary/backup/default)
//! - Async I/O operations for jitter-free performance
//! - Comprehensive validation and error handling
//! - Integration with existing Bevy resources and plugin system
//!
//! This system ensures all user preferences, plugin states, calibration data,
//! and application configuration persists seamlessly across restarts.

use anyhow::Result;
use bevy::prelude::*;

pub mod schema;
pub mod serialization;
pub mod storage;
pub mod validation;
pub mod recovery;
pub mod systems;

// Re-export key types
pub use schema::*;
pub use serialization::*;
pub use storage::*;
pub use validation::*;
pub use recovery::*;
pub use systems::*;

/// Error types for state persistence operations
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Storage operation failed: {0}")]
    StorageError(#[from] std::io::Error),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Recovery failed: {0}")]
    RecoveryError(String),
    
    #[error("State schema version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },
    
    #[error("State file corrupted: {0}")]
    CorruptedState(String),
    
    #[error("Plugin state error: {0}")]
    PluginStateError(String),
}

/// Main state persistence manager
#[derive(Resource)]
pub struct StatePersistenceManager {
    /// Current application state
    pub current_state: AppState,
    /// Storage backend
    pub storage: StateStorage,
    /// Validation engine
    pub validator: StateValidator,
    /// Recovery system
    pub recovery: StateRecovery,
    /// Auto-save configuration
    pub auto_save_config: AutoSaveConfig,
    /// State change tracking
    pub change_tracker: StateChangeTracker,
}

impl StatePersistenceManager {
    /// Create new state persistence manager
    pub fn new() -> Result<Self> {
        let storage = StateStorage::new()?;
        let validator = StateValidator::new();
        let recovery = StateRecovery::new();
        let auto_save_config = AutoSaveConfig::default();
        let change_tracker = StateChangeTracker::new();
        
        // Try to load existing state, fall back to defaults
        let current_state = match recovery.load_state(&storage) {
            Ok(state) => {
                info!("✅ Application state loaded successfully");
                state
            }
            Err(e) => {
                warn!("State loading failed, using defaults: {}", e);
                AppState::default()
            }
        };
        
        Ok(Self {
            current_state,
            storage,
            validator,
            recovery,
            auto_save_config,
            change_tracker,
        })
    }
    
    /// Save current state to storage
    pub async fn save_state(&self) -> Result<()> {
        // Validate state before saving
        self.validator.validate(&self.current_state)?;
        
        // Perform atomic save
        self.storage.save_state(&self.current_state).await?;
        
        info!("✅ Application state saved successfully");
        Ok(())
    }
    
    /// Load state from storage
    pub async fn load_state(&mut self) -> Result<()> {
        self.current_state = self.recovery.load_state(&self.storage)?;
        info!("✅ Application state loaded successfully");
        Ok(())
    }
    
    /// Update state from Bevy resources
    pub fn update_from_resources(&mut self, world: &World) -> Result<()> {
        // Update state from Bevy resources
        if let Some(screen_distance) = world.get_resource::<crate::ScreenDistance>() {
            self.current_state.user_preferences.screen_distance = screen_distance.0;
        }
        
        if let Some(display_mode) = world.get_resource::<crate::DisplayModeState>() {
            self.current_state.user_preferences.display_mode_3d = display_mode.is_3d_enabled;
        }
        
        if let Some(roll_lock) = world.get_resource::<crate::RollLockState>() {
            self.current_state.user_preferences.roll_lock_enabled = roll_lock.is_enabled;
        }
        
        if let Some(brightness) = world.get_resource::<crate::BrightnessState>() {
            self.current_state.user_preferences.brightness_level = brightness.current_level;
        }
        
        if let Some(settings_panel) = world.get_resource::<crate::SettingsPanelState>() {
            self.current_state.ui_state.settings_panel_open = settings_panel.is_open;
            self.current_state.ui_state.selected_preset = settings_panel.selected_preset;
            self.current_state.ui_state.performance_monitoring = settings_panel.performance_monitoring;
            self.current_state.ui_state.advanced_calibration = settings_panel.advanced_calibration;
        }
        
        if let Some(top_menu) = world.get_resource::<crate::TopMenuState>() {
            self.current_state.ui_state.selected_tab = top_menu.selected_tab;
        }
        
        if let Some(calibration) = world.get_resource::<crate::tracking::CalibrationState>() {
            self.current_state.calibration_data = CalibrationData::from_bevy_state(calibration);
        }
        
        // Mark state as changed
        self.change_tracker.mark_changed();
        
        Ok(())
    }
    
    /// Apply state to Bevy resources
    pub fn apply_to_resources(&self, world: &mut World) -> Result<()> {
        // Apply state to Bevy resources
        if let Some(mut screen_distance) = world.get_resource_mut::<crate::ScreenDistance>() {
            screen_distance.0 = self.current_state.user_preferences.screen_distance;
        }
        
        if let Some(mut display_mode) = world.get_resource_mut::<crate::DisplayModeState>() {
            display_mode.is_3d_enabled = self.current_state.user_preferences.display_mode_3d;
        }
        
        if let Some(mut roll_lock) = world.get_resource_mut::<crate::RollLockState>() {
            roll_lock.is_enabled = self.current_state.user_preferences.roll_lock_enabled;
        }
        
        if let Some(mut brightness) = world.get_resource_mut::<crate::BrightnessState>() {
            brightness.current_level = self.current_state.user_preferences.brightness_level;
        }
        
        if let Some(mut settings_panel) = world.get_resource_mut::<crate::SettingsPanelState>() {
            settings_panel.is_open = self.current_state.ui_state.settings_panel_open;
            settings_panel.selected_preset = self.current_state.ui_state.selected_preset;
            settings_panel.performance_monitoring = self.current_state.ui_state.performance_monitoring;
            settings_panel.advanced_calibration = self.current_state.ui_state.advanced_calibration;
        }
        
        if let Some(mut top_menu) = world.get_resource_mut::<crate::TopMenuState>() {
            top_menu.selected_tab = self.current_state.ui_state.selected_tab;
        }
        
        if let Some(mut calibration) = world.get_resource_mut::<crate::tracking::CalibrationState>() {
            self.current_state.calibration_data.apply_to_bevy_state(calibration);
        }
        
        info!("✅ State applied to Bevy resources");
        Ok(())
    }
    
    /// Check if state has changed and needs saving
    pub fn needs_save(&self) -> bool {
        self.change_tracker.has_changed()
    }
    
    /// Reset change tracking
    pub fn reset_change_tracking(&mut self) {
        self.change_tracker.reset();
    }
}

/// Auto-save configuration
#[derive(Debug, Clone)]
pub struct AutoSaveConfig {
    /// Enable auto-save on state changes
    pub enabled: bool,
    /// Debounce delay in seconds
    pub debounce_delay: f32,
    /// Save on application exit
    pub save_on_exit: bool,
    /// Save interval in seconds (0 = disabled)
    pub periodic_save_interval: f32,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_delay: 2.0,
            save_on_exit: true,
            periodic_save_interval: 30.0,
        }
    }
}

/// State change tracking
#[derive(Debug, Default)]
pub struct StateChangeTracker {
    /// Has state changed since last save
    changed: bool,
    /// Last change timestamp
    last_change_time: f32,
}

impl StateChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn mark_changed(&mut self) {
        self.changed = true;
        // In a real implementation, this would use actual time
        self.last_change_time = 0.0;
    }
    
    pub fn has_changed(&self) -> bool {
        self.changed
    }
    
    pub fn reset(&mut self) {
        self.changed = false;
    }
    
    pub fn time_since_change(&self) -> f32 {
        // In a real implementation, this would calculate actual time difference
        0.0
    }
}

/// Initialize state persistence system for Bevy app
pub fn add_state_persistence(app: &mut App) -> Result<()> {
    // Initialize state persistence manager
    let state_manager = StatePersistenceManager::new()?;
    
    // Add state persistence resources
    app.insert_resource(state_manager);
    
    // Add state persistence systems
    app.add_systems(Startup, systems::state_restoration_system);
    app.add_systems(FixedUpdate, (
        systems::state_auto_save_system,
        systems::state_monitoring_system,
    ));
    
    info!("✅ State persistence system initialized");
    Ok(())
}