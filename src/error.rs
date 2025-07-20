use std::fmt;
use thiserror::Error;

/// Error type for input-related operations
#[derive(Debug, Error)]
pub enum InputError {
    /// Failed to initialize the input system
    #[error("Failed to initialize input system: {0}")]
    Initialization(String),
    
    /// Failed to move the mouse
    #[error("Failed to move mouse: {0}")]
    MouseMove(String),
    
    /// Input operation was rate-limited
    #[error("Input operation rate limited")]
    RateLimited,
    
    /// Input system is not initialized
    #[error("Input system not initialized")]
    NotInitialized,
}

impl From<enigo::Error> for InputError {
    fn from(err: enigo::Error) -> Self {
        InputError::MouseMove(err.to_string())
    }
}

/// Result type for input operations
pub type InputResult<T> = Result<T, InputError>;
