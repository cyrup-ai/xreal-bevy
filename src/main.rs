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
mod plugins;

use driver::configure_display;
use tracking::{Orientation, CalibrationState, Command, Data};
use capture::ScreenCaptures;
use render::setup_3d_scene;
use ui::{settings_ui, reset_ui_guard};
use input::handle_input;
use setup::{LibusbCheckState, LibusbInstallStatus, GlassesConnectionState, CacheValidityState, DependencyCheckState, handle_libusb_check_task, handle_libusb_install_task, handle_glasses_check_task, handle_cache_check_task, handle_cache_update_task};

#[derive(Component)]
struct ImuTask(Task<CommandQueue>);

#[derive(Component)]
struct CaptureInitTask(Task<CommandQueue>);

#[derive(Resource)]
struct ScreenDistance(f32);

#[derive(Resource)]
struct DisplayModeState {
    is_3d_enabled: bool,
    pending_change: Option<bool>,
}

impl Default for DisplayModeState {
    fn default() -> Self {
        Self {
            is_3d_enabled: true, // Default to 3D mode
            pending_change: None,
        }
    }
}

#[derive(Resource)]
struct RollLockState {
    is_enabled: bool,
    pending_change: Option<bool>,
}

impl Default for RollLockState {
    fn default() -> Self {
        Self {
            is_enabled: false, // Default roll lock disabled
            pending_change: None,
        }
    }
}

#[derive(Resource)]
struct BrightnessState {
    current_level: u8,        // Current brightness 0-7
    pending_change: Option<u8>, // Pending brightness change
}

impl Default for BrightnessState {
    fn default() -> Self {
        Self {
            current_level: 4,  // Mid-range default
            pending_change: None,
        }
    }
}

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

#[derive(Resource, Default)]
struct SystemStatus {
    current_fps: Option<f32>,
    connection_status: bool,
    capture_active: bool,
}

#[derive(Resource, Default)]
struct SettingsPanelState {
    is_open: bool,
    selected_preset: DisplayPreset,
    performance_monitoring: bool,
    advanced_calibration: bool,
}

#[derive(Resource, Default)]
struct TopMenuState {
    is_hovering: bool,
    is_menu_open: bool,
    selected_tab: AppTab,
    hover_timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AppTab {
    #[default]
    Browser,
    Terminal,
    VSCode,
    Files,
    Media,
    Games,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayPreset {
    Gaming,
    Productivity,
    Cinema,
}

impl Default for DisplayPreset {
    fn default() -> Self {
        DisplayPreset::Productivity
    }
}

/// Zero-allocation jitter measurement with fixed-size ring buffers
/// Provides O(1) operations and incremental statistics tracking
#[derive(Resource)]
struct JitterMetrics<const BUFFER_SIZE: usize = 1000> {
    // Fixed-size ring buffers - zero heap allocations
    frame_times: [f32; BUFFER_SIZE],
    imu_intervals: [f32; BUFFER_SIZE],
    capture_intervals: [f32; BUFFER_SIZE],
    
    // Ring buffer indices for O(1) operations
    frame_write_idx: usize,
    imu_write_idx: usize,
    capture_write_idx: usize,
    
    // Element counts for partial buffer fills
    frame_count: usize,
    imu_count: usize,
    capture_count: usize,
    
    // Welford's algorithm state for incremental variance calculation
    frame_mean: f32,
    frame_m2: f32,  // Sum of squares of differences from mean
    imu_mean: f32,
    imu_m2: f32,
    
    // Previous timing values
    last_frame_time: f32,
    last_imu_time: f32,
    last_capture_time: f32,
    
    // Configuration constants
    frame_variance_threshold: f32,
    stats_update_interval: usize,
    stats_counter: usize,
}

impl<const BUFFER_SIZE: usize> Default for JitterMetrics<BUFFER_SIZE> {
    #[inline]
    fn default() -> Self {
        Self {
            frame_times: [0.0; BUFFER_SIZE],
            imu_intervals: [0.0; BUFFER_SIZE],
            capture_intervals: [0.0; BUFFER_SIZE],
            frame_write_idx: 0,
            imu_write_idx: 0,
            capture_write_idx: 0,
            frame_count: 0,
            imu_count: 0,
            capture_count: 0,
            frame_mean: 0.0,
            frame_m2: 0.0,
            imu_mean: 0.0,
            imu_m2: 0.0,
            last_frame_time: 0.0,
            last_imu_time: 0.0,
            last_capture_time: 0.0,
            frame_variance_threshold: 1.0, // 1ms threshold
            stats_update_interval: 60,     // Log every 60 frames
            stats_counter: 0,
        }
    }
}

impl<const BUFFER_SIZE: usize> JitterMetrics<BUFFER_SIZE> {
    /// Add frame timing measurement using Welford's online algorithm
    /// Provides O(1) variance calculation without storing all values
    #[inline]
    fn add_frame_measurement(&mut self, interval: f32) {
        // Update ring buffer
        self.frame_times[self.frame_write_idx] = interval;
        self.frame_write_idx = (self.frame_write_idx + 1) % BUFFER_SIZE;
        
        // Update Welford's algorithm state
        self.frame_count += 1;
        let delta = interval - self.frame_mean;
        self.frame_mean += delta / self.frame_count as f32;
        let delta2 = interval - self.frame_mean;
        self.frame_m2 += delta * delta2;
        
        // Limit count to buffer size for proper variance calculation
        if self.frame_count > BUFFER_SIZE {
            self.frame_count = BUFFER_SIZE;
        }
    }
    
    /// Add IMU timing measurement using Welford's online algorithm
    /// Provides O(1) variance calculation for IMU interval consistency
    #[inline]
    fn add_imu_measurement(&mut self, interval: f32) {
        // Update ring buffer
        self.imu_intervals[self.imu_write_idx] = interval;
        self.imu_write_idx = (self.imu_write_idx + 1) % BUFFER_SIZE;
        
        // Update Welford's algorithm state for IMU
        self.imu_count += 1;
        let delta = interval - self.imu_mean;
        self.imu_mean += delta / self.imu_count as f32;
        let delta2 = interval - self.imu_mean;
        self.imu_m2 += delta * delta2;
        
        // Limit count to buffer size for proper variance calculation
        if self.imu_count > BUFFER_SIZE {
            self.imu_count = BUFFER_SIZE;
        }
    }
    
    /// Add capture timing measurement using ring buffer
    /// Tracks screen capture frame delivery timing consistency
    #[inline]
    fn add_capture_measurement(&mut self, interval: f32) {
        // Update ring buffer
        self.capture_intervals[self.capture_write_idx] = interval;
        self.capture_write_idx = (self.capture_write_idx + 1) % BUFFER_SIZE;
        
        // Update count
        if self.capture_count < BUFFER_SIZE {
            self.capture_count += 1;
        }
    }
    
    /// Get current IMU variance using Welford's algorithm
    #[inline]
    fn imu_variance(&self) -> f32 {
        if self.imu_count < 2 {
            return 0.0;
        }
        self.imu_m2 / (self.imu_count - 1) as f32
    }
    
    /// Get current IMU standard deviation
    #[inline]
    fn imu_std_dev(&self) -> f32 {
        self.imu_variance().sqrt()
    }
    
    /// Get current frame variance using Welford's algorithm
    #[inline]
    fn frame_variance(&self) -> f32 {
        if self.frame_count < 2 {
            return 0.0;
        }
        self.frame_m2 / (self.frame_count - 1) as f32
    }
    
    /// Get current frame standard deviation
    #[inline]
    fn frame_std_dev(&self) -> f32 {
        self.frame_variance().sqrt()
    }
    
    /// Calculate percentile using quickselect algorithm on ring buffer
    /// Returns percentile value without heap allocation
    #[inline]
    fn frame_percentile(&self, percentile: f32) -> f32 {
        if self.frame_count == 0 {
            return 0.0;
        }
        
        let count = self.frame_count.min(BUFFER_SIZE);
        let target_idx = ((count as f32 * percentile / 100.0) as usize).min(count - 1);
        
        // Create indices array for sorting without allocating the values
        let mut indices = [0usize; BUFFER_SIZE];
        for i in 0..count {
            indices[i] = i;
        }
        
        // Quickselect implementation using indices
        Self::quickselect(&self.frame_times, &mut indices[..count], target_idx)
    }
    
    /// Quickselect algorithm implementation for finding kth element
    /// Operates on indices to avoid copying data
    #[inline]
    fn quickselect(values: &[f32], indices: &mut [usize], k: usize) -> f32 {
        if indices.len() <= 1 {
            return if indices.is_empty() { 0.0 } else { values[indices[0]] };
        }
        
        let pivot_idx = Self::partition(values, indices);
        
        if k == pivot_idx {
            values[indices[k]]
        } else if k < pivot_idx {
            Self::quickselect(values, &mut indices[..pivot_idx], k)
        } else {
            Self::quickselect(values, &mut indices[pivot_idx + 1..], k - pivot_idx - 1)
        }
    }
    
    /// Partition function for quickselect
    #[inline]
    fn partition(values: &[f32], indices: &mut [usize]) -> usize {
        let len = indices.len();
        if len <= 1 {
            return 0;
        }
        
        let pivot_value = values[indices[len / 2]];
        indices.swap(len / 2, len - 1);
        
        let mut store_idx = 0;
        for i in 0..len - 1 {
            if values[indices[i]] <= pivot_value {
                indices.swap(i, store_idx);
                store_idx += 1;
            }
        }
        indices.swap(store_idx, len - 1);
        store_idx
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

fn display_mode_system(
    mut display_mode_state: ResMut<DisplayModeState>,
) {
    // Handle pending display mode changes
    if let Some(new_mode) = display_mode_state.pending_change {
        // Initialize glasses device for this operation
        match driver::init_glasses() {
            Ok(device) => {
                match device.set_display_mode(new_mode) {
                    Ok(_) => {
                        display_mode_state.is_3d_enabled = new_mode;
                        debug!("✅ Display mode changed to {}", if new_mode { "3D Stereo" } else { "2D Mirror" });
                    }
                    Err(e) => {
                        error!("❌ Failed to change display mode: {}", e);
                        // Keep the current state, don't update is_3d_enabled
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to initialize glasses device for display mode change: {}", e);
            }
        }
        // Clear pending change regardless of success/failure
        display_mode_state.pending_change = None;
    }
}

fn roll_lock_system(
    mut roll_lock_state: ResMut<RollLockState>,
    command_channel: Res<CommandChannel>,
) {
    // Handle pending roll lock changes
    if let Some(new_enabled) = roll_lock_state.pending_change {
        // Send roll lock command to tracking system
        match command_channel.0.try_send(Command::SetRollLock(new_enabled)) {
            Ok(_) => {
                roll_lock_state.is_enabled = new_enabled;
                debug!("✅ Roll lock {}", if new_enabled { "enabled" } else { "disabled" });
            }
            Err(e) => {
                error!("❌ Failed to send roll lock command: {}", e);
                // Keep the current state, don't update is_enabled
            }
        }
        // Clear pending change regardless of success/failure
        roll_lock_state.pending_change = None;
    }
}

fn brightness_control_system(
    mut brightness_state: ResMut<BrightnessState>,
    command_channel: Res<CommandChannel>,
) {
    if let Some(new_level) = brightness_state.pending_change {
        // Validate brightness range (0-7)
        let clamped_level = new_level.min(7);
        
        match command_channel.0.try_send(Command::SetBrightness(clamped_level)) {
            Ok(_) => {
                brightness_state.current_level = clamped_level;
                debug!("✅ Brightness set to level {}", clamped_level);
            }
            Err(e) => {
                error!("❌ Failed to send brightness command: {}", e);
                // Keep the current state on failure
            }
        }
        brightness_state.pending_change = None;
    }
}

fn system_status_update_system(
    mut system_status: ResMut<SystemStatus>,
    diagnostics: Res<DiagnosticsStore>,
    glasses_state: Res<GlassesConnectionState>,
    screen_captures: Option<Res<ScreenCaptures>>,
) {
    // Update FPS from FrameTimeDiagnosticsPlugin
    system_status.current_fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
        .map(|fps| fps as f32);
    
    // Update connection status from GlassesConnectionState
    system_status.connection_status = match glasses_state.is_connected {
        Some(connected) => connected,
        None => false,
    };
    
    // Update capture status from ScreenCaptures resource existence
    system_status.capture_active = screen_captures.is_some();
}

/// Zero-allocation jitter measurement system with blazing-fast performance
/// Uses Welford's algorithm for O(1) variance calculation and ring buffers for zero heap allocation
#[inline]
fn jitter_measurement_system(
    time: Res<Time>,
    mut jitter_metrics: ResMut<JitterMetrics>,
    data_channel: Res<DataChannel>,
) {
    // Use high-precision timing with f64 conversion to minimize floating point errors
    let current_time = time.elapsed_secs_f64() as f32 * 1000.0;
    
    // Measure frame timing with zero allocations
    if jitter_metrics.last_frame_time > 0.0 {
        let frame_interval = current_time - jitter_metrics.last_frame_time;
        
        // Add measurement using optimized ring buffer and Welford's algorithm
        jitter_metrics.add_frame_measurement(frame_interval);
        
        // Check for jitter violations using cached statistics
        if jitter_metrics.frame_count > 10 {
            let std_dev = jitter_metrics.frame_std_dev();
            
            // Log jitter violations with zero string allocations
            if std_dev > jitter_metrics.frame_variance_threshold {
                warn!("⚠️ Jitter violation detected! Frame variance: {:.2}ms (threshold: {:.2}ms)", 
                      std_dev, jitter_metrics.frame_variance_threshold);
            }
            
            // Increment counter and log statistics periodically
            jitter_metrics.stats_counter += 1;
            if jitter_metrics.stats_counter >= jitter_metrics.stats_update_interval {
                jitter_metrics.stats_counter = 0;
                
                // Calculate percentile using zero-allocation quickselect
                let p99 = jitter_metrics.frame_percentile(99.0);
                
                debug!("📊 Jitter Stats - Mean: {:.2}ms, StdDev: {:.2}ms, 99th: {:.2}ms", 
                       jitter_metrics.frame_mean, std_dev, p99);
            }
        }
    }
    jitter_metrics.last_frame_time = current_time;
    
    // Measure IMU data intervals with zero allocations using Welford's algorithm
    while let Ok(_data) = data_channel.0.try_recv() {
        if jitter_metrics.last_imu_time > 0.0 {
            let imu_interval = current_time - jitter_metrics.last_imu_time;
            
            // Add measurement using optimized Welford's algorithm
            jitter_metrics.add_imu_measurement(imu_interval);
            
            // Check for IMU jitter violations
            if jitter_metrics.imu_count > 10 {
                let imu_std_dev = jitter_metrics.imu_std_dev();
                
                // Log IMU jitter violations (target: ~1ms intervals, threshold: 0.5ms variance)
                if imu_std_dev > 0.5 {
                    warn!("⚠️ IMU jitter violation detected! IMU variance: {:.2}ms (threshold: 0.5ms)", imu_std_dev);
                }
            }
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

    let mut app = App::new();
    
    app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "XREAL Virtual Desktop".into(),
                    resolution: (450., 350.).into(),
                    resizable: true,
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
        .insert_resource(DisplayModeState::default())
        .insert_resource(RollLockState::default())
        .insert_resource(BrightnessState::default())
        .insert_resource(ui::UiRenderGuard::default())
        .insert_resource(LibusbCheckState::default())
        .insert_resource(LibusbInstallStatus::default())
        .insert_resource(GlassesConnectionState::default())
        .insert_resource(CacheValidityState::default())
        .insert_resource(DependencyCheckState::default())
        .insert_resource(SystemsSpawnedState::default())
        .insert_resource(SystemStatus::default())
        .insert_resource(SettingsPanelState::default())
        .insert_resource(TopMenuState::default())
        .insert_resource(JitterMetrics::<1000>::default());
    
    // Initialize plugin system
    if let Err(e) = plugins::add_plugin_system(&mut app, plugins::PluginSystemConfig::default()) {
        error!("Failed to initialize plugin system: {}", e);
    } else {
        info!("✅ Plugin system initialized successfully");
    }
    
    app.add_systems(Startup, (setup_3d_scene, orchestrate_dependency_startup))
        .add_systems(FixedUpdate, (update_from_data_channel, handle_imu_task, handle_capture_init_task))
        .add_systems(FixedUpdate, render::update_camera_from_orientation)
        .add_systems(FixedUpdate, render::spawn_capture_tasks)
        .add_systems(FixedUpdate, render::handle_capture_tasks)
        .add_systems(FixedUpdate, handle_libusb_check_task)
        .add_systems(FixedUpdate, handle_libusb_install_task)
        .add_systems(FixedUpdate, handle_glasses_check_task)
        .add_systems(FixedUpdate, handle_cache_check_task)
        .add_systems(FixedUpdate, handle_cache_update_task)
        .add_systems(FixedUpdate, orchestrate_dependency_flow)
        .add_systems(FixedUpdate, conditional_system_startup.after(orchestrate_dependency_flow))
        .add_systems(Update, (reset_ui_guard, settings_ui).chain())
        .add_systems(FixedUpdate, handle_input)
        .add_systems(FixedUpdate, log_fps)
        .add_systems(FixedUpdate, render::update_screen_positions)
        .add_systems(FixedUpdate, jitter_measurement_system)
        .add_systems(FixedUpdate, display_mode_system)
        .add_systems(FixedUpdate, roll_lock_system)
        .add_systems(FixedUpdate, brightness_control_system)
        .add_systems(FixedUpdate, system_status_update_system)
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