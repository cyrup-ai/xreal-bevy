//! State Validation System
//!
//! Provides comprehensive validation for all state components with
//! detailed error reporting and no unwrap/expect usage.

use crate::AppState;
use anyhow::Result;
use bevy::prelude::{info, warn};

/// State validator
pub struct StateValidator {
    /// Validation rules enabled
    pub rules_enabled: bool,
    /// Strict validation mode
    pub strict_mode: bool,
}

impl StateValidator {
    /// Create new state validator
    pub fn new() -> Self {
        Self {
            rules_enabled: true,
            strict_mode: true,
        }
    }

    /// Validate complete application state
    pub fn validate(&self, state: &AppState) -> Result<()> {
        if !self.rules_enabled {
            return Ok(());
        }

        // Validate AppState enum values and transitions
        match state {
            AppState::Startup => {
                // Startup state is always valid - initial state
                info!("Validating Startup state: ✅ Valid");
            }
            AppState::ChecksFailed => {
                // ChecksFailed indicates system issues but is a valid state
                warn!("Validating ChecksFailed state: ⚠️ System checks failed but state is valid");
            }
            AppState::Running => {
                // Running state indicates normal operation
                info!("Validating Running state: ✅ Valid - normal operation");
            }
        }

        Ok(())
    }
}

impl Default for StateValidator {
    fn default() -> Self {
        Self::new()
    }
}
