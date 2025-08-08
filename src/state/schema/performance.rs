//! Performance settings schema for XREAL application optimization
//!
//! This module provides performance configuration structures with validation and
//! serialization support for the XREAL application state system.

use super::core::StateValidation;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Performance settings and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Target frame rate
    pub target_fps: u32,
    /// VSync enabled
    pub vsync_enabled: bool,
    /// Render quality level
    pub render_quality: RenderQuality,
    /// Anti-aliasing settings
    pub anti_aliasing: AntiAliasingSettings,
    /// Shadow settings
    pub shadow_settings: ShadowSettings,
    /// Texture settings
    pub texture_settings: TextureSettings,
    /// Performance monitoring enabled
    pub monitoring_enabled: bool,
    /// Performance thresholds
    pub thresholds: PerformanceThresholds,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            target_fps: 90,
            vsync_enabled: true,
            render_quality: RenderQuality::High,
            anti_aliasing: AntiAliasingSettings::default(),
            shadow_settings: ShadowSettings::default(),
            texture_settings: TextureSettings::default(),
            monitoring_enabled: true,
            thresholds: PerformanceThresholds::default(),
        }
    }
}

impl StateValidation for PerformanceSettings {
    fn validate(&self) -> Result<()> {
        // Validate target FPS
        if self.target_fps < 30 || self.target_fps > 240 {
            anyhow::bail!("Target FPS out of range: {}", self.target_fps);
        }

        // Validate sub-components
        self.anti_aliasing.validate()?;
        self.shadow_settings.validate()?;
        self.texture_settings.validate()?;
        self.thresholds.validate()?;

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.target_fps = other.target_fps;
        self.vsync_enabled = other.vsync_enabled;
        self.render_quality = other.render_quality;
        self.anti_aliasing.merge(&other.anti_aliasing)?;
        self.shadow_settings.merge(&other.shadow_settings)?;
        self.texture_settings.merge(&other.texture_settings)?;
        self.monitoring_enabled = other.monitoring_enabled;
        self.thresholds.merge(&other.thresholds)?;
        Ok(())
    }
}

/// Render quality levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

impl Default for RenderQuality {
    fn default() -> Self {
        Self::High
    }
}

/// Anti-aliasing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiAliasingSettings {
    /// Anti-aliasing enabled
    pub enabled: bool,
    /// Anti-aliasing type
    pub aa_type: AntiAliasingType,
    /// Sample count for MSAA
    pub sample_count: u32,
}

impl Default for AntiAliasingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            aa_type: AntiAliasingType::MSAA,
            sample_count: 4,
        }
    }
}

impl StateValidation for AntiAliasingSettings {
    fn validate(&self) -> Result<()> {
        // Validate sample count
        if self.sample_count > 16
            || (self.sample_count != 0 && !self.sample_count.is_power_of_two())
        {
            anyhow::bail!("Invalid AA sample count: {}", self.sample_count);
        }
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.aa_type = other.aa_type;
        self.sample_count = other.sample_count;
        Ok(())
    }
}

/// Anti-aliasing types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AntiAliasingType {
    None,
    FXAA,
    MSAA,
    TAA,
}

impl Default for AntiAliasingType {
    fn default() -> Self {
        Self::MSAA
    }
}

/// Shadow settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSettings {
    /// Shadows enabled
    pub enabled: bool,
    /// Shadow quality
    pub quality: ShadowQuality,
    /// Shadow map resolution
    pub map_resolution: u32,
    /// Shadow distance
    pub distance: f32,
    /// Cascade count
    pub cascade_count: u32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            quality: ShadowQuality::Medium,
            map_resolution: 2048,
            distance: 100.0,
            cascade_count: 4,
        }
    }
}

impl StateValidation for ShadowSettings {
    fn validate(&self) -> Result<()> {
        // Validate map resolution
        if self.map_resolution < 256
            || self.map_resolution > 8192
            || !self.map_resolution.is_power_of_two()
        {
            anyhow::bail!("Invalid shadow map resolution: {}", self.map_resolution);
        }

        // Validate distance
        if self.distance < 1.0 || self.distance > 1000.0 {
            anyhow::bail!("Shadow distance out of range: {}", self.distance);
        }

        // Validate cascade count
        if self.cascade_count < 1 || self.cascade_count > 8 {
            anyhow::bail!("Cascade count out of range: {}", self.cascade_count);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.quality = other.quality;
        self.map_resolution = other.map_resolution;
        self.distance = other.distance;
        self.cascade_count = other.cascade_count;
        Ok(())
    }
}

/// Shadow quality levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShadowQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for ShadowQuality {
    fn default() -> Self {
        Self::Medium
    }
}

/// Texture settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureSettings {
    /// Texture quality
    pub quality: TextureQuality,
    /// Anisotropic filtering level
    pub anisotropic_filtering: u32,
    /// Texture streaming enabled
    pub streaming_enabled: bool,
    /// Texture cache size in MB
    pub cache_size_mb: u32,
}

impl Default for TextureSettings {
    fn default() -> Self {
        Self {
            quality: TextureQuality::High,
            anisotropic_filtering: 8,
            streaming_enabled: true,
            cache_size_mb: 512,
        }
    }
}

impl StateValidation for TextureSettings {
    fn validate(&self) -> Result<()> {
        // Validate anisotropic filtering
        if self.anisotropic_filtering > 16
            || (self.anisotropic_filtering != 0 && !self.anisotropic_filtering.is_power_of_two())
        {
            anyhow::bail!(
                "Invalid anisotropic filtering: {}",
                self.anisotropic_filtering
            );
        }

        // Validate cache size
        if self.cache_size_mb < 64 || self.cache_size_mb > 4096 {
            anyhow::bail!("Texture cache size out of range: {}", self.cache_size_mb);
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.quality = other.quality;
        self.anisotropic_filtering = other.anisotropic_filtering;
        self.streaming_enabled = other.streaming_enabled;
        self.cache_size_mb = other.cache_size_mb;
        Ok(())
    }
}

/// Texture quality levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for TextureQuality {
    fn default() -> Self {
        Self::High
    }
}

/// Performance monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Low FPS threshold
    pub low_fps_threshold: f32,
    /// High frame time threshold in ms
    pub high_frame_time_ms: f32,
    /// Memory usage warning threshold in MB
    pub memory_warning_mb: u64,
    /// CPU usage warning threshold percentage
    pub cpu_warning_percent: f32,
    /// GPU usage warning threshold percentage
    pub gpu_warning_percent: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            low_fps_threshold: 60.0,
            high_frame_time_ms: 16.67, // ~60 FPS
            memory_warning_mb: 1024,
            cpu_warning_percent: 80.0,
            gpu_warning_percent: 90.0,
        }
    }
}

impl StateValidation for PerformanceThresholds {
    fn validate(&self) -> Result<()> {
        // Validate FPS threshold
        if self.low_fps_threshold < 10.0 || self.low_fps_threshold > 240.0 {
            anyhow::bail!("Low FPS threshold out of range: {}", self.low_fps_threshold);
        }

        // Validate frame time
        if self.high_frame_time_ms < 1.0 || self.high_frame_time_ms > 100.0 {
            anyhow::bail!("High frame time out of range: {}", self.high_frame_time_ms);
        }

        // Validate memory threshold
        if self.memory_warning_mb < 128 || self.memory_warning_mb > 16384 {
            anyhow::bail!(
                "Memory warning threshold out of range: {}",
                self.memory_warning_mb
            );
        }

        // Validate CPU threshold
        if self.cpu_warning_percent < 10.0 || self.cpu_warning_percent > 100.0 {
            anyhow::bail!(
                "CPU warning threshold out of range: {}",
                self.cpu_warning_percent
            );
        }

        // Validate GPU threshold
        if self.gpu_warning_percent < 10.0 || self.gpu_warning_percent > 100.0 {
            anyhow::bail!(
                "GPU warning threshold out of range: {}",
                self.gpu_warning_percent
            );
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.low_fps_threshold = other.low_fps_threshold;
        self.high_frame_time_ms = other.high_frame_time_ms;
        self.memory_warning_mb = other.memory_warning_mb;
        self.cpu_warning_percent = other.cpu_warning_percent;
        self.gpu_warning_percent = other.gpu_warning_percent;
        Ok(())
    }
}
