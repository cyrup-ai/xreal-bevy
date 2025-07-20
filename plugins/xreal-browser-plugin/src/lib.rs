//! XREAL Browser Plugin
//! 
//! A production-quality browser plugin for XREAL AR glasses providing webview integration
//! with WGPU rendering, input handling, and navigation capabilities.
//! 
//! This plugin follows Bevy's Plugin trait architecture for seamless integration
//! with the XREAL Bevy application.

pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

// Re-export the main plugin and key types for easy access
pub use plugin::BrowserPlugin;
pub use resources::{BrowserState, BrowserConfig, NavigationHistory};
pub use components::{BrowserEntity, BrowserSurface, BrowserInput};

/// Browser plugin capabilities and feature flags
pub mod capabilities {
    use serde::{Deserialize, Serialize};

    /// Browser plugin capability flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct BrowserCapabilities(u32);

    impl BrowserCapabilities {
        pub const NONE: Self = Self(0);
        pub const WEBVIEW: Self = Self(1 << 0);
        pub const NAVIGATION: Self = Self(1 << 1);
        pub const INPUT_HANDLING: Self = Self(1 << 2);
        pub const TRANSPARENCY: Self = Self(1 << 3);
        pub const AUDIO: Self = Self(1 << 4);
        pub const NETWORK_ACCESS: Self = Self(1 << 5);
        pub const KEYBOARD_FOCUS: Self = Self(1 << 6);
        pub const MULTI_WINDOW: Self = Self(1 << 7);

        /// Create new capabilities with no flags set
        #[inline(always)]
        pub const fn new() -> Self {
            Self::NONE
        }

        /// Add a capability flag
        #[inline(always)]
        pub const fn with_flag(self, flag: Self) -> Self {
            Self(self.0 | flag.0)
        }

        /// Check if capability is supported
        #[inline(always)]
        pub const fn contains(self, other: Self) -> bool {
            (self.0 & other.0) == other.0
        }

        /// Get default browser capabilities
        #[inline(always)]
        pub const fn default_capabilities() -> Self {
            Self::WEBVIEW
                .with_flag(Self::NAVIGATION)
                .with_flag(Self::INPUT_HANDLING)
                .with_flag(Self::TRANSPARENCY)
                .with_flag(Self::AUDIO)
                .with_flag(Self::NETWORK_ACCESS)
                .with_flag(Self::KEYBOARD_FOCUS)
        }
    }

    impl Default for BrowserCapabilities {
        #[inline(always)]
        fn default() -> Self {
            Self::default_capabilities()
        }
    }
}

/// Browser plugin error types
pub mod error {
    use std::fmt;

    /// Browser plugin error types
    #[derive(Debug, Clone)]
    pub enum BrowserError {
        /// Navigation failed
        NavigationFailed(String),
        /// Rendering failed
        RenderingFailed(String),
        /// Input handling failed
        InputFailed(String),
        /// Resource loading failed
        ResourceLoadFailed(String),
        /// Configuration error
        ConfigError(String),
        /// Network error
        NetworkError(String),
    }

    impl fmt::Display for BrowserError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                BrowserError::NavigationFailed(msg) => write!(f, "Navigation failed: {}", msg),
                BrowserError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
                BrowserError::InputFailed(msg) => write!(f, "Input handling failed: {}", msg),
                BrowserError::ResourceLoadFailed(msg) => write!(f, "Resource loading failed: {}", msg),
                BrowserError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
                BrowserError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            }
        }
    }

    impl std::error::Error for BrowserError {}

    /// Browser plugin result type
    pub type BrowserResult<T> = Result<T, BrowserError>;
}

/// Browser plugin prelude for convenient imports
pub mod prelude {
    pub use crate::{
        BrowserPlugin,
        components::*,
        resources::*,
        capabilities::*,
        error::*,
    };
}