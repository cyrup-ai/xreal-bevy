//! XREAL Virtual Desktop Library
//!
//! This library provides the core functionality for the XREAL virtual desktop application,
//! including plugin system, input handling, tracking, and rendering capabilities.

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender};

// Include all modules that need to be available for both binary and library
pub mod capture;
pub mod cursor;
pub mod driver;
pub mod input;
pub mod plugins;
pub mod render;
pub mod setup;
pub mod state;
pub mod tracking;
pub mod ui;
pub mod usb_debug;
pub mod xreal_stereo;

// Re-export commonly used types
pub use capture::ScreenCaptures;
pub use tracking::{CalibrationState, Command, Data, Orientation};
pub use ui::state::*;

// Re-export AppState for setup.rs - define here to avoid import conflicts
#[derive(
    States, Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum AppState {
    #[default]
    Startup,
    ChecksFailed,
    Running,
}

// Define shared types that are used across modules
#[derive(Resource)]
pub struct CommandChannel(pub Sender<tracking::Command>);

#[derive(Resource)]
pub struct DataChannel(pub Sender<tracking::Data>, pub Receiver<tracking::Data>);

#[derive(Resource, Default)]
pub struct ScreenDistance(pub f32);

#[derive(Resource, Default)]
pub struct DisplayModeState {
    pub is_3d_enabled: bool,
    pub pending_change: Option<bool>,
}

#[derive(Resource, Default)]
pub struct RollLockState {
    pub is_enabled: bool,
    pub pending_change: Option<bool>,
}

#[derive(Resource, Default)]
pub struct BrightnessState {
    pub current_level: u8,
    pub pending_change: Option<u8>,
}

#[derive(Resource, Default)]
pub struct FrameCounter {
    pub count: u64,
}
