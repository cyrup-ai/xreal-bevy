//! Browser plugin implementation for Bevy
//!
//! This module contains the main BrowserPlugin struct that implements Bevy's Plugin trait,
//! providing seamless integration with the Bevy ECS architecture.

use bevy::prelude::*;
use tracing::{info, error};
use crate::{
    components::{BrowserBundle, BrowserEntity},
    resources::{BrowserConfig, BrowserState},
    systems::{
        browser_update_system, browser_input_system, browser_render_system,
        browser_navigation_system, browser_lifecycle_system, browser_cleanup_system,
        browser_command_system, browser_performance_system,
    },
    capabilities::BrowserCapabilities,
    error::{BrowserError, BrowserResult},
};

/// Main browser plugin for Bevy applications
/// 
/// This plugin provides complete browser functionality including webview integration,
/// input handling, navigation, and rendering within the Bevy ECS architecture.
#[derive(Debug, Clone)]
pub struct BrowserPlugin {
    /// Plugin configuration
    pub config: BrowserConfig,
    /// Plugin capabilities
    pub capabilities: BrowserCapabilities,
}

impl BrowserPlugin {
    /// Create a new browser plugin with default configuration
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            config: BrowserConfig::new(),
            capabilities: BrowserCapabilities::default_capabilities(),
        }
    }

    /// Create a browser plugin with custom configuration
    #[inline(always)]
    pub fn with_config(config: BrowserConfig) -> Self {
        Self {
            config,
            capabilities: BrowserCapabilities::default_capabilities(),
        }
    }

    /// Create a browser plugin optimized for development
    #[inline(always)]
    pub fn development() -> Self {
        Self::new().with_default_url("https://example.com".to_string())
    }
    
    /// Set the default URL for new browser instances
    #[inline(always)]
    pub fn with_default_url(mut self, url: String) -> Self {
        self.config.default_url = url;
        self
    }
    
    /// Set the cache size in megabytes
    #[inline(always)]
    pub fn with_cache_size(mut self, size_mb: u64) -> Self {
        self.config.cache_size_mb = size_mb;
        self
    }

    /// Create a browser plugin optimized for performance
    #[inline(always)]
    pub fn performance_optimized() -> Self {
        Self {
            config: BrowserConfig::performance_optimized(),
            capabilities: BrowserCapabilities::default_capabilities(),
        }
    }

    /// Set custom capabilities for the browser plugin
    #[inline(always)]
    pub fn with_capabilities(mut self, capabilities: BrowserCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Enable specific capability
    #[inline(always)]
    pub fn enable_capability(mut self, capability: BrowserCapabilities) -> Self {
        self.capabilities = self.capabilities.with_flag(capability);
        self
    }

    /// Get plugin capabilities
    #[inline(always)]
    pub fn capabilities(&self) -> BrowserCapabilities {
        self.capabilities
    }

    /// Validate plugin configuration
    #[inline(always)]
    pub fn validate(&self) -> BrowserResult<()> {
        self.config.validate()?;
        
        // Validate capabilities
        if !self.capabilities.contains(BrowserCapabilities::WEBVIEW) {
            return Err(BrowserError::ConfigError(
                "Browser plugin requires webview capability".to_string()
            ));
        }
        
        Ok(())
    }

    /// Create a new browser entity with this plugin's configuration
    #[inline(always)]
    pub fn create_browser_entity(&self, commands: &mut Commands, id: String, url: String) -> Entity {
        let bundle = BrowserBundle::new(id, url);
        commands.spawn(bundle).id()
    }

    /// Create a browser entity with custom viewport size
    #[inline(always)]
    pub fn create_browser_entity_with_size(
        &self,
        commands: &mut Commands,
        id: String,
        url: String,
        viewport_size: (u32, u32),
    ) -> Entity {
        let mut bundle = BrowserBundle::new(id, url);
        bundle.entity.set_viewport_size(viewport_size.0, viewport_size.1);
        commands.spawn(bundle).id()
    }
}

impl Default for BrowserPlugin {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for BrowserPlugin {
    fn build(&self, app: &mut App) {
        // Validate configuration before building
        if let Err(e) = self.validate() {
            error!("Browser plugin configuration validation failed: {}", e);
            return;
        }

        info!("Initializing XREAL Browser Plugin v1.0.0");
        info!("Configuration: {} instances max, {}MB cache", 
              self.config.max_instances, self.config.cache_size_mb);

        // Insert plugin resources
        app.insert_resource(self.config.clone())
           .insert_resource(BrowserState::new());

        // Register component types
        app.register_type::<BrowserEntity>();

        // Add browser systems to the appropriate system sets
        app.add_systems(
            Update,
            (
                // Core browser systems
                browser_update_system,
                browser_navigation_system,
                browser_lifecycle_system,
                browser_performance_system,
                
                // Input handling
                browser_input_system,
                
                // Command processing
                browser_command_system,
            ).chain(), // Run in sequence for proper state management
        );

        // Add render systems to PostUpdate for proper ordering
        app.add_systems(
            PostUpdate,
            browser_render_system,
        );

        // Add cleanup systems
        app.add_systems(
            Last,
            browser_cleanup_system,
        );

        // Initialize browser state
        app.world_mut().resource_mut::<BrowserState>().set_initialized(true);

        info!("✅ XREAL Browser Plugin initialized successfully");
        info!("Capabilities: webview={}, navigation={}, input={}, transparency={}", 
              self.capabilities.contains(BrowserCapabilities::WEBVIEW),
              self.capabilities.contains(BrowserCapabilities::NAVIGATION),
              self.capabilities.contains(BrowserCapabilities::INPUT_HANDLING),
              self.capabilities.contains(BrowserCapabilities::TRANSPARENCY));
    }

    fn finish(&self, app: &mut App) {
        // Perform final initialization after all plugins are loaded
        let browser_state = app.world().resource::<BrowserState>();
        if browser_state.is_initialized {
            info!("🌐 Browser plugin finish phase completed");
            
            // In real implementation, this would:
            // 1. Initialize webview backend
            // 2. Set up IPC communication
            // 3. Configure security settings
            // 4. Load initial pages if specified
        }
    }

    fn cleanup(&self, app: &mut App) {
        // Clean up browser resources when plugin is removed
        info!("🧹 Cleaning up browser plugin resources");
        
        // Remove all browser entities
        let mut query = app.world_mut().query_filtered::<Entity, With<BrowserEntity>>();
        let entities: Vec<Entity> = query.iter(app.world()).collect();
        
        for entity in entities {
            if let Ok(entity_commands) = app.world_mut().get_entity_mut(entity) {
                entity_commands.despawn();
            }
        }
        
        // Reset browser state
        if let Some(mut browser_state) = app.world_mut().get_resource_mut::<BrowserState>() {
            *browser_state = BrowserState::new();
        }
        
        info!("✅ Browser plugin cleanup completed");
    }
}

/// Browser plugin builder for advanced configuration
#[derive(Debug, Clone)]
pub struct BrowserPluginBuilder {
    config: BrowserConfig,
    capabilities: BrowserCapabilities,
}

impl BrowserPluginBuilder {
    /// Create a new browser plugin builder
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            config: BrowserConfig::new(),
            capabilities: BrowserCapabilities::default_capabilities(),
        }
    }

    /// Set default URL
    #[inline(always)]
    pub fn default_url(mut self, url: impl Into<String>) -> Self {
        self.config.default_url = url.into();
        self
    }

    /// Set cache size in megabytes
    #[inline(always)]
    pub fn cache_size_mb(mut self, size: u64) -> Self {
        self.config.cache_size_mb = size;
        self
    }

    /// Set user agent string
    #[inline(always)]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Enable or disable JavaScript
    #[inline(always)]
    pub fn javascript_enabled(mut self, enabled: bool) -> Self {
        self.config.javascript_enabled = enabled;
        self
    }

    /// Enable or disable images
    #[inline(always)]
    pub fn images_enabled(mut self, enabled: bool) -> Self {
        self.config.images_enabled = enabled;
        self
    }

    /// Enable or disable plugins
    #[inline(always)]
    pub fn plugins_enabled(mut self, enabled: bool) -> Self {
        self.config.plugins_enabled = enabled;
        self
    }

    /// Set maximum number of browser instances
    #[inline(always)]
    pub fn max_instances(mut self, max: usize) -> Self {
        self.config.max_instances = max;
        self
    }

    /// Set default viewport size
    #[inline(always)]
    pub fn default_viewport_size(mut self, width: u32, height: u32) -> Self {
        self.config.default_viewport_size = (width, height);
        self
    }

    /// Enable or disable developer tools
    #[inline(always)]
    pub fn dev_tools_enabled(mut self, enabled: bool) -> Self {
        self.config.dev_tools_enabled = enabled;
        self
    }

    /// Add capability
    #[inline(always)]
    pub fn with_capability(mut self, capability: BrowserCapabilities) -> Self {
        self.capabilities = self.capabilities.with_flag(capability);
        self
    }

    /// Build the browser plugin
    #[inline(always)]
    pub fn build(self) -> BrowserPlugin {
        BrowserPlugin {
            config: self.config,
            capabilities: self.capabilities,
        }
    }
}

impl Default for BrowserPluginBuilder {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a browser entity
pub fn spawn_browser(
    commands: &mut Commands,
    id: impl Into<String>,
    url: impl Into<String>,
) -> Entity {
    let bundle = BrowserBundle::new(id.into(), url.into());
    commands.spawn(bundle).id()
}

/// Convenience function to create a browser entity with custom size
pub fn spawn_browser_with_size(
    commands: &mut Commands,
    id: impl Into<String>,
    url: impl Into<String>,
    viewport_size: (u32, u32),
) -> Entity {
    let mut bundle = BrowserBundle::new(id.into(), url.into());
    bundle.entity.set_viewport_size(viewport_size.0, viewport_size.1);
    commands.spawn(bundle).id()
}

/// System to spawn a default browser instance (useful for testing)
pub fn spawn_default_browser_system(
    mut commands: Commands,
    config: Res<BrowserConfig>,
    query: Query<&BrowserEntity>,
) {
    // Only spawn if no browsers exist
    if query.is_empty() {
        let entity = spawn_browser(
            &mut commands,
            "default_browser",
            config.default_url.clone(),
        );
        info!("Spawned default browser instance: {:?}", entity);
    }
}