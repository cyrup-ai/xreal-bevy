//! Tests for state schema system
//!
//! Extracted from src/state/schema/mod.rs to maintain clean separation
//! between source code and test code following Rust best practices.

use xreal_virtual_desktop::state::schema::*;

#[test]
fn test_app_state_creation() {
    let state = AppState::new();
    assert_eq!(state.schema_version, STATE_SCHEMA_VERSION);
    assert!(state.is_compatible());
}

#[test]
fn test_app_state_validation() {
    let state = AppState::default();
    assert!(state.validate().is_ok());
}

#[test]
fn test_user_preferences_validation() {
    let mut prefs = UserPreferences::default();
    assert!(prefs.validate().is_ok());

    // Test invalid screen distance
    prefs.screen_distance = -100.0;
    assert!(prefs.validate().is_err());

    // Test invalid brightness
    prefs.screen_distance = -5.0; // Reset to valid
    prefs.brightness_level = 10;
    assert!(prefs.validate().is_err());
}

#[test]
fn test_performance_settings_validation() {
    let mut perf = PerformanceSettings::default();
    assert!(perf.validate().is_ok());

    // Test invalid target FPS
    perf.target_fps = 300;
    assert!(perf.validate().is_err());
}

#[test]
fn test_state_merge() {
    let mut state1 = AppState::default();
    let mut state2 = AppState::default();
    
    state2.user_preferences.brightness_level = 6;
    state2.touch();
    
    assert!(state1.merge(&state2).is_ok());
    assert_eq!(state1.user_preferences.brightness_level, 6);
}

#[test]
fn test_serialization() {
    let state = AppState::default();
    
    // Test JSON serialization
    let json_bytes = core::serialization::to_json_bytes(&state).expect("Failed to serialize state");
    let deserialized = core::serialization::from_json_bytes(&json_bytes).expect("Failed to deserialize state");
    
    assert_eq!(state.schema_version, deserialized.schema_version);
    assert_eq!(state.user_preferences.brightness_level, deserialized.user_preferences.brightness_level);
}

#[tokio::test]
async fn test_file_operations() {
    let state = AppState::default();
    let temp_path = std::env::temp_dir().join("test_state.json");
    
    // Test save and load
    core::serialization::save_to_file(&state, &temp_path).await.expect("Failed to save state");
    let loaded_state = core::serialization::load_from_file(&temp_path).await.expect("Failed to load state");
    
    assert_eq!(state.schema_version, loaded_state.schema_version);
    
    // Cleanup
    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn test_calibration_data_validation() {
    let mut cal = CalibrationData::default();
    assert!(cal.validate().is_ok());

    // Test invalid quality score
    cal.quality_score = 1.5;
    assert!(cal.validate().is_err());

    // Test invalid temperature
    cal.quality_score = 0.8; // Reset to valid
    cal.temperature_celsius = 100.0;
    assert!(cal.validate().is_err());
}

#[test]
fn test_plugin_config_validation() {
    let config = PluginConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_window_layout_validation() {
    let layout = WindowLayout::default();
    assert!(layout.validate().is_ok());
}

#[test]
fn test_input_config_validation() {
    let mut input = InputConfig::default();
    assert!(input.validate().is_ok());

    // Test invalid dwell time
    input.gaze_input.dwell_time_ms = 10000;
    assert!(input.validate().is_err());
}

#[test]
fn test_audio_settings_validation() {
    let mut audio = AudioSettings::default();
    assert!(audio.validate().is_ok());

    // Test invalid master volume
    audio.master_volume = 1.5;
    assert!(audio.validate().is_err());
}

#[test]
fn test_network_config_validation() {
    let mut network = NetworkConfig::default();
    assert!(network.validate().is_ok());

    // Test invalid timeout
    network.connection_timeout_secs = 0;
    assert!(network.validate().is_err());
}

#[test]
fn test_security_settings_validation() {
    let security = SecuritySettings::default();
    assert!(security.validate().is_ok());
}