//! Tests for builder demo examples
//!
//! Extracted from src/plugins/examples/builder_demo.rs to maintain clean separation
//! between source code and test code following Rust best practices.//! Tests for builder demo examples
//!
//! Extracted from src/plugins/examples/builder_demo.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use xreal_virtual_desktop::plugins::examples::builder_demo::*;

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
fn test_terminal_plugin() {    let plugin = create_terminal_plugin();
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
fn test_capabilities_extraction() {    let plugin = ExamplePlugin;
    let caps = plugin.capabilities();
    assert!(caps.requires_network_access);
    assert!(caps.supports_audio);
    assert!(caps.supports_transparency);
    assert_eq!(caps.preferred_update_rate, Some(30));
}