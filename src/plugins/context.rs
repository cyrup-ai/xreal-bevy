//! Plugin Context and Render Context Structures
//!
//! Provides context structures that give plugins safe access to WGPU resources,
//! following Bevy's render world patterns. Integrates with existing src/main.rs
//! render systems and maintains jitter-free performance.
//!
//! Reference: XREAL_GUIDE.md Resource Integration Plugin (lines 249-293)

use crate::tracking::Orientation;
use anyhow::Result;
use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};

/// Plugin context providing safe access to WGPU resources
///
/// Follows XRealStereoTextures pattern from XREAL_GUIDE.md and integrates
/// with existing JitterMetrics system.
#[derive(Resource, Clone)]
pub struct PluginContext {
    pub render_device: RenderDevice,
    pub render_queue: RenderQueue,
    pub surface_format: wgpu::TextureFormat,
    pub orientation_access: OrientationAccess,
    pub performance_budget: PerformanceBudget,
}

/// Safe access to orientation data for plugins
#[allow(dead_code)]
#[derive(Clone)]
pub struct OrientationAccess {
    pub current_quat: Quat,
    pub angular_velocity: Vec3,
    pub last_update_time: f32,
}

impl OrientationAccess {
    #[allow(dead_code)] // Plugin context infrastructure
    pub fn new(_orientation_rx: Option<()>, _calibration_rx: Option<()>) -> Self {
        // For now, return a default instance
        // In full implementation, this would manage the channels
        Self {
            current_quat: Quat::IDENTITY,
            angular_velocity: Vec3::ZERO,
            last_update_time: 0.0,
        }
    }

    #[allow(dead_code)] // Plugin context infrastructure
    pub fn from_orientation(orientation: &Orientation, time: f32) -> Self {
        Self {
            current_quat: orientation.quat,
            angular_velocity: Vec3::ZERO,
            last_update_time: time,
        }
    }
}

/// Performance budget allocation for plugins
#[allow(dead_code)]
#[derive(Clone)]
pub struct PerformanceBudget {
    pub max_frame_time_ms: f32,
    pub max_memory_usage_mb: u64,
    pub target_fps: u32,
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self {
            max_frame_time_ms: 16.0, // 60fps budget
            max_memory_usage_mb: 64,
            target_fps: 60,
        }
    }
}

/// Render context for frame-based plugin rendering
pub struct RenderContext<'a> {
    pub render_device: &'a RenderDevice,
    pub render_queue: &'a RenderQueue,
    pub command_encoder: &'a mut wgpu::CommandEncoder,
    pub surface_texture: &'a wgpu::SurfaceTexture,
    pub surface_format: wgpu::TextureFormat,
    pub delta_time: f32,
    pub frame_count: u64,
    pub orientation: OrientationAccess,
    pub performance_metrics: &'a mut PluginPerformanceMetrics,
    /// Current frame time budget in milliseconds
    pub frame_budget_ms: f32,
    /// Amount of budget consumed so far this frame
    pub budget_consumed_ms: f32,
}

impl<'a> RenderContext<'a> {
    /// Check if there's sufficient frame budget remaining
    ///
    /// Returns true if the plugin can proceed with rendering based on available
    /// frame time budget. This helps maintain consistent frame rates.
    ///
    /// # Returns
    /// * `bool` - True if budget is available for rendering
    #[inline]
    #[allow(dead_code)] // Used by plugin examples through trait objects
    pub fn has_frame_budget(&self) -> bool {
        self.budget_consumed_ms < self.frame_budget_ms
    }

    /// Get remaining frame budget in milliseconds
    ///
    /// # Returns
    /// * `f32` - Remaining budget in milliseconds
    #[inline]
    #[allow(dead_code)] // Used by plugin examples through trait objects
    pub fn remaining_budget_ms(&self) -> f32 {
        (self.frame_budget_ms - self.budget_consumed_ms).max(0.0)
    }

    /// Get budget utilization percentage
    ///
    /// # Returns
    /// * `f32` - Budget used as percentage (0.0 to 1.0+)
    #[inline]
    #[allow(dead_code)] // Used by plugin examples through trait objects
    pub fn budget_utilization(&self) -> f32 {
        if self.frame_budget_ms > 0.0 {
            self.budget_consumed_ms / self.frame_budget_ms
        } else {
            0.0
        }
    }

    /// Consume frame budget time
    ///
    /// Records time spent on rendering operations to track budget consumption.
    /// Should be called after completing rendering work.
    ///
    /// # Arguments
    /// * `time_ms` - Time consumed in milliseconds
    #[inline]
    #[allow(dead_code)] // Used by plugin examples through trait objects
    pub fn consume_budget(&mut self, time_ms: f32) {
        self.budget_consumed_ms += time_ms;

        // Record performance metrics
        self.performance_metrics.frame_render_time = time_ms;

        // Warn if budget exceeded
        if self.budget_consumed_ms > self.frame_budget_ms {
            let overage = self.budget_consumed_ms - self.frame_budget_ms;
            warn!(
                "Plugin frame budget exceeded by {:.2}ms ({:.1}%)",
                overage,
                (overage / self.frame_budget_ms) * 100.0
            );
        }
    }

    /// Reset budget consumption for new frame
    ///
    /// Should be called at the start of each frame cycle.
    #[inline]
    #[allow(dead_code)] // Used by plugin examples through trait objects
    pub fn reset_budget(&mut self) {
        self.budget_consumed_ms = 0.0;
    }

    /// Check if plugin is within performance thresholds
    ///
    /// # Returns
    /// * `bool` - True if performance is acceptable
    #[inline]
    #[allow(dead_code)] // Used by plugin examples through trait objects
    pub fn is_performing_well(&self) -> bool {
        self.budget_utilization() <= 1.2 // Allow 20% overage tolerance
    }
}

/// Plugin performance metrics for monitoring
#[derive(Default)]
pub struct PluginPerformanceMetrics {
    pub frame_render_time: f32,
    pub memory_usage: u64,
    pub gpu_memory_usage: u64,
    pub last_error: Option<String>,
}

/// Resource for plugin resource management
#[derive(Resource)]
pub struct PluginResourceManager {
    pub total_memory_usage: u64,
    pub total_gpu_memory_usage: u64,
    pub active_plugins: u32,
    pub resource_limits: ResourceLimits,
}

/// Resource limits configuration
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_total_memory_mb: u64,
    pub max_plugin_memory_mb: u64,
    pub max_texture_size: u32,
    pub max_buffer_size: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_total_memory_mb: 1024, // 1GB total
            max_plugin_memory_mb: 64,  // 64MB per plugin
            max_texture_size: 4096,
            max_buffer_size: 64 * 1024 * 1024, // 64MB
        }
    }
}

impl PluginResourceManager {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            total_memory_usage: 0,
            total_gpu_memory_usage: 0,
            active_plugins: 0,
            resource_limits: limits,
        }
    }

    /// Check if plugin resource allocation is within limits
    pub fn can_allocate(&self, memory_mb: u64) -> bool {
        self.total_memory_usage + memory_mb <= self.resource_limits.max_total_memory_mb
            && memory_mb <= self.resource_limits.max_plugin_memory_mb
    }

    /// Register plugin resource usage
    pub fn register_plugin(&mut self, memory_mb: u64) -> Result<()> {
        if !self.can_allocate(memory_mb) {
            return Err(anyhow::anyhow!("Plugin memory allocation exceeds limits"));
        }

        self.total_memory_usage += memory_mb;
        self.active_plugins += 1;
        Ok(())
    }

    /// Unregister plugin and free resources
    pub fn unregister_plugin(&mut self, memory_mb: u64) {
        self.total_memory_usage = self.total_memory_usage.saturating_sub(memory_mb);
        self.active_plugins = self.active_plugins.saturating_sub(1);
    }

    /// Cleanup plugin resources
    pub fn cleanup_plugin(&mut self, plugin_id: &str) {
        // In full implementation, this would track per-plugin resource usage
        debug!("Cleaning up resources for plugin: {}", plugin_id);
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> u64 {
        self.total_memory_usage
    }
}

/// Plugin performance tracker
#[derive(Resource)]
pub struct PluginPerformanceTracker {
    pub frame_times: Vec<f32>,
    pub memory_snapshots: Vec<u64>,
    pub thresholds: PerformanceThresholds,
    pub violations: u32,
}

/// Performance thresholds for monitoring
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_frame_time_ms: f32,
    pub max_memory_growth_mb: u64,
    pub jitter_threshold_ms: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_frame_time_ms: 16.0, // 60fps
            max_memory_growth_mb: 100,
            jitter_threshold_ms: 1.0, // Match XREAL jitter requirements
        }
    }
}

impl PluginPerformanceTracker {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            frame_times: Vec::with_capacity(1000), // Match JitterMetrics buffer size
            memory_snapshots: Vec::with_capacity(100),
            thresholds,
            violations: 0,
        }
    }

    /// Record frame timing for jitter analysis
    pub fn record_frame_time(&mut self, time_ms: f32) {
        self.frame_times.push(time_ms);

        // Keep buffer size manageable
        if self.frame_times.len() > 1000 {
            self.frame_times.remove(0);
        }

        // Check for violations
        if time_ms > self.thresholds.max_frame_time_ms {
            self.violations += 1;
            warn!(
                "Plugin frame time violation: {:.2}ms > {:.2}ms threshold",
                time_ms, self.thresholds.max_frame_time_ms
            );
        }
    }

    /// Calculate current jitter level
    pub fn calculate_jitter(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let mean = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        let variance = self
            .frame_times
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>()
            / self.frame_times.len() as f32;

        variance.sqrt()
    }

    /// Cleanup plugin performance tracking
    pub fn cleanup_plugin(&mut self, plugin_id: &str) {
        // In full implementation, this would clean up per-plugin metrics
        debug!("Cleaning up performance tracking for plugin: {}", plugin_id);
    }

    /// Check if plugin is performing well
    pub fn is_plugin_performing_well(&self, _plugin_id: &str) -> bool {
        // In full implementation, this would check per-plugin metrics
        let current_jitter = self.calculate_jitter();
        current_jitter < self.thresholds.jitter_threshold_ms
    }

    /// Get average frame time across all plugins
    pub fn get_average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Record frame time for specific plugin
    pub fn record_frame_time_for_plugin(&mut self, _plugin_id: String, time_ms: f32) {
        // For now, record globally - in full implementation, track per-plugin
        self.record_frame_time(time_ms);
    }
}

/// System to update plugin contexts with current XREAL state
pub fn update_plugin_contexts_system(_orientation: Res<Orientation>, _time: Res<Time>) {
    // Update plugin contexts with latest XREAL data
    // Implementation will be added as plugin system develops
}

/// System to monitor plugin resource usage
pub fn plugin_resource_monitoring_system(
    _resource_manager: ResMut<PluginResourceManager>,
    _performance_tracker: ResMut<PluginPerformanceTracker>,
    time: Res<Time>,
) {
    let frame_time_ms = time.delta_secs() * 1000.0;

    // Monitor plugin performance integration with existing jitter measurement
    if frame_time_ms > 16.0 {
        debug!("Frame time above 60fps budget: {:.2}ms", frame_time_ms);
    }
}
