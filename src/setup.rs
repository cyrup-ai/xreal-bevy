use anyhow::{Context, Result};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::process::Command;

pub const XREAL_VENDOR_ID: u16 = 0x3318;
pub const XREAL_PRODUCT_ID: u16 = 0x0424;
const CACHE_DURATION_SECS: u64 = 86400; // 24 hours

#[derive(Resource, Default, Debug)]
pub struct LibusbCheckState(pub Option<bool>);

#[derive(Resource, Default, Debug)]
pub struct LibusbInstallStatus(pub Option<bool>);

#[derive(Resource, Default, Debug)]
pub struct GlassesConnectionState(pub Option<bool>);

#[derive(Resource, Default, Debug)]
pub struct CacheValidityState(pub Option<bool>);

#[derive(Resource, Default, Debug)]
pub struct DependencyCheckState(pub Option<bool>);

#[derive(Component)]
pub struct LibusbCheckTask(pub Task<bool>);

#[derive(Component)]
#[allow(dead_code)]
pub struct LibusbInstallTask(pub Task<bool>);

#[derive(Component)]
#[allow(dead_code)]
pub struct GlassesCheckTask(pub Task<bool>);

#[derive(Component)]
#[allow(dead_code)]
pub struct CacheCheckTask(pub Task<bool>);

#[derive(Component)]
#[allow(dead_code)]
pub struct CacheUpdateTask(pub Task<bool>);

#[allow(dead_code)]
pub fn spawn_startup_tasks(mut commands: Commands) {
    info!("🚀 Spawning startup tasks...");
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async_check_libusb_task());
    commands.spawn(LibusbCheckTask(task));

    let task = thread_pool.spawn(async_check_glasses_task());
    commands.spawn(GlassesCheckTask(task));

    let task = thread_pool.spawn(async_check_cache_task());
    commands.spawn(CacheCheckTask(task));
}

#[allow(dead_code)]
pub fn handle_libusb_check_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut LibusbCheckTask)>,
    mut state: ResMut<LibusbCheckState>,
) {
    use futures_lite::future::FutureExt;
    use std::task::{Context, Poll, Waker};

    for (entity, mut task) in task_query.iter_mut() {
        if task.0.is_finished() {
            // Create a no-op waker for non-blocking poll
            let waker = Waker::noop();
            let mut context = Context::from_waker(&waker);

            // Poll the task directly without blocking
            match task.0.poll(&mut context) {
                Poll::Ready(result) => {
                    state.0 = Some(result);
                    if !result {
                        info!("🔧 libusb not found. Spawning installation task...");
                        let install_task =
                            AsyncComputeTaskPool::get().spawn(async_install_libusb_task());
                        commands.spawn(LibusbInstallTask(install_task));
                    }
                    commands.entity(entity).despawn();
                }
                Poll::Pending => {
                    // This shouldn't happen for a finished task, but handle gracefully
                    error!("Task reported as finished but poll returned Pending");
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn handle_libusb_install_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut LibusbInstallTask)>,
    mut status: ResMut<LibusbInstallStatus>,
) {
    use futures_lite::future::FutureExt;
    use std::task::{Context, Poll, Waker};

    for (entity, mut task) in task_query.iter_mut() {
        if task.0.is_finished() {
            let waker = Waker::noop();
            let mut context = Context::from_waker(&waker);

            match task.0.poll(&mut context) {
                Poll::Ready(result) => {
                    status.0 = Some(result);
                    commands.entity(entity).despawn();
                }
                Poll::Pending => {
                    error!("Task reported as finished but poll returned Pending");
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn handle_glasses_check_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut GlassesCheckTask)>,
    mut state: ResMut<GlassesConnectionState>,
) {
    use futures_lite::future::FutureExt;
    use std::task::{Context, Poll, Waker};

    for (entity, mut task) in task_query.iter_mut() {
        if task.0.is_finished() {
            let waker = Waker::noop();
            let mut context = Context::from_waker(&waker);

            match task.0.poll(&mut context) {
                Poll::Ready(result) => {
                    state.0 = Some(result);
                    commands.entity(entity).despawn();
                }
                Poll::Pending => {
                    error!("Task reported as finished but poll returned Pending");
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn handle_cache_check_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut CacheCheckTask)>,
    mut state: ResMut<CacheValidityState>,
) {
    use futures_lite::future::FutureExt;
    use std::task::{Context, Poll, Waker};

    for (entity, mut task) in task_query.iter_mut() {
        if task.0.is_finished() {
            let waker = Waker::noop();
            let mut context = Context::from_waker(&waker);

            match task.0.poll(&mut context) {
                Poll::Ready(result) => {
                    state.0 = Some(result);
                    if !result {
                        info!("🗃️ Cache is stale or invalid. Spawning update task...");
                        let update_task =
                            AsyncComputeTaskPool::get().spawn(async_update_cache_task());
                        commands.spawn(CacheUpdateTask(update_task));
                    }
                    commands.entity(entity).despawn();
                }
                Poll::Pending => {
                    error!("Task reported as finished but poll returned Pending");
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn handle_cache_update_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut CacheUpdateTask)>,
    mut state: ResMut<CacheValidityState>,
) {
    use futures_lite::future::FutureExt;
    use std::task::{Context, Poll, Waker};

    for (entity, mut task) in task_query.iter_mut() {
        if task.0.is_finished() {
            let waker = Waker::noop();
            let mut context = Context::from_waker(&waker);

            match task.0.poll(&mut context) {
                Poll::Ready(result) => {
                    if result {
                        state.0 = Some(true);
                    }
                    commands.entity(entity).despawn();
                }
                Poll::Pending => {
                    error!("Task reported as finished but poll returned Pending");
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn check_startup_completion(
    mut next_state: ResMut<NextState<crate::AppState>>,
    libusb_check: Res<LibusbCheckState>,
    libusb_install: Res<LibusbInstallStatus>,
    glasses_check: Res<GlassesConnectionState>,
    cache_check: Res<CacheValidityState>,
    q_libusb_install: Query<&LibusbInstallTask>,
    q_cache_update: Query<&CacheUpdateTask>,
) {
    if !q_libusb_install.is_empty() || !q_cache_update.is_empty() {
        return;
    }

    let libusb_ok = match libusb_check.0 {
        Some(true) => true,
        Some(false) => libusb_install.0.unwrap_or(false),
        None => return,
    };

    let glasses_ok = glasses_check.0.unwrap_or(false);
    let cache_ok = cache_check.0.unwrap_or(false);

    if libusb_ok && glasses_ok && cache_ok {
        info!("✅ All startup checks passed. Transitioning to Running state.");
        next_state.set(crate::AppState::Running);
    } else if libusb_check.0.is_some() && glasses_check.0.is_some() && cache_check.0.is_some() {
        error!("❌ A startup check failed. Transitioning to ChecksFailed state.");
        next_state.set(crate::AppState::ChecksFailed);
    }
}

#[allow(dead_code)]
pub fn show_failure_message() {
    error!("FATAL: Startup checks failed. Please check the logs for more details. The application cannot continue.");
}

#[allow(dead_code)]
async fn async_check_libusb_task() -> bool {
    info!("Checking for libusb...");
    match Command::new("pkg-config")
        .arg("--exists")
        .arg("libusb-1.0")
        .status()
        .await
    {
        Ok(status) if status.success() => {
            info!("libusb found via pkg-config.");
            true
        }
        _ => {
            info!("pkg-config check failed, trying 'brew list libusb'");
            match Command::new("brew")
                .arg("list")
                .arg("libusb")
                .status()
                .await
            {
                Ok(status) => status.success(),
                Err(_) => false,
            }
        }
    }
}

#[allow(dead_code)]
async fn async_install_libusb_task() -> bool {
    info!("Attempting to install libusb via Homebrew...");
    let brew_installed = match Command::new("which").arg("brew").status().await {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    if !brew_installed {
        error!("Homebrew is not installed. Cannot install libusb automatically.");
        return false;
    }

    match Command::new("brew")
        .arg("install")
        .arg("libusb")
        .status()
        .await
    {
        Ok(status) if status.success() => {
            info!("libusb installed successfully.");
            true
        }
        Ok(_) => {
            error!("'brew install libusb' command failed.");
            false
        }
        Err(e) => {
            error!("Failed to execute 'brew install libusb': {}", e);
            false
        }
    }
}

#[allow(dead_code)]
async fn async_check_glasses_task() -> bool {
    info!("Checking for XREAL glasses connection...");
    match ar_drivers::any_glasses() {
        Ok(_glasses) => {
            info!("✅ XREAL glasses detected.");
            true
        }
        Err(e) => {
            error!("❌ XREAL glasses not detected: {}", e);
            false
        }
    }
}

#[allow(dead_code)]
fn get_cache_file_path() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir().context("Failed to find cache directory")?;
    let app_cache_dir = cache_dir.join("xreal_bevy");
    Ok(app_cache_dir.join("dependency_check.timestamp"))
}

#[allow(dead_code)]
async fn async_check_cache_task() -> bool {
    info!("Checking cache validity...");
    let Ok(cache_file) = get_cache_file_path() else {
        return false;
    };

    let Ok(metadata) = fs::metadata(&cache_file).await else {
        return false;
    };

    let Ok(modified_time) = metadata.modified() else {
        return false;
    };

    let now = SystemTime::now();
    match now.duration_since(modified_time) {
        Ok(duration) if duration.as_secs() < CACHE_DURATION_SECS => {
            info!("✅ Cache is valid.");
            true
        }
        _ => {
            info!("Cache is stale or invalid.");
            false
        }
    }
}

#[allow(dead_code)]
async fn async_update_cache_task() -> bool {
    info!("Updating cache...");
    let Ok(cache_file) = get_cache_file_path() else {
        return false;
    };

    if let Some(parent) = cache_file.parent() {
        if let Err(e) = fs::create_dir_all(parent).await {
            error!("Failed to create cache directory: {}", e);
            return false;
        }
    }

    let now_str = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_secs().to_string(),
        Err(_) => return false, // Should not happen
    };

    match fs::write(&cache_file, now_str.as_bytes()).await {
        Ok(_) => {
            info!("✅ Cache updated successfully.");
            true
        }
        Err(e) => {
            error!("Failed to write to cache file: {}", e);
            false
        }
    }
}
