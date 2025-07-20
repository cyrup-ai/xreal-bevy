//! Terminal plugin implementation using Bevy's Plugin trait
//!
//! This module implements the main TerminalPlugin struct that integrates with Bevy's
//! plugin system for seamless terminal functionality in XREAL AR applications.

use bevy::prelude::*;
use tracing::{info, error};
use crate::{
    components::*,
    resources::*,
    systems::*,
    color_scheme::ColorSchemeVariant,
    capabilities::TerminalCapabilities,
    error::{TerminalError, TerminalResult},
};

/// Main terminal plugin for Bevy integration
#[derive(Debug)]
pub struct TerminalPlugin {
    /// Terminal configuration
    config: TerminalConfig,
    /// Plugin capabilities
    capabilities: TerminalCapabilities,
    /// Whether to auto-start terminals
    auto_start: bool,
    /// Default terminal instances to create
    default_terminals: Vec<TerminalInstanceConfig>,
}

/// Configuration for a terminal instance
#[derive(Debug, Clone)]
pub struct TerminalInstanceConfig {
    /// Unique identifier for the terminal
    pub id: String,
    /// Shell command to execute
    pub shell_command: String,
    /// Grid size (columns, rows)
    pub grid_size: (usize, usize),
    /// Font size
    pub font_size: f32,
    /// Whether to start automatically
    pub auto_start: bool,
}

impl TerminalInstanceConfig {
    /// Create a new terminal instance configuration
    #[inline(always)]
    pub fn new(id: String, shell_command: String) -> Self {
        Self {
            id,
            shell_command,
            grid_size: (80, 24),
            font_size: 14.0,
            auto_start: true,
        }
    }

    /// Set grid size
    #[inline(always)]
    pub fn with_grid_size(mut self, cols: usize, rows: usize) -> Self {
        self.grid_size = (cols, rows);
        self
    }

    /// Set font size
    #[inline(always)]
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set auto-start behavior
    #[inline(always)]
    pub fn with_auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start = auto_start;
        self
    }
}

impl Default for TerminalInstanceConfig {
    #[inline(always)]
    fn default() -> Self {
        Self::new("default".to_string(), "/bin/zsh".to_string())
    }
}

impl TerminalPlugin {
    /// Create a new terminal plugin with default configuration
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            config: TerminalConfig::new(),
            capabilities: TerminalCapabilities::default_capabilities(),
            auto_start: true,
            default_terminals: vec![TerminalInstanceConfig::default()],
        }
    }

    /// Create terminal plugin with custom configuration
    #[inline(always)]
    pub fn with_config(config: TerminalConfig) -> Self {
        Self {
            config,
            capabilities: TerminalCapabilities::default_capabilities(),
            auto_start: true,
            default_terminals: vec![TerminalInstanceConfig::default()],
        }
    }

    /// Create terminal plugin optimized for development
    #[inline(always)]
    pub fn development() -> Self {
        let config = TerminalConfig::development();
        let mut plugin = Self::with_config(config);
        
        // Add multiple terminal instances for development
        plugin.default_terminals = vec![
            TerminalInstanceConfig::new("main".to_string(), "/bin/bash".to_string())
                .with_grid_size(120, 30)
                .with_font_size(12.0),
            TerminalInstanceConfig::new("logs".to_string(), "tail -f /var/log/system.log".to_string())
                .with_grid_size(100, 20)
                .with_font_size(10.0),
        ];
        
        plugin
    }

    /// Create terminal plugin optimized for performance
    #[inline(always)]
    pub fn performance_optimized() -> Self {
        let config = TerminalConfig::performance_optimized();
        let mut plugin = Self::with_config(config);
        
        // Minimal capabilities for performance
        plugin.capabilities = TerminalCapabilities::PTY_SUPPORT
            .with_flag(TerminalCapabilities::INPUT_HANDLING);
        
        plugin
    }

    /// Set color scheme variant
    #[inline(always)]
    pub fn with_color_scheme(mut self, variant: ColorSchemeVariant) -> Self {
        self.config = self.config.with_color_scheme(variant);
        self
    }

    /// Set default shell
    #[inline(always)]
    pub fn with_shell(mut self, shell: String) -> Self {
        self.config = self.config.with_shell(shell);
        self
    }

    /// Set font size
    #[inline(always)]
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.config = self.config.with_font_size(size);
        self
    }

    /// Set grid size
    #[inline(always)]
    pub fn with_grid_size(mut self, cols: usize, rows: usize) -> Self {
        self.config = self.config.with_grid_size(cols, rows);
        self
    }

    /// Add terminal instance configuration
    #[inline(always)]
    pub fn add_terminal(mut self, terminal_config: TerminalInstanceConfig) -> Self {
        self.default_terminals.push(terminal_config);
        self
    }

    /// Set terminal instances
    #[inline(always)]
    pub fn with_terminals(mut self, terminals: Vec<TerminalInstanceConfig>) -> Self {
        self.default_terminals = terminals;
        self
    }

    /// Set capabilities
    #[inline(always)]
    pub fn with_capabilities(mut self, capabilities: TerminalCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set auto-start behavior
    #[inline(always)]
    pub fn with_auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start = auto_start;
        self
    }

    /// Get plugin capabilities
    #[inline(always)]
    pub fn capabilities(&self) -> TerminalCapabilities {
        self.capabilities
    }

    /// Get plugin configuration
    #[inline(always)]
    pub fn config(&self) -> &TerminalConfig {
        &self.config
    }

    /// Validate plugin configuration
    #[inline(always)]
    pub fn validate(&self) -> TerminalResult<()> {
        // Validate configuration
        self.config.validate()?;
        
        // Validate terminal instances
        if self.default_terminals.is_empty() {
            return Err(TerminalError::ConfigError("No terminal instances configured".to_string()));
        }
        
        // Check for duplicate terminal IDs
        let mut ids = std::collections::HashSet::new();
        for terminal in &self.default_terminals {
            if !ids.insert(&terminal.id) {
                return Err(TerminalError::ConfigError(
                    format!("Duplicate terminal ID: {}", terminal.id)
                ));
            }
        }
        
        Ok(())
    }

    /// Create default terminal instances
    fn create_default_terminals(&self, commands: &mut Commands) -> TerminalResult<()> {
        for terminal_config in &self.default_terminals {
            let bundle = TerminalBundle::new(
                terminal_config.id.clone(),
                terminal_config.shell_command.clone(),
            );
            
            let mut entity_commands = commands.spawn(bundle);
            
            // Configure the terminal entity
            entity_commands.insert(Name::new(format!("Terminal-{}", terminal_config.id)));
            
            if terminal_config.auto_start {
                // Mark as active and running
                entity_commands.insert(TerminalEntity {
                    id: terminal_config.id.clone(),
                    shell_command: terminal_config.shell_command.clone(),
                    is_active: true,
                    grid_size: terminal_config.grid_size,
                    font_size: terminal_config.font_size,
                    is_running: true,
                });
            }
            
            info!("Created terminal instance: {}", terminal_config.id);
        }
        
        Ok(())
    }
}

impl Default for TerminalPlugin {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        info!("Building terminal plugin");
        
        // Validate configuration before building
        if let Err(e) = self.validate() {
            error!("Terminal plugin validation failed: {}", e);
            return;
        }
        
        // Insert resources
        app.insert_resource(self.config.clone())
           .insert_resource(TerminalState::new());
        
        // Register component types
        app.register_type::<TerminalEntity>()
           .register_type::<TerminalInput>()
           .register_type::<TerminalCursor>()
           .register_type::<TerminalScrollback>();
        
        // Add systems to appropriate schedules
        app.add_systems(Startup, initialize_terminal_system);
        
        app.add_systems(Update, (
            update_terminal_system,
            process_terminal_input_system,
            handle_terminal_scroll_system,
            process_terminal_commands_system,
            update_terminal_performance_system,
            manage_terminal_lifecycle_system,
        ).chain()); // Chain systems to ensure proper execution order
        
        app.add_systems(PostUpdate, render_terminal_system);
        
        // Add cleanup system
        app.add_systems(Last, cleanup_terminal_system);
        
        info!("Terminal plugin systems registered");
    }

    fn finish(&self, app: &mut App) {
        info!("Finishing terminal plugin setup");
        
        // Create default terminal instances if auto-start is enabled
        if self.auto_start {
            let world = app.world_mut();
            let mut commands = world.commands();
            
            if let Err(e) = self.create_default_terminals(&mut commands) {
                error!("Failed to create default terminals: {}", e);
            } else {
                info!("Default terminal instances created successfully");
            }
        }
        
        info!("Terminal plugin setup completed");
    }

    fn cleanup(&self, app: &mut App) {
        info!("Cleaning up terminal plugin");
        
        // Remove resources
        app.world_mut().remove_resource::<TerminalConfig>();
        app.world_mut().remove_resource::<TerminalState>();
        
        info!("Terminal plugin cleanup completed");
    }
}

/// Builder for creating terminal plugin with fluent API
pub struct TerminalPluginBuilder {
    plugin: TerminalPlugin,
}

impl TerminalPluginBuilder {
    /// Create a new terminal plugin builder
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            plugin: TerminalPlugin::new(),
        }
    }

    /// Set color scheme
    #[inline(always)]
    pub fn color_scheme(mut self, variant: ColorSchemeVariant) -> Self {
        self.plugin = self.plugin.with_color_scheme(variant);
        self
    }

    /// Set shell
    #[inline(always)]
    pub fn shell(mut self, shell: String) -> Self {
        self.plugin = self.plugin.with_shell(shell);
        self
    }

    /// Set font size
    #[inline(always)]
    pub fn font_size(mut self, size: f32) -> Self {
        self.plugin = self.plugin.with_font_size(size);
        self
    }

    /// Set grid size
    #[inline(always)]
    pub fn grid_size(mut self, cols: usize, rows: usize) -> Self {
        self.plugin = self.plugin.with_grid_size(cols, rows);
        self
    }

    /// Add terminal instance
    #[inline(always)]
    pub fn add_terminal(mut self, id: String, shell_command: String) -> Self {
        let terminal_config = TerminalInstanceConfig::new(id, shell_command);
        self.plugin = self.plugin.add_terminal(terminal_config);
        self
    }

    /// Add terminal instance with configuration
    #[inline(always)]
    pub fn add_terminal_with_config(mut self, terminal_config: TerminalInstanceConfig) -> Self {
        self.plugin = self.plugin.add_terminal(terminal_config);
        self
    }

    /// Set capabilities
    #[inline(always)]
    pub fn capabilities(mut self, capabilities: TerminalCapabilities) -> Self {
        self.plugin = self.plugin.with_capabilities(capabilities);
        self
    }

    /// Set auto-start
    #[inline(always)]
    pub fn auto_start(mut self, auto_start: bool) -> Self {
        self.plugin = self.plugin.with_auto_start(auto_start);
        self
    }

    /// Build the terminal plugin
    #[inline(always)]
    pub fn build(self) -> TerminalPlugin {
        self.plugin
    }
}

impl Default for TerminalPluginBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common terminal plugin configurations

/// Create a simple terminal plugin with default settings
#[inline(always)]
pub fn simple_terminal() -> TerminalPlugin {
    TerminalPlugin::new()
}

/// Create a terminal plugin for development work
#[inline(always)]
pub fn development_terminal() -> TerminalPlugin {
    TerminalPlugin::development()
}

/// Create a terminal plugin optimized for performance
#[inline(always)]
pub fn performance_terminal() -> TerminalPlugin {
    TerminalPlugin::performance_optimized()
}

/// Create a terminal plugin with dark theme
#[inline(always)]
pub fn dark_terminal() -> TerminalPlugin {
    TerminalPlugin::new().with_color_scheme(ColorSchemeVariant::Dark)
}

/// Create a terminal plugin with light theme
#[inline(always)]
pub fn light_terminal() -> TerminalPlugin {
    TerminalPlugin::new().with_color_scheme(ColorSchemeVariant::Light)
}

/// Create a terminal plugin with high contrast theme
#[inline(always)]
pub fn high_contrast_terminal() -> TerminalPlugin {
    TerminalPlugin::new().with_color_scheme(ColorSchemeVariant::HighContrast)
}

/// Create a retro green terminal plugin
#[inline(always)]
pub fn retro_terminal() -> TerminalPlugin {
    TerminalPlugin::new().with_color_scheme(ColorSchemeVariant::RetroGreen)
}