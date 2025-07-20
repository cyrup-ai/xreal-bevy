//! Tests for fast data structures in plugin system
//!
//! Extracted from src/plugins/fast_data/mod.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use xreal_virtual_desktop::plugins::fast_data::*;

#[test]
fn test_small_string_basic_operations() {
    let mut s = SmallString::<32>::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
    assert_eq!(s.remaining_capacity(), 32);

    s.push_str("hello").expect("Failed to push string");
    assert_eq!(s.as_str(), "hello");
    assert_eq!(s.len(), 5);
    assert_eq!(s.remaining_capacity(), 27);

    s.push(' ').expect("Failed to push character");
    s.push_str("world").expect("Failed to push string");
    assert_eq!(s.as_str(), "hello world");
    assert_eq!(s.len(), 11);
}

#[test]
fn test_ring_buffer_operations() {
    let buffer = RingBuffer::<i32, 4>::new();
    assert!(buffer.is_empty());
    assert_eq!(buffer.capacity(), 4);

    buffer.push(1).expect("Failed to push to buffer");
    buffer.push(2).expect("Failed to push to buffer");
    buffer.push(3).expect("Failed to push to buffer");
    assert_eq!(buffer.len(), 3);
    assert!(!buffer.is_full());

    buffer.push(4).expect("Failed to push to buffer");
    assert!(buffer.is_full());
    assert!(buffer.push(5).is_err()); // Should fail when full

    assert_eq!(buffer.pop().expect("Failed to pop from buffer"), 1);
    assert_eq!(buffer.pop().expect("Failed to pop from buffer"), 2);
    assert_eq!(buffer.len(), 2);
}

#[test]
fn test_plugin_metadata_builder() {
    let metadata = PluginMetadata::builder()
        .name("test-plugin").expect("Failed to set name")
        .version("1.0.0").expect("Failed to set version")
        .description("A test plugin").expect("Failed to set description")
        .author("Test Author").expect("Failed to set author")
        .category("test").expect("Failed to set category")
        .capability("testing").expect("Failed to add capability")
        .dependency("core").expect("Failed to add dependency")
        .config("enabled", "true").expect("Failed to add config")
        .build().expect("Failed to build metadata");

    assert_eq!(metadata.name(), "test-plugin");
    assert_eq!(metadata.version(), "1.0.0");
    assert_eq!(metadata.description(), "A test plugin");
    assert_eq!(metadata.author(), "Test Author");
    assert_eq!(metadata.category(), "test");
    assert_eq!(metadata.capability_count(), 1);
    assert_eq!(metadata.dependency_count(), 1);
    assert!(metadata.has_capability("testing"));
    assert!(metadata.has_dependency("core"));
    assert_eq!(metadata.get_config("enabled").expect("Failed to get config"), "true");
}

#[test]
fn test_performance_metrics() {
    let metrics = PerformanceMetrics::with_all_metrics();
    
    // Test operation tracking
    {
        let _tracker = metrics.start_operation().success();
        // Tracker automatically records on drop
    }
    
    // Test manual recording
    metrics.record_success(1000); // 1μs
    metrics.record_failure();
    
    let summary = metrics.summary();
    assert_eq!(summary.total_operations, 3); // 1 from tracker + 1 success + 1 failure
    assert!(summary.success_rate > 0.0);
    assert!(summary.operations_per_second >= 0.0);
}

#[test]
fn test_plugin_registry() {
    let mut registry = PluginRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.capacity(), 256);

    let metadata = PluginMetadata::builder()
        .name("test-plugin").expect("Failed to set name")
        .version("1.0.0").expect("Failed to set version")
        .build().expect("Failed to build metadata");

    registry.register(metadata).expect("Failed to register plugin");
    assert_eq!(registry.registered_count(), 1);
    assert!(registry.contains("test-plugin"));

    let entry = registry.get("test-plugin").expect("Failed to get plugin");
    assert_eq!(entry.name(), "test-plugin");
    assert_eq!(entry.state, PluginState::Registered);

    registry.set_state("test-plugin", PluginState::Running).expect("Failed to set state");
    assert_eq!(registry.running_count(), 1);

    let stats = registry.stats();
    assert_eq!(stats.registered_count, 1);
    assert_eq!(stats.running_count, 1);
    assert!(stats.utilization_percentage() > 0.0);
}