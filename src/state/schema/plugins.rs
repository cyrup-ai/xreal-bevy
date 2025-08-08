//! Plugin system state schema for XREAL application
//!
//! This module provides plugin state structures with validation and
//! serialization support for the XREAL application state system.

use super::core::StateValidation;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemState {
    /// Enabled plugins
    pub enabled_plugins: Vec<String>,
    /// Plugin configurations
    pub plugin_configs: HashMap<String, PluginConfig>,
    /// Plugin load order
    pub load_order: Vec<String>,
    /// Auto-load plugins on startup
    pub auto_load: bool,
    /// Plugin sandbox enabled
    pub sandbox_enabled: bool,
    /// Maximum plugin memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum plugin CPU usage percentage
    pub max_cpu_percent: f32,
}

impl Default for PluginSystemState {
    fn default() -> Self {
        Self {
            enabled_plugins: Vec::new(),
            plugin_configs: HashMap::new(),
            load_order: Vec::new(),
            auto_load: true,
            sandbox_enabled: true,
            max_memory_mb: 512,
            max_cpu_percent: 25.0,
        }
    }
}

impl StateValidation for PluginSystemState {
    fn validate(&self) -> Result<()> {
        // Validate memory limit
        if self.max_memory_mb < 64 || self.max_memory_mb > 8192 {
            anyhow::bail!("Max memory out of range: {}", self.max_memory_mb);
        }

        // Validate CPU limit
        if self.max_cpu_percent < 1.0 || self.max_cpu_percent > 100.0 {
            anyhow::bail!("Max CPU percent out of range: {}", self.max_cpu_percent);
        }

        // Validate plugin configs
        for (name, config) in &self.plugin_configs {
            if name.is_empty() {
                anyhow::bail!("Empty plugin name in config");
            }
            config.validate()?;
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        // Merge enabled plugins (union)
        for plugin in &other.enabled_plugins {
            if !self.enabled_plugins.contains(plugin) {
                self.enabled_plugins.push(plugin.clone());
            }
        }

        // Merge plugin configs
        for (name, config) in &other.plugin_configs {
            self.plugin_configs.insert(name.clone(), config.clone());
        }

        // Use other's load order
        self.load_order = other.load_order.clone();
        self.auto_load = other.auto_load;
        self.sandbox_enabled = other.sandbox_enabled;
        self.max_memory_mb = other.max_memory_mb;
        self.max_cpu_percent = other.max_cpu_percent;

        Ok(())
    }
}

/// Individual plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin enabled
    pub enabled: bool,
    /// Plugin priority (0 = highest)
    pub priority: u8,
    /// Plugin-specific settings
    pub settings: HashMap<String, String>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Permissions
    pub permissions: PluginPermissions,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 128,
            settings: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            permissions: PluginPermissions::default(),
        }
    }
}

impl StateValidation for PluginConfig {
    fn validate(&self) -> Result<()> {
        self.resource_limits.validate()?;
        self.permissions.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.priority = other.priority;

        // Merge settings
        for (key, value) in &other.settings {
            self.settings.insert(key.clone(), value.clone());
        }

        self.resource_limits.merge(&other.resource_limits)?;
        self.permissions.merge(&other.permissions)?;
        Ok(())
    }
}

/// Plugin resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f32,
    /// Maximum file handles
    pub max_file_handles: u32,
    /// Maximum network connections
    pub max_network_connections: u32,
    /// Execution timeout in seconds
    pub execution_timeout_secs: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 256,
            max_cpu_percent: 10.0,
            max_file_handles: 100,
            max_network_connections: 10,
            execution_timeout_secs: 30,
        }
    }
}

impl StateValidation for ResourceLimits {
    fn validate(&self) -> Result<()> {
        if self.max_memory_mb < 1 || self.max_memory_mb > 4096 {
            anyhow::bail!("Max memory out of range: {}", self.max_memory_mb);
        }

        if self.max_cpu_percent < 0.1 || self.max_cpu_percent > 100.0 {
            anyhow::bail!("Max CPU percent out of range: {}", self.max_cpu_percent);
        }

        if self.max_file_handles > 10000 {
            anyhow::bail!("Max file handles too high: {}", self.max_file_handles);
        }

        if self.max_network_connections > 1000 {
            anyhow::bail!(
                "Max network connections too high: {}",
                self.max_network_connections
            );
        }

        if self.execution_timeout_secs < 1 || self.execution_timeout_secs > 3600 {
            anyhow::bail!(
                "Execution timeout out of range: {}",
                self.execution_timeout_secs
            );
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.max_memory_mb = other.max_memory_mb;
        self.max_cpu_percent = other.max_cpu_percent;
        self.max_file_handles = other.max_file_handles;
        self.max_network_connections = other.max_network_connections;
        self.execution_timeout_secs = other.execution_timeout_secs;
        Ok(())
    }
}

/// Plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// File system access
    pub filesystem_access: bool,
    /// Network access
    pub network_access: bool,
    /// System command execution
    pub system_commands: bool,
    /// Hardware access
    pub hardware_access: bool,
    /// Inter-plugin communication
    pub plugin_communication: bool,
    /// UI modification
    pub ui_modification: bool,
    /// Data collection
    pub data_collection: bool,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            filesystem_access: false,
            network_access: false,
            system_commands: false,
            hardware_access: false,
            plugin_communication: true,
            ui_modification: true,
            data_collection: false,
        }
    }
}

impl StateValidation for PluginPermissions {
    fn validate(&self) -> Result<()> {
        // No specific validation needed for boolean permissions
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.filesystem_access = other.filesystem_access;
        self.network_access = other.network_access;
        self.system_commands = other.system_commands;
        self.hardware_access = other.hardware_access;
        self.plugin_communication = other.plugin_communication;
        self.ui_modification = other.ui_modification;
        self.data_collection = other.data_collection;
        Ok(())
    }
}
