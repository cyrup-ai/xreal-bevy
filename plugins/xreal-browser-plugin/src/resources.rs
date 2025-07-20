//! Browser plugin resources for Bevy ECS
//!
//! This module defines all resources used by the browser plugin for global state management,
//! configuration, and shared data within the Bevy ECS architecture.

use bevy::prelude::*;
use bevy::render::render_resource::{BindGroupLayout, RenderPipeline};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::error::{BrowserError, BrowserResult};

/// Global browser plugin configuration resource
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Default URL to load when creating new browser instances
    pub default_url: String,
    /// Maximum cache size in megabytes
    pub cache_size_mb: u64,
    /// User agent string to use for requests
    pub user_agent: String,
    /// Whether JavaScript is enabled
    pub javascript_enabled: bool,
    /// Whether images are loaded automatically
    pub images_enabled: bool,
    /// Whether plugins (Flash, etc.) are enabled
    pub plugins_enabled: bool,
    /// Maximum number of concurrent browser instances
    pub max_instances: usize,
    /// Default viewport size for new browser instances
    pub default_viewport_size: (u32, u32),
    /// Whether to enable developer tools
    pub dev_tools_enabled: bool,
}

impl BrowserConfig {
    /// Create a new browser configuration with sensible defaults
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            default_url: "https://www.google.com".to_string(),
            cache_size_mb: 128,
            user_agent: "XREAL Browser/1.0 (AR Glasses; Bevy Engine)".to_string(),
            javascript_enabled: true,
            images_enabled: true,
            plugins_enabled: false, // Disabled for security
            max_instances: 4, // Reasonable limit for AR environment
            default_viewport_size: (1920, 1080), // Standard AR glasses resolution
            dev_tools_enabled: false, // Disabled by default for performance
        }
    }

    /// Create configuration for development with dev tools enabled
    #[inline(always)]
    pub fn development() -> Self {
        let mut config = Self::new();
        config.dev_tools_enabled = true;
        config.default_url = "about:blank".to_string();
        config
    }

    /// Create configuration optimized for performance
    #[inline(always)]
    pub fn performance_optimized() -> Self {
        let mut config = Self::new();
        config.cache_size_mb = 64; // Smaller cache
        config.images_enabled = true; // Keep images for visual content
        config.javascript_enabled = true; // Keep JS for functionality
        config.plugins_enabled = false; // Disable plugins for security/performance
        config.dev_tools_enabled = false; // Disable dev tools
        config
    }

    /// Validate configuration settings
    #[inline(always)]
    pub fn validate(&self) -> BrowserResult<()> {
        if self.cache_size_mb == 0 {
            return Err(BrowserError::ConfigError("Cache size cannot be zero".to_string()));
        }
        if self.max_instances == 0 {
            return Err(BrowserError::ConfigError("Max instances cannot be zero".to_string()));
        }
        if self.default_viewport_size.0 == 0 || self.default_viewport_size.1 == 0 {
            return Err(BrowserError::ConfigError("Viewport size cannot be zero".to_string()));
        }
        Ok(())
    }
}

impl Default for BrowserConfig {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Global browser state resource
#[derive(Resource, Debug)]
pub struct BrowserState {
    /// Number of active browser instances
    pub active_instances: usize,
    /// Total memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Global navigation history (shared across instances)
    pub global_history: NavigationHistory,
    /// Shared render resources
    pub render_resources: BrowserRenderResources,
    /// Performance metrics
    pub performance_metrics: BrowserPerformanceMetrics,
    /// Whether the browser system is initialized
    pub is_initialized: bool,
}

impl BrowserState {
    /// Create a new browser state with default values
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            active_instances: 0,
            memory_usage_bytes: 0,
            global_history: NavigationHistory::new(),
            render_resources: BrowserRenderResources::new(),
            performance_metrics: BrowserPerformanceMetrics::new(),
            is_initialized: false,
        }
    }

    /// Register a new browser instance
    #[inline(always)]
    pub fn register_instance(&mut self) {
        self.active_instances = self.active_instances.saturating_add(1);
    }

    /// Unregister a browser instance
    #[inline(always)]
    pub fn unregister_instance(&mut self) {
        self.active_instances = self.active_instances.saturating_sub(1);
    }

    /// Update memory usage
    #[inline(always)]
    pub fn update_memory_usage(&mut self, bytes: u64) {
        self.memory_usage_bytes = bytes;
    }

    /// Mark as initialized
    #[inline(always)]
    pub fn set_initialized(&mut self, initialized: bool) {
        self.is_initialized = initialized;
    }

    /// Get memory usage in megabytes
    #[inline(always)]
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

impl Default for BrowserState {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation history management
#[derive(Debug, Clone)]
pub struct NavigationHistory {
    /// History entries (URL, title, timestamp)
    entries: VecDeque<NavigationEntry>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Current position in history
    current_position: usize,
}

/// Single navigation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationEntry {
    /// URL of the page
    pub url: String,
    /// Page title
    pub title: String,
    /// Timestamp when visited
    pub timestamp: f64,
    /// Favicon URL if available
    pub favicon_url: Option<String>,
}

impl NavigationEntry {
    /// Create a new navigation entry
    #[inline(always)]
    pub fn new(url: String, title: String, timestamp: f64) -> Self {
        Self {
            url,
            title,
            timestamp,
            favicon_url: None,
        }
    }

    /// Create a new navigation entry with favicon
    #[inline(always)]
    pub fn with_favicon(url: String, title: String, timestamp: f64, favicon_url: String) -> Self {
        Self {
            url,
            title,
            timestamp,
            favicon_url: Some(favicon_url),
        }
    }
}

impl NavigationHistory {
    /// Create a new navigation history
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 100, // Reasonable default
            current_position: 0,
        }
    }

    /// Add a new entry to history
    #[inline(always)]
    pub fn add_entry(&mut self, entry: NavigationEntry) {
        // Remove entries after current position (for new navigation)
        while self.entries.len() > self.current_position {
            self.entries.pop_back();
        }

        // Add new entry
        self.entries.push_back(entry);
        self.current_position = self.entries.len();

        // Trim to max entries
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
            if self.current_position > 0 {
                self.current_position -= 1;
            }
        }
    }

    /// Navigate back in history
    #[inline(always)]
    pub fn go_back(&mut self) -> Option<&NavigationEntry> {
        if self.can_go_back() {
            self.current_position -= 1;
            self.entries.get(self.current_position - 1)
        } else {
            None
        }
    }

    /// Navigate forward in history
    #[inline(always)]
    pub fn go_forward(&mut self) -> Option<&NavigationEntry> {
        if self.can_go_forward() {
            self.current_position += 1;
            self.entries.get(self.current_position - 1)
        } else {
            None
        }
    }

    /// Check if can navigate back
    #[inline(always)]
    pub fn can_go_back(&self) -> bool {
        self.current_position > 1
    }

    /// Check if can navigate forward
    #[inline(always)]
    pub fn can_go_forward(&self) -> bool {
        self.current_position < self.entries.len()
    }

    /// Get current entry
    #[inline(always)]
    pub fn current_entry(&self) -> Option<&NavigationEntry> {
        if self.current_position > 0 {
            self.entries.get(self.current_position - 1)
        } else {
            None
        }
    }

    /// Get recent entries (last N entries)
    #[inline(always)]
    pub fn recent_entries(&self, count: usize) -> Vec<&NavigationEntry> {
        self.entries
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Clear all history
    #[inline(always)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_position = 0;
    }
}

impl Default for NavigationHistory {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Shared render resources for browser instances
#[derive(Debug)]
pub struct BrowserRenderResources {
    /// Shared render pipeline
    pub render_pipeline: Option<RenderPipeline>,
    /// Shared bind group layout
    pub bind_group_layout: Option<BindGroupLayout>,
    /// Whether resources are initialized
    pub initialized: bool,
}

impl BrowserRenderResources {
    /// Create new render resources
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            render_pipeline: None,
            bind_group_layout: None,
            initialized: false,
        }
    }

    /// Mark resources as initialized
    #[inline(always)]
    pub fn set_initialized(&mut self, initialized: bool) {
        self.initialized = initialized;
    }
}

impl Default for BrowserRenderResources {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for browser plugin
#[derive(Debug, Clone)]
pub struct BrowserPerformanceMetrics {
    /// Total number of pages loaded
    pub pages_loaded: u64,
    /// Average page load time in milliseconds
    pub average_load_time_ms: f32,
    /// Total rendering time in milliseconds
    pub total_render_time_ms: f64,
    /// Number of render frames
    pub render_frames: u64,
    /// Last frame render time in milliseconds
    pub last_frame_time_ms: f32,
    /// Memory usage peak in bytes
    pub memory_peak_bytes: u64,
}

impl BrowserPerformanceMetrics {
    /// Create new performance metrics
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            pages_loaded: 0,
            average_load_time_ms: 0.0,
            total_render_time_ms: 0.0,
            render_frames: 0,
            last_frame_time_ms: 0.0,
            memory_peak_bytes: 0,
        }
    }

    /// Record a page load
    #[inline(always)]
    pub fn record_page_load(&mut self, load_time_ms: f32) {
        self.pages_loaded = self.pages_loaded.saturating_add(1);
        // Update running average
        let total_time = self.average_load_time_ms * (self.pages_loaded - 1) as f32 + load_time_ms;
        self.average_load_time_ms = total_time / self.pages_loaded as f32;
    }

    /// Record a render frame
    #[inline(always)]
    pub fn record_render_frame(&mut self, frame_time_ms: f32) {
        self.render_frames = self.render_frames.saturating_add(1);
        self.total_render_time_ms += frame_time_ms as f64;
        self.last_frame_time_ms = frame_time_ms;
    }

    /// Update memory usage peak
    #[inline(always)]
    pub fn update_memory_peak(&mut self, bytes: u64) {
        if bytes > self.memory_peak_bytes {
            self.memory_peak_bytes = bytes;
        }
    }

    /// Get average frame time in milliseconds
    #[inline(always)]
    pub fn average_frame_time_ms(&self) -> f32 {
        if self.render_frames > 0 {
            (self.total_render_time_ms / self.render_frames as f64) as f32
        } else {
            0.0
        }
    }

    /// Get frames per second
    #[inline(always)]
    pub fn fps(&self) -> f32 {
        let avg_frame_time = self.average_frame_time_ms();
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }
}

impl Default for BrowserPerformanceMetrics {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}