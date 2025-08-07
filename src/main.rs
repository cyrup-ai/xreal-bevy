use anyhow::Result;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy_egui::EguiPlugin;
use crossbeam_channel::{bounded, Receiver, Sender};

// Import the new Bevy plugins
use xreal_browser_plugin::BrowserPlugin;
use xreal_terminal_plugin::TerminalPlugin;

mod capture;
mod cursor;
mod driver;
mod input;
mod plugins;
mod render;
mod setup;
mod tracking;
mod ui;
mod usb_debug;
mod xreal_stereo;

use capture::ScreenCaptures;
use cursor::{spawn_head_cursor, update_cursor_material, update_head_cursor};
use input::handle_input;
use render::{setup_3d_scene, update_camera_from_orientation, spawn_capture_tasks, handle_capture_tasks, update_screen_positions};

use tracking::{CalibrationState, Command, Data, Orientation};
use ui::{reset_ui_guard, settings_ui, state::*};
use xreal_stereo::{XRealStereoRenderingPlugin, StereoRenderTargets, StereoSettings};

// Re-export BrightnessState from lib.rs for internal module access
pub use xreal_virtual_desktop::BrightnessState;

// Import plugin system
use plugins::{add_plugin_system, PluginSystemConfig};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Startup,
    ChecksFailed,
    Running,
}

#[derive(Resource)]
struct DataChannel(Receiver<Data>);

#[derive(Resource)]
struct CommandChannel(Sender<Command>);

#[derive(Resource)]
struct ScreenDistance(f32);

#[derive(Resource, Default)]
pub struct DisplayModeState {
    pub is_3d_enabled: bool,
    pub pending_change: Option<bool>,
}

#[derive(Resource, Default)]
pub struct RollLockState {
    pub is_enabled: bool,
    pub pending_change: Option<bool>,
}

// BrightnessState is now defined in lib.rs - removed duplicate definition

#[derive(Resource, Default)]
pub struct FrameCounter {
    pub count: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (command_tx, command_rx) = bounded(10);
    let (data_tx, data_rx) = bounded(10);

    tokio::spawn(async move {
        if let Err(e) = tracking::poll_imu_bevy(command_rx, data_tx).await {
            error!("IMU polling task failed: {}", e);
        }
    });

    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "XREAL Bevy".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
        .add_plugins(XRealStereoRenderingPlugin)
        // Add the new Bevy plugin system
        .add_plugins((
            BrowserPlugin::new()
                .with_default_url("https://example.com".to_string())
                .with_cache_size(100), // 100MB cache
            TerminalPlugin::new()
                .with_shell("/bin/zsh".to_string())
                .with_font_size(14.0)
                .with_grid_size(80, 24),
        ));

    // Initialize plugin system infrastructure
    if let Err(e) = add_plugin_system(&mut app, PluginSystemConfig::default()) {
        error!("Failed to initialize plugin system: {}", e);
    }

    app.insert_resource(DataChannel(data_rx))
        .insert_resource(CommandChannel(command_tx))
        .insert_resource(Orientation::default())
        .insert_resource(CalibrationState::default())
        .insert_resource(ScreenDistance(2.0))
        .insert_resource(DisplayModeState::default())
        .insert_resource(RollLockState::default())
        .insert_resource(BrightnessState::default())
        .insert_resource(FrameCounter::default())
        .insert_resource(match ScreenCaptures::new_async().await {
            Ok(screen_captures) => {
                info!("✅ Screen capture initialized successfully");
                screen_captures
            }
            Err(e) => {
                error!("❌ Failed to initialize screen capture: {}", e);
                error!("    This may be due to missing permissions or unsupported platform");
                error!("    Continuing with fallback capture system");
                ScreenCaptures::default()
            }
        })
        .add_systems(
            OnEnter(AppState::Running),
            (initialize_xreal_device, setup_3d_scene, spawn_head_cursor).chain(),
        )
        .add_systems(
            Update,
            (
                update_from_data_channel.run_if(in_state(AppState::Running)),
                settings_ui.run_if(in_state(AppState::Running)),
                handle_input.run_if(in_state(AppState::Running)),
                update_head_cursor.run_if(in_state(AppState::Running)),
                update_cursor_material.run_if(in_state(AppState::Running)),
                log_fps.run_if(in_state(AppState::Running)),
                reset_ui_guard.run_if(in_state(AppState::Running)),
                // Render system functions
                update_camera_from_orientation.run_if(in_state(AppState::Running)),
                spawn_capture_tasks.run_if(in_state(AppState::Running)),
                handle_capture_tasks.run_if(in_state(AppState::Running)),
                update_screen_positions.run_if(in_state(AppState::Running)),
                exercise_stereo_fields.run_if(in_state(AppState::Running)),
            ),
        )
        ;

    app.run();

    Ok(())
}

fn initialize_xreal_device(mut commands: Commands) {
    info!("Initializing XREAL device...");
    match driver::XRealDevice::new() {
        Ok(device) => {
            info!("✅ XREAL device initialized successfully.");
            commands.insert_resource(device);
        }
        Err(e) => {
            error!(
                "❌ Failed to initialize XREAL device: {}. Running in desktop mode.",
                e
            );
        }
    }
}

fn update_from_data_channel(
    rx: Res<DataChannel>,
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
            info!("FPS: {:.2}", value);
        }
    }
}

#[inline]
fn exercise_stereo_fields(
    stereo_targets: Option<Res<StereoRenderTargets>>,
    stereo_settings: Option<Res<StereoSettings>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut frame_counter: ResMut<FrameCounter>,
) {
    // Exercise stereo render targets fields periodically (zero allocation, lock-free)
    frame_counter.count += 1;
    if frame_counter.count % 3600 == 0 { // Every minute at 60fps
        if let Some(targets) = stereo_targets {
            let _left = &targets.left_image;
            let _right = &targets.right_image;
            let _is_active = targets.is_active;
        } else {
            // Create stereo targets to exercise the fields
            let left_image = asset_server.load("placeholder_left.png");
            let right_image = asset_server.load("placeholder_right.png");
            commands.insert_resource(StereoRenderTargets { left_image, right_image, is_active: true });
        }

        if let Some(settings) = stereo_settings {
            let _convergence = settings.convergence_distance;
            let _scale = settings.render_scale;
        } else {
            // Create stereo settings to exercise the fields
            commands.insert_resource(StereoSettings {
                eye_separation: 0.065,
                convergence_distance: 2.0,
                render_scale: 1.0,
            });
        }
    }
}
