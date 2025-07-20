//! Typestate Plugin Builder for Type-Safe Plugin Construction
//! 
//! Provides a fluent, type-safe builder API for creating XREAL plugins with compile-time
//! validation of required fields and configuration. Uses phantom types to ensure plugins
//! are properly configured before construction.

use bevy::prelude::*;
use std::marker::PhantomData;
use std::path::PathBuf;
use super::PluginCapabilitiesFlags;

use super::{PluginCapabilities, PluginMetadata, SurfaceRequirements};

/// Marker traits for typestate validation
pub mod state {
    /// Plugin ID has been set
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct HasId;
    /// Plugin ID not yet set
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct NoId;
    
    /// Plugin name has been set
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct HasName;
    /// Plugin name not yet set
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct NoName;
    
    /// Plugin capabilities have been configured
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct HasCapabilities;
    /// Plugin capabilities not yet configured
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct NoCapabilities;
    
    /// Plugin is fully configured and ready to build
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct Ready;
    /// Plugin configuration incomplete
    #[allow(dead_code)] // Type-state marker, not directly constructed
    pub struct Incomplete;
}

/// Type-safe plugin builder with compile-time validation
/// 
/// Uses phantom types to ensure all required fields are set before allowing build().
/// Provides fluent API with excellent IDE autocomplete and documentation.
#[allow(dead_code)]
pub struct PluginBuilder<ID, NAME, CAPS, STATE> {
    id: Option<String>,
    name: Option<String>,
    version: String,
    description: String,
    author: String,
    capabilities: Option<PluginCapabilities>,
    dependencies: Vec<String>,
    surface_requirements: Option<SurfaceRequirements>,
    icon_path: Option<PathBuf>,
    minimum_engine_version: String,
    
    // Phantom data for compile-time state tracking
    _phantom_id: PhantomData<ID>,
    _phantom_name: PhantomData<NAME>, 
    _phantom_caps: PhantomData<CAPS>,
    _phantom_state: PhantomData<STATE>,
}

/// Starting point for plugin builder - no fields configured yet
impl PluginBuilder<state::NoId, state::NoName, state::NoCapabilities, state::Incomplete> {
    /// Create a new plugin builder
    /// 
    /// # Example
    /// ```rust
    /// let plugin = PluginBuilder::new()
    ///     .id("com.xreal.browser")
    ///     .name("XREAL Browser")
    ///     .version("1.0.0")
    ///     .description("Web browser for XREAL AR glasses")
    ///     .author("XREAL Team")
    ///     .requires_network()
    ///     .supports_keyboard()
    ///     .build();
    /// ```
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            capabilities: None,
            dependencies: Vec::new(),
            surface_requirements: None,
            icon_path: None,
            minimum_engine_version: "1.0.0".to_string(),
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
}

/// Builder with ID set - can now set name
impl<NAME, CAPS> PluginBuilder<state::HasId, NAME, CAPS, state::Incomplete> {
    /// Set the human-readable plugin name
    /// 
    /// This name is displayed in the XREAL plugin manager UI and should be
    /// descriptive and user-friendly.
    /// 
    /// # Example
    /// ```rust
    /// .name("XREAL Web Browser")
    /// ```
    pub fn name(mut self, name: impl Into<String>) -> PluginBuilder<state::HasId, state::HasName, CAPS, state::Incomplete> {
        self.name = Some(name.into());
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: self.capabilities,
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
}

/// Builder with no ID - must set ID first
impl<NAME, CAPS> PluginBuilder<state::NoId, NAME, CAPS, state::Incomplete> {
    /// Set the unique plugin identifier
    /// 
    /// This should be a reverse domain name style identifier that uniquely
    /// identifies your plugin across the XREAL ecosystem.
    /// 
    /// # Example
    /// ```rust
    /// .id("com.mycompany.browser")
    /// ```
    pub fn id(mut self, id: impl Into<String>) -> PluginBuilder<state::HasId, NAME, CAPS, state::Incomplete> {
        self.id = Some(id.into());
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: self.capabilities,
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
}

/// Common builder methods available in all states
impl<ID, NAME, CAPS, STATE> PluginBuilder<ID, NAME, CAPS, STATE> {
    /// Set the plugin version
    /// 
    /// Should follow semantic versioning (semver) format: MAJOR.MINOR.PATCH
    /// 
    /// # Example
    /// ```rust
    /// .version("2.1.3")
    /// ```
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    /// Set the plugin description
    /// 
    /// Provide a detailed description of what your plugin does and its features.
    /// This is shown in the plugin manager and store.
    /// 
    /// # Example
    /// ```rust
    /// .description("Full-featured web browser with WebGL support for XREAL AR glasses")
    /// ```
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    
    /// Set the plugin author/organization
    /// 
    /// # Example
    /// ```rust
    /// .author("XREAL Development Team")
    /// ```
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }
    
    /// Add a dependency on another plugin
    /// 
    /// Dependencies will be loaded automatically before this plugin.
    /// 
    /// # Example
    /// ```rust
    /// .depends_on("com.xreal.webview-engine")
    /// ```
    pub fn depends_on(mut self, plugin_id: impl Into<String>) -> Self {
        self.dependencies.push(plugin_id.into());
        self
    }
    
    /// Set the minimum XREAL engine version required
    /// 
    /// # Example
    /// ```rust
    /// .requires_engine("1.2.0")
    /// ```
    pub fn requires_engine(mut self, version: impl Into<String>) -> Self {
        self.minimum_engine_version = version.into();
        self
    }
    
    /// Set the path to the plugin icon
    /// 
    /// Icon should be PNG format, 64x64 pixels for best results.
    /// 
    /// # Example
    /// ```rust
    /// .icon("assets/browser-icon.png")
    /// ```
    pub fn icon<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.icon_path = Some(path.into());
        self
    }
    
    /// Configure custom surface requirements
    /// 
    /// # Example
    /// ```rust
    /// .surface_size(1920, 1080)
    /// .surface_format(wgpu::TextureFormat::Bgra8UnormSrgb)
    /// ```
    pub fn surface_requirements(mut self, requirements: SurfaceRequirements) -> Self {
        self.surface_requirements = Some(requirements);
        self
    }
    
    /// Set surface dimensions
    /// 
    /// # Example
    /// ```rust
    /// .surface_size(1280, 720)
    /// ```
    pub fn surface_size(mut self, width: u32, height: u32) -> Self {
        let mut requirements = self.surface_requirements.unwrap_or_default();
        requirements.width = width;
        requirements.height = height;
        self.surface_requirements = Some(requirements);
        self
    }
    
    /// Set surface texture format
    /// 
    /// # Example
    /// ```rust
    /// .surface_format(wgpu::TextureFormat::Rgba8UnormSrgb)
    /// ```
    pub fn surface_format(mut self, format: wgpu::TextureFormat) -> Self {
        let mut requirements = self.surface_requirements.unwrap_or_default();
        requirements.format = format;
        self.surface_requirements = Some(requirements);
        self
    }
}

/// Capability configuration methods - returns builder with capabilities set
impl<ID, NAME> PluginBuilder<ID, NAME, state::NoCapabilities, state::Incomplete> {
    /// Enable transparency support
    /// 
    /// Allows the plugin to render with alpha blending and transparent backgrounds.
    /// 
    /// # Example
    /// ```rust
    /// .supports_transparency()
    /// ```
    pub fn supports_transparency(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.supports_transparency = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Require keyboard focus for input handling
    /// 
    /// Plugin will receive keyboard events when focused. Essential for text input.
    /// 
    /// # Example
    /// ```rust
    /// .requires_keyboard()
    /// ```
    pub fn requires_keyboard(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.requires_keyboard_focus = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Enable multi-window support
    /// 
    /// Plugin can create and manage multiple windows in the AR space.
    /// 
    /// # Example
    /// ```rust
    /// .supports_multi_window()
    /// ```
    pub fn supports_multi_window(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.supports_multi_window = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Enable 3D rendering capabilities
    /// 
    /// Plugin will render 3D graphics with depth and spatial positioning.
    /// 
    /// # Example
    /// ```rust
    /// .supports_3d_rendering()
    /// ```
    pub fn supports_3d_rendering(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.supports_3d_rendering = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Enable compute shader support
    /// 
    /// Plugin can use GPU compute shaders for parallel processing.
    /// 
    /// # Example
    /// ```rust
    /// .supports_compute_shaders()
    /// ```
    pub fn supports_compute_shaders(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.supports_compute_shaders = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Require network access
    /// 
    /// Plugin needs internet connectivity for web requests, streaming, etc.
    /// 
    /// # Example
    /// ```rust
    /// .requires_network()
    /// ```
    pub fn requires_network(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.requires_network_access = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Enable file system access
    /// 
    /// Plugin can read and write files on the local system.
    /// 
    /// # Example
    /// ```rust
    /// .supports_file_system()
    /// ```
    pub fn supports_file_system(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.supports_file_system = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Enable audio playback support
    /// 
    /// Plugin can play audio and handle audio streams.
    /// 
    /// # Example
    /// ```rust
    /// .supports_audio()
    /// ```
    pub fn supports_audio(self) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.supports_audio = true;
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
    
    /// Set preferred update rate in Hz
    /// 
    /// Specify how often the plugin needs to be updated. Lower rates save battery.
    /// 
    /// # Example
    /// ```rust
    /// .update_rate(30) // 30 FPS for terminal apps
    /// .update_rate(60) // 60 FPS for games
    /// ```
    pub fn update_rate(self, hz: u32) -> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
        let mut caps = PluginCapabilities::default();
        caps.preferred_update_rate = Some(hz);
        
        PluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities: Some(caps),
            dependencies: self.dependencies,
            surface_requirements: self.surface_requirements,
            icon_path: self.icon_path,
            minimum_engine_version: self.minimum_engine_version,
            _phantom_id: PhantomData,
            _phantom_name: PhantomData,
            _phantom_caps: PhantomData,
            _phantom_state: PhantomData,
        }
    }
}

/// Additional capability methods for already-configured capabilities
impl<ID, NAME> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
    /// Chain additional capabilities onto existing ones
    /// 
    /// # Example
    /// ```rust
    /// .requires_network()
    /// .also_supports_transparency()
    /// .also_supports_audio()
    /// ```
    pub fn also_supports_transparency(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.supports_transparency = true;
        }
        self
    }
    
    pub fn also_requires_keyboard(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.requires_keyboard_focus = true;
        }
        self
    }
    
    pub fn also_supports_multi_window(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.supports_multi_window = true;
        }
        self
    }
    
    pub fn also_supports_3d_rendering(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.supports_3d_rendering = true;
        }
        self
    }
    
    pub fn also_supports_compute_shaders(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.supports_compute_shaders = true;
        }
        self
    }
    
    pub fn also_requires_network(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.requires_network_access = true;
        }
        self
    }
    
    pub fn also_supports_file_system(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.supports_file_system = true;
        }
        self
    }
    
    pub fn also_supports_audio(mut self) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.supports_audio = true;
        }
        self
    }
    
    pub fn also_update_rate(mut self, hz: u32) -> Self {
        if let Some(ref mut caps) = self.capabilities {
            caps.preferred_update_rate = Some(hz);
        }
        self
    }
}

/// Capability extraction method - available when capabilities are configured
impl<ID, NAME> PluginBuilder<ID, NAME, state::HasCapabilities, state::Incomplete> {
    /// Extract just the capabilities without building full metadata
    /// 
    /// Useful for implementing the capabilities() method in plugin implementations
    /// where you want to reuse the same capability configuration.
    /// 
    /// # Example
    /// ```rust
    /// fn capabilities(&self) -> PluginCapabilities {
    ///     PluginBuilder::new()
    ///         .requires_network()
    ///         .also_requires_keyboard()
    ///         .also_supports_audio()
    ///         .capabilities()
    /// }
    /// ```
    pub fn capabilities(self) -> PluginCapabilities {
        self.capabilities.expect("Capabilities should be set in HasCapabilities state")
    }
}

/// Simple builder implementation for examples compatibility
/// 
/// This provides a simpler API that doesn't use type states but still provides
/// the fluent interface expected by the plugin examples.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SimplePluginBuilder {
    id: Option<String>,
    name: Option<String>,
    version: String,
    description: String,
    author: String,
    capabilities: PluginCapabilities,
    dependencies: Vec<String>,
    minimum_engine_version: String,
    icon_path: Option<PathBuf>,
    surface_width: Option<u32>,
    surface_height: Option<u32>,
    surface_format: Option<wgpu::TextureFormat>,
}

impl SimplePluginBuilder {
    /// Create new simple builder
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            minimum_engine_version: "1.0.0".to_string(),
            capabilities: PluginCapabilities::default(),
            ..Default::default()
        }
    }
    
    /// Set plugin ID
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    
    /// Set plugin name
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set version
    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }
    
    /// Set description
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
    
    /// Set author
    pub fn author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }
    
    /// Set icon path
    pub fn icon(mut self, path: &str) -> Self {
        self.icon_path = Some(PathBuf::from(path));
        self
    }
    
    /// Set engine requirement
    pub fn requires_engine(mut self, version: &str) -> Self {
        self.minimum_engine_version = version.to_string();
        self
    }
    
    /// Set surface size
    pub fn surface_size(mut self, width: u32, height: u32) -> Self {
        self.surface_width = Some(width);
        self.surface_height = Some(height);
        self
    }
    
    /// Set surface format
    pub fn surface_format(mut self, format: wgpu::TextureFormat) -> Self {
        self.surface_format = Some(format);
        self
    }
    
    // Capability methods
    
    /// Require network access
    pub fn requires_network(mut self) -> Self {
        self.capabilities.requires_network_access = true;
        self
    }
    
    /// Require keyboard focus
    pub fn requires_keyboard(mut self) -> Self {
        self.capabilities.requires_keyboard_focus = true;
        self
    }
    
    /// Support multiple windows
    pub fn supports_multi_window(mut self) -> Self {
        self.capabilities.supports_multi_window = true;
        self
    }
    
    /// Support audio
    pub fn supports_audio(mut self) -> Self {
        self.capabilities.supports_audio = true;
        self
    }
    
    /// Support file system access
    pub fn supports_file_system(mut self) -> Self {
        self.capabilities.supports_file_system = true;
        self
    }
    
    /// Support transparency rendering
    pub fn supports_transparency(mut self) -> Self {
        self.capabilities.supports_transparency = true;
        self
    }
    
    /// Support 3D rendering
    pub fn supports_3d_rendering(mut self) -> Self {
        self.capabilities.supports_3d_rendering = true;
        self
    }
    
    /// Support compute shaders
    pub fn supports_compute_shaders(mut self) -> Self {
        self.capabilities.supports_compute_shaders = true;
        self
    }
    
    /// Add plugin dependency
    pub fn depends_on(mut self, plugin_id: &str) -> Self {
        self.dependencies.push(plugin_id.to_string());
        self
    }
    
    /// Set update rate
    pub fn update_rate(mut self, hz: u32) -> Self {
        self.capabilities.preferred_update_rate = Some(hz);
        self
    }
    
    // "Also" variants for chaining
    
    pub fn also_requires_keyboard(mut self) -> Self {
        self.capabilities.requires_keyboard_focus = true;
        self
    }
    
    pub fn also_supports_multi_window(mut self) -> Self {
        self.capabilities.supports_multi_window = true;
        self
    }
    
    pub fn also_supports_audio(mut self) -> Self {
        self.capabilities.supports_audio = true;
        self
    }
    
    pub fn also_supports_file_system(mut self) -> Self {
        self.capabilities.supports_file_system = true;
        self
    }
    
    pub fn also_update_rate(mut self, hz: u32) -> Self {
        self.capabilities.preferred_update_rate = Some(hz);
        self
    }
    
    pub fn also_supports_transparency(mut self) -> Self {
        self.capabilities.supports_transparency = true;
        self
    }
    
    pub fn also_supports_3d_rendering(mut self) -> Self {
        self.capabilities.supports_3d_rendering = true;
        self
    }
    
    pub fn also_supports_compute_shaders(mut self) -> Self {
        self.capabilities.supports_compute_shaders = true;
        self
    }
    
    /// Get capabilities only
    pub fn capabilities(self) -> PluginCapabilities {
        self.capabilities
    }
    
    /// Build metadata
    pub fn build(self) -> PluginMetadata {
        use super::fast_data::PluginDependencies;
        
        // Convert dependencies Vec<String> to PluginDependencies
        let mut ultra_deps = PluginDependencies::new();
        for dep in self.dependencies {
            if !ultra_deps.push(super::fast_data::create_plugin_id(&dep)) {
                warn!("Dependency list full, skipping: {}", dep);
                break;
            }
        }
        
        PluginMetadata {
            id: super::fast_data::create_plugin_id(&self.id.unwrap_or_else(|| "unknown.plugin".to_string())),
            name: super::fast_data::create_plugin_name(&self.name.unwrap_or_else(|| "Unknown Plugin".to_string())),
            version: super::fast_data::create_plugin_version(&self.version),
            description: super::fast_data::create_plugin_description(&self.description),
            author: super::fast_data::create_plugin_author(&self.author),
            capabilities: convert_capabilities_to_flags(&self.capabilities),
            dependencies: ultra_deps,
            minimum_engine_version: super::fast_data::create_plugin_version(&self.minimum_engine_version),
            icon_path: self.icon_path,
            library_path: PathBuf::new(),
        }
    }
}

/// Final build method - only available when all required fields are set
impl PluginBuilder<state::HasId, state::HasName, state::HasCapabilities, state::Incomplete> {
    /// Build the final plugin metadata
    /// 
    /// This method is only available when all required fields (id, name, capabilities)
    /// have been configured. Compilation will fail if any required field is missing.
    /// 
    /// # Example
    /// ```rust
    /// let metadata = PluginBuilder::new()
    ///     .id("com.xreal.browser")
    ///     .name("XREAL Browser")
    ///     .description("Full-featured web browser")
    ///     .author("XREAL Team")
    ///     .requires_network()
    ///     .also_requires_keyboard()
    ///     .build();
    /// ```
    pub fn build(self) -> PluginMetadata {
        use super::fast_data::PluginDependencies;
        
        // Convert dependencies Vec<String> to PluginDependencies
        let mut ultra_deps = PluginDependencies::new();
        for dep in self.dependencies {
            if !ultra_deps.push(super::fast_data::create_plugin_id(&dep)) {
                warn!("Dependency list full, skipping: {}", dep);
                break;
            }
        }
        
        PluginMetadata {
            id: super::fast_data::create_plugin_id(&self.id.expect("ID should be set in HasId state")),
            name: super::fast_data::create_plugin_name(&self.name.expect("Name should be set in HasName state")),
            version: super::fast_data::create_plugin_version(&self.version),
            description: super::fast_data::create_plugin_description(&self.description),
            author: super::fast_data::create_plugin_author(&self.author),
            capabilities: convert_capabilities_to_flags(&self.capabilities.unwrap_or_default()),
            dependencies: ultra_deps,
            minimum_engine_version: super::fast_data::create_plugin_version(&self.minimum_engine_version),
            icon_path: self.icon_path,
            library_path: PathBuf::new(), // Set by the plugin loader
        }
    }
}

/// Convenience macro for common plugin configurations
/// 
/// # Examples
/// 
/// Simple plugin with minimal configuration:
/// ```rust
/// plugin_metadata! {
///     id: "com.example.simple",
///     name: "Simple Plugin",
///     basic
/// }
/// ```
/// 
/// Web browser plugin:
/// ```rust
/// plugin_metadata! {
///     id: "com.xreal.browser",
///     name: "XREAL Browser",
///     description: "Full-featured web browser for AR",
///     author: "XREAL Team",
///     version: "2.0.0",
///     browser_like
/// }
/// ```
/// 
/// Terminal application:
/// ```rust
/// plugin_metadata! {
///     id: "com.xreal.terminal",
///     name: "XREAL Terminal",
///     terminal_like
/// }
/// ```
#[macro_export]
macro_rules! plugin_metadata {
    // Basic plugin with minimal setup
    (id: $id:expr, name: $name:expr, basic) => {
        $crate::plugins::builder::PluginBuilder::new()
            .id($id)
            .name($name)
            .supports_transparency()
            .build()
    };
    
    // Browser-like plugin with network and keyboard
    (id: $id:expr, name: $name:expr, browser_like) => {
        $crate::plugins::builder::PluginBuilder::new()
            .id($id)
            .name($name)
            .requires_network()
            .also_requires_keyboard()
            .also_supports_audio()
            .also_update_rate(60)
            .build()
    };
    
    // Terminal-like plugin with keyboard and file system
    (id: $id:expr, name: $name:expr, terminal_like) => {
        $crate::plugins::builder::PluginBuilder::new()
            .id($id)
            .name($name)
            .requires_keyboard()
            .also_supports_file_system()
            .also_update_rate(30)
            .build()
    };
    
    // Game-like plugin with 3D and high framerate
    (id: $id:expr, name: $name:expr, game_like) => {
        $crate::plugins::builder::PluginBuilder::new()
            .id($id)
            .name($name)
            .supports_3d_rendering()
            .also_requires_keyboard()
            .also_supports_audio()
            .also_supports_compute_shaders()
            .also_update_rate(60)
            .build()
    };
    
    // Full specification with all optional fields
    (
        id: $id:expr,
        name: $name:expr,
        description: $desc:expr,
        author: $author:expr,
        version: $version:expr,
        $capabilities:ident
    ) => {
        $crate::plugins::builder::PluginBuilder::new()
            .id($id)
            .name($name)
            .description($desc)
            .author($author)
            .version($version)
            .$capabilities
            .build()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_plugin_builder() {
        let metadata = PluginBuilder::new()
            .id("test.plugin")
            .name("Test Plugin")
            .supports_transparency()
            .build();
        
        assert_eq!(metadata.id, "test.plugin");
        assert_eq!(metadata.name, "Test Plugin");
        assert!(metadata.capabilities.supports_transparency);
    }
    
    #[test]
    fn test_browser_plugin_builder() {
        let metadata = PluginBuilder::new()
            .id("com.xreal.browser")
            .name("XREAL Browser")
            .description("Web browser for AR")
            .author("XREAL Team")
            .version("1.0.0")
            .requires_network()
            .also_requires_keyboard()
            .also_supports_audio()
            .build();
        
        assert_eq!(metadata.id, "com.xreal.browser");
        assert!(metadata.capabilities.requires_network_access);
        assert!(metadata.capabilities.requires_keyboard_focus);
        assert!(metadata.capabilities.supports_audio);
    }
    
    #[test]
    fn test_chained_capabilities() {
        let metadata = PluginBuilder::new()
            .id("test.chain")
            .name("Chain Test")
            .supports_3d_rendering()
            .also_supports_audio()
            .also_requires_keyboard()
            .also_supports_compute_shaders()
            .build();
        
        assert!(metadata.capabilities.supports_3d_rendering);
        assert!(metadata.capabilities.supports_audio);
        assert!(metadata.capabilities.requires_keyboard_focus);
        assert!(metadata.capabilities.supports_compute_shaders);
    }
    
    // These should fail to compile if uncommented:
    // #[test]
    // fn test_incomplete_builder_fails() {
    //     let metadata = PluginBuilder::new()
    //         .id("test")
    //         // Missing name!
    //         .supports_transparency()
    //         .build(); // This should fail to compile
    // }
}

/// Convert old PluginCapabilities to new PluginCapabilitiesFlags
fn convert_capabilities_to_flags(capabilities: &PluginCapabilities) -> PluginCapabilitiesFlags {
    let mut flags = PluginCapabilitiesFlags::new();
    
    if capabilities.supports_transparency {
        flags.set_flag(PluginCapabilitiesFlags::SUPPORTS_TRANSPARENCY);
    }
    if capabilities.requires_keyboard_focus {
        flags.set_flag(PluginCapabilitiesFlags::REQUIRES_KEYBOARD_FOCUS);
    }
    if capabilities.supports_multi_window {
        flags.set_flag(PluginCapabilitiesFlags::SUPPORTS_MULTI_WINDOW);
    }
    if capabilities.supports_3d_rendering {
        flags.set_flag(PluginCapabilitiesFlags::SUPPORTS_3D_RENDERING);
    }
    if capabilities.supports_compute_shaders {
        flags.set_flag(PluginCapabilitiesFlags::SUPPORTS_COMPUTE_SHADERS);
    }
    if capabilities.requires_network_access {
        flags.set_flag(PluginCapabilitiesFlags::REQUIRES_NETWORK_ACCESS);
    }
    if capabilities.supports_file_system {
        flags.set_flag(PluginCapabilitiesFlags::SUPPORTS_FILE_SYSTEM);
    }
    if capabilities.supports_audio {
        flags.set_flag(PluginCapabilitiesFlags::SUPPORTS_AUDIO);
    }
    
    flags
}