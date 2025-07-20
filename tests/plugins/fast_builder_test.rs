//! Tests for fast plugin builder
//!
//! Extracted from src/plugins/fast_builder.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use xreal_virtual_desktop::plugins::fast_builder::*;

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