//! Input configuration schema for XREAL application
//!
//! This module provides input system structures with validation and
//! serialization support for the XREAL application state system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::core::StateValidation;

/// Input system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Gaze input settings
    pub gaze_input: GazeInputSettings,
    /// Gesture input settings
    pub gesture_input: GestureInputSettings,
    /// Voice input settings
    pub voice_input: VoiceInputSettings,
    /// Keyboard shortcuts
    pub keyboard_shortcuts: HashMap<String, String>,
    /// Input sensitivity settings
    pub sensitivity: SensitivitySettings,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            gaze_input: GazeInputSettings::default(),
            gesture_input: GestureInputSettings::default(),
            voice_input: VoiceInputSettings::default(),
            keyboard_shortcuts: HashMap::new(),
            sensitivity: SensitivitySettings::default(),
        }
    }
}

impl StateValidation for InputConfig {
    fn validate(&self) -> Result<()> {
        self.gaze_input.validate()?;
        self.gesture_input.validate()?;
        self.voice_input.validate()?;
        self.sensitivity.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.gaze_input.merge(&other.gaze_input)?;
        self.gesture_input.merge(&other.gesture_input)?;
        self.voice_input.merge(&other.voice_input)?;
        
        // Merge keyboard shortcuts
        for (key, value) in &other.keyboard_shortcuts {
            self.keyboard_shortcuts.insert(key.clone(), value.clone());
        }
        
        self.sensitivity.merge(&other.sensitivity)?;
        Ok(())
    }
}

/// Gaze input settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazeInputSettings {
    /// Gaze input enabled
    pub enabled: bool,
    /// Dwell time in milliseconds
    pub dwell_time_ms: u32,
    /// Gaze cursor visible
    pub cursor_visible: bool,
    /// Smooth tracking enabled
    pub smooth_tracking: bool,
    /// Calibration required
    pub calibration_required: bool,
}

impl Default for GazeInputSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            dwell_time_ms: 800,
            cursor_visible: true,
            smooth_tracking: true,
            calibration_required: false,
        }
    }
}

impl StateValidation for GazeInputSettings {
    fn validate(&self) -> Result<()> {
        if self.dwell_time_ms < 100 || self.dwell_time_ms > 5000 {
            anyhow::bail!("Dwell time out of range: {}", self.dwell_time_ms);
        }
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.dwell_time_ms = other.dwell_time_ms;
        self.cursor_visible = other.cursor_visible;
        self.smooth_tracking = other.smooth_tracking;
        self.calibration_required = other.calibration_required;
        Ok(())
    }
}

/// Gesture input settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureInputSettings {
    /// Gesture input enabled
    pub enabled: bool,
    /// Gesture sensitivity (0.0-1.0)
    pub sensitivity: f32,
    /// Minimum gesture duration in ms
    pub min_duration_ms: u32,
    /// Maximum gesture duration in ms
    pub max_duration_ms: u32,
}

impl Default for GestureInputSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            sensitivity: 0.7,
            min_duration_ms: 200,
            max_duration_ms: 3000,
        }
    }
}

impl StateValidation for GestureInputSettings {
    fn validate(&self) -> Result<()> {
        if self.sensitivity < 0.0 || self.sensitivity > 1.0 {
            anyhow::bail!("Gesture sensitivity out of range: {}", self.sensitivity);
        }
        
        if self.min_duration_ms >= self.max_duration_ms {
            anyhow::bail!("Invalid gesture duration range");
        }
        
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.sensitivity = other.sensitivity;
        self.min_duration_ms = other.min_duration_ms;
        self.max_duration_ms = other.max_duration_ms;
        Ok(())
    }
}

/// Voice input settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInputSettings {
    /// Voice input enabled
    pub enabled: bool,
    /// Wake word required
    pub wake_word_required: bool,
    /// Wake word
    pub wake_word: String,
    /// Voice sensitivity (0.0-1.0)
    pub sensitivity: f32,
    /// Language code
    pub language: String,
}

impl Default for VoiceInputSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_word_required: true,
            wake_word: "Hey XREAL".to_string(),
            sensitivity: 0.8,
            language: "en-US".to_string(),
        }
    }
}

impl StateValidation for VoiceInputSettings {
    fn validate(&self) -> Result<()> {
        if self.sensitivity < 0.0 || self.sensitivity > 1.0 {
            anyhow::bail!("Voice sensitivity out of range: {}", self.sensitivity);
        }
        
        if self.wake_word.is_empty() && self.wake_word_required {
            anyhow::bail!("Wake word required but empty");
        }
        
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.wake_word_required = other.wake_word_required;
        self.wake_word = other.wake_word.clone();
        self.sensitivity = other.sensitivity;
        self.language = other.language.clone();
        Ok(())
    }
}

/// Input sensitivity settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivitySettings {
    /// Mouse sensitivity multiplier
    pub mouse_sensitivity: f32,
    /// Scroll sensitivity multiplier
    pub scroll_sensitivity: f32,
    /// Touch sensitivity multiplier
    pub touch_sensitivity: f32,
    /// Head tracking sensitivity
    pub head_tracking_sensitivity: f32,
}

impl Default for SensitivitySettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            scroll_sensitivity: 1.0,
            touch_sensitivity: 1.0,
            head_tracking_sensitivity: 1.0,
        }
    }
}

impl StateValidation for SensitivitySettings {
    fn validate(&self) -> Result<()> {
        let sensitivities = [
            ("mouse", self.mouse_sensitivity),
            ("scroll", self.scroll_sensitivity),
            ("touch", self.touch_sensitivity),
            ("head_tracking", self.head_tracking_sensitivity),
        ];
        
        for (name, value) in sensitivities {
            if value < 0.1 || value > 5.0 {
                anyhow::bail!("{} sensitivity out of range: {}", name, value);
            }
        }
        
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.mouse_sensitivity = other.mouse_sensitivity;
        self.scroll_sensitivity = other.scroll_sensitivity;
        self.touch_sensitivity = other.touch_sensitivity;
        self.head_tracking_sensitivity = other.head_tracking_sensitivity;
        Ok(())
    }
}