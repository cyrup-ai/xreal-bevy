use anyhow::Result;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task}
};

const CACHE_FILE: &str = "/tmp/.xreal_libusb_check";
const CACHE_DURATION_HOURS: u64 = 24;

#[derive(Resource, Clone)]
pub struct LibusbCheckState {
    pub is_installed: Option<bool>,
    pub is_checking: bool,
}

impl Default for LibusbCheckState {
    fn default() -> Self {
        Self {
            is_installed: None,
            is_checking: false,
        }
    }
}

#[derive(Resource, Clone)]
pub struct LibusbInstallStatus {
    pub install_result: Option<Result<(), String>>,
    pub is_installing: bool,
}

impl Default for LibusbInstallStatus {
    fn default() -> Self {
        Self {
            install_result: None,
            is_installing: false,
        }
    }
}


#[derive(Resource, Clone)]
pub struct GlassesConnectionState {
    pub is_connected: Option<bool>,
    pub is_checking: bool,
}

impl Default for GlassesConnectionState {
    fn default() -> Self {
        Self {
            is_connected: None,
            is_checking: false,
        }
    }
}

#[derive(Resource, Clone)]
pub struct CacheValidityState {
    pub is_valid: Option<bool>,
    pub is_checking: bool,
    pub is_updating: bool,
}

impl Default for CacheValidityState {
    fn default() -> Self {
        Self {
            is_valid: None,
            is_checking: false,
            is_updating: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct DependencyCheckState {
    pub cache_checked: bool,
    pub libusb_checked: bool,
    pub glasses_checked: bool,
    pub dependencies_ready: bool,
    pub needs_libusb_install: bool,
}

#[derive(Component)]
pub struct LibusbCheckTask(pub Task<CommandQueue>);

#[derive(Component)]
pub struct LibusbInstallTask(pub Task<CommandQueue>);


#[derive(Component)]
pub struct GlassesCheckTask(pub Task<CommandQueue>);

#[derive(Component)]
pub struct CacheCheckTask(pub Task<CommandQueue>);

#[derive(Component)]
pub struct CacheUpdateTask(pub Task<CommandQueue>);

#[inline]
pub fn ensure_dependencies() -> Result<()> {
    // Only perform quick synchronous cache check
    if is_cache_valid() {
        println!("✅ Dependencies verified (cached for 24h)");
        return Ok(());
    }
    
    println!("🔍 Checking system dependencies...");
    println!("📋 Dependency verification proceeding asynchronously...");
    
    // All actual dependency checking now handled by Bevy async systems
    // This allows the main thread to continue without blocking
    Ok(())
}

#[inline]
fn is_cache_valid() -> bool {
    if let Ok(metadata) = fs::metadata(CACHE_FILE) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(duration) => duration,
                    Err(_) => return false,
                };
                let hours_elapsed = (now.as_secs() - duration.as_secs()) / 3600;
                return hours_elapsed < CACHE_DURATION_HOURS;
            }
        }
    }
    false
}

/// Async task to check if libusb is installed using pkg-config
pub fn async_check_libusb_task() -> LibusbCheckTask {
    let thread_pool = AsyncComputeTaskPool::get();
    
    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Use async-process::Command for non-blocking system call
        let result = async_process::Command::new("pkg-config")
            .args(&["--exists", "libusb-1.0"])
            .output()
            .await;
            
        let is_installed = result
            .map(|output| output.status.success())
            .unwrap_or(false);
            
        command_queue.push(move |world: &mut World| {
            if let Some(mut libusb_state) = world.get_resource_mut::<LibusbCheckState>() {
                libusb_state.is_installed = Some(is_installed);
                libusb_state.is_checking = false;
            }
        });
        
        command_queue
    });
    
    LibusbCheckTask(task)
}

/// Async task to install libusb using Homebrew
pub fn async_install_libusb_task() -> LibusbInstallTask {
    let thread_pool = AsyncComputeTaskPool::get();
    
    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Check if Homebrew is installed first
        let brew_check = async_process::Command::new("which")
            .arg("brew")
            .output()
            .await;
            
        if let Err(_) = brew_check {
            command_queue.push(move |world: &mut World| {
                if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                    install_status.install_result = Some(Err("Homebrew not found. Please install Homebrew first".to_string()));
                    install_status.is_installing = false;
                }
            });
            return command_queue;
        }
        
        let brew_output = match brew_check {
            Ok(output) => output,
            Err(_) => {
                command_queue.push(move |world: &mut World| {
                    if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                        install_status.install_result = Some(Err("Failed to execute which brew command".to_string()));
                        install_status.is_installing = false;
                    }
                });
                return command_queue;
            }
        };
        
        if !brew_output.status.success() {
            command_queue.push(move |world: &mut World| {
                if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                    install_status.install_result = Some(Err("Homebrew not found. Please install Homebrew first".to_string()));
                    install_status.is_installing = false;
                }
            });
            return command_queue;
        }
        
        // Install libusb via brew
        let install_result = async_process::Command::new("brew")
            .args(&["install", "libusb"])
            .output()
            .await;
            
        match install_result {
            Ok(output) => {
                if output.status.success() {
                    command_queue.push(move |world: &mut World| {
                        if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                            install_status.install_result = Some(Ok(()));
                            install_status.is_installing = false;
                        }
                    });
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("already installed") {
                        command_queue.push(move |world: &mut World| {
                            if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                                install_status.install_result = Some(Ok(()));
                                install_status.is_installing = false;
                            }
                        });
                    } else {
                        let error_msg = format!("Failed to install libusb: {}", stderr);
                        command_queue.push(move |world: &mut World| {
                            if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                                install_status.install_result = Some(Err(error_msg));
                                install_status.is_installing = false;
                            }
                        });
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute brew command: {}", e);
                command_queue.push(move |world: &mut World| {
                    if let Some(mut install_status) = world.get_resource_mut::<LibusbInstallStatus>() {
                        install_status.install_result = Some(Err(error_msg));
                        install_status.is_installing = false;
                    }
                });
            }
        }
        
        command_queue
    });
    
    LibusbInstallTask(task)
}


/// Async task to check glasses connection using system_profiler
pub fn async_check_glasses_task() -> GlassesCheckTask {
    let thread_pool = AsyncComputeTaskPool::get();
    
    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Check USB devices for XREAL/Nreal identifiers using async system_profiler
        let lsusb_result = async_process::Command::new("system_profiler")
            .args(&["SPUSBDataType"])
            .output()
            .await;
            
        let is_connected = if let Ok(output) = lsusb_result {
            let usb_info = String::from_utf8_lossy(&output.stdout);
            usb_info.to_lowercase().contains("nreal") || 
            usb_info.to_lowercase().contains("xreal") ||
            usb_info.to_lowercase().contains("0x3318") ||  // XREAL vendor ID
            usb_info.to_lowercase().contains("0x0486")     // Alternative vendor ID
        } else {
            false
        };
        
        command_queue.push(move |world: &mut World| {
            if let Some(mut glasses_state) = world.get_resource_mut::<GlassesConnectionState>() {
                glasses_state.is_connected = Some(is_connected);
                glasses_state.is_checking = false;
            }
        });
        
        command_queue
    });
    
    GlassesCheckTask(task)
}

/// Async task to check cache validity using async file operations
pub fn async_check_cache_task() -> CacheCheckTask {
    let thread_pool = AsyncComputeTaskPool::get();
    
    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Check cache file validity using async file operations
        let is_valid = if let Ok(metadata) = async_std::fs::metadata(CACHE_FILE).await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                        Ok(duration) => duration,
                        Err(_) => {
                            command_queue.push(move |world: &mut World| {
                                if let Some(mut cache_state) = world.get_resource_mut::<CacheValidityState>() {
                                    cache_state.is_valid = Some(false);
                                    cache_state.is_checking = false;
                                }
                            });
                            return command_queue;
                        }
                    };
                    let hours_elapsed = (now.as_secs() - duration.as_secs()) / 3600;
                    hours_elapsed < CACHE_DURATION_HOURS
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        
        command_queue.push(move |world: &mut World| {
            if let Some(mut cache_state) = world.get_resource_mut::<CacheValidityState>() {
                cache_state.is_valid = Some(is_valid);
                cache_state.is_checking = false;
            }
        });
        
        command_queue
    });
    
    CacheCheckTask(task)
}

/// Async task to update cache using async file operations
pub fn async_update_cache_task() -> CacheUpdateTask {
    let thread_pool = AsyncComputeTaskPool::get();
    
    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();
        
        // Update cache file using async file operations
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());
        
        match async_std::fs::write(CACHE_FILE, timestamp).await {
            Ok(_) => {
                command_queue.push(move |world: &mut World| {
                    if let Some(mut cache_state) = world.get_resource_mut::<CacheValidityState>() {
                        cache_state.is_updating = false;
                        // Mark as valid since we just updated it
                        cache_state.is_valid = Some(true);
                    }
                });
            }
            Err(_) => {
                command_queue.push(move |world: &mut World| {
                    if let Some(mut cache_state) = world.get_resource_mut::<CacheValidityState>() {
                        cache_state.is_updating = false;
                    }
                });
            }
        }
        
        command_queue
    });
    
    CacheUpdateTask(task)
}


/// System to handle completed libusb check tasks
pub fn handle_libusb_check_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut LibusbCheckTask)>,
) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for (entity, mut task) in &mut tasks {
        // Poll the task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply the command queue to execute deferred world modifications
            commands.append(&mut command_queue);
            // Remove the completed task
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle completed libusb install tasks
pub fn handle_libusb_install_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut LibusbInstallTask)>,
) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for (entity, mut task) in &mut tasks {
        // Poll the task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply the command queue to execute deferred world modifications
            commands.append(&mut command_queue);
            // Remove the completed task
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle completed glasses check tasks
pub fn handle_glasses_check_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut GlassesCheckTask)>,
) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for (entity, mut task) in &mut tasks {
        // Poll the task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply the command queue to execute deferred world modifications
            commands.append(&mut command_queue);
            // Remove the completed task
            commands.entity(entity).despawn();
        }
    }
}


/// System to handle completed cache check tasks
pub fn handle_cache_check_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CacheCheckTask)>,
) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for (entity, mut task) in &mut tasks {
        // Poll the task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply the command queue to execute deferred world modifications
            commands.append(&mut command_queue);
            // Remove the completed task
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle completed cache update tasks
pub fn handle_cache_update_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut CacheUpdateTask)>,
) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for (entity, mut task) in &mut tasks {
        // Poll the task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply the command queue to execute deferred world modifications
            commands.append(&mut command_queue);
            // Remove the completed task
            commands.entity(entity).despawn();
        }
    }
}