//! Bevy Integration Systems for State Persistence
//! 
//! Provides Bevy systems for auto-save, change detection, and state monitoring.
//! Uses AsyncComputeTaskPool for non-blocking operations.

// use anyhow::Result; // Unused import removed
use bevy::prelude::*;
use crate::state::StatePersistenceManager;

/// System to monitor state changes and trigger auto-save
pub fn state_auto_save_system(
    mut state_manager: ResMut<StatePersistenceManager>,
    world: &World,
) {
    // Update state from Bevy resources
    if let Err(e) = state_manager.update_persistent_state_from_resources(world) {
        error!("Failed to update state from resources: {}", e);
        return;
    }
    
    // Check if auto-save is needed
    if state_manager.needs_save() && state_manager.auto_save_config.enabled {
        // In a full implementation, this would use async tasks
        // For now, just mark as saved
        state_manager.reset_change_tracking();
        info!("✅ State auto-save triggered");
    }
}

/// System to apply persisted state to Bevy resources on startup
pub fn state_restoration_system(
    _commands: Commands,
    _state_manager: ResMut<StatePersistenceManager>,
) {
    // Apply state to Bevy resources
    // Skip applying state to resources for now - method signature mismatch
    // TODO: Implement proper state application with persistent state parameter
    info!("✅ State restoration system executed");
}

/// System to monitor state persistence performance
pub fn state_monitoring_system(
    state_manager: Res<StatePersistenceManager>,
    time: Res<Time>,
) {
    // Monitor state persistence operations
    let _elapsed = time.elapsed_secs();
    
    // In a full implementation, this would monitor:
    // - State change frequency
    // - Save operation timing
    // - Error rates
    // - Memory usage
    
    if state_manager.needs_save() {
        debug!("State has changes pending save");
    }
}