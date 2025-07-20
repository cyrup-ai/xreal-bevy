//! Tests for plugin builder system
//!
//! Extracted from src/plugins/builder/mod.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use xreal_virtual_desktop::plugins::builder::*;
use xreal_virtual_desktop::plugins::{PluginCapabilitiesFlags, PluginCapabilities};

#[test]
fn test_basic_plugin_builder() {
    let result = PluginBuilder::new()
        .id("com.test.basic")
        .expect("Failed to set id")
        .name("Test Plugin")
        .expect("Failed to set name")
        .version("1.0.0")
        .expect("Failed to set version")
        .description("A test plugin")
        .expect("Failed to set description")
        .author("Test Author")
        .expect("Failed to set author")
        .basic_capabilities()
        .build();

    assert!(result.is_ok());
    let plugin = result.expect("Failed to build plugin");
    assert_eq!(plugin.id, "com.test.basic");
    assert_eq!(plugin.name, "Test Plugin");
    assert_eq!(plugin.version, "1.0.0");
}

#[test]
fn test_multimedia_plugin_builder() {
    let result = PluginBuilder::multimedia()
        .id("com.test.multimedia")
        .expect("Failed to set id")
        .name("Multimedia Plugin")
        .expect("Failed to set name")
        .version("2.0.0")
        .expect("Failed to set version")
        .description("A multimedia test plugin")
        .expect("Failed to set description")
        .author("Multimedia Team")
        .expect("Failed to set author")
        .dependency("com.test.audio")
        .expect("Failed to add dependency")
        .high_performance_surface()
        .build();

    assert!(result.is_ok());
    let plugin = result.expect("Failed to build plugin");
    assert_eq!(plugin.dependencies.len(), 1);
    assert_eq!(plugin.dependencies[0], "com.test.audio");
    assert!(plugin.surface_requirements.is_some());
}

#[test]
fn test_network_plugin_builder() {
    let result = PluginBuilder::network()
        .id("com.test.network")
        .expect("Failed to set id")
        .name("Network Plugin")
        .expect("Failed to set name")
        .version("1.5.0")
        .expect("Failed to set version")
        .description("A network test plugin")
        .expect("Failed to set description")
        .author("Network Team")
        .expect("Failed to set author")
        .dependencies(["com.test.http", "com.test.security"])
        .expect("Failed to set dependencies")
        .minimum_engine_version("0.3.0")
        .expect("Failed to set minimum engine version")
        .build();

    assert!(result.is_ok());
    let plugin = result.expect("Failed to build plugin");
    assert_eq!(plugin.dependencies.len(), 2);
    assert!(plugin.dependencies.contains(&"com.test.http".to_string()));
    assert!(plugin.dependencies.contains(&"com.test.security".to_string()));
    assert_eq!(plugin.minimum_engine_version, "0.3.0");
}

#[test]
fn test_builder_validation() {
    // Test invalid ID
    let result = PluginBuilder::new()
        .id("")
        .expect_err("Empty ID should fail");
    assert!(matches!(result, PluginBuilderError::InvalidId(_)));

    // Test invalid name
    let result = PluginBuilder::new()
        .id("com.test.valid")
        .expect("Failed to set valid id")
        .name("")
        .expect_err("Empty name should fail");
    assert!(matches!(result, PluginBuilderError::InvalidName(_)));

    // Test invalid version
    let result = PluginBuilder::new()
        .id("com.test.valid")
        .expect("Failed to set valid id")
        .name("Valid Name")
        .expect("Failed to set valid name")
        .version("")
        .expect_err("Empty version should fail");
    assert!(matches!(result, PluginBuilderError::InvalidVersion(_)));
}

#[test]
fn test_incomplete_builder() {
    // Builder without required fields should not compile to Complete state
    let incomplete = PluginBuilder::new()
        .version("1.0.0")
        .expect("Failed to set version")
        .description("Test")
        .expect("Failed to set description")
        .author("Test Author")
        .expect("Failed to set author");

    // try_complete should fail with missing fields
    let result = incomplete.try_complete();
    assert!(result.is_err());
    assert!(matches!(result.expect_err("Should fail with missing fields"), PluginBuilderError::MissingField(_)));
}

#[test]
fn test_dependency_management() {
    let mut builder = PluginBuilder::new()
        .id("com.test.deps")
        .expect("Failed to set id")
        .name("Dependency Test")
        .expect("Failed to set name")
        .basic_capabilities();

    // Add dependencies
    builder = builder.dependency("com.test.dep1").expect("Failed to add dependency");
    builder = builder.dependency("com.test.dep2").expect("Failed to add dependency");
    
    // Add duplicate dependency (should not duplicate)
    builder = builder.dependency("com.test.dep1").expect("Failed to add dependency");
    
    assert_eq!(builder.get_dependencies().len(), 2);
    
    // Remove dependency
    builder = builder.remove_dependency("com.test.dep1");
    assert_eq!(builder.get_dependencies().len(), 1);
    assert_eq!(builder.get_dependencies()[0], "com.test.dep2");
    
    // Clear all dependencies
    builder = builder.clear_dependencies();
    assert_eq!(builder.get_dependencies().len(), 0);
}

#[test]
fn test_surface_requirements() {
    let plugin = PluginBuilder::new()
        .id("com.test.surface")
        .expect("Failed to set id")
        .name("Surface Test")
        .expect("Failed to set name")
        .basic_capabilities()
        .basic_surface()
        .build()
        .expect("Failed to build plugin");

    assert!(plugin.surface_requirements.is_some());
    let requirements = plugin.surface_requirements.expect("Surface requirements should be set");
    assert_eq!(requirements.width, 1920);
    assert_eq!(requirements.height, 1080);
}

#[test]
fn test_icon_path() {
    let plugin = PluginBuilder::new()
        .id("com.test.icon")
        .expect("Failed to set id")
        .name("Icon Test")
        .expect("Failed to set name")
        .basic_capabilities()
        .icon("assets/test-icon.png")
        .build()
        .expect("Failed to build plugin");

    assert!(plugin.icon_path.is_some());
    assert_eq!(plugin.icon_path.expect("Icon path should be set").to_string_lossy(), "assets/test-icon.png");
}

#[test]
fn test_validation_functions() {
    // Test ID validation
    assert!(validation::quick::is_valid_id("com.test.valid"));
    assert!(!validation::quick::is_valid_id(""));
    assert!(!validation::quick::is_valid_id("invalid id"));

    // Test name validation
    assert!(validation::quick::is_valid_name("Valid Name"));
    assert!(!validation::quick::is_valid_name(""));

    // Test version validation
    assert!(validation::quick::is_valid_version("1.0.0"));
    assert!(validation::quick::is_valid_version("2.1.3-beta"));
    assert!(!validation::quick::is_valid_version(""));
    assert!(!validation::quick::is_valid_version("invalid"));
}

#[test]
fn test_capabilities_validation() {
    let valid_caps = PluginCapabilities {
        flags: PluginCapabilitiesFlags::RENDERING,
        max_memory_mb: 256,
        max_cpu_percent: 10.0,
        requires_network: false,
        requires_filesystem: false,
        requires_audio: false,
        requires_input: false,
    };
    assert!(validation::quick::is_valid_capabilities(&valid_caps));

    let invalid_caps = PluginCapabilities {
        flags: PluginCapabilitiesFlags::RENDERING,
        max_memory_mb: 0, // Invalid: zero memory
        max_cpu_percent: 10.0,
        requires_network: false,
        requires_filesystem: false,
        requires_audio: false,
        requires_input: false,
    };
    assert!(!validation::quick::is_valid_capabilities(&invalid_caps));
}

#[test]
fn test_complete_validation() {
    let plugin = PluginBuilder::new()
        .id("com.test.complete")
        .expect("Failed to set id")
        .name("Complete Test")
        .expect("Failed to set name")
        .version("1.0.0")
        .expect("Failed to set version")
        .description("A complete test plugin")
        .expect("Failed to set description")
        .author("Test Author")
        .expect("Failed to set author")
        .basic_capabilities()
        .build()
        .expect("Failed to build plugin");

    // Validate the complete plugin metadata
    assert!(validation::PluginValidator::validate_metadata(&plugin).is_ok());
}