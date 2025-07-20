//! Terminal color scheme configuration
//!
//! This module provides comprehensive ANSI color support and terminal color scheme
//! management for the XREAL terminal plugin.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// ANSI color enumeration for terminal text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnsiColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl AnsiColor {
    /// Get all standard ANSI colors
    #[inline(always)]
    pub const fn all_colors() -> [AnsiColor; 16] {
        [
            AnsiColor::Black,
            AnsiColor::Red,
            AnsiColor::Green,
            AnsiColor::Yellow,
            AnsiColor::Blue,
            AnsiColor::Magenta,
            AnsiColor::Cyan,
            AnsiColor::White,
            AnsiColor::BrightBlack,
            AnsiColor::BrightRed,
            AnsiColor::BrightGreen,
            AnsiColor::BrightYellow,
            AnsiColor::BrightBlue,
            AnsiColor::BrightMagenta,
            AnsiColor::BrightCyan,
            AnsiColor::BrightWhite,
        ]
    }

    /// Convert ANSI color to index
    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Create ANSI color from index
    #[inline(always)]
    pub const fn from_index(index: usize) -> Option<AnsiColor> {
        match index {
            0 => Some(AnsiColor::Black),
            1 => Some(AnsiColor::Red),
            2 => Some(AnsiColor::Green),
            3 => Some(AnsiColor::Yellow),
            4 => Some(AnsiColor::Blue),
            5 => Some(AnsiColor::Magenta),
            6 => Some(AnsiColor::Cyan),
            7 => Some(AnsiColor::White),
            8 => Some(AnsiColor::BrightBlack),
            9 => Some(AnsiColor::BrightRed),
            10 => Some(AnsiColor::BrightGreen),
            11 => Some(AnsiColor::BrightYellow),
            12 => Some(AnsiColor::BrightBlue),
            13 => Some(AnsiColor::BrightMagenta),
            14 => Some(AnsiColor::BrightCyan),
            15 => Some(AnsiColor::BrightWhite),
            _ => None,
        }
    }
}

/// Terminal color scheme configuration with full ANSI support
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct TerminalColorScheme {
    /// Background color (RGBA)
    pub background: [f32; 4],
    /// Default foreground color (RGBA)
    pub foreground: [f32; 4],
    /// Cursor color (RGBA)
    pub cursor: [f32; 4],
    /// Selection background color (RGBA)
    pub selection: [f32; 4],
    /// ANSI color palette (16 colors)
    pub ansi_colors: [[f32; 4]; 16],
    /// Bold text brightness multiplier
    pub bold_brightness: f32,
    /// Dim text brightness multiplier
    pub dim_brightness: f32,
}

impl TerminalColorScheme {
    /// Create a new terminal color scheme with default colors
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 1.0], // Black
            foreground: [1.0, 1.0, 1.0, 1.0], // White
            cursor: [0.5, 0.5, 0.5, 1.0],     // Gray
            selection: [0.2, 0.4, 0.8, 0.3],  // Blue with alpha
            ansi_colors: [
                [0.0, 0.0, 0.0, 1.0], // Black
                [0.8, 0.0, 0.0, 1.0], // Red
                [0.0, 0.8, 0.0, 1.0], // Green
                [0.8, 0.8, 0.0, 1.0], // Yellow
                [0.0, 0.0, 0.8, 1.0], // Blue
                [0.8, 0.0, 0.8, 1.0], // Magenta
                [0.0, 0.8, 0.8, 1.0], // Cyan
                [0.8, 0.8, 0.8, 1.0], // White
                [0.4, 0.4, 0.4, 1.0], // Bright Black (Gray)
                [1.0, 0.4, 0.4, 1.0], // Bright Red
                [0.4, 1.0, 0.4, 1.0], // Bright Green
                [1.0, 1.0, 0.4, 1.0], // Bright Yellow
                [0.4, 0.4, 1.0, 1.0], // Bright Blue
                [1.0, 0.4, 1.0, 1.0], // Bright Magenta
                [0.4, 1.0, 1.0, 1.0], // Bright Cyan
                [1.0, 1.0, 1.0, 1.0], // Bright White
            ],
            bold_brightness: 1.2,
            dim_brightness: 0.7,
        }
    }

    /// Create a dark theme color scheme
    #[inline(always)]
    pub fn dark_theme() -> Self {
        Self {
            background: [0.1, 0.1, 0.1, 1.0], // Dark gray
            foreground: [0.9, 0.9, 0.9, 1.0], // Light gray
            cursor: [0.0, 1.0, 0.0, 1.0],     // Green cursor
            selection: [0.3, 0.3, 0.6, 0.4],  // Dark blue selection
            ..Self::new()
        }
    }

    /// Create a light theme color scheme
    #[inline(always)]
    pub fn light_theme() -> Self {
        Self {
            background: [0.95, 0.95, 0.95, 1.0], // Light gray
            foreground: [0.1, 0.1, 0.1, 1.0],    // Dark gray
            cursor: [0.0, 0.0, 1.0, 1.0],        // Blue cursor
            selection: [0.7, 0.8, 1.0, 0.4],     // Light blue selection
            ansi_colors: [
                [0.2, 0.2, 0.2, 1.0], // Black (darker for light theme)
                [0.6, 0.0, 0.0, 1.0], // Red (darker)
                [0.0, 0.6, 0.0, 1.0], // Green (darker)
                [0.6, 0.6, 0.0, 1.0], // Yellow (darker)
                [0.0, 0.0, 0.6, 1.0], // Blue (darker)
                [0.6, 0.0, 0.6, 1.0], // Magenta (darker)
                [0.0, 0.6, 0.6, 1.0], // Cyan (darker)
                [0.6, 0.6, 0.6, 1.0], // White (darker)
                [0.4, 0.4, 0.4, 1.0], // Bright Black
                [0.8, 0.2, 0.2, 1.0], // Bright Red
                [0.2, 0.8, 0.2, 1.0], // Bright Green
                [0.8, 0.8, 0.2, 1.0], // Bright Yellow
                [0.2, 0.2, 0.8, 1.0], // Bright Blue
                [0.8, 0.2, 0.8, 1.0], // Bright Magenta
                [0.2, 0.8, 0.8, 1.0], // Bright Cyan
                [0.8, 0.8, 0.8, 1.0], // Bright White
            ],
            bold_brightness: 1.1,
            dim_brightness: 0.8,
        }
    }

    /// Create a high contrast color scheme for accessibility
    #[inline(always)]
    pub fn high_contrast() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 1.0], // Pure black
            foreground: [1.0, 1.0, 1.0, 1.0], // Pure white
            cursor: [1.0, 1.0, 0.0, 1.0],     // Yellow cursor
            selection: [1.0, 1.0, 1.0, 0.3],  // White selection
            ansi_colors: [
                [0.0, 0.0, 0.0, 1.0], // Black
                [1.0, 0.0, 0.0, 1.0], // Red (pure)
                [0.0, 1.0, 0.0, 1.0], // Green (pure)
                [1.0, 1.0, 0.0, 1.0], // Yellow (pure)
                [0.0, 0.0, 1.0, 1.0], // Blue (pure)
                [1.0, 0.0, 1.0, 1.0], // Magenta (pure)
                [0.0, 1.0, 1.0, 1.0], // Cyan (pure)
                [1.0, 1.0, 1.0, 1.0], // White (pure)
                [0.5, 0.5, 0.5, 1.0], // Bright Black (gray)
                [1.0, 0.5, 0.5, 1.0], // Bright Red
                [0.5, 1.0, 0.5, 1.0], // Bright Green
                [1.0, 1.0, 0.5, 1.0], // Bright Yellow
                [0.5, 0.5, 1.0, 1.0], // Bright Blue
                [1.0, 0.5, 1.0, 1.0], // Bright Magenta
                [0.5, 1.0, 1.0, 1.0], // Bright Cyan
                [1.0, 1.0, 1.0, 1.0], // Bright White
            ],
            bold_brightness: 1.0, // No brightness change for high contrast
            dim_brightness: 0.6,
        }
    }

    /// Create a retro green terminal color scheme
    #[inline(always)]
    pub fn retro_green() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 1.0],   // Black
            foreground: [0.0, 1.0, 0.0, 1.0],   // Green
            cursor: [0.0, 1.0, 0.0, 1.0],       // Green cursor
            selection: [0.0, 0.5, 0.0, 0.3],    // Dark green selection
            ansi_colors: [
                [0.0, 0.0, 0.0, 1.0],   // Black
                [0.0, 0.8, 0.0, 1.0],   // Red -> Green
                [0.0, 1.0, 0.0, 1.0],   // Green
                [0.0, 0.9, 0.0, 1.0],   // Yellow -> Light Green
                [0.0, 0.7, 0.0, 1.0],   // Blue -> Dark Green
                [0.0, 0.8, 0.0, 1.0],   // Magenta -> Green
                [0.0, 0.9, 0.0, 1.0],   // Cyan -> Light Green
                [0.0, 1.0, 0.0, 1.0],   // White -> Green
                [0.0, 0.4, 0.0, 1.0],   // Bright Black -> Very Dark Green
                [0.0, 0.9, 0.0, 1.0],   // Bright Red -> Light Green
                [0.0, 1.0, 0.0, 1.0],   // Bright Green
                [0.2, 1.0, 0.2, 1.0],   // Bright Yellow -> Very Light Green
                [0.0, 0.8, 0.0, 1.0],   // Bright Blue -> Green
                [0.0, 0.9, 0.0, 1.0],   // Bright Magenta -> Light Green
                [0.2, 1.0, 0.2, 1.0],   // Bright Cyan -> Very Light Green
                [0.5, 1.0, 0.5, 1.0],   // Bright White -> Pale Green
            ],
            bold_brightness: 1.3,
            dim_brightness: 0.6,
        }
    }

    /// Get color for ANSI color index
    #[inline(always)]
    pub fn get_ansi_color(&self, color: AnsiColor) -> [f32; 4] {
        self.ansi_colors[color.to_index()]
    }

    /// Set color for ANSI color index
    #[inline(always)]
    pub fn set_ansi_color(&mut self, color: AnsiColor, rgba: [f32; 4]) {
        self.ansi_colors[color.to_index()] = rgba;
    }

    /// Get foreground color with brightness modifier
    #[inline(always)]
    pub fn get_foreground_with_brightness(&self, bold: bool, dim: bool) -> [f32; 4] {
        let mut color = self.foreground;
        if bold {
            for i in 0..3 {
                color[i] = (color[i] * self.bold_brightness).min(1.0);
            }
        } else if dim {
            for i in 0..3 {
                color[i] *= self.dim_brightness;
            }
        }
        color
    }

    /// Get ANSI color with brightness modifier
    #[inline(always)]
    pub fn get_ansi_color_with_brightness(&self, color: AnsiColor, bold: bool, dim: bool) -> [f32; 4] {
        let mut rgba = self.get_ansi_color(color);
        if bold {
            for i in 0..3 {
                rgba[i] = (rgba[i] * self.bold_brightness).min(1.0);
            }
        } else if dim {
            for i in 0..3 {
                rgba[i] *= self.dim_brightness;
            }
        }
        rgba
    }

    /// Convert to Bevy Color
    #[inline(always)]
    pub fn to_bevy_color(rgba: [f32; 4]) -> Color {
        Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3])
    }

    /// Get background as Bevy Color
    #[inline(always)]
    pub fn background_color(&self) -> Color {
        Self::to_bevy_color(self.background)
    }

    /// Get foreground as Bevy Color
    #[inline(always)]
    pub fn foreground_color(&self) -> Color {
        Self::to_bevy_color(self.foreground)
    }

    /// Get cursor as Bevy Color
    #[inline(always)]
    pub fn cursor_color(&self) -> Color {
        Self::to_bevy_color(self.cursor)
    }

    /// Get selection as Bevy Color
    #[inline(always)]
    pub fn selection_color(&self) -> Color {
        Self::to_bevy_color(self.selection)
    }

    /// Get ANSI color as Bevy Color
    #[inline(always)]
    pub fn ansi_bevy_color(&self, color: AnsiColor) -> Color {
        Self::to_bevy_color(self.get_ansi_color(color))
    }
}

impl Default for TerminalColorScheme {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined color scheme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSchemeVariant {
    Default,
    Dark,
    Light,
    HighContrast,
    RetroGreen,
}

impl ColorSchemeVariant {
    /// Get all available color scheme variants
    #[inline(always)]
    pub const fn all_variants() -> [ColorSchemeVariant; 5] {
        [
            ColorSchemeVariant::Default,
            ColorSchemeVariant::Dark,
            ColorSchemeVariant::Light,
            ColorSchemeVariant::HighContrast,
            ColorSchemeVariant::RetroGreen,
        ]
    }

    /// Get the name of the color scheme variant
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        match self {
            ColorSchemeVariant::Default => "Default",
            ColorSchemeVariant::Dark => "Dark",
            ColorSchemeVariant::Light => "Light",
            ColorSchemeVariant::HighContrast => "High Contrast",
            ColorSchemeVariant::RetroGreen => "Retro Green",
        }
    }

    /// Create a terminal color scheme from this variant
    #[inline(always)]
    pub fn to_color_scheme(self) -> TerminalColorScheme {
        match self {
            ColorSchemeVariant::Default => TerminalColorScheme::new(),
            ColorSchemeVariant::Dark => TerminalColorScheme::dark_theme(),
            ColorSchemeVariant::Light => TerminalColorScheme::light_theme(),
            ColorSchemeVariant::HighContrast => TerminalColorScheme::high_contrast(),
            ColorSchemeVariant::RetroGreen => TerminalColorScheme::retro_green(),
        }
    }
}

impl Default for ColorSchemeVariant {
    #[inline(always)]
    fn default() -> Self {
        ColorSchemeVariant::Default
    }
}