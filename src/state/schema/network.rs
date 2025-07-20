//! Network configuration schema for XREAL application
//!
//! This module provides network settings structures with validation and
//! serialization support for the XREAL application state system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::core::StateValidation;

/// Network configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network enabled
    pub enabled: bool,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u32,
    /// Request timeout in seconds
    pub request_timeout_secs: u32,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Proxy settings
    pub proxy_settings: ProxySettings,
    /// SSL/TLS settings
    pub ssl_settings: SslSettings,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            connection_timeout_secs: 30,
            request_timeout_secs: 60,
            max_connections: 10,
            proxy_settings: ProxySettings::default(),
            ssl_settings: SslSettings::default(),
        }
    }
}

impl StateValidation for NetworkConfig {
    fn validate(&self) -> Result<()> {
        if self.connection_timeout_secs < 1 || self.connection_timeout_secs > 300 {
            anyhow::bail!("Connection timeout out of range: {}", self.connection_timeout_secs);
        }
        
        if self.request_timeout_secs < 1 || self.request_timeout_secs > 600 {
            anyhow::bail!("Request timeout out of range: {}", self.request_timeout_secs);
        }
        
        if self.max_connections < 1 || self.max_connections > 100 {
            anyhow::bail!("Max connections out of range: {}", self.max_connections);
        }
        
        self.proxy_settings.validate()?;
        self.ssl_settings.validate()?;
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.connection_timeout_secs = other.connection_timeout_secs;
        self.request_timeout_secs = other.request_timeout_secs;
        self.max_connections = other.max_connections;
        self.proxy_settings.merge(&other.proxy_settings)?;
        self.ssl_settings.merge(&other.ssl_settings)?;
        Ok(())
    }
}

/// Proxy configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    /// Proxy enabled
    pub enabled: bool,
    /// Proxy type
    pub proxy_type: ProxyType,
    /// Proxy host
    pub host: String,
    /// Proxy port
    pub port: u16,
    /// Authentication required
    pub auth_required: bool,
    /// Username for authentication
    pub username: String,
    /// Password for authentication (stored securely)
    pub password_hash: String,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            proxy_type: ProxyType::Http,
            host: String::new(),
            port: 8080,
            auth_required: false,
            username: String::new(),
            password_hash: String::new(),
        }
    }
}

impl StateValidation for ProxySettings {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.host.is_empty() {
            anyhow::bail!("Proxy host required when proxy is enabled");
        }
        
        if self.port == 0 {
            anyhow::bail!("Invalid proxy port: {}", self.port);
        }
        
        if self.auth_required && self.username.is_empty() {
            anyhow::bail!("Username required when proxy auth is enabled");
        }
        
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.enabled = other.enabled;
        self.proxy_type = other.proxy_type;
        self.host = other.host.clone();
        self.port = other.port;
        self.auth_required = other.auth_required;
        self.username = other.username.clone();
        self.password_hash = other.password_hash.clone();
        Ok(())
    }
}

/// Proxy type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl Default for ProxyType {
    fn default() -> Self {
        Self::Http
    }
}

/// SSL/TLS configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslSettings {
    /// SSL verification enabled
    pub verify_ssl: bool,
    /// Accept invalid certificates
    pub accept_invalid_certs: bool,
    /// Minimum TLS version
    pub min_tls_version: TlsVersion,
    /// Certificate pinning enabled
    pub certificate_pinning: bool,
}

impl Default for SslSettings {
    fn default() -> Self {
        Self {
            verify_ssl: true,
            accept_invalid_certs: false,
            min_tls_version: TlsVersion::Tls12,
            certificate_pinning: false,
        }
    }
}

impl StateValidation for SslSettings {
    fn validate(&self) -> Result<()> {
        // No specific validation needed for SSL settings
        Ok(())
    }

    fn merge(&mut self, other: &Self) -> Result<()> {
        self.verify_ssl = other.verify_ssl;
        self.accept_invalid_certs = other.accept_invalid_certs;
        self.min_tls_version = other.min_tls_version;
        self.certificate_pinning = other.certificate_pinning;
        Ok(())
    }
}

/// TLS version enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
}

impl Default for TlsVersion {
    fn default() -> Self {
        Self::Tls12
    }
}