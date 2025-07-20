//! Calibration data schema for IMU sensor fusion
//!
//! This module provides calibration structures with validation and
//! serialization support for the XREAL application state system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::core::StateValidation;

/// IMU calibration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    /// Calibration state
    pub state: CalibrationState,
    /// Accelerometer bias values
    pub accel_bias: [f32; 3],
    /// Gyroscope bias values
    pub gyro_bias: [f32; 3],
    /// Magnetometer bias values
    pub mag_bias: [f32; 3],
    /// Calibration timestamp
    pub calibrated_at: u64,
    /// Number of calibration samples
    pub sample_count: u32,
    /// Calibration quality score (0.0-1.0)
    pub quality_score: f32,
    /// Temperature at calibration
    pub temperature_celsius: f32,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            state: CalibrationState::Idle,
            accel_bias: [0.0; 3],
            gyro_bias: [0.0; 3],
            mag_bias: [0.0; 3],
            calibrated_at: 0,
            sample_count: 0,
            quality_score: 0.0,
            temperature_celsius: 20.0,
        }
    }
}

impl StateValidation for CalibrationData {
    fn validate(&self) -> Result<()> {
        // Validate quality score
        if self.quality_score < 0.0 || self.quality_score > 1.0 {
            anyhow::bail!("Quality score out of range: {}", self.quality_score);
        }

        // Validate temperature
        if self.temperature_celsius < -40.0 || self.temperature_celsius > 85.0 {
            anyhow::bail!("Temperature out of range: {}", self.temperature_celsius);
        }

        // Validate bias values are reasonable
        for &bias in &self.accel_bias {
            if bias.abs() > 10.0 {
                anyhow::bail!("Accelerometer bias out of range: {}", bias);
            }
        }

        for &bias in &self.gyro_bias {
            if bias.abs() > 1000.0 {
                anyhow::bail!("Gyroscope bias out of range: {}", bias);
            }
        }

        for &bias in &self.mag_bias {
            if bias.abs() > 1000.0 {
                anyhow::bail!("Magnetometer bias out of range: {}", bias);
            }
        }

        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        // Only merge if other calibration is newer and better quality
        if other.calibrated_at > self.calibrated_at && other.quality_score >= self.quality_score {
            self.state = other.state;
            self.accel_bias = other.accel_bias;
            self.gyro_bias = other.gyro_bias;
            self.mag_bias = other.mag_bias;
            self.calibrated_at = other.calibrated_at;
            self.sample_count = other.sample_count;
            self.quality_score = other.quality_score;
            self.temperature_celsius = other.temperature_celsius;
        }
        Ok(())
    }
}

/// Calibration state enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CalibrationState {
    /// Not calibrated
    Idle,
    /// Currently calibrating
    Calibrating,
    /// Successfully calibrated
    Calibrated,
    /// Calibration failed
    Failed,
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self::Idle
    }
}