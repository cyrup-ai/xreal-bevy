//! Window layout schema for XREAL application display management
//!
//! This module provides window and display structures with validation and
//! serialization support for the XREAL application state system.

use super::core::StateValidation;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Window layout and positioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    /// Primary display configuration
    pub primary_display: DisplayConfig,
    /// Virtual screen configuration
    pub virtual_screen: VirtualScreenConfig,
    /// Multi-monitor setup
    pub multi_monitor: MultiMonitorConfig,
    /// Window management settings
    pub window_management: WindowManagementSettings,
}

impl Default for WindowLayout {
    fn default() -> Self {
        Self {
            primary_display: DisplayConfig::default(),
            virtual_screen: VirtualScreenConfig::default(),
            multi_monitor: MultiMonitorConfig::default(),
            window_management: WindowManagementSettings::default(),
        }
    }
}

impl StateValidation for WindowLayout {
    fn validate(&self) -> Result<()> {
        self.primary_display.validate()?;
        self.virtual_screen.validate()?;
        self.multi_monitor.validate()?;
        self.window_management.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.primary_display.merge(&other.primary_display)?;
        self.virtual_screen.merge(&other.virtual_screen)?;
        self.multi_monitor.merge(&other.multi_monitor)?;
        self.window_management.merge(&other.window_management)?;
        Ok(())
    }
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Display resolution
    pub resolution: [u32; 2],
    /// Refresh rate in Hz
    pub refresh_rate: u32,
    /// Color depth in bits
    pub color_depth: u32,
    /// HDR enabled
    pub hdr_enabled: bool,
    /// Gamma correction
    pub gamma: f32,
    /// Brightness adjustment
    pub brightness: f32,
    /// Contrast adjustment
    pub contrast: f32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            resolution: [1920, 1080],
            refresh_rate: 90,
            color_depth: 32,
            hdr_enabled: false,
            gamma: 2.2,
            brightness: 1.0,
            contrast: 1.0,
        }
    }
}

impl StateValidation for DisplayConfig {
    fn validate(&self) -> Result<()> {
        // Validate resolution
        if self.resolution[0] < 640 || self.resolution[1] < 480 {
            anyhow::bail!(
                "Resolution too low: {}x{}",
                self.resolution[0],
                self.resolution[1]
            );
        }
        if self.resolution[0] > 7680 || self.resolution[1] > 4320 {
            anyhow::bail!(
                "Resolution too high: {}x{}",
                self.resolution[0],
                self.resolution[1]
            );
        }

        // Validate refresh rate
        if self.refresh_rate < 30 || self.refresh_rate > 240 {
            anyhow::bail!("Refresh rate out of range: {}", self.refresh_rate);
        }

        // Validate color depth
        if ![16, 24, 32].contains(&self.color_depth) {
            anyhow::bail!("Invalid color depth: {}", self.color_depth);
        }

        // Validate gamma
        if self.gamma < 1.0 || self.gamma > 3.0 {
            anyhow::bail!("Gamma out of range: {}", self.gamma);
        }

        // Validate brightness and contrast
        if self.brightness < 0.1 || self.brightness > 2.0 {
            anyhow::bail!("Brightness out of range: {}", self.brightness);
        }
        if self.contrast < 0.1 || self.contrast > 2.0 {
            anyhow::bail!("Contrast out of range: {}", self.contrast);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.resolution = other.resolution;
        self.refresh_rate = other.refresh_rate;
        self.color_depth = other.color_depth;
        self.hdr_enabled = other.hdr_enabled;
        self.gamma = other.gamma;
        self.brightness = other.brightness;
        self.contrast = other.contrast;
        Ok(())
    }
}

/// Virtual screen configuration for XREAL glasses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualScreenConfig {
    /// Virtual screen size in inches
    pub screen_size_inches: f32,
    /// Distance from user in meters
    pub distance_meters: f32,
    /// Screen curvature (0.0 = flat, 1.0 = full curve)
    pub curvature: f32,
    /// Screen tilt angle in degrees
    pub tilt_angle: f32,
    /// Stereoscopic 3D enabled
    pub stereo_3d: bool,
    /// Inter-pupillary distance in mm
    pub ipd_mm: f32,
    /// Eye relief in mm
    pub eye_relief_mm: f32,
}

impl Default for VirtualScreenConfig {
    fn default() -> Self {
        Self {
            screen_size_inches: 130.0,
            distance_meters: 5.0,
            curvature: 0.1,
            tilt_angle: 0.0,
            stereo_3d: true,
            ipd_mm: 63.0,
            eye_relief_mm: 12.0,
        }
    }
}

impl StateValidation for VirtualScreenConfig {
    fn validate(&self) -> Result<()> {
        // Validate screen size
        if self.screen_size_inches < 50.0 || self.screen_size_inches > 300.0 {
            anyhow::bail!("Screen size out of range: {}", self.screen_size_inches);
        }

        // Validate distance
        if self.distance_meters < 1.0 || self.distance_meters > 50.0 {
            anyhow::bail!("Distance out of range: {}", self.distance_meters);
        }

        // Validate curvature
        if self.curvature < 0.0 || self.curvature > 1.0 {
            anyhow::bail!("Curvature out of range: {}", self.curvature);
        }

        // Validate tilt angle
        if self.tilt_angle < -45.0 || self.tilt_angle > 45.0 {
            anyhow::bail!("Tilt angle out of range: {}", self.tilt_angle);
        }

        // Validate IPD
        if self.ipd_mm < 50.0 || self.ipd_mm > 80.0 {
            anyhow::bail!("IPD out of range: {}", self.ipd_mm);
        }

        // Validate eye relief
        if self.eye_relief_mm < 5.0 || self.eye_relief_mm > 25.0 {
            anyhow::bail!("Eye relief out of range: {}", self.eye_relief_mm);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.screen_size_inches = other.screen_size_inches;
        self.distance_meters = other.distance_meters;
        self.curvature = other.curvature;
        self.tilt_angle = other.tilt_angle;
        self.stereo_3d = other.stereo_3d;
        self.ipd_mm = other.ipd_mm;
        self.eye_relief_mm = other.eye_relief_mm;
        Ok(())
    }
}

/// Multi-monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMonitorConfig {
    /// Multi-monitor enabled
    pub enabled: bool,
    /// Monitor arrangement
    pub arrangement: MonitorArrangement,
    /// Primary monitor index
    pub primary_monitor: u32,
    /// Bezel compensation enabled
    pub bezel_compensation: bool,
    /// Bezel width in pixels
    pub bezel_width_px: u32,
}

impl Default for MultiMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            arrangement: MonitorArrangement::Horizontal,
            primary_monitor: 0,
            bezel_compensation: true,
            bezel_width_px: 10,
        }
    }
}

impl StateValidation for MultiMonitorConfig {
    fn validate(&self) -> Result<()> {
        // Validate bezel width
        if self.bezel_width_px > 100 {
            anyhow::bail!("Bezel width too large: {}", self.bezel_width_px);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.arrangement = other.arrangement;
        self.primary_monitor = other.primary_monitor;
        self.bezel_compensation = other.bezel_compensation;
        self.bezel_width_px = other.bezel_width_px;
        Ok(())
    }
}

/// Monitor arrangement options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MonitorArrangement {
    Horizontal,
    Vertical,
    Grid,
    Custom,
}

impl Default for MonitorArrangement {
    fn default() -> Self {
        Self::Horizontal
    }
}

/// Window management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowManagementSettings {
    /// Auto-arrange windows
    pub auto_arrange: bool,
    /// Snap to edges
    pub snap_to_edges: bool,
    /// Snap threshold in pixels
    pub snap_threshold_px: u32,
    /// Window transparency enabled
    pub transparency_enabled: bool,
    /// Always on top for certain windows
    pub always_on_top: bool,
    /// Window animations enabled
    pub animations_enabled: bool,
    /// Animation duration in ms
    pub animation_duration_ms: u32,
}

impl Default for WindowManagementSettings {
    fn default() -> Self {
        Self {
            auto_arrange: true,
            snap_to_edges: true,
            snap_threshold_px: 20,
            transparency_enabled: true,
            always_on_top: false,
            animations_enabled: true,
            animation_duration_ms: 200,
        }
    }
}

impl StateValidation for WindowManagementSettings {
    fn validate(&self) -> Result<()> {
        // Validate snap threshold
        if self.snap_threshold_px > 100 {
            anyhow::bail!("Snap threshold too large: {}", self.snap_threshold_px);
        }

        // Validate animation duration
        if self.animation_duration_ms < 50 || self.animation_duration_ms > 2000 {
            anyhow::bail!(
                "Animation duration out of range: {}",
                self.animation_duration_ms
            );
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.auto_arrange = other.auto_arrange;
        self.snap_to_edges = other.snap_to_edges;
        self.snap_threshold_px = other.snap_threshold_px;
        self.transparency_enabled = other.transparency_enabled;
        self.always_on_top = other.always_on_top;
        self.animations_enabled = other.animations_enabled;
        self.animation_duration_ms = other.animation_duration_ms;
        Ok(())
    }
}
