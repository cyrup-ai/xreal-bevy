//! UI state schema for XREAL application interface
//!
//! This module provides UI state structures with validation and
//! serialization support for the XREAL application state system.

use super::core::StateValidation;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// UI state and layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    /// Settings panel open state
    pub settings_panel_open: bool,
    /// Debug panel open state
    pub debug_panel_open: bool,
    /// Performance overlay visible
    pub performance_overlay: bool,
    /// UI scale factor
    pub ui_scale: f32,
    /// Window positions and sizes
    pub window_positions: WindowPositions,
    /// Panel configurations
    pub panel_configs: PanelConfigs,
    /// Toolbar state
    pub toolbar_state: ToolbarState,
    /// Notification settings
    pub notification_settings: NotificationSettings,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            settings_panel_open: false,
            debug_panel_open: false,
            performance_overlay: false,
            ui_scale: 1.0,
            window_positions: WindowPositions::default(),
            panel_configs: PanelConfigs::default(),
            toolbar_state: ToolbarState::default(),
            notification_settings: NotificationSettings::default(),
        }
    }
}

impl StateValidation for UiState {
    fn validate(&self) -> Result<()> {
        // Validate UI scale
        if self.ui_scale < 0.5 || self.ui_scale > 3.0 {
            anyhow::bail!("UI scale out of range: {}", self.ui_scale);
        }

        // Validate sub-components
        self.window_positions.validate()?;
        self.panel_configs.validate()?;
        self.toolbar_state.validate()?;
        self.notification_settings.validate()?;

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        // Merge primitive fields
        self.settings_panel_open = other.settings_panel_open;
        self.debug_panel_open = other.debug_panel_open;
        self.performance_overlay = other.performance_overlay;
        self.ui_scale = other.ui_scale;

        // Merge complex fields
        self.window_positions.merge(&other.window_positions)?;
        self.panel_configs.merge(&other.panel_configs)?;
        self.toolbar_state.merge(&other.toolbar_state)?;
        self.notification_settings
            .merge(&other.notification_settings)?;

        Ok(())
    }
}

/// Window positions and sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPositions {
    /// Main window position
    pub main_window: WindowRect,
    /// Settings window position
    pub settings_window: WindowRect,
    /// Debug window position
    pub debug_window: WindowRect,
    /// Plugin manager window position
    pub plugin_manager_window: WindowRect,
}

impl Default for WindowPositions {
    fn default() -> Self {
        Self {
            main_window: WindowRect::default(),
            settings_window: WindowRect {
                x: 100.0,
                y: 100.0,
                width: 400.0,
                height: 600.0,
            },
            debug_window: WindowRect {
                x: 200.0,
                y: 200.0,
                width: 500.0,
                height: 400.0,
            },
            plugin_manager_window: WindowRect {
                x: 150.0,
                y: 150.0,
                width: 600.0,
                height: 500.0,
            },
        }
    }
}

impl StateValidation for WindowPositions {
    fn validate(&self) -> Result<()> {
        self.main_window.validate()?;
        self.settings_window.validate()?;
        self.debug_window.validate()?;
        self.plugin_manager_window.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.main_window.merge(&other.main_window)?;
        self.settings_window.merge(&other.settings_window)?;
        self.debug_window.merge(&other.debug_window)?;
        self.plugin_manager_window
            .merge(&other.plugin_manager_window)?;
        Ok(())
    }
}

/// Window rectangle definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRect {
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Default for WindowRect {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }
}

impl StateValidation for WindowRect {
    fn validate(&self) -> Result<()> {
        // Validate dimensions
        if self.width <= 0.0 || self.height <= 0.0 {
            anyhow::bail!("Invalid window dimensions: {}x{}", self.width, self.height);
        }

        // Validate reasonable size limits
        if self.width > 10000.0 || self.height > 10000.0 {
            anyhow::bail!(
                "Window dimensions too large: {}x{}",
                self.width,
                self.height
            );
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.x = other.x;
        self.y = other.y;
        self.width = other.width;
        self.height = other.height;
        Ok(())
    }
}

/// Panel configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfigs {
    /// Settings panel configuration
    pub settings_panel: PanelConfig,
    /// Debug panel configuration
    pub debug_panel: PanelConfig,
    /// Performance panel configuration
    pub performance_panel: PanelConfig,
    /// Plugin panel configuration
    pub plugin_panel: PanelConfig,
}

impl Default for PanelConfigs {
    fn default() -> Self {
        Self {
            settings_panel: PanelConfig::default(),
            debug_panel: PanelConfig::default(),
            performance_panel: PanelConfig::default(),
            plugin_panel: PanelConfig::default(),
        }
    }
}

impl StateValidation for PanelConfigs {
    fn validate(&self) -> Result<()> {
        self.settings_panel.validate()?;
        self.debug_panel.validate()?;
        self.performance_panel.validate()?;
        self.plugin_panel.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.settings_panel.merge(&other.settings_panel)?;
        self.debug_panel.merge(&other.debug_panel)?;
        self.performance_panel.merge(&other.performance_panel)?;
        self.plugin_panel.merge(&other.plugin_panel)?;
        Ok(())
    }
}

/// Individual panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Panel is visible
    pub visible: bool,
    /// Panel is docked
    pub docked: bool,
    /// Panel opacity (0.0-1.0)
    pub opacity: f32,
    /// Panel size
    pub size: [f32; 2],
    /// Panel position
    pub position: [f32; 2],
    /// Panel is resizable
    pub resizable: bool,
    /// Panel is movable
    pub movable: bool,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            visible: false,
            docked: true,
            opacity: 0.95,
            size: [300.0, 400.0],
            position: [0.0, 0.0],
            resizable: true,
            movable: true,
        }
    }
}

impl StateValidation for PanelConfig {
    fn validate(&self) -> Result<()> {
        // Validate opacity
        if self.opacity < 0.0 || self.opacity > 1.0 {
            anyhow::bail!("Panel opacity out of range: {}", self.opacity);
        }

        // Validate size
        if self.size[0] <= 0.0 || self.size[1] <= 0.0 {
            anyhow::bail!("Invalid panel size: {:?}", self.size);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.visible = other.visible;
        self.docked = other.docked;
        self.opacity = other.opacity;
        self.size = other.size;
        self.position = other.position;
        self.resizable = other.resizable;
        self.movable = other.movable;
        Ok(())
    }
}

/// Toolbar state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarState {
    /// Toolbar is visible
    pub visible: bool,
    /// Toolbar position
    pub position: ToolbarPosition,
    /// Toolbar size
    pub size: ToolbarSize,
    /// Auto-hide enabled
    pub auto_hide: bool,
    /// Auto-hide delay in milliseconds
    pub auto_hide_delay: u32,
    /// Toolbar buttons configuration
    pub buttons: ToolbarButtons,
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self {
            visible: true,
            position: ToolbarPosition::Bottom,
            size: ToolbarSize::Medium,
            auto_hide: false,
            auto_hide_delay: 3000,
            buttons: ToolbarButtons::default(),
        }
    }
}

impl StateValidation for ToolbarState {
    fn validate(&self) -> Result<()> {
        // Validate auto-hide delay
        if self.auto_hide_delay < 100 || self.auto_hide_delay > 30000 {
            anyhow::bail!("Auto-hide delay out of range: {}", self.auto_hide_delay);
        }

        self.buttons.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.visible = other.visible;
        self.position = other.position;
        self.size = other.size;
        self.auto_hide = other.auto_hide;
        self.auto_hide_delay = other.auto_hide_delay;
        self.buttons.merge(&other.buttons)?;
        Ok(())
    }
}

/// Toolbar position options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToolbarPosition {
    Top,
    Bottom,
    Left,
    Right,
    Floating,
}

impl Default for ToolbarPosition {
    fn default() -> Self {
        Self::Bottom
    }
}

/// Toolbar size options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToolbarSize {
    Small,
    Medium,
    Large,
}

impl Default for ToolbarSize {
    fn default() -> Self {
        Self::Medium
    }
}

/// Toolbar buttons configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarButtons {
    /// Settings button visible
    pub settings: bool,
    /// Debug button visible
    pub debug: bool,
    /// Performance button visible
    pub performance: bool,
    /// Plugin manager button visible
    pub plugin_manager: bool,
    /// Calibration button visible
    pub calibration: bool,
    /// Help button visible
    pub help: bool,
}

impl Default for ToolbarButtons {
    fn default() -> Self {
        Self {
            settings: true,
            debug: false,
            performance: false,
            plugin_manager: true,
            calibration: true,
            help: true,
        }
    }
}

impl StateValidation for ToolbarButtons {
    fn validate(&self) -> Result<()> {
        // No specific validation needed for boolean flags
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.settings = other.settings;
        self.debug = other.debug;
        self.performance = other.performance;
        self.plugin_manager = other.plugin_manager;
        self.calibration = other.calibration;
        self.help = other.help;
        Ok(())
    }
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Notifications enabled
    pub enabled: bool,
    /// Notification position
    pub position: NotificationPosition,
    /// Notification duration in milliseconds
    pub duration: u32,
    /// Maximum number of notifications
    pub max_notifications: u32,
    /// Sound enabled
    pub sound_enabled: bool,
    /// Animation enabled
    pub animation_enabled: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            position: NotificationPosition::TopRight,
            duration: 5000,
            max_notifications: 5,
            sound_enabled: true,
            animation_enabled: true,
        }
    }
}

impl StateValidation for NotificationSettings {
    fn validate(&self) -> Result<()> {
        // Validate duration
        if self.duration < 1000 || self.duration > 60000 {
            anyhow::bail!("Notification duration out of range: {}", self.duration);
        }

        // Validate max notifications
        if self.max_notifications < 1 || self.max_notifications > 20 {
            anyhow::bail!("Max notifications out of range: {}", self.max_notifications);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.position = other.position;
        self.duration = other.duration;
        self.max_notifications = other.max_notifications;
        self.sound_enabled = other.sound_enabled;
        self.animation_enabled = other.animation_enabled;
        Ok(())
    }
}

/// Notification position options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::TopRight
    }
}
