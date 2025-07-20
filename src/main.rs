use anyhow::Result;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use bevy_egui::EguiPlugin;
use crossbeam_channel::{bounded, Receiver, Sender};


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
use render::setup_3d_scene;

use tracking::{CalibrationState, Command, Data, Orientation};
use ui::{reset_ui_guard, settings_ui, state::*};
use xreal_stereo::XRealStereoRenderingPlugin;

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

#[derive(Resource, Default)]
pub struct BrightnessState {
    pub value: u8,
    pub pending_change: Option<u8>,
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

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "XREAL Bevy".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((EguiPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
        .add_plugins(XRealStereoRenderingPlugin)
        .insert_resource(DataChannel(data_rx))
        .insert_resource(CommandChannel(command_tx))
        .insert_resource(Orientation::default())
        .insert_resource(CalibrationState::default())
        .insert_resource(ScreenDistance(2.0))
        .insert_resource(DisplayModeState::default())
        .insert_resource(RollLockState::default())
        .insert_resource(BrightnessState::default())
        .insert_resource(ScreenCaptures::new_async().await.unwrap())
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
            ),
        )
        .run();

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
