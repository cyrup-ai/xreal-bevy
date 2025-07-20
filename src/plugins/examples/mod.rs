//! Example plugin implementations demonstrating the XREAL plugin system
//!
//! These examples show how to create plugins that integrate with the XREAL
//! virtual desktop environment using WGPU surface rendering.

pub mod browser;
pub mod builder_demo;
pub mod fast_demo;
pub mod terminal;
pub mod utils;

// Plugin exports for system usage (used internally by plugin initialization)
pub(crate) use browser::XRealBrowserPlugin;
pub(crate) use terminal::{TerminalColorScheme, XRealTerminalPlugin};
