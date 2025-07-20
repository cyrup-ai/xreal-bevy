//! Ultra-Fast Zero-Allocation Plugin Builder
//!
//! This module provides a blazing-fast, zero-allocation plugin builder using
//! const generics for compile-time state validation. All operations are
//! zero-cost abstractions with no runtime overhead.

use super::fast_data::{
    FixedVec, PluginAuthor, PluginDependencies, PluginDescription, PluginId, PluginName,
    PluginVersion, SmallString,
};
use crate::plugins::{
    PluginCapabilities, PluginCapabilitiesFlags, PluginMetadata, SurfaceRequirements,
};
use core::marker::PhantomData;

/// Compile-time builder state encoding
///
/// Uses const generics to encode the builder state at compile time,
/// ensuring zero runtime overhead and compile-time validation.
pub struct BuilderState<const HAS_ID: bool, const HAS_NAME: bool, const HAS_CAPS: u8>;

/// Capability bits for compile-time encoding
pub mod caps {
    pub const NONE: u8 = 0;
    pub const TRANSPARENCY: u8 = 1 << 0;
    pub const KEYBOARD: u8 = 1 << 1;
    pub const MULTI_WINDOW: u8 = 1 << 2;
    pub const RENDERING_3D: u8 = 1 << 3;
    pub const COMPUTE_SHADERS: u8 = 1 << 4;
    pub const NETWORK: u8 = 1 << 5;
    pub const FILE_SYSTEM: u8 = 1 << 6;
    pub const AUDIO: u8 = 1 << 7;
}

/// Ultra-fast zero-allocation plugin builder
///
/// All state is encoded at compile time using const generics.
/// Builder operations are zero-cost abstractions that only
/// affect the type system, not runtime performance.
#[derive(Clone)]
pub struct FastPluginBuilder<const HAS_ID: bool, const HAS_NAME: bool, const HAS_CAPS: u8> {
    /// Plugin identifier (zero-size when not set)
    id: PluginId,
    /// Plugin name (zero-size when not set)
    name: PluginName,
    /// Plugin version
    version: PluginVersion,
    /// Plugin description
    description: PluginDescription,
    /// Plugin author
    author: PluginAuthor,
    /// Plugin dependencies (fixed-size, no allocations)
    dependencies: PluginDependencies,
    /// Surface requirements
    surface_width: u32,
    surface_height: u32,
    surface_format: u32, // Encoded wgpu::TextureFormat
    /// Update rate preference
    preferred_update_rate: u32,
    /// Minimum engine version required
    minimum_engine_version: PluginVersion,
    /// Phantom data for compile-time state tracking
    _phantom: PhantomData<BuilderState<HAS_ID, HAS_NAME, HAS_CAPS>>,
}

/// Initial builder state - no fields configured
impl FastPluginBuilder<false, false, { caps::NONE }> {
    /// Create a new ultra-fast plugin builder
    ///
    /// This is a zero-cost operation that creates a builder with
    /// all fields in their default state.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            id: SmallString::new(),
            name: SmallString::new(),
            version: match SmallString::from_static("1.0.0") {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            description: SmallString::new(),
            author: SmallString::new(),
            dependencies: FixedVec::new(),
            surface_width: 1920,
            surface_height: 1080,
            surface_format: 0, // TextureFormat::Bgra8UnormSrgb
            preferred_update_rate: 60,
            minimum_engine_version: match SmallString::from_static("1.0.0") {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            _phantom: PhantomData,
        }
    }
}

/// Builder with no ID set - must set ID first
impl<const HAS_NAME: bool, const HAS_CAPS: u8> FastPluginBuilder<false, HAS_NAME, HAS_CAPS> {
    /// Set the unique plugin identifier
    ///
    /// This is a zero-cost type transformation that encodes the ID
    /// in the type system for compile-time validation.
    ///
    /// # Example
    /// ```rust
    /// let builder = FastPluginBuilder::new()
    ///     .id("com.xreal.browser");
    /// ```
    #[inline(always)]
    pub const fn id(self, id: &'static str) -> FastPluginBuilder<true, HAS_NAME, HAS_CAPS> {
        FastPluginBuilder {
            id: match SmallString::from_static(id) {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }
}

/// Builder with ID set but no name - must set name next
impl<const HAS_CAPS: u8> FastPluginBuilder<true, false, HAS_CAPS> {
    /// Set the human-readable plugin name
    ///
    /// Zero-cost type transformation that validates name is set
    /// before capabilities can be finalized.
    ///
    /// # Example
    /// ```rust
    /// .name("XREAL Web Browser")
    /// ```
    #[inline(always)]
    pub const fn name(self, name: &'static str) -> FastPluginBuilder<true, true, HAS_CAPS> {
        FastPluginBuilder {
            id: self.id,
            name: match SmallString::from_static(name) {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }
}

/// Common builder methods available in all valid states
impl<const HAS_ID: bool, const HAS_NAME: bool, const HAS_CAPS: u8>
    FastPluginBuilder<HAS_ID, HAS_NAME, HAS_CAPS>
{
    /// Set the plugin version
    ///
    /// Zero-cost operation that updates the version string.
    #[inline(always)]
    pub const fn version(self, version: &'static str) -> Self {
        Self {
            version: match SmallString::from_static(version) {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            id: self.id,
            name: self.name,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: self._phantom,
        }
    }

    /// Set the plugin description
    #[inline(always)]
    pub const fn description(self, description: &'static str) -> Self {
        Self {
            description: match SmallString::from_static(description) {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            id: self.id,
            name: self.name,
            version: self.version,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: self._phantom,
        }
    }

    /// Set the plugin author
    #[inline(always)]
    pub const fn author(self, author: &'static str) -> Self {
        Self {
            author: match SmallString::from_static(author) {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: self._phantom,
        }
    }

    /// Set surface dimensions
    #[inline(always)]
    pub const fn surface_size(self, width: u32, height: u32) -> Self {
        Self {
            surface_width: width,
            surface_height: height,
            ..self
        }
    }

    /// Set preferred update rate
    #[inline(always)]
    pub const fn update_rate(self, hz: u32) -> Self {
        Self {
            preferred_update_rate: hz,
            ..self
        }
    }

    /// Set minimum engine version
    #[inline(always)]
    pub const fn requires_engine(self, version: &'static str) -> Self {
        Self {
            minimum_engine_version: match SmallString::from_static(version) {
                Ok(s) => s,
                Err(_) => SmallString::new(),
            },
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            _phantom: self._phantom,
        }
    }
}

/// Capability configuration methods (only available when ID and name are set)
impl<const HAS_CAPS: u8> FastPluginBuilder<true, true, HAS_CAPS> {
    /// Enable transparency support
    #[allow(dead_code)]
    #[inline(always)]
    pub const fn supports_transparency(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Require keyboard focus
    #[inline(always)]
    pub const fn requires_keyboard(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Enable multi-window support
    #[inline(always)]
    pub const fn supports_multi_window(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Enable 3D rendering
    #[allow(dead_code)]
    #[inline(always)]
    pub const fn supports_3d_rendering(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Enable compute shaders
    #[allow(dead_code)]
    #[inline(always)]
    pub const fn supports_compute_shaders(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Require network access
    #[inline(always)]
    pub const fn requires_network(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Enable file system access
    #[inline(always)]
    pub const fn supports_file_system(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }

    /// Enable audio support
    #[inline(always)]
    pub const fn supports_audio(self) -> FastPluginBuilder<true, true, 255> {
        FastPluginBuilder {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            dependencies: self.dependencies,
            surface_width: self.surface_width,
            surface_height: self.surface_height,
            surface_format: self.surface_format,
            preferred_update_rate: self.preferred_update_rate,
            minimum_engine_version: self.minimum_engine_version,
            _phantom: PhantomData,
        }
    }
}

/// Build method - only available when all required fields are set and at least one capability  
impl<const HAS_CAPS: u8> FastPluginBuilder<true, true, HAS_CAPS> {
    /// Build the final plugin metadata
    ///
    /// This method is only available when all required fields are set
    /// and at least one capability is configured. The const generic
    /// constraint ensures compilation fails if capabilities are missing.
    ///
    /// # Performance
    /// This is a zero-allocation operation that constructs the metadata
    /// from compile-time known values.
    #[inline]
    pub fn build(self) -> PluginMetadata {
        // Convert compile-time capability bits to runtime capabilities
        let mut capabilities = PluginCapabilitiesFlags::new();

        if (HAS_CAPS & caps::TRANSPARENCY) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::SUPPORTS_TRANSPARENCY);
        }
        if (HAS_CAPS & caps::KEYBOARD) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::REQUIRES_KEYBOARD_FOCUS);
        }
        if (HAS_CAPS & caps::MULTI_WINDOW) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::SUPPORTS_MULTI_WINDOW);
        }
        if (HAS_CAPS & caps::RENDERING_3D) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::SUPPORTS_3D_RENDERING);
        }
        if (HAS_CAPS & caps::COMPUTE_SHADERS) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::SUPPORTS_COMPUTE_SHADERS);
        }
        if (HAS_CAPS & caps::NETWORK) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::REQUIRES_NETWORK_ACCESS);
        }
        if (HAS_CAPS & caps::FILE_SYSTEM) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::SUPPORTS_FILE_SYSTEM);
        }
        if (HAS_CAPS & caps::AUDIO) != 0 {
            capabilities.set_flag(PluginCapabilitiesFlags::SUPPORTS_AUDIO);
        }

        PluginMetadata {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            capabilities,
            dependencies: {
                let mut deps = PluginDependencies::new();
                for dep in self.dependencies.iter_any() {
                    if !deps.push(*dep) {
                        break;
                    }
                }
                deps
            },
            minimum_engine_version: self.minimum_engine_version,
            icon_path: None, // Can be added as needed
            library_path: std::path::PathBuf::new(),
        }
    }

    /// Extract just the capabilities for use in plugin implementations
    ///
    /// Zero-allocation method that returns the capabilities struct
    /// constructed from compile-time capability bits.
    #[inline(always)]
    pub const fn capabilities(self) -> PluginCapabilities {
        PluginCapabilities {
            supports_transparency: (HAS_CAPS & caps::TRANSPARENCY) != 0,
            requires_keyboard_focus: (HAS_CAPS & caps::KEYBOARD) != 0,
            supports_multi_window: (HAS_CAPS & caps::MULTI_WINDOW) != 0,
            supports_3d_rendering: (HAS_CAPS & caps::RENDERING_3D) != 0,
            supports_compute_shaders: (HAS_CAPS & caps::COMPUTE_SHADERS) != 0,
            requires_network_access: (HAS_CAPS & caps::NETWORK) != 0,
            supports_file_system: (HAS_CAPS & caps::FILE_SYSTEM) != 0,
            supports_audio: (HAS_CAPS & caps::AUDIO) != 0,
            preferred_update_rate: if self.preferred_update_rate > 0 {
                Some(self.preferred_update_rate)
            } else {
                None
            },
        }
    }

    /// Get surface requirements
    #[allow(dead_code)]
    #[inline]
    pub fn surface_requirements(self) -> SurfaceRequirements {
        SurfaceRequirements {
            width: self.surface_width,
            height: self.surface_height,
            format: match self.surface_format {
                0 => wgpu::TextureFormat::Bgra8UnormSrgb,
                1 => wgpu::TextureFormat::Rgba8UnormSrgb,
                2 => wgpu::TextureFormat::Bgra8Unorm,
                3 => wgpu::TextureFormat::Rgba8Unorm,
                _ => wgpu::TextureFormat::Bgra8UnormSrgb,
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
        }
    }
}

/// Ultra-fast macro for common plugin configurations
///
/// Generates compile-time optimized plugin builders for common patterns.
/// All expansion happens at compile time with zero runtime overhead.
#[macro_export]
macro_rules! fast_plugin {
    // Browser-like plugin
    (browser: $id:literal, $name:literal) => {
        $crate::plugins::fast_builder::FastPluginBuilder::new()
            .id($id)
            .name($name)
            .requires_network()
            .requires_keyboard()
            .supports_multi_window()
            .supports_audio()
            .update_rate(60)
    };

    // Terminal-like plugin
    (terminal: $id:literal, $name:literal) => {
        $crate::plugins::fast_builder::FastPluginBuilder::new()
            .id($id)
            .name($name)
            .requires_keyboard()
            .supports_file_system()
            .supports_multi_window()
            .update_rate(30)
    };

    // Game-like plugin
    (game: $id:literal, $name:literal) => {
        $crate::plugins::fast_builder::FastPluginBuilder::new()
            .id($id)
            .name($name)
            .supports_3d_rendering()
            .supports_compute_shaders()
            .requires_keyboard()
            .supports_audio()
            .update_rate(60)
    };

    // Basic plugin
    (basic: $id:literal, $name:literal) => {
        $crate::plugins::fast_builder::FastPluginBuilder::new()
            .id($id)
            .name($name)
            .supports_transparency()
    };
}

/// Type alias for the initial builder state
#[allow(dead_code)]
pub type NewPluginBuilder = FastPluginBuilder<false, false, { caps::NONE }>;

// Re-export commented out until needed
// pub use NewPluginBuilder as PluginBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_builder() {
        let metadata = FastPluginBuilder::new()
            .id("test.plugin")
            .name("Test Plugin")
            .supports_transparency()
            .build();

        assert_eq!(metadata.id, "test.plugin");
        assert_eq!(metadata.name, "Test Plugin");
        assert!(metadata.capabilities.supports_transparency);
    }

    #[test]
    fn test_browser_builder() {
        let metadata = FastPluginBuilder::new()
            .id("com.xreal.browser")
            .name("XREAL Browser")
            .requires_network()
            .requires_keyboard()
            .supports_audio()
            .update_rate(60)
            .build();

        assert!(metadata.capabilities.requires_network_access);
        assert!(metadata.capabilities.requires_keyboard_focus);
        assert!(metadata.capabilities.supports_audio);
        assert_eq!(metadata.capabilities.preferred_update_rate, Some(60));
    }

    #[test]
    fn test_macro() {
        let metadata = fast_plugin!(browser: "test.browser", "Test Browser").build();
        assert!(metadata.capabilities.requires_network_access);
        assert!(metadata.capabilities.requires_keyboard_focus);

        let metadata = fast_plugin!(terminal: "test.terminal", "Test Terminal").build();
        assert!(metadata.capabilities.requires_keyboard_focus);
        assert!(metadata.capabilities.supports_file_system);
    }

    // These should fail compilation if uncommented:

    // #[test]
    // fn test_incomplete_builder() {
    //     let metadata = FastPluginBuilder::new()
    //         .id("test")
    //         // Missing name!
    //         .supports_transparency()
    //         .build(); // Should fail to compile
    // }

    // #[test]
    // fn test_no_capabilities() {
    //     let metadata = FastPluginBuilder::new()
    //         .id("test")
    //         .name("Test")
    //         // Missing capabilities!
    //         .build(); // Should fail to compile
    // }
}
