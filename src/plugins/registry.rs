use anyhow::Result;
use bevy::prelude::*;
use bevy::app::PluginGroupBuilder;
use libloading::{Library, Symbol};
use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
};
use crossbeam_channel::{Receiver, Sender, bounded};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::tracking::{Orientation, CalibrationState};
use super::{
    PluginApp, PluginInstance, PluginMetadata, PluginError,
    PluginSystemConfig, PluginState, context::OrientationAccess
};

/// Plugin registry managing dynamic loading and Bevy integration
/// Implements PluginGroup pattern for seamless Bevy integration
#[allow(dead_code)]
#[derive(Resource)]
pub struct PluginRegistry {
    /// Currently loaded plugin instances
    instances: HashMap<String, PluginInstance>,
    /// Dynamically loaded libraries
    libraries: HashMap<String, Arc<Library>>,
    /// Plugin metadata cache
    metadata_cache: HashMap<String, PluginMetadata>,
    /// Configuration for plugin system
    config: PluginSystemConfig,
    /// Channels for XREAL system integration
    orientation_channels: HashMap<String, (Sender<Quat>, Receiver<Quat>)>,
    calibration_channels: HashMap<String, (Sender<CalibrationState>, Receiver<CalibrationState>)>,
    /// Hot reload file watcher
    hot_reload_watcher: Option<RecommendedWatcher>,
    /// Pending reload requests
    reload_queue: Vec<String>,
}

impl PluginRegistry {
    pub fn new(config: PluginSystemConfig) -> Result<Self> {
        let mut registry = Self {
            instances: HashMap::new(),
            libraries: HashMap::new(),
            metadata_cache: HashMap::new(),
            orientation_channels: HashMap::new(),
            calibration_channels: HashMap::new(),
            hot_reload_watcher: None,
            reload_queue: Vec::new(),
            config,
        };
        
        // Setup hot reload watcher if enabled
        if registry.config.enable_hot_reload {
            registry.setup_hot_reload()?;
        }
        
        Ok(registry)
    }
    
    /// Discover plugins in configured directories
    pub fn discover_plugins(&mut self) -> Result<Vec<PluginMetadata>> {
        let mut discovered = Vec::new();
        
        for plugin_dir in &self.config.plugin_directories {
            if !plugin_dir.exists() {
                continue;
            }
            
            debug!("Scanning plugin directory: {:?}", plugin_dir);
            
            for entry in std::fs::read_dir(plugin_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                // Look for dynamic libraries
                if self.is_plugin_library(&path) {
                    match self.extract_metadata(&path) {
                        Ok(metadata) => {
                            self.metadata_cache.insert(metadata.id.as_str().to_string(), metadata.clone());
                            discovered.push(metadata);
                        }
                        Err(e) => {
                            warn!("Failed to extract metadata from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        
        info!("Discovered {} plugins", discovered.len());
        Ok(discovered)
    }
    
    /// Load a plugin by ID
    pub fn load_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let metadata = self.metadata_cache.get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?
            .clone();
        
        // Check if already loaded
        if self.instances.contains_key(plugin_id) {
            return Ok(());
        }
        
        info!("Loading plugin: {} v{}", metadata.name, metadata.version);
        
        // Validate dependencies
        self.validate_dependencies(&metadata)?;
        
        // Load dynamic library
        let library = self.load_library(&metadata.library_path)?;
        
        // Get plugin factory function
        let create_plugin: Symbol<extern "C" fn() -> Box<dyn PluginApp>> = unsafe {
            library.get(b"create_plugin")?
        };
        
        // Create plugin instance
        let app = create_plugin();
        
        // Setup XREAL system access channels
        let (orientation_tx, orientation_rx) = bounded(10);
        let (calibration_tx, calibration_rx) = bounded(10);
        
        self.orientation_channels.insert(plugin_id.to_string(), (orientation_tx, orientation_rx.clone()));
        self.calibration_channels.insert(plugin_id.to_string(), (calibration_tx, calibration_rx.clone()));
        
        // Create plugin context
        let _orientation_access = OrientationAccess::new(None, None);
        // Note: In full implementation, this would get actual render device/queue from Bevy
        // For now, showing the integration pattern
        
        let mut instance = PluginInstance::new(metadata.clone());
        instance.state = PluginState::Loaded;
        instance.app = Some(app);
        
        // Store library reference to prevent unloading
        self.libraries.insert(plugin_id.to_string(), Arc::new(library));
        self.instances.insert(plugin_id.to_string(), instance);
        
        info!("✅ Plugin loaded successfully: {}", plugin_id);
        Ok(())
    }
    
    /// Unload a plugin by ID
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let instance = self.instances.get_mut(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;
        
        info!("Unloading plugin: {}", plugin_id);
        
        // Shutdown plugin
        if let Some(ref mut app) = instance.app {
            if let Err(e) = app.shutdown() {
                error!("Plugin shutdown error for {}: {}", plugin_id, e);
            }
        }
        
        // Update state
        instance.state = PluginState::Unloaded;
        instance.app = None;
        
        // Cleanup resources
        self.libraries.remove(plugin_id);
        self.orientation_channels.remove(plugin_id);
        self.calibration_channels.remove(plugin_id);
        
        info!("✅ Plugin unloaded: {}", plugin_id);
        Ok(())
    }
    
    /// Get plugin instance
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInstance> {
        self.instances.get(plugin_id)
    }
    
    /// Get mutable plugin instance
    pub fn get_plugin_mut(&mut self, plugin_id: &str) -> Option<&mut PluginInstance> {
        self.instances.get_mut(plugin_id)
    }
    
    /// List all active plugins
    pub fn list_active_plugins(&self) -> Vec<&str> {
        self.instances
            .iter()
            .filter(|(_, instance)| instance.is_active())
            .map(|(id, _)| id.as_str())
            .collect()
    }
    
    /// List all plugin IDs (regardless of state)
    pub fn list_all_plugin_ids(&self) -> Vec<String> {
        self.instances.keys().cloned().collect()
    }
    
    /// Register a plugin instance directly
    pub fn register_plugin(&mut self, plugin_id: String, instance: PluginInstance) {
        self.instances.insert(plugin_id, instance);
    }
    
    /// Update plugin with latest XREAL data
    pub fn update_plugin_systems(&mut self, orientation: &Orientation, calibration: &CalibrationState) {
        // Send latest data to all active plugins
        for (_plugin_id, (orientation_tx, _)) in &self.orientation_channels {
            if let Err(_) = orientation_tx.try_send(orientation.quat) {
                // Channel full, which is fine - plugins get latest available data
            }
        }
        
        for (_plugin_id, (calibration_tx, _)) in &self.calibration_channels {
            if let Err(_) = calibration_tx.try_send(calibration.clone()) {
                // Channel full, which is fine - plugins get latest available data
            }
        }
    }
    
    /// Process hot reload requests
    pub fn process_hot_reload(&mut self) -> Result<()> {
        if !self.config.enable_hot_reload {
            return Ok(());
        }
        
        let reload_requests = std::mem::take(&mut self.reload_queue);
        
        for plugin_id in reload_requests {
            info!("Hot reloading plugin: {}", plugin_id);
            
            // Unload current version
            if self.instances.contains_key(&plugin_id) {
                self.unload_plugin(&plugin_id)?;
            }
            
            // Reload from disk
            self.load_plugin(&plugin_id)?;
        }
        
        Ok(())
    }
    
    /// Check if path is a plugin library
    fn is_plugin_library(&self, path: &Path) -> bool {
        match path.extension().and_then(|s| s.to_str()) {
            Some("so") | Some("dylib") | Some("dll") => {
                // Check if it's a plugin by looking for plugin prefix/suffix
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|name| name.starts_with("xreal_plugin_") || name.ends_with("_plugin"))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }
    
    /// Extract metadata from plugin library
    fn extract_metadata(&self, library_path: &Path) -> Result<PluginMetadata> {
        // Load library temporarily to get metadata
        let library = unsafe { Library::new(library_path)? };
        
        let get_metadata: Symbol<extern "C" fn() -> PluginMetadata> = unsafe {
            library.get(b"get_plugin_metadata")?
        };
        
        let mut metadata = get_metadata();
        metadata.library_path = library_path.to_path_buf();
        
        Ok(metadata)
    }
    
    /// Validate plugin dependencies
    fn validate_dependencies(&self, metadata: &PluginMetadata) -> Result<()> {
        for dep in metadata.dependencies.iter() {
            if !self.instances.contains_key(dep.as_str()) {
                return Err(PluginError::MissingDependency(dep.as_str().to_string()).into());
            }
        }
        Ok(())
    }
    
    /// Load dynamic library with error handling
    fn load_library(&self, path: &Path) -> Result<Library> {
        unsafe {
            Library::new(path).map_err(|e| {
                PluginError::LoadFailed(format!("Failed to load {}: {}", path.display(), e)).into()
            })
        }
    }
    
    /// Setup hot reload file watching
    fn setup_hot_reload(&mut self) -> Result<()> {
        use notify::Event;
        
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            tx,
            notify::Config::default().with_poll_interval(std::time::Duration::from_secs(1))
        )?;
        
        // Watch plugin directories
        for dir in &self.config.plugin_directories {
            if dir.exists() {
                watcher.watch(dir, RecursiveMode::NonRecursive)?;
            }
        }
        
        self.hot_reload_watcher = Some(watcher);
        
        // Spawn background task to handle file events
        let reload_queue = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let queue_clone = reload_queue.clone();
        
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if let Ok(Event { paths, .. }) = event {
                    for path in paths {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if file_stem.contains("plugin") {
                                if let Ok(mut queue) = queue_clone.lock() {
                                    // Extract plugin ID from filename
                                    let plugin_id = file_stem.replace("xreal_plugin_", "").replace("_plugin", "");
                                    queue.push(plugin_id);
                                }
                            }
                        }
                    }
                }
            }
        });
        
        info!("Hot reload watcher enabled for {} directories", self.config.plugin_directories.len());
        Ok(())
    }
}

/// Bevy plugin group for dynamic plugins
/// Integrates plugin system with Bevy's plugin architecture
/// 
/// NOTE: Alternative integration pattern. Current implementation uses direct
/// add_plugin_system() integration in main.rs. Preserved for future modular loading.
#[allow(dead_code)]
pub struct DynamicPluginGroup {
    registry: PluginRegistry,
}

impl DynamicPluginGroup {
    pub fn new(config: PluginSystemConfig) -> Result<Self> {
        Ok(Self {
            registry: PluginRegistry::new(config)?,
        })
    }
}

impl PluginGroup for DynamicPluginGroup {
    fn build(mut self) -> PluginGroupBuilder {
        let builder = PluginGroupBuilder::start::<Self>();
        
        // Discover and auto-load plugins
        match self.registry.discover_plugins() {
            Ok(plugins) => {
                info!("Auto-loading {} discovered plugins", plugins.len());
                for metadata in plugins {
                    if let Err(e) = self.registry.load_plugin(metadata.id.as_str()) {
                        error!("Failed to auto-load plugin {}: {}", metadata.id.as_str(), e);
                    }
                }
            }
            Err(e) => {
                error!("Plugin discovery failed: {}", e);
            }
        }
        
        builder
    }
}

// NOTE: The old PluginRegistry systems have been removed to avoid conflicts
// with the new FastPluginRegistry implementation. The following systems were
// replaced by equivalent systems in lifecycle.rs that use FastPluginRegistry:
// - plugin_discovery_system -> lifecycle::plugin_health_monitoring_system
// - plugin_lifecycle_system -> lifecycle::plugin_lifecycle_system
// - plugin_cleanup_system -> lifecycle::plugin_resource_coordination_system
//
// This legacy registry implementation is preserved for reference but is not
// actively used in the current system.