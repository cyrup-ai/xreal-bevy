//! Complete state schema with versioned serialization support for XREAL application
//!
//! This module provides the complete application state schema with modular organization
//! for better maintainability. All state components are designed for atomic operations,
//! validation, and seamless serialization/deserialization.
//!
//! # Architecture
//!
//! The schema is decomposed into logical submodules:
//!
//! - [`core`]: Core schema definitions, versioning, and serialization utilities
//! - [`preferences`]: User preference settings and accessibility options
//! - [`ui`]: UI state, window positions, and interface configuration
//! - [`calibration`]: IMU calibration data and sensor fusion settings
//! - [`plugins`]: Plugin system state and configuration
//! - [`performance`]: Performance settings and rendering configuration
//! - [`window`]: Window layout and display management
//! - [`input`]: Input system configuration and sensitivity settings
//! - [`audio`]: Audio settings and spatial audio configuration
//! - [`network`]: Network configuration and proxy settings
//! - [`security`]: Security settings and access control
//!
//! # Usage Examples
//!
//! ```rust
//! use state::schema::{AppState, STATE_SCHEMA_VERSION};
//!
//! // Create a new application state
//! let mut state = AppState::new();
//!
//! // Validate the state
//! state.validate()?;
//!
//! // Save to file
//! state::schema::core::serialization::save_to_file(&state, &path).await?;
//!
//! // Load from file with validation and migration
//! let loaded_state = state::schema::core::serialization::load_from_file(&path).await?;
//! let migrated_state = state::schema::core::serialization::validate_and_migrate(loaded_state)?;
//! ```

pub mod core;
pub mod preferences;
pub mod ui;
pub mod calibration;
pub mod plugins;
pub mod performance;
pub mod window;
pub mod input;
pub mod audio;
pub mod network;
pub mod security;

// Re-export commonly used types for convenience
pub use core::{
    AppState, StateValidation, StateMigration, STATE_SCHEMA_VERSION,
};

pub use preferences::{
    UserPreferences, ComfortSettings, AccessibilitySettings, ColorBlindType,
    PrivacySettings, AppearanceSettings,
};

pub use ui::{
    UiState, WindowPositions, WindowRect, PanelConfigs, PanelConfig,
    ToolbarState, ToolbarPosition, ToolbarSize, ToolbarButtons,
    NotificationSettings, NotificationPosition,
};

pub use calibration::{
    CalibrationData, CalibrationState,
};

pub use plugins::{
    PluginSystemState, PluginConfig, ResourceLimits, PluginPermissions,
};

pub use performance::{
    PerformanceSettings, RenderQuality, AntiAliasingSettings, AntiAliasingType,
    ShadowSettings, ShadowQuality, TextureSettings, TextureQuality,
    PerformanceThresholds,
};

pub use window::{
    WindowLayout, DisplayConfig, VirtualScreenConfig, MultiMonitorConfig,
    MonitorArrangement, WindowManagementSettings,
};

pub use input::{
    InputConfig, GazeInputSettings, GestureInputSettings, VoiceInputSettings,
    SensitivitySettings,
};

pub use audio::{
    AudioSettings, AudioDeviceConfig, SpatialAudioSettings, AudioEffectsSettings,
};

pub use network::{
    NetworkConfig, ProxySettings, ProxyType, SslSettings, TlsVersion,
};

pub use security::{
    SecuritySettings, AuthenticationSettings, AuthenticationMethod,
    AccessControlSettings, SecurityLevel,
};

