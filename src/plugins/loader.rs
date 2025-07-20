use anyhow::Result;
use bevy::prelude::*;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use super::{PluginApp, PluginCapabilitiesFlags, PluginError, PluginMetadata, PluginSystemConfig};

/// Dynamic plugin loader with security validation and dependency resolution
///
/// NOTE: Comprehensive plugin loading infrastructure for future dynamic plugin support.
/// Currently unused as the system uses static plugin integration.
#[allow(dead_code)]
pub struct PluginLoader {
    /// Configuration for loading behavior
    config: PluginSystemConfig,
    /// Cache of loaded library handles
    library_cache: HashMap<PathBuf, Arc<Library>>,
    /// Metadata cache for quick access
    metadata_cache: HashMap<String, PluginMetadata>,
    /// Security validator
    security_validator: SecurityValidator,
}

impl PluginLoader {
    pub fn new(config: &PluginSystemConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            library_cache: HashMap::new(),
            metadata_cache: HashMap::new(),
            security_validator: SecurityValidator::new(config),
        })
    }

    /// Load plugin from path with full validation
    pub fn load_plugin(
        &mut self,
        library_path: &Path,
    ) -> Result<(Box<dyn PluginApp>, PluginMetadata)> {
        info!("Loading plugin from: {:?}", library_path);

        // Security validation
        self.security_validator.validate_library(library_path)?;

        // Load or get cached library
        let library = self.get_or_load_library(library_path)?;

        // Extract metadata first
        let metadata = self.extract_metadata(&library)?;

        // Validate plugin compatibility
        self.validate_plugin_compatibility(&metadata)?;

        // Validate capabilities against security policy
        self.security_validator
            .validate_capabilities(&metadata.capabilities)?;

        // Create plugin instance
        let plugin_app = self.create_plugin_instance(&library)?;

        // Cache metadata
        self.metadata_cache
            .insert(metadata.id.as_str().to_string(), metadata.clone());

        info!(
            "✅ Successfully loaded plugin: {} v{}",
            metadata.name.as_str(),
            metadata.version.as_str()
        );
        Ok((plugin_app, metadata))
    }

    /// Unload plugin and cleanup resources
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        // Remove from caches
        if let Some(metadata) = self.metadata_cache.remove(plugin_id) {
            // Remove library from cache if no other plugins are using it
            self.library_cache.remove(&metadata.library_path);
            info!("✅ Unloaded plugin: {}", plugin_id);
        }

        Ok(())
    }

    /// Get cached metadata
    pub fn get_metadata(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        self.metadata_cache.get(plugin_id)
    }

    /// List all cached plugins
    pub fn list_cached_plugins(&self) -> Vec<&PluginMetadata> {
        self.metadata_cache.values().collect()
    }

    /// Validate plugin manifest file
    pub fn validate_manifest(&self, manifest_path: &Path) -> Result<PluginManifest> {
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&manifest_content)?;

        // Validate manifest contents
        if manifest.plugin.id.is_empty() {
            return Err(PluginError::LoadFailed("Plugin ID cannot be empty".to_string()).into());
        }

        if manifest.plugin.version.is_empty() {
            return Err(
                PluginError::LoadFailed("Plugin version cannot be empty".to_string()).into(),
            );
        }

        // Validate version format
        if !self.is_valid_version(&manifest.plugin.version) {
            return Err(PluginError::LoadFailed("Invalid version format".to_string()).into());
        }

        Ok(manifest)
    }

    /// Get or load library from cache
    fn get_or_load_library(&mut self, path: &Path) -> Result<Arc<Library>> {
        if let Some(library) = self.library_cache.get(path) {
            return Ok(library.clone());
        }

        // Load new library
        let library = unsafe {
            Library::new(path).map_err(|e| {
                PluginError::LoadFailed(format!("Failed to load library {}: {}", path.display(), e))
            })?
        };

        let library_arc = Arc::new(library);
        self.library_cache
            .insert(path.to_path_buf(), library_arc.clone());

        Ok(library_arc)
    }

    /// Extract plugin metadata from library
    fn extract_metadata(&self, library: &Library) -> Result<PluginMetadata> {
        let get_metadata: Symbol<extern "C" fn() -> PluginMetadata> = unsafe {
            library.get(b"get_plugin_metadata").map_err(|e| {
                PluginError::LoadFailed(format!(
                    "Plugin missing get_plugin_metadata function: {}",
                    e
                ))
            })?
        };

        Ok(get_metadata())
    }

    /// Create plugin instance from library
    fn create_plugin_instance(&self, library: &Library) -> Result<Box<dyn PluginApp>> {
        let create_plugin: Symbol<extern "C" fn() -> Box<dyn PluginApp>> = unsafe {
            library.get(b"create_plugin").map_err(|e| {
                PluginError::LoadFailed(format!("Plugin missing create_plugin function: {}", e))
            })?
        };

        Ok(create_plugin())
    }

    /// Validate plugin compatibility with current system
    fn validate_plugin_compatibility(&self, metadata: &PluginMetadata) -> Result<()> {
        // Check minimum engine version
        if !self.is_version_compatible(
            metadata.minimum_engine_version.as_str(),
            env!("CARGO_PKG_VERSION"),
        ) {
            return Err(PluginError::IncompatibleVersion(format!(
                "Plugin requires engine version {} or higher, current: {}",
                metadata.minimum_engine_version.as_str(),
                env!("CARGO_PKG_VERSION")
            ))
            .into());
        }

        // Check if plugin ID conflicts with existing plugins
        if self.metadata_cache.contains_key(metadata.id.as_str()) {
            return Err(PluginError::LoadFailed(format!(
                "Plugin ID '{}' already exists",
                metadata.id.as_str()
            ))
            .into());
        }

        Ok(())
    }

    /// Check if version strings are compatible
    fn is_version_compatible(&self, required: &str, current: &str) -> bool {
        // Simple version comparison - in production, use a proper semver library
        let required_parts: Vec<u32> = required.split('.').filter_map(|s| s.parse().ok()).collect();
        let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

        if required_parts.is_empty() || current_parts.is_empty() {
            return false;
        }

        // Major version must match, minor version must be >= required
        if required_parts[0] != current_parts[0] {
            return current_parts[0] > required_parts[0];
        }

        if required_parts.len() > 1 && current_parts.len() > 1 {
            return current_parts[1] >= required_parts[1];
        }

        true
    }

    /// Validate version string format
    fn is_valid_version(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() >= 2 && parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
}

/// Security validator for plugin loading
#[allow(dead_code)]
pub struct SecurityValidator {
    config: PluginSystemConfig,
    allowed_capabilities: PluginCapabilitiesFlags,
}

impl SecurityValidator {
    fn new(config: &PluginSystemConfig) -> Self {
        Self {
            config: config.clone(),
            allowed_capabilities: config.allowed_capabilities,
        }
    }

    /// Validate library file security
    fn validate_library(&self, library_path: &Path) -> Result<()> {
        // Check file exists and is readable
        if !library_path.exists() {
            return Err(PluginError::LoadFailed("Library file does not exist".to_string()).into());
        }

        // Check file extension
        let valid_extensions = ["so", "dylib", "dll"];
        let extension = library_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| PluginError::LoadFailed("Invalid library file extension".to_string()))?;

        if !valid_extensions.contains(&extension) {
            return Err(PluginError::LoadFailed(format!(
                "Unsupported library extension: {}",
                extension
            ))
            .into());
        }

        // Check if library is in allowed directory
        let is_in_allowed_dir = self
            .config
            .plugin_directories
            .iter()
            .any(|dir| library_path.starts_with(dir));

        if !is_in_allowed_dir {
            return Err(PluginError::LoadFailed(
                "Library not in allowed plugin directory".to_string(),
            )
            .into());
        }

        // Additional security checks in sandbox mode
        if self.config.sandbox_mode {
            self.validate_sandbox_permissions(library_path)?;
        }

        Ok(())
    }

    /// Validate plugin capabilities against security policy
    fn validate_capabilities(&self, capabilities: &PluginCapabilitiesFlags) -> Result<()> {
        if capabilities.has_flag(PluginCapabilitiesFlags::REQUIRES_NETWORK_ACCESS)
            && !self
                .allowed_capabilities
                .has_flag(PluginCapabilitiesFlags::REQUIRES_NETWORK_ACCESS)
        {
            return Err(PluginError::LoadFailed(
                "Network access not permitted by security policy".to_string(),
            )
            .into());
        }

        if capabilities.has_flag(PluginCapabilitiesFlags::SUPPORTS_FILE_SYSTEM)
            && !self
                .allowed_capabilities
                .has_flag(PluginCapabilitiesFlags::SUPPORTS_FILE_SYSTEM)
        {
            return Err(PluginError::LoadFailed(
                "File system access not permitted by security policy".to_string(),
            )
            .into());
        }

        if capabilities.has_flag(PluginCapabilitiesFlags::SUPPORTS_COMPUTE_SHADERS)
            && !self
                .allowed_capabilities
                .has_flag(PluginCapabilitiesFlags::SUPPORTS_COMPUTE_SHADERS)
        {
            return Err(PluginError::LoadFailed(
                "Compute shader access not permitted by security policy".to_string(),
            )
            .into());
        }

        Ok(())
    }

    /// Validate sandbox permissions (placeholder for full implementation)
    fn validate_sandbox_permissions(&self, _library_path: &Path) -> Result<()> {
        // In full implementation, this would:
        // 1. Check code signing/digital signatures
        // 2. Validate library dependencies
        // 3. Check for suspicious patterns
        // 4. Verify library comes from trusted source

        Ok(())
    }
}

/// Plugin manifest structure for metadata
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,
    pub capabilities: Option<PluginCapabilitiesFlags>,
    pub dependencies: Option<Vec<String>>,
    pub resources: Option<ResourceRequirements>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub memory_mb: Option<u64>,
    pub texture_size: Option<u32>,
    pub buffer_size: Option<u64>,
    pub compute_units: Option<u32>,
}

/// Plugin development helper functions
pub mod dev_tools {
    use super::*;

    /// Generate plugin template
    #[allow(dead_code)]
    pub fn generate_plugin_template(plugin_id: &str, plugin_name: &str) -> String {
        format!(
            r#"
[plugin]
id = "{}"
name = "{}"
version = "0.1.0"
description = "Description of {}"
author = "Your Name"
license = "MIT"

[capabilities]
supports_transparency = true
requires_keyboard_focus = false
supports_multi_window = false
supports_3d_rendering = true
supports_compute_shaders = false
requires_network_access = false
supports_file_system = false
supports_audio = false

[resources]
memory_mb = 64
texture_size = 1024
buffer_size = 16777216

[[dependencies]]
# Add plugin dependencies here
"#,
            plugin_id, plugin_name, plugin_name
        )
    }

    /// Validate plugin development setup
    #[allow(dead_code)]
    pub fn validate_dev_setup(plugin_dir: &Path) -> Result<()> {
        // Check for required files
        let required_files = ["Cargo.toml", "src/lib.rs", "plugin.toml"];

        for file in &required_files {
            let file_path = plugin_dir.join(file);
            if !file_path.exists() {
                return Err(anyhow::anyhow!("Missing required file: {}", file));
            }
        }

        // Validate Cargo.toml has correct crate type
        let cargo_toml = plugin_dir.join("Cargo.toml");
        let cargo_content = std::fs::read_to_string(cargo_toml)?;

        if !cargo_content.contains(r#"crate-type = ["cdylib"]"#) {
            return Err(anyhow::anyhow!(
                "Cargo.toml must specify crate-type = [\"cdylib\"]"
            ));
        }

        info!("✅ Plugin development setup is valid");
        Ok(())
    }
}
