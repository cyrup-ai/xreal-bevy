//! Demonstration of the Type-Safe Plugin Builder
//!
//! This file shows various examples of using the XREAL plugin builder system
//! to create self-documenting, type-safe plugin configurations.

use crate::{
    plugin_metadata,
    plugins::{PluginBuilder, PluginCapabilitiesFlags, PluginMetadata},
};

/// Example 1: Simple minimal plugin
///
/// Creates a basic plugin with just the required fields and minimal capabilities.
pub fn create_minimal_plugin() -> PluginMetadata {
    PluginBuilder::new()
        .id("com.example.minimal")
        .name("Minimal Plugin")
        .description("A simple example plugin")
        .author("Example Developer")
        .supports_transparency() // At least one capability required
        .build()
}

/// Example 2: Web browser plugin with full configuration
///
/// Demonstrates a complex plugin with multiple capabilities and detailed configuration.
pub fn create_browser_plugin() -> PluginMetadata {
    PluginBuilder::new()
        .id("com.company.webbrowser")
        .name("Advanced Web Browser")
        .version("2.1.0")
        .description("Full-featured web browser with WebGL support and AR integration")
        .author("Web Technologies Inc.")
        .depends_on("com.xreal.webview-engine")
        .depends_on("com.xreal.javascript-runtime")
        .requires_engine("1.5.0")
        .icon("assets/browser-icon.png")
        .surface_size(1920, 1200)
        .surface_format(wgpu::TextureFormat::Bgra8UnormSrgb)
        .requires_network()
        .also_requires_keyboard()
        .also_supports_multi_window()
        .also_supports_audio()
        .also_supports_compute_shaders() // For WebGL acceleration
        .also_update_rate(60)
        .build()
}

/// Example 3: Terminal emulator with focused capabilities
///
/// Shows a plugin optimized for text-based interaction.
pub fn create_terminal_plugin() -> PluginMetadata {
    PluginBuilder::new()
        .id("com.terminal.xreal")
        .name("XREAL Terminal Pro")
        .version("1.3.2")
        .description(
            "Professional terminal emulator with syntax highlighting and shell integration",
        )
        .author("Terminal Technologies")
        .icon("assets/terminal-icon.png")
        .surface_size(1024, 768)
        .requires_keyboard()
        .also_supports_file_system()
        .also_supports_multi_window()
        .also_update_rate(30) // Optimized for battery life
        .build()
}

/// Example 4: 3D Game plugin with high-performance requirements
///
/// Demonstrates a plugin that needs maximum GPU resources and high frame rates.
pub fn create_game_plugin() -> PluginMetadata {
    PluginBuilder::new()
        .id("com.studio.spacegame")
        .name("Space Explorer VR")
        .version("3.0.0")
        .description("Immersive 3D space exploration game designed for AR glasses")
        .author("Game Studio XR")
        .depends_on("com.xreal.physics-engine")
        .requires_engine("2.0.0")
        .icon("assets/game-icon.png")
        .surface_size(2560, 1440) // High resolution for immersion
        .supports_3d_rendering()
        .also_requires_keyboard()
        .also_supports_audio()
        .also_supports_compute_shaders()
        .also_supports_multi_window()
        .also_update_rate(60) // High frame rate for smooth gameplay
        .build()
}

/// Example 5: Media player with audio focus
///
/// Shows a plugin designed primarily for media consumption.
pub fn create_media_player() -> PluginMetadata {
    PluginBuilder::new()
        .id("com.media.xrealplayer")
        .name("XREAL Media Player")
        .version("1.0.5")
        .description("High-quality video and audio player with spatial audio support")
        .author("Media Solutions LLC")
        .icon("assets/media-icon.png")
        .surface_size(1920, 1080)
        .requires_network() // For streaming
        .also_supports_audio()
        .also_supports_file_system() // For local files
        .also_supports_transparency() // For UI overlays
        .also_update_rate(60) // Smooth video playback
        .build()
}

/// Example 6: Using the convenience macro for common patterns
///
/// The plugin_metadata! macro provides shortcuts for common plugin types.
pub fn create_plugins_with_macro() -> Vec<PluginMetadata> {
    vec![
        // Basic plugin
        plugin_metadata! {
            id: "com.example.basic",
            name: "Basic Plugin",
            basic
        },
        // Browser-like plugin
        plugin_metadata! {
            id: "com.example.browser",
            name: "Example Browser",
            browser_like
        },
        // Terminal-like plugin
        plugin_metadata! {
            id: "com.example.terminal",
            name: "Example Terminal",
            terminal_like
        },
        // Game-like plugin
        plugin_metadata! {
            id: "com.example.game",
            name: "Example Game",
            game_like
        },
    ]
}

/// Example 7: Extract capabilities for plugin implementation
///
/// Shows how to use the builder to define capabilities in your plugin's
/// capabilities() method for consistency.
pub struct ExamplePlugin;

impl crate::plugins::PluginApp for ExamplePlugin {
    fn id(&self) -> &str {
        "com.example.demo"
    }
    fn name(&self) -> &str {
        "Demo Plugin"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn capabilities(&self) -> PluginCapabilitiesFlags {
        use crate::plugins::PluginCapabilitiesFlags;

        PluginCapabilitiesFlags::new()
            .with_flag(PluginCapabilitiesFlags::REQUIRES_NETWORK_ACCESS)
            .with_flag(PluginCapabilitiesFlags::SUPPORTS_AUDIO)
            .with_flag(PluginCapabilitiesFlags::SUPPORTS_TRANSPARENCY)
    }

    // ... other required methods would be implemented here
    fn initialize(&mut self, _context: &crate::plugins::PluginContext) -> anyhow::Result<()> {
        Ok(())
    }
    fn render(&mut self, _context: &mut crate::plugins::RenderContext) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_input(&mut self, _event: &crate::plugins::InputEvent) -> anyhow::Result<bool> {
        Ok(false)
    }
    fn update(&mut self, _delta_time: f32) -> anyhow::Result<()> {
        Ok(())
    }
    fn resize(&mut self, _new_size: (u32, u32)) -> anyhow::Result<()> {
        Ok(())
    }
    fn shutdown(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Example 8: Compile-time safety demonstration
///
/// These examples show how the type system prevents common mistakes.
pub fn compile_time_safety_examples() {
    // ✅ This compiles - all required fields provided
    let _valid = PluginBuilder::new()
        .id("com.example.valid")
        .name("Valid Plugin")
        .supports_transparency()
        .build();

    // ❌ These would fail to compile if uncommented:

    // Missing name:
    // let _invalid = PluginBuilder::new()
    //     .id("com.example.invalid")
    //     .supports_transparency()
    //     .build(); // Compile error: name required

    // Missing capabilities:
    // let _invalid = PluginBuilder::new()
    //     .id("com.example.invalid")
    //     .name("Invalid Plugin")
    //     .build(); // Compile error: capabilities required

    // Missing ID:
    // let _invalid = PluginBuilder::new()
    //     .name("Invalid Plugin")
    //     .supports_transparency()
    //     .build(); // Compile error: id required
}

/// Example 9: Fluent API chaining
///
/// Shows how capabilities can be chained in any order for maximum flexibility.
pub fn capability_chaining_examples() -> Vec<PluginMetadata> {
    vec![
        // Start with network, add others
        PluginBuilder::new()
            .id("com.example.chain1")
            .name("Chain Example 1")
            .requires_network()
            .also_supports_audio()
            .also_requires_keyboard()
            .also_supports_3d_rendering()
            .build(),
        // Start with 3D, add others
        PluginBuilder::new()
            .id("com.example.chain2")
            .name("Chain Example 2")
            .supports_3d_rendering()
            .also_supports_compute_shaders()
            .also_supports_audio()
            .also_update_rate(60)
            .build(),
        // Mixed order - all valid
        PluginBuilder::new()
            .id("com.example.chain3")
            .name("Chain Example 3")
            .version("2.0.0")
            .supports_transparency()
            .author("Developer")
            .also_supports_file_system()
            .description("Mixed order example")
            .also_supports_multi_window()
            .build(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_plugin() {
        let plugin = create_minimal_plugin();
        assert_eq!(plugin.id, "com.example.minimal");
        assert_eq!(plugin.name, "Minimal Plugin");
        assert!(plugin.capabilities.supports_transparency);
    }

    #[test]
    fn test_browser_plugin() {
        let plugin = create_browser_plugin();
        assert_eq!(plugin.id, "com.company.webbrowser");
        assert!(plugin.capabilities.requires_network_access);
        assert!(plugin.capabilities.requires_keyboard_focus);
        assert!(plugin.capabilities.supports_audio);
        assert!(plugin.capabilities.supports_compute_shaders);
        assert_eq!(plugin.capabilities.preferred_update_rate, Some(60));
        assert_eq!(plugin.dependencies.len(), 2);
    }

    #[test]
    fn test_terminal_plugin() {
        let plugin = create_terminal_plugin();
        assert_eq!(plugin.id, "com.terminal.xreal");
        assert!(plugin.capabilities.requires_keyboard_focus);
        assert!(plugin.capabilities.supports_file_system);
        assert_eq!(plugin.capabilities.preferred_update_rate, Some(30));
        assert!(!plugin.capabilities.requires_network_access);
    }

    #[test]
    fn test_game_plugin() {
        let plugin = create_game_plugin();
        assert_eq!(plugin.id, "com.studio.spacegame");
        assert!(plugin.capabilities.supports_3d_rendering);
        assert!(plugin.capabilities.supports_compute_shaders);
        assert!(plugin.capabilities.supports_audio);
        assert_eq!(plugin.capabilities.preferred_update_rate, Some(60));
    }

    #[test]
    fn test_macro_plugins() {
        let plugins = create_plugins_with_macro();
        assert_eq!(plugins.len(), 4);

        // Check browser-like plugin
        let browser = &plugins[1];
        assert!(browser.capabilities.requires_network_access);
        assert!(browser.capabilities.requires_keyboard_focus);

        // Check terminal-like plugin
        let terminal = &plugins[2];
        assert!(terminal.capabilities.requires_keyboard_focus);
        assert!(terminal.capabilities.supports_file_system);
    }

    #[test]
    fn test_capabilities_extraction() {
        let plugin = ExamplePlugin;
        let caps = plugin.capabilities();
        assert!(caps.requires_network_access);
        assert!(caps.supports_audio);
        assert!(caps.supports_transparency);
        assert_eq!(caps.preferred_update_rate, Some(30));
    }
}
