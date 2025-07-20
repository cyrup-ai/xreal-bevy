//! Plugin Lifecycle Management Systems
//!
//! Provides comprehensive lifecycle management that integrates with Bevy's system scheduling
//! and existing XREAL resource management. Coordinates with src/main.rs system ordering
//! and maintains jitter-free operation.

use super::{
    context::{PluginPerformanceTracker, PluginResourceManager},
    AtomicPluginState, FastPluginRegistry, PluginLifecycleEvent, PluginLifecycleState,
};
use crate::ui::state::JitterMetrics;
use anyhow::Result;
use bevy::prelude::*;

/// Plugin lifecycle management resource
/// Coordinates with existing FixedUpdate scheduling and XREAL resource management
#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct PluginLifecycleManager {
    /// Current lifecycle states for all plugins
    plugin_states: std::collections::HashMap<String, PluginLifecycleState>,
    /// State transition queue for coordinated updates
    transition_queue: Vec<PluginStateTransition>,
    /// Error states requiring attention
    error_states: std::collections::HashMap<String, String>,
    /// Resource cleanup coordination
    cleanup_pending: Vec<String>,
}

/// Plugin state transition for queued processing
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PluginStateTransition {
    pub plugin_id: String,
    pub from_state: PluginLifecycleState,
    pub to_state: PluginLifecycleState,
    pub timestamp: std::time::Instant,
}

impl PluginLifecycleManager {
    /// Request state transition for plugin
    pub fn request_transition(
        &mut self,
        plugin_id: String,
        to_state: PluginLifecycleState,
    ) -> Result<()> {
        let current_state = self
            .plugin_states
            .get(&plugin_id)
            .copied()
            .unwrap_or(PluginLifecycleState::Loading);

        // Validate transition is allowed
        if !self.is_valid_transition(current_state, to_state) {
            return Err(anyhow::anyhow!(
                "Invalid state transition for plugin {}: {:?} -> {:?}",
                plugin_id,
                current_state,
                to_state
            ));
        }

        let transition = PluginStateTransition {
            plugin_id: plugin_id.clone(),
            from_state: current_state,
            to_state,
            timestamp: std::time::Instant::now(),
        };

        self.transition_queue.push(transition);
        Ok(())
    }

    /// Check if state transition is valid
    fn is_valid_transition(&self, from: PluginLifecycleState, to: PluginLifecycleState) -> bool {
        use PluginLifecycleState::*;
        match (from, to) {
            (Loading, Initializing) => true,
            (Loading, Error) => true,
            (Initializing, Running) => true,
            (Initializing, Error) => true,
            (Running, Paused) => true,
            (Running, Error) => true,
            (Running, Unloading) => true,
            (Paused, Running) => true,
            (Paused, Unloading) => true,
            (Error, Unloading) => true,
            (Error, Loading) => true, // Allow retry
            _ => false,
        }
    }

    /// Get current state for plugin
    pub fn get_plugin_state(&self, plugin_id: &str) -> PluginLifecycleState {
        self.plugin_states
            .get(plugin_id)
            .copied()
            .unwrap_or(PluginLifecycleState::Loading)
    }

    /// Mark plugin for cleanup
    pub fn schedule_cleanup(&mut self, plugin_id: String) {
        if !self.cleanup_pending.contains(&plugin_id) {
            self.cleanup_pending.push(plugin_id);
        }
    }

    /// Record plugin error
    pub fn record_error(&mut self, plugin_id: String, error: String) {
        self.error_states.insert(plugin_id.clone(), error);
        let _ = self.request_transition(plugin_id, PluginLifecycleState::Error);
    }

    /// Get plugins in specific state
    pub fn get_plugins_in_state(&self, state: PluginLifecycleState) -> Vec<String> {
        self.plugin_states
            .iter()
            .filter_map(|(id, &plugin_state)| {
                if plugin_state == state {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Clear error state for plugin (for retry)
    pub fn clear_error(&mut self, plugin_id: &str) {
        self.error_states.remove(plugin_id);
    }
}

/// System to process plugin lifecycle state transitions
/// Integrates with FixedUpdate scheduling and coordinates with existing XREAL systems
pub fn plugin_lifecycle_system(
    mut lifecycle_manager: ResMut<PluginLifecycleManager>,
    plugin_registry: ResMut<FastPluginRegistry>,
    mut lifecycle_events: EventWriter<PluginLifecycleEvent>,
    mut resource_manager: ResMut<PluginResourceManager>,
    mut performance_tracker: ResMut<PluginPerformanceTracker>,
) {
    // Process pending state transitions
    let transitions = std::mem::take(&mut lifecycle_manager.transition_queue);

    for transition in transitions {
        // Apply state transition
        lifecycle_manager
            .plugin_states
            .insert(transition.plugin_id.clone(), transition.to_state);

        // Send lifecycle event
        match transition.to_state {
            PluginLifecycleState::Loading => {
                lifecycle_events.write(PluginLifecycleEvent::PluginLoaded {
                    plugin_id: transition.plugin_id.clone(),
                });
            }
            PluginLifecycleState::Initializing => {
                lifecycle_events.write(PluginLifecycleEvent::PluginInitialized {
                    plugin_id: transition.plugin_id.clone(),
                });
            }
            PluginLifecycleState::Running => {
                lifecycle_events.write(PluginLifecycleEvent::PluginStarted {
                    plugin_id: transition.plugin_id.clone(),
                });
            }
            PluginLifecycleState::Paused => {
                lifecycle_events.write(PluginLifecycleEvent::PluginStopped {
                    plugin_id: transition.plugin_id.clone(),
                });
            }
            PluginLifecycleState::Error => {
                let error_msg = lifecycle_manager
                    .error_states
                    .get(&transition.plugin_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown error".to_string());
                lifecycle_events.write(PluginLifecycleEvent::PluginError {
                    plugin_id: transition.plugin_id.clone(),
                    error: error_msg,
                });
            }
            PluginLifecycleState::Unloading => {
                lifecycle_events.write(PluginLifecycleEvent::PluginUnloaded {
                    plugin_id: transition.plugin_id.clone(),
                });
            }
        }

        debug!(
            "Plugin {} transitioned from {:?} to {:?}",
            transition.plugin_id, transition.from_state, transition.to_state
        );
    }

    // Process cleanup queue
    let cleanup_plugins = std::mem::take(&mut lifecycle_manager.cleanup_pending);
    for plugin_id in cleanup_plugins {
        // Cleanup plugin resources
        resource_manager.cleanup_plugin(&plugin_id);
        performance_tracker.cleanup_plugin(&plugin_id);

        // Remove from registry - FastPluginRegistry handles this internally
        if let Some(entry) = plugin_registry.get_plugin(&plugin_id) {
            // Update state to unloading
            if let Err(e) = entry
                .state
                .set_lifecycle_state(AtomicPluginState::STATE_UNLOADING)
            {
                warn!(
                    "Failed to set plugin {} state to unloading: {}",
                    plugin_id, e
                );
            }

            // Set state to unloaded
            if let Err(e) = entry
                .state
                .set_lifecycle_state(AtomicPluginState::STATE_UNLOADED)
            {
                warn!(
                    "Failed to set plugin {} state to unloaded: {}",
                    plugin_id, e
                );
            }
        }

        // FastPluginRegistry doesn't have unload_plugin method, so we mark it as unloaded
        // The actual cleanup will be handled by the registry's internal systems
        lifecycle_manager.plugin_states.remove(&plugin_id);
        lifecycle_manager.error_states.remove(&plugin_id);

        info!("✅ Plugin {} cleanup completed", plugin_id);
    }
}

/// System to monitor plugin health and handle errors
/// Coordinates with existing JitterMetrics system for performance monitoring
pub fn plugin_health_monitoring_system(
    mut lifecycle_manager: ResMut<PluginLifecycleManager>,
    plugin_registry: Res<FastPluginRegistry>,
    performance_tracker: Res<PluginPerformanceTracker>,
    _jitter_metrics: Res<JitterMetrics>,
) {
    // Monitor running plugins for health issues
    let running_plugins = lifecycle_manager.get_plugins_in_state(PluginLifecycleState::Running);

    for plugin_id in running_plugins {
        // Check performance violations
        if !performance_tracker.is_plugin_performing_well(&plugin_id) {
            warn!("Plugin {} performance degradation detected", plugin_id);

            // Optionally pause plugin if performance is severely impacted
            let avg_frame_time = performance_tracker.get_average_frame_time();
            if avg_frame_time > 32.0 {
                // More than 2x 60fps budget
                warn!(
                    "Pausing plugin {} due to severe performance impact",
                    plugin_id
                );
                let _ = lifecycle_manager
                    .request_transition(plugin_id.clone(), PluginLifecycleState::Paused);
            }
        }

        // Check if plugin instance exists and is responsive
        if plugin_registry.get_plugin(&plugin_id).is_none() {
            error!("Plugin {} is missing from registry", plugin_id);
            lifecycle_manager.record_error(
                plugin_id,
                "Plugin instance missing from registry".to_string(),
            );
        }
    }
}

/// System to handle plugin error recovery
/// Implements retry logic and error isolation
pub fn plugin_error_recovery_system(
    mut lifecycle_manager: ResMut<PluginLifecycleManager>,
    plugin_registry: ResMut<FastPluginRegistry>,
    _time: Res<Time>,
) {
    let error_plugins = lifecycle_manager.get_plugins_in_state(PluginLifecycleState::Error);

    for plugin_id in error_plugins {
        if let Some(instance) = plugin_registry.get_plugin(&plugin_id) {
            // Check if enough time has passed for retry (5 seconds)
            let load_time = instance.get_load_time();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let time_since_error = now.saturating_sub(load_time).as_secs();
            if time_since_error > 5 {
                info!("Attempting recovery for plugin: {}", plugin_id);

                // Clear error state and attempt reload
                lifecycle_manager.clear_error(&plugin_id);

                match lifecycle_manager
                    .request_transition(plugin_id.clone(), PluginLifecycleState::Loading)
                {
                    Ok(_) => {
                        info!("Plugin {} queued for recovery", plugin_id);
                    }
                    Err(e) => {
                        error!("Failed to queue plugin {} for recovery: {}", plugin_id, e);
                    }
                }
            }
        }
    }
}

/// System to coordinate plugin lifecycle with XREAL resource management
/// Ensures proper integration with existing resource systems
pub fn plugin_resource_coordination_system(
    lifecycle_manager: Res<PluginLifecycleManager>,
    mut resource_manager: ResMut<PluginResourceManager>,
    _plugin_registry: Res<FastPluginRegistry>,
) {
    // Coordinate resource allocation with plugin states
    let initializing_plugins =
        lifecycle_manager.get_plugins_in_state(PluginLifecycleState::Initializing);

    for plugin_id in initializing_plugins {
        // Pre-allocate base resources for initializing plugins
        if resource_manager.get_memory_usage()
            < resource_manager.resource_limits.max_total_memory_mb
        {
            if let Err(e) = resource_manager.register_plugin(16) {
                // 16MB base allocation
                warn!(
                    "Failed to allocate base memory for plugin {}: {}",
                    plugin_id, e
                );
            }
        }
    }

    // Clean up resources for unloading plugins
    let unloading_plugins = lifecycle_manager.get_plugins_in_state(PluginLifecycleState::Unloading);
    for plugin_id in unloading_plugins {
        resource_manager.cleanup_plugin(&plugin_id);
    }
}
