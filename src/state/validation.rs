//! State Validation System
//! 
//! Provides comprehensive validation for all state components with
//! detailed error reporting and no unwrap/expect usage.

use anyhow::Result;
use crate::state::{StateError, AppState};

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
        
        // Use the StateValidation trait implementation
        state.validate()
    }
}

impl Default for StateValidator {
    fn default() -> Self {
        Self::new()
    }
}