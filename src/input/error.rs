// use std::fmt; // Currently unused
use thiserror::Error;

/// Errors that can occur in the input system
#[derive(Debug, Error)]
pub enum InputError {
    /// Failed to initialize input device
    #[error("Failed to initialize input device: {0}")]
    Initialization(String),
    
    /// Input operation failed
    #[error("Input operation failed: {0}")]
    OperationFailed(String),
    
    /// Input device not available
    #[error("Input device not available: {0}")]
    DeviceNotAvailable(String),
    
    /// Invalid input configuration
    #[error("Invalid input configuration: {0}")]
    InvalidConfig(String),
    
    /// Unsupported input type or value
    #[error("Unsupported input: {0}")]
    UnsupportedInput(String),
    
    /// Thread safety violation
    #[error("Thread safety violation: {0}")]
    ThreadSafety(String),
    
    /// Mouse move operation failed
    #[error("Failed to move mouse: {0}")]
    MouseMove(String),
    
    /// Mouse click operation failed
    #[error("Failed to click mouse: {0}")]
    MouseClick(String),
    
    /// Key press operation failed
    #[error("Failed to press key: {0}")]
    KeyPress(String),
    
    /// Key release operation failed
    #[error("Failed to release key: {0}")]
    KeyRelease(String),
    
    /// Text input operation failed
    #[error("Failed to input text: {0}")]
    TextInput(String),
    
    /// Other types of errors
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for input operations
pub type InputResult<T> = Result<T, InputError>;

impl From<std::io::Error> for InputError {
    fn from(err: std::io::Error) -> Self {
        InputError::OperationFailed(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for InputError {
    fn from(err: std::time::SystemTimeError) -> Self {
        InputError::OperationFailed(format!("System time error: {}", err))
    }
}

// Enigo error conversion is handled by the `enigo` feature flag in Cargo.toml
// and the `enabled` flag in InputConfig
