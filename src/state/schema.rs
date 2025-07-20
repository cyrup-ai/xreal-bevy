//! Complete state schema with versioned serialization support for XREAL application
//!
//! This module provides the complete application state schema with modular organization
//! for better maintainability. The module has been decomposed into logical submodules
//! while preserving the original API surface for backward compatibility.

// Re-export the entire schema module structure
pub use schema::*;

// Import the modular implementation
mod schema;