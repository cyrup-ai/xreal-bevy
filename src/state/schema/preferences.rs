//! User preferences schema for XREAL application settings
//!
//! This module provides user preference structures with validation and
//! serialization support for the XREAL application state system.

use super::core::StateValidation;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// User preference settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Virtual screen distance from user
    pub screen_distance: f32,
    /// 3D stereoscopic display mode enabled
    pub display_mode_3d: bool,
    /// Roll lock enabled for head tracking
    pub roll_lock_enabled: bool,
    /// Brightness level (0-7)
    pub brightness_level: u8,
    /// Auto-brightness enabled
    pub auto_brightness: bool,
    /// Comfort settings
    pub comfort_settings: ComfortSettings,
    /// Accessibility settings
    pub accessibility_settings: AccessibilitySettings,
    /// Privacy settings
    pub privacy_settings: PrivacySettings,
    /// Theme and appearance settings
    pub appearance_settings: AppearanceSettings,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            screen_distance: -5.0,
            display_mode_3d: true,
            roll_lock_enabled: false,
            brightness_level: 4,
            auto_brightness: false,
            comfort_settings: ComfortSettings::default(),
            accessibility_settings: AccessibilitySettings::default(),
            privacy_settings: PrivacySettings::default(),
            appearance_settings: AppearanceSettings::default(),
        }
    }
}

impl StateValidation for UserPreferences {
    fn validate(&self) -> Result<()> {
        // Validate screen distance
        if self.screen_distance < -50.0 || self.screen_distance > 50.0 {
            anyhow::bail!("Screen distance out of range: {}", self.screen_distance);
        }

        // Validate brightness level
        if self.brightness_level > 7 {
            anyhow::bail!("Brightness level out of range: {}", self.brightness_level);
        }

        // Validate sub-components
        self.comfort_settings.validate()?;
        self.accessibility_settings.validate()?;
        self.privacy_settings.validate()?;
        self.appearance_settings.validate()?;

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        // Merge primitive fields (prefer other's values)
        self.screen_distance = other.screen_distance;
        self.display_mode_3d = other.display_mode_3d;
        self.roll_lock_enabled = other.roll_lock_enabled;
        self.brightness_level = other.brightness_level;
        self.auto_brightness = other.auto_brightness;

        // Merge complex fields
        self.comfort_settings.merge(&other.comfort_settings)?;
        self.accessibility_settings
            .merge(&other.accessibility_settings)?;
        self.privacy_settings.merge(&other.privacy_settings)?;
        self.appearance_settings.merge(&other.appearance_settings)?;

        Ok(())
    }
}

/// Comfort settings for user experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfortSettings {
    /// Motion sickness reduction enabled
    pub motion_sickness_reduction: bool,
    /// Comfort vignette enabled
    pub comfort_vignette: bool,
    /// Snap turning enabled
    pub snap_turning: bool,
    /// Snap turn angle in degrees
    pub snap_turn_angle: f32,
    /// Smooth locomotion enabled
    pub smooth_locomotion: bool,
    /// Locomotion speed multiplier
    pub locomotion_speed: f32,
    /// Eye strain reduction enabled
    pub eye_strain_reduction: bool,
    /// Blue light filter strength (0.0-1.0)
    pub blue_light_filter: f32,
}

impl Default for ComfortSettings {
    fn default() -> Self {
        Self {
            motion_sickness_reduction: true,
            comfort_vignette: true,
            snap_turning: false,
            snap_turn_angle: 30.0,
            smooth_locomotion: true,
            locomotion_speed: 1.0,
            eye_strain_reduction: true,
            blue_light_filter: 0.2,
        }
    }
}

impl StateValidation for ComfortSettings {
    fn validate(&self) -> Result<()> {
        // Validate snap turn angle
        if self.snap_turn_angle < 5.0 || self.snap_turn_angle > 90.0 {
            anyhow::bail!("Snap turn angle out of range: {}", self.snap_turn_angle);
        }

        // Validate locomotion speed
        if self.locomotion_speed < 0.1 || self.locomotion_speed > 5.0 {
            anyhow::bail!("Locomotion speed out of range: {}", self.locomotion_speed);
        }

        // Validate blue light filter
        if self.blue_light_filter < 0.0 || self.blue_light_filter > 1.0 {
            anyhow::bail!("Blue light filter out of range: {}", self.blue_light_filter);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.motion_sickness_reduction = other.motion_sickness_reduction;
        self.comfort_vignette = other.comfort_vignette;
        self.snap_turning = other.snap_turning;
        self.snap_turn_angle = other.snap_turn_angle;
        self.smooth_locomotion = other.smooth_locomotion;
        self.locomotion_speed = other.locomotion_speed;
        self.eye_strain_reduction = other.eye_strain_reduction;
        self.blue_light_filter = other.blue_light_filter;
        Ok(())
    }
}

/// Accessibility settings for inclusive design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// High contrast mode enabled
    pub high_contrast: bool,
    /// Large text mode enabled
    pub large_text: bool,
    /// Text scaling factor
    pub text_scale: f32,
    /// Color blind assistance enabled
    pub color_blind_assistance: bool,
    /// Color blind type
    pub color_blind_type: ColorBlindType,
    /// Audio descriptions enabled
    pub audio_descriptions: bool,
    /// Haptic feedback enabled
    pub haptic_feedback: bool,
    /// Haptic intensity (0.0-1.0)
    pub haptic_intensity: f32,
    /// Voice commands enabled
    pub voice_commands: bool,
    /// Gesture navigation enabled
    pub gesture_navigation: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            high_contrast: false,
            large_text: false,
            text_scale: 1.0,
            color_blind_assistance: false,
            color_blind_type: ColorBlindType::None,
            audio_descriptions: false,
            haptic_feedback: true,
            haptic_intensity: 0.7,
            voice_commands: false,
            gesture_navigation: true,
        }
    }
}

impl StateValidation for AccessibilitySettings {
    fn validate(&self) -> Result<()> {
        // Validate text scale
        if self.text_scale < 0.5 || self.text_scale > 3.0 {
            anyhow::bail!("Text scale out of range: {}", self.text_scale);
        }

        // Validate haptic intensity
        if self.haptic_intensity < 0.0 || self.haptic_intensity > 1.0 {
            anyhow::bail!("Haptic intensity out of range: {}", self.haptic_intensity);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.high_contrast = other.high_contrast;
        self.large_text = other.large_text;
        self.text_scale = other.text_scale;
        self.color_blind_assistance = other.color_blind_assistance;
        self.color_blind_type = other.color_blind_type;
        self.audio_descriptions = other.audio_descriptions;
        self.haptic_feedback = other.haptic_feedback;
        self.haptic_intensity = other.haptic_intensity;
        self.voice_commands = other.voice_commands;
        self.gesture_navigation = other.gesture_navigation;
        Ok(())
    }
}

/// Color blind assistance types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ColorBlindType {
    /// No color blindness
    None,
    /// Protanopia (red-blind)
    Protanopia,
    /// Deuteranopia (green-blind)
    Deuteranopia,
    /// Tritanopia (blue-blind)
    Tritanopia,
    /// Protanomaly (red-weak)
    Protanomaly,
    /// Deuteranomaly (green-weak)
    Deuteranomaly,
    /// Tritanomaly (blue-weak)
    Tritanomaly,
}

impl Default for ColorBlindType {
    fn default() -> Self {
        Self::None
    }
}

/// Privacy settings for data protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Analytics data collection enabled
    pub analytics_enabled: bool,
    /// Crash reporting enabled
    pub crash_reporting: bool,
    /// Usage statistics collection enabled
    pub usage_statistics: bool,
    /// Personalization data collection enabled
    pub personalization_data: bool,
    /// Location data collection enabled
    pub location_data: bool,
    /// Biometric data collection enabled
    pub biometric_data: bool,
    /// Data retention period in days
    pub data_retention_days: u32,
    /// Automatic data deletion enabled
    pub auto_delete_data: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            analytics_enabled: false,
            crash_reporting: true,
            usage_statistics: false,
            personalization_data: false,
            location_data: false,
            biometric_data: false,
            data_retention_days: 90,
            auto_delete_data: true,
        }
    }
}

impl StateValidation for PrivacySettings {
    fn validate(&self) -> Result<()> {
        // Validate data retention period
        if self.data_retention_days < 1 || self.data_retention_days > 3650 {
            anyhow::bail!(
                "Data retention period out of range: {}",
                self.data_retention_days
            );
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.analytics_enabled = other.analytics_enabled;
        self.crash_reporting = other.crash_reporting;
        self.usage_statistics = other.usage_statistics;
        self.personalization_data = other.personalization_data;
        self.location_data = other.location_data;
        self.biometric_data = other.biometric_data;
        self.data_retention_days = other.data_retention_days;
        self.auto_delete_data = other.auto_delete_data;
        Ok(())
    }
}

/// Appearance and theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Current theme name
    pub theme: String,
    /// Dark mode enabled
    pub dark_mode: bool,
    /// Custom accent color (RGB)
    pub accent_color: [u8; 3],
    /// UI animation speed (0.0-2.0)
    pub animation_speed: f32,
    /// UI transparency (0.0-1.0)
    pub ui_transparency: f32,
    /// Font family
    pub font_family: String,
    /// Font size multiplier
    pub font_size_multiplier: f32,
    /// Show advanced settings
    pub show_advanced: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "cyrup_dark".to_string(),
            dark_mode: true,
            accent_color: [0, 150, 255], // CYRUP blue
            animation_speed: 1.0,
            ui_transparency: 0.95,
            font_family: "Inter".to_string(),
            font_size_multiplier: 1.0,
            show_advanced: false,
        }
    }
}

impl StateValidation for AppearanceSettings {
    fn validate(&self) -> Result<()> {
        // Validate animation speed
        if self.animation_speed < 0.0 || self.animation_speed > 2.0 {
            anyhow::bail!("Animation speed out of range: {}", self.animation_speed);
        }

        // Validate UI transparency
        if self.ui_transparency < 0.0 || self.ui_transparency > 1.0 {
            anyhow::bail!("UI transparency out of range: {}", self.ui_transparency);
        }

        // Validate font size multiplier
        if self.font_size_multiplier < 0.5 || self.font_size_multiplier > 3.0 {
            anyhow::bail!(
                "Font size multiplier out of range: {}",
                self.font_size_multiplier
            );
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.theme = other.theme.clone();
        self.dark_mode = other.dark_mode;
        self.accent_color = other.accent_color;
        self.animation_speed = other.animation_speed;
        self.ui_transparency = other.ui_transparency;
        self.font_family = other.font_family.clone();
        self.font_size_multiplier = other.font_size_multiplier;
        self.show_advanced = other.show_advanced;
        Ok(())
    }
}
