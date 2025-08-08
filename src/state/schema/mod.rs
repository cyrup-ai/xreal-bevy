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

pub mod audio;
pub mod calibration;
pub mod core;
pub mod input;
pub mod network;
pub mod performance;
pub mod plugins;
pub mod preferences;
pub mod security;
pub mod ui;
pub mod window;

// Re-export commonly used types for convenience
pub use core::{StateMigration, StateValidation, STATE_SCHEMA_VERSION};

pub use preferences::{
    AccessibilitySettings, AppearanceSettings, ColorBlindType, ComfortSettings, PrivacySettings,
    UserPreferences,
};

pub use ui::{
    NotificationPosition, NotificationSettings, PanelConfig, PanelConfigs, ToolbarButtons,
    ToolbarPosition, ToolbarSize, ToolbarState, UiState, WindowPositions, WindowRect,
};

pub use calibration::{CalibrationData, CalibrationState};

pub use plugins::{PluginConfig, PluginPermissions, PluginSystemState, ResourceLimits};

pub use performance::{
    AntiAliasingSettings, AntiAliasingType, PerformanceSettings, PerformanceThresholds,
    RenderQuality, ShadowQuality, ShadowSettings, TextureQuality, TextureSettings,
};

pub use window::{
    DisplayConfig, MonitorArrangement, MultiMonitorConfig, VirtualScreenConfig, WindowLayout,
    WindowManagementSettings,
};

pub use input::{
    GazeInputSettings, GestureInputSettings, InputConfig, SensitivitySettings, VoiceInputSettings,
};

pub use audio::{AudioDeviceConfig, AudioEffectsSettings, AudioSettings, SpatialAudioSettings};

pub use network::{NetworkConfig, ProxySettings, ProxyType, SslSettings, TlsVersion};

pub use security::{
    AccessControlSettings, AuthenticationMethod, AuthenticationSettings, SecurityLevel,
    SecuritySettings,
};
