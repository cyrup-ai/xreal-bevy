//! Audio settings schema for XREAL application
//!
//! This module provides audio configuration structures with validation and
//! serialization support for the XREAL application state system.

use super::core::StateValidation;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Audio system settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0-1.0)
    pub master_volume: f32,
    /// Audio device configuration
    pub device_config: AudioDeviceConfig,
    /// Spatial audio settings
    pub spatial_audio: SpatialAudioSettings,
    /// Audio effects settings
    pub effects: AudioEffectsSettings,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            device_config: AudioDeviceConfig::default(),
            spatial_audio: SpatialAudioSettings::default(),
            effects: AudioEffectsSettings::default(),
        }
    }
}

impl StateValidation for AudioSettings {
    fn validate(&self) -> Result<()> {
        if self.master_volume < 0.0 || self.master_volume > 1.0 {
            anyhow::bail!("Master volume out of range: {}", self.master_volume);
        }

        self.device_config.validate()?;
        self.spatial_audio.validate()?;
        self.effects.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.master_volume = other.master_volume;
        self.device_config.merge(&other.device_config)?;
        self.spatial_audio.merge(&other.spatial_audio)?;
        self.effects.merge(&other.effects)?;
        Ok(())
    }
}

/// Audio device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceConfig {
    /// Output device name
    pub output_device: String,
    /// Input device name
    pub input_device: String,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Buffer size in samples
    pub buffer_size: u32,
    /// Bit depth
    pub bit_depth: u32,
}

impl Default for AudioDeviceConfig {
    fn default() -> Self {
        Self {
            output_device: "default".to_string(),
            input_device: "default".to_string(),
            sample_rate: 48000,
            buffer_size: 1024,
            bit_depth: 16,
        }
    }
}

impl StateValidation for AudioDeviceConfig {
    fn validate(&self) -> Result<()> {
        if ![44100, 48000, 96000].contains(&self.sample_rate) {
            anyhow::bail!("Unsupported sample rate: {}", self.sample_rate);
        }

        if ![16, 24, 32].contains(&self.bit_depth) {
            anyhow::bail!("Unsupported bit depth: {}", self.bit_depth);
        }

        if !self.buffer_size.is_power_of_two() || self.buffer_size < 64 || self.buffer_size > 8192 {
            anyhow::bail!("Invalid buffer size: {}", self.buffer_size);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.output_device = other.output_device.clone();
        self.input_device = other.input_device.clone();
        self.sample_rate = other.sample_rate;
        self.buffer_size = other.buffer_size;
        self.bit_depth = other.bit_depth;
        Ok(())
    }
}

/// Spatial audio settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudioSettings {
    /// Spatial audio enabled
    pub enabled: bool,
    /// HRTF enabled
    pub hrtf_enabled: bool,
    /// Room simulation enabled
    pub room_simulation: bool,
    /// Distance attenuation factor
    pub distance_attenuation: f32,
}

impl Default for SpatialAudioSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            hrtf_enabled: true,
            room_simulation: false,
            distance_attenuation: 1.0,
        }
    }
}

impl StateValidation for SpatialAudioSettings {
    fn validate(&self) -> Result<()> {
        if self.distance_attenuation < 0.0 || self.distance_attenuation > 2.0 {
            anyhow::bail!(
                "Distance attenuation out of range: {}",
                self.distance_attenuation
            );
        }
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.hrtf_enabled = other.hrtf_enabled;
        self.room_simulation = other.room_simulation;
        self.distance_attenuation = other.distance_attenuation;
        Ok(())
    }
}

/// Audio effects settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEffectsSettings {
    /// Reverb enabled
    pub reverb_enabled: bool,
    /// Reverb amount (0.0-1.0)
    pub reverb_amount: f32,
    /// Echo enabled
    pub echo_enabled: bool,
    /// Echo delay in ms
    pub echo_delay_ms: u32,
}

impl Default for AudioEffectsSettings {
    fn default() -> Self {
        Self {
            reverb_enabled: false,
            reverb_amount: 0.3,
            echo_enabled: false,
            echo_delay_ms: 200,
        }
    }
}

impl StateValidation for AudioEffectsSettings {
    fn validate(&self) -> Result<()> {
        if self.reverb_amount < 0.0 || self.reverb_amount > 1.0 {
            anyhow::bail!("Reverb amount out of range: {}", self.reverb_amount);
        }

        if self.echo_delay_ms < 10 || self.echo_delay_ms > 2000 {
            anyhow::bail!("Echo delay out of range: {}", self.echo_delay_ms);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.reverb_enabled = other.reverb_enabled;
        self.reverb_amount = other.reverb_amount;
        self.echo_enabled = other.echo_enabled;
        self.echo_delay_ms = other.echo_delay_ms;
        Ok(())
    }
}
