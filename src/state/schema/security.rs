//! Security settings schema for XREAL application
//!
//! This module provides security configuration structures with validation and
//! serialization support for the XREAL application state system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::core::StateValidation;

/// Security configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Authentication settings
    pub authentication: AuthenticationSettings,
    /// Access control settings
    pub access_control: AccessControlSettings,
    /// Audit logging enabled
    pub audit_logging: bool,
    /// Security level
    pub security_level: SecurityLevel,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            authentication: AuthenticationSettings::default(),
            access_control: AccessControlSettings::default(),
            audit_logging: true,
            security_level: SecurityLevel::High,
        }
    }
}

impl StateValidation for SecuritySettings {
    fn validate(&self) -> Result<()> {
        self.authentication.validate()?;
        self.access_control.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.encryption_enabled = other.encryption_enabled;
        self.authentication.merge(&other.authentication)?;
        self.access_control.merge(&other.access_control)?;
        self.audit_logging = other.audit_logging;
        self.security_level = other.security_level;
        Ok(())
    }
}

/// Authentication settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSettings {
    /// Authentication required
    pub required: bool,
    /// Authentication method
    pub method: AuthenticationMethod,
    /// Session timeout in minutes
    pub session_timeout_mins: u32,
    /// Multi-factor authentication enabled
    pub mfa_enabled: bool,
}

impl Default for AuthenticationSettings {
    fn default() -> Self {
        Self {
            required: false,
            method: AuthenticationMethod::None,
            session_timeout_mins: 60,
            mfa_enabled: false,
        }
    }
}

impl StateValidation for AuthenticationSettings {
    fn validate(&self) -> Result<()> {
        if self.session_timeout_mins < 5 || self.session_timeout_mins > 1440 {
            anyhow::bail!("Session timeout out of range: {}", self.session_timeout_mins);
        }
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.required = other.required;
        self.method = other.method;
        self.session_timeout_mins = other.session_timeout_mins;
        self.mfa_enabled = other.mfa_enabled;
        Ok(())
    }
}

/// Authentication method enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    None,
    Password,
    Biometric,
    Token,
    Certificate,
}

impl Default for AuthenticationMethod {
    fn default() -> Self {
        Self::None
    }
}

/// Access control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlSettings {
    /// Sandbox enabled
    pub sandbox_enabled: bool,
    /// File system access restricted
    pub restrict_filesystem: bool,
    /// Network access restricted
    pub restrict_network: bool,
    /// System command execution restricted
    pub restrict_system_commands: bool,
    /// Hardware access restricted
    pub restrict_hardware: bool,
}

impl Default for AccessControlSettings {
    fn default() -> Self {
        Self {
            sandbox_enabled: true,
            restrict_filesystem: true,
            restrict_network: true,
            restrict_system_commands: true,
            restrict_hardware: true,
        }
    }
}

impl StateValidation for AccessControlSettings {
    fn validate(&self) -> Result<()> {
        // No specific validation needed for boolean access controls
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.sandbox_enabled = other.sandbox_enabled;
        self.restrict_filesystem = other.restrict_filesystem;
        self.restrict_network = other.restrict_network;
        self.restrict_system_commands = other.restrict_system_commands;
        self.restrict_hardware = other.restrict_hardware;
        Ok(())
    }
}

/// Security level enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Maximum,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::High
    }
}