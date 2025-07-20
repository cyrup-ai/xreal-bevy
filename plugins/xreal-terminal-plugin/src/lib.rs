//! XREAL Terminal Plugin
//! 
//! A production-quality terminal plugin for XREAL AR glasses providing PTY integration,
//! command execution, and terminal emulation with full ANSI color support.
//! 
//! This plugin follows Bevy's Plugin trait architecture for seamless integration
//! with the XREAL Bevy application.

pub mod color_scheme;
pub use tracing::{debug, error, info, trace, warn};
pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

// Re-export the main plugin and key types for easy access
pub use plugin::TerminalPlugin;
pub use color_scheme::{TerminalColorScheme, AnsiColor};
pub use resources::{TerminalState, TerminalConfig, TerminalGrid};
pub use components::{TerminalEntity, TerminalSurface, TerminalInput};

/// Terminal plugin capabilities and feature flags
pub mod capabilities {
    use serde::{Deserialize, Serialize};

    /// Terminal plugin capability flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TerminalCapabilities(u32);

    impl TerminalCapabilities {
        pub const NONE: Self = Self(0);
        pub const PTY_SUPPORT: Self = Self(1 << 0);
        pub const ANSI_COLORS: Self = Self(1 << 1);
        pub const INPUT_HANDLING: Self = Self(1 << 2);
        pub const SCROLLBACK: Self = Self(1 << 3);
        pub const COPY_PASTE: Self = Self(1 << 4);
        pub const SEARCH: Self = Self(1 << 5);
        pub const TRANSPARENCY: Self = Self(1 << 6);
        pub const FONT_SCALING: Self = Self(1 << 7);
        pub const MULTI_SESSION: Self = Self(1 << 8);

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

        /// Get default terminal capabilities
        #[inline(always)]
        pub const fn default_capabilities() -> Self {
            Self::PTY_SUPPORT
                .with_flag(Self::ANSI_COLORS)
                .with_flag(Self::INPUT_HANDLING)
                .with_flag(Self::SCROLLBACK)
                .with_flag(Self::COPY_PASTE)
                .with_flag(Self::TRANSPARENCY)
                .with_flag(Self::FONT_SCALING)
        }
    }

    impl Default for TerminalCapabilities {
        #[inline(always)]
        fn default() -> Self {
            Self::default_capabilities()
        }
    }
}

/// Terminal plugin error types
pub mod error {
    use std::fmt;

    /// Terminal plugin error types
    #[derive(Debug, Clone)]
    pub enum TerminalError {
        /// PTY creation failed
        PtyCreationFailed(String),
        /// Command execution failed
        CommandFailed(String),
        /// Input handling failed
        InputFailed(String),
        /// Rendering failed
        RenderingFailed(String),
        /// Configuration error
        ConfigError(String),
        /// ANSI parsing error
        AnsiParseError(String),
        /// Font loading error
        FontError(String),
    }

    impl fmt::Display for TerminalError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                TerminalError::PtyCreationFailed(msg) => write!(f, "PTY creation failed: {}", msg),
                TerminalError::CommandFailed(msg) => write!(f, "Command execution failed: {}", msg),
                TerminalError::InputFailed(msg) => write!(f, "Input handling failed: {}", msg),
                TerminalError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
                TerminalError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
                TerminalError::AnsiParseError(msg) => write!(f, "ANSI parsing error: {}", msg),
                TerminalError::FontError(msg) => write!(f, "Font loading error: {}", msg),
            }
        }
    }

    impl std::error::Error for TerminalError {}

    /// Terminal plugin result type
    pub type TerminalResult<T> = Result<T, TerminalError>;
}

/// Terminal plugin prelude for convenient imports
pub mod prelude {
    pub use crate::{
        TerminalPlugin,
        components::*,
        resources::*,
        color_scheme::*,
        capabilities::*,
        error::*,
    };
}