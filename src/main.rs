use anyhow::Result;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::window::{WindowPlugin, WindowPosition};
use bevy::{
    ecs::world::CommandQueue,
    tasks::{AsyncComputeTaskPool, Task}
};
use crossbeam_channel::{bounded, Receiver, Sender};
use std::time::Duration;


mod setup;
mod driver;
mod tracking;
mod capture;
mod render;
mod ui;
mod input;

use driver::configure_display;
use tracking::{Orientation, CalibrationState, Command, Data};
use capture::ScreenCaptures;
use render::setup_3d_scene;
use ui::{settings_ui, reset_ui_guard};
use input::handle_input;
use setup::{LibusbCheckState, LibusbInstallStatus, GlassesConnectionState, FramerateDetectionResult, CacheValidityState, DependencyCheckState, handle_libusb_check_task, handle_libusb_install_task, handle_glasses_check_task, handle_framerate_detection_task, handle_cache_check_task, handle_cache_update_task};

#[derive(Component)]
struct ImuTask(Task<CommandQueue>);

#[derive(Component)]
struct CaptureInitTask(Task<CommandQueue>);

#[derive(Resource)]
struct ScreenDistance(f32);

#[derive(Resource)]
struct CommandChannel(Sender<Command>);

#[derive(Resource)]
struct DataChannel(Receiver<Data>);

#[derive(Resource)]
struct ImuChannels {
    tx_data: Sender<Data>,
    rx_command: Receiver<Command>,
}

#[derive(Resource, Default)]
struct SystemsSpawnedState {
    imu_spawned: bool,
    capture_spawned: bool,
}

#[derive(Resource)]
struct JitterMetrics {
    frame_times: Vec<f32>,
    imu_intervals: Vec<f32>,
    capture_intervals: Vec<f32>,
    last_frame_time: f32,
    last_imu_time: f32,
    last_capture_time: f32,
    frame_variance_threshold: f32,
    history_buffer_size: usize,
}

impl Default for JitterMetrics {
    fn default() -> Self {
        Self {
            frame_times: Vec::with_capacity(1000),
            imu_intervals: Vec::with_capacity(1000),
            capture_intervals: Vec::with_capacity(1000),
            last_frame_time: 0.0,
            last_imu_time: 0.0,
            last_capture_time: 0.0,
            frame_variance_threshold: 1.0, // 1ms threshold
            history_buffer_size: 1000,
        }
    }
}

fn orchestrate_dependency_startup(
    mut commands: Commands,
    dependency_state: Res<DependencyCheckState>,
    cache_state: Res<CacheValidityState>,
) {
    // If cache was invalid (checked in ensure_dependencies), start async dependency flow
    if !dependency_state.cache_checked && cache_state.is_valid.is_none() {
        info!("🔍 Starting async dependency verification...");
        // Spawn cache check task first
        commands.spawn(setup::async_check_cache_task());
    }
}

fn orchestrate_dependency_flow(
    mut commands: Commands,
    mut dependency_state: ResMut<DependencyCheckState>,
    cache_state: Res<CacheValidityState>,
    libusb_state: Res<LibusbCheckState>,
    install_status: Res<LibusbInstallStatus>,
    glasses_state: Res<GlassesConnectionState>,
    // Check for active tasks to avoid duplicate spawning
    cache_tasks: Query<Entity, With<setup::CacheCheckTask>>,
    libusb_tasks: Query<Entity, With<setup::LibusbCheckTask>>,
    install_tasks: Query<Entity, With<setup::LibusbInstallTask>>,
    glasses_tasks: Query<Entity, With<setup::GlassesCheckTask>>,
    cache_update_tasks: Query<Entity, With<setup::CacheUpdateTask>>,
) {
    // Step 1: Cache check completed, proceed to libusb check
    if !dependency_state.cache_checked && cache_state.is_valid == Some(false) && cache_tasks.is_empty() {
        info!("📦 Cache invalid, checking libusb installation...");
        commands.spawn(setup::async_check_libusb_task());
        dependency_state.cache_checked = true;
    }
    
    // Step 2: Libusb check completed, install if needed
    if dependency_state.cache_checked && !dependency_state.libusb_checked && libusb_tasks.is_empty() {
        if let Some(is_installed) = libusb_state.is_installed {
            if !is_installed && install_tasks.is_empty() {
                info!("📦 libusb not found. Installing via Homebrew...");
                commands.spawn(setup::async_install_libusb_task());
                dependency_state.needs_libusb_install = true;
            } else if is_installed {
                info!("✅ libusb is installed");
                dependency_state.libusb_checked = true;
            }
        }
    }
    
    // Step 3: Installation completed or not needed, proceed to glasses check
    if dependency_state.cache_checked && install_tasks.is_empty() && glasses_tasks.is_empty() {
        let libusb_ready = if dependency_state.needs_libusb_install {
            match install_status.install_result.as_ref() {
                Some(result) => match result {
                    Ok(_) => {
                        debug!("✅ Libusb installation completed successfully");
                        true
                    }
                    Err(err) => {
                        error!("❌ Libusb installation failed: {}", err);
                        false
                    }
                }
                None => {
                    debug!("⏳ Libusb installation still in progress");
                    false
                }
            }
        } else {
            match libusb_state.is_installed {
                Some(installed) => {
                    if installed {
                        debug!("✅ Libusb already installed");
                    } else {
                        debug!("❌ Libusb not installed");
                    }
                    installed
                }
                None => {
                    debug!("⏳ Libusb installation status still being checked");
                    false
                }
            }
        };
        
        if libusb_ready && !dependency_state.glasses_checked {
            info!("🔌 Checking for XREAL glasses...");
            commands.spawn(setup::async_check_glasses_task());
            dependency_state.libusb_checked = true;
        }
    }
    
    // Step 4: Glasses check completed, update cache
    if dependency_state.libusb_checked && !dependency_state.glasses_checked && glasses_tasks.is_empty() {
        if let Some(connected) = glasses_state.is_connected {
            match connected {
                true => {
                    info!("✅ XREAL glasses detected via USB");
                }
                false => {
                    info!("⚠️  XREAL glasses not detected - proceeding anyway");
                    debug!("🔍 Glasses connection check completed with negative result");
                }
            }
            
            if cache_update_tasks.is_empty() {
                info!("💾 Updating dependency cache...");
                commands.spawn(setup::async_update_cache_task());
            }
            dependency_state.glasses_checked = true;
        }
    }
    
    // Step 5: All checks completed
    if dependency_state.cache_checked && dependency_state.libusb_checked && dependency_state.glasses_checked && cache_update_tasks.is_empty() && !dependency_state.dependencies_ready {
        info!("✅ All dependency checks completed asynchronously");
        dependency_state.dependencies_ready = true;
    }
}

fn conditional_system_startup(
    mut commands: Commands,
    dependency_state: Res<DependencyCheckState>,
    mut spawned_state: ResMut<SystemsSpawnedState>,
    mut channels: ResMut<ImuChannels>,
) {
    // Only spawn systems after dependencies are ready
    if dependency_state.dependencies_ready {
        // Spawn IMU task if not already spawned
        if !spawned_state.imu_spawned {
            let thread_pool = AsyncComputeTaskPool::get();
            let tx_data = channels.tx_data.clone();
            let rx_command = std::mem::replace(&mut channels.rx_command, bounded(1).1);
            
            let task = thread_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();
                
                // Spawn the IMU polling task using the new pattern
                if let Ok(result) = tracking::poll_imu_bevy(rx_command, tx_data).await {
                    // Use command queue for any world updates needed
                    command_queue.push(move |_world: &mut World| {
                        // IMU task completed successfully
                        info!("IMU task completed: {:?}", result);
                    });
                } else {
                    command_queue.push(move |_world: &mut World| {
                        error!("IMU task failed");
                    });
                }
                
                command_queue
            });
            
            commands.spawn(ImuTask(task));
            spawned_state.imu_spawned = true;
            info!("✅ IMU system spawned after dependencies ready");
        }
        
        // Spawn capture init task if not already spawned
        if !spawned_state.capture_spawned {
            let thread_pool = AsyncComputeTaskPool::get();
            
            let task = thread_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();
                
                // Initialize screen capture asynchronously
                match ScreenCaptures::new_async().await {
                    Ok(capture) => {
                        command_queue.push(move |world: &mut World| {
                            world.insert_resource(capture);
                            info!("Screen capture initialized asynchronously with optimal framerate");
                        });
                    }
                    Err(e) => {
                        command_queue.push(move |world: &mut World| {
                            // Fall back to synchronous initialization
                            match ScreenCaptures::new() {
                                Ok(capture) => {
                                    world.insert_resource(capture);
                                    warn!("Screen capture initialized synchronously after async failure: {}", e);
                                }
                                Err(sync_err) => {
                                    error!("Both async and sync screen capture initialization failed: {} / {}", e, sync_err);
                                }
                            }
                        });
                    }
                }
                
                command_queue
            });
            
            commands.spawn(CaptureInitTask(task));
            spawned_state.capture_spawned = true;
            info!("✅ Screen capture system spawned after dependencies ready");
        }
    }
}

fn jitter_measurement_system(
    time: Res<Time>,
    mut jitter_metrics: ResMut<JitterMetrics>,
    data_channel: Res<DataChannel>,
) {
    use std::time::Instant;
    
    let current_time = time.elapsed_secs() * 1000.0; // Convert to milliseconds
    
    // Measure frame timing
    if jitter_metrics.last_frame_time > 0.0 {
        let frame_interval = current_time - jitter_metrics.last_frame_time;
        
        // Add to circular buffer
        if jitter_metrics.frame_times.len() >= jitter_metrics.history_buffer_size {
            jitter_metrics.frame_times.remove(0);
        }
        jitter_metrics.frame_times.push(frame_interval);
        
        // Check for jitter violations
        if jitter_metrics.frame_times.len() > 10 {
            let mean = jitter_metrics.frame_times.iter().sum::<f32>() / jitter_metrics.frame_times.len() as f32;
            let variance = jitter_metrics.frame_times.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f32>() / jitter_metrics.frame_times.len() as f32;
            let std_dev = variance.sqrt();
            
            // Log jitter violations
            if std_dev > jitter_metrics.frame_variance_threshold {
                warn!("⚠️ Jitter violation detected! Frame variance: {:.2}ms (threshold: {:.2}ms)", 
                      std_dev, jitter_metrics.frame_variance_threshold);
            }
            
            // Log statistics every 60 frames
            if jitter_metrics.frame_times.len() % 60 == 0 {
                let mut sorted_times = jitter_metrics.frame_times.clone();
                sorted_times.sort_by(|a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                });
                let p99_index = (sorted_times.len() as f32 * 0.99) as usize;
                let p99 = match sorted_times.get(p99_index) {
                    Some(value) => *value,
                    None => {
                        error!("❌ Failed to calculate 99th percentile: index {} out of bounds for {} samples", 
                               p99_index, sorted_times.len());
                        0.0
                    }
                };
                
                debug!("📊 Jitter Stats - Mean: {:.2}ms, StdDev: {:.2}ms, 99th: {:.2}ms", 
                       mean, std_dev, p99);
            }
        }
    }
    jitter_metrics.last_frame_time = current_time;
    
    // Measure IMU data intervals
    let mut imu_data_received = false;
    while let Ok(_data) = data_channel.0.try_recv() {
        imu_data_received = true;
        if jitter_metrics.last_imu_time > 0.0 {
            let imu_interval = current_time - jitter_metrics.last_imu_time;
            
            // Add to circular buffer
            if jitter_metrics.imu_intervals.len() >= jitter_metrics.history_buffer_size {
                jitter_metrics.imu_intervals.remove(0);
            }
            jitter_metrics.imu_intervals.push(imu_interval);
        }
        jitter_metrics.last_imu_time = current_time;
    }
}

fn main() -> Result<()> {
    println!("🥽 XREAL Virtual Desktop - Starting up...");
    
    // Ensure dependencies are installed
    setup::ensure_dependencies()?;
    let (tx_command, rx_command) = bounded::<Command>(1);
    let (tx_data, rx_data) = bounded::<Data>(1);

    configure_display()?;

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "XREAL Virtual Desktop".into(),
                    resolution: (400., 300.).into(),
                    resizable: false,
                    decorations: true,
                    transparent: false,
                    window_level: bevy::window::WindowLevel::AlwaysOnTop,
                    position: WindowPosition::Automatic,
                    ..default()
                }),
                ..default()
            }), 
            EguiPlugin::default(), 
            FrameTimeDiagnosticsPlugin::default()
        ))
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(1)))
        .insert_resource(DataChannel(rx_data))
        .insert_resource(CommandChannel(tx_command))
        .insert_resource(ImuChannels { tx_data, rx_command })
        .insert_resource(Orientation::default())
        .insert_resource(CalibrationState::default())
        .insert_resource(ScreenDistance(-5.0))
        .insert_resource(ui::UiRenderGuard::default())
        .insert_resource(LibusbCheckState::default())
        .insert_resource(LibusbInstallStatus::default())
        .insert_resource(GlassesConnectionState::default())
        .insert_resource(FramerateDetectionResult::default())
        .insert_resource(CacheValidityState::default())
        .insert_resource(DependencyCheckState::default())
        .insert_resource(SystemsSpawnedState::default())
        .insert_resource(JitterMetrics::default())
        .add_systems(Startup, (setup_3d_scene, orchestrate_dependency_startup))
        .add_systems(FixedUpdate, (update_from_data_channel, handle_imu_task, handle_capture_init_task))
        .add_systems(FixedUpdate, render::update_camera_from_orientation)
        .add_systems(FixedUpdate, render::spawn_capture_tasks)
        .add_systems(FixedUpdate, render::handle_capture_tasks)
        .add_systems(FixedUpdate, handle_libusb_check_task)
        .add_systems(FixedUpdate, handle_libusb_install_task)
        .add_systems(FixedUpdate, handle_glasses_check_task)
        .add_systems(FixedUpdate, handle_framerate_detection_task)
        .add_systems(FixedUpdate, handle_cache_check_task)
        .add_systems(FixedUpdate, handle_cache_update_task)
        .add_systems(FixedUpdate, orchestrate_dependency_flow)
        .add_systems(FixedUpdate, conditional_system_startup.after(orchestrate_dependency_flow))
        .add_systems(Update, (reset_ui_guard, settings_ui).chain())
        .add_systems(FixedUpdate, handle_input)
        .add_systems(FixedUpdate, log_fps)
        .add_systems(FixedUpdate, render::update_screen_positions)
        .add_systems(FixedUpdate, jitter_measurement_system)
        .run();

    Ok(())
}

fn handle_capture_init_task(mut commands: Commands, mut tasks: Query<(Entity, &mut CaptureInitTask)>) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for (entity, mut task) in &mut tasks {
        // Poll the capture init task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply any world updates from the capture init task
            commands.append(&mut command_queue);
            // Remove the completed task
            commands.entity(entity).despawn();
        }
    }
}

fn handle_imu_task(mut commands: Commands, mut tasks: Query<&mut ImuTask>) {
    use bevy::tasks::{futures_lite::future, block_on};
    
    for mut task in &mut tasks {
        // Poll the IMU task non-blocking
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Apply any world updates from the IMU task
            commands.append(&mut command_queue);
        }
    }
}

fn update_from_data_channel(
    rx: ResMut<DataChannel>,
    mut orientation: ResMut<Orientation>,
    mut cal_state: ResMut<CalibrationState>,
) {
    while let Ok(data) = rx.0.try_recv() {
        match data {
            Data::Orientation(q) => orientation.quat = q,
            Data::CalState(s) => *cal_state = s,
        }
    }
}

fn log_fps(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.average() {
            trace!("FPS: {}", value);
        }
    }
}