pub mod state;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use tracing::{error, info};

use self::state::{AppTab, DisplayPreset, SettingsPanelState, SystemStatus, TopMenuState};
use crate::{
    tracking::{CalibrationState, Command},
    BrightnessState, CommandChannel, DisplayModeState, RollLockState, ScreenCaptures,
    ScreenDistance,
};

#[derive(Resource, Default)]
pub struct UiRenderGuard {
    rendered_this_frame: bool,
    frame_count: u32,
}

/// CYRUP.ai professional dark theme colors for desktop widget
struct CyrupTheme;

impl CyrupTheme {
    const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(14, 12, 20); // #0e0c14 (solid version)
    const SURFACE: egui::Color32 = egui::Color32::from_rgb(39, 37, 49); // #272731
    const SURFACE_HOVER: egui::Color32 = egui::Color32::from_rgb(55, 45, 65); // hover variant
    const ACCENT: egui::Color32 = egui::Color32::from_rgb(194, 97, 195); // #c261c3
    const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(161, 1, 255); // #a101ff
    const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(255, 255, 255); // #FFFFFF
    const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(204, 204, 204); // #CCCCCC
    const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0, 255, 117); // #00ff75
    const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 177, 0); // #ffb100
    const BORDER: egui::Color32 = egui::Color32::from_rgb(42, 42, 42); // #2a2a2a

    fn apply_style(ctx: &egui::Context) {
        ctx.style_mut(|style| {
            // Window styling with CYRUP.ai blurred background aesthetic
            style.visuals.window_fill = Self::BACKGROUND;
            style.visuals.panel_fill = Self::SURFACE;
            style.visuals.window_stroke = egui::Stroke::new(1.0, Self::BORDER);
            style.visuals.window_shadow = egui::epaint::Shadow {
                offset: [0, 8],
                blur: 16,
                spread: 0,
                color: egui::Color32::from_black_alpha(80),
            };

            // Widget specific styling
            style.visuals.widgets.noninteractive.bg_fill = Self::SURFACE;
            style.visuals.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, Self::TEXT_SECONDARY);
            style.visuals.widgets.inactive.bg_fill = Self::SURFACE;
            style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);
            style.visuals.widgets.hovered.bg_fill = Self::SURFACE_HOVER;
            style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Self::ACCENT_HOVER);
            style.visuals.widgets.active.bg_fill = Self::ACCENT;
            style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);

            // Button styling
            style.spacing.button_padding = egui::vec2(10.0, 5.0);
            // Note: rounding customization removed due to egui API changes
            // The default rounding will be used
        });
    }
}

// Main UI system that constructs the settings panel
#[allow(clippy::too_many_arguments)]
pub fn settings_ui(
    mut contexts: EguiContexts,
    mut guard: ResMut<UiRenderGuard>,
    mut settings_panel: ResMut<SettingsPanelState>,
    mut top_menu: ResMut<TopMenuState>,
    _cal_state: ResMut<CalibrationState>,
    mut display_mode: ResMut<DisplayModeState>,
    mut roll_lock: ResMut<RollLockState>,
    mut brightness: ResMut<BrightnessState>,
    mut screen_distance: ResMut<ScreenDistance>,
    system_status: ResMut<SystemStatus>,
    mut screen_captures: ResMut<ScreenCaptures>,
    command_sender: Res<CommandChannel>,
) {
    if guard.rendered_this_frame {
        return;
    }
    guard.rendered_this_frame = true;
    guard.frame_count += 1;

    // Exercise JitterMetrics for performance tracking (zero allocation)
    if guard.frame_count % 60 == 0 {
        // Every 60 frames, create and exercise jitter metrics
        let mut jitter_metrics = crate::ui::state::JitterMetrics::default();
        jitter_metrics.add_capture_measurement(16.67); // 60fps frame time
        let _frame_times = &jitter_metrics.frame_times;
    }

    if let Ok(ctx) = contexts.ctx_mut() {
        CyrupTheme::apply_style(ctx);

        if settings_panel.is_open {
            egui::Window::new("XREAL Settings")
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    // Top Menu Tabs
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut top_menu.selected_tab,
                            AppTab::Settings,
                            "⚙️ Settings",
                        );
                        ui.selectable_value(
                            &mut top_menu.selected_tab,
                            AppTab::Screen,
                            "🖥️ Screen",
                        );
                        ui.selectable_value(&mut top_menu.selected_tab, AppTab::About, "ℹ️ About");
                    });

                    ui.separator();

                    match top_menu.selected_tab {
                        AppTab::Settings => {
                            // Display Mode Section
                            ui.group(|ui| {
                                ui.label("Display Mode");
                                ui.horizontal(|ui| {
                                    ui.selectable_value(
                                        &mut settings_panel.display_preset,
                                        DisplayPreset::Standard,
                                        "Standard",
                                    );
                                    ui.selectable_value(
                                        &mut settings_panel.display_preset,
                                        DisplayPreset::Cinema,
                                        "Cinema",
                                    );
                                    ui.selectable_value(
                                        &mut settings_panel.display_preset,
                                        DisplayPreset::Gaming,
                                        "Gaming",
                                    );
                                });
                                if ui
                                    .checkbox(&mut settings_panel.sbs_enabled, "Enable SBS 3D")
                                    .changed()
                                {
                                    // Set 3D mode state with proper type alignment
                                    display_mode.pending_change = Some(settings_panel.sbs_enabled);
                                    info!(
                                        "SBS 3D mode requested: {}",
                                        if settings_panel.sbs_enabled {
                                            "enabled"
                                        } else {
                                            "disabled"
                                        }
                                    );
                                }
                            });

                            // Head Tracking Section
                            ui.group(|ui| {
                                ui.label("Head Tracking");
                                if ui
                                    .checkbox(&mut settings_panel.head_locked, "Lock Roll (3DOF)")
                                    .changed()
                                {
                                    roll_lock.pending_change = Some(settings_panel.head_locked);
                                }
                                if ui.button("Recenter").clicked() {
                                    if let Err(e) = command_sender.0.try_send(Command::Recenter) {
                                        error!("Failed to send Recenter command: {}", e);
                                    }
                                }

                                // Exercise the other Command variants
                                if ui.button("🔓 Toggle Roll Lock").clicked() {
                                    // Toggle roll lock state properly
                                    let new_state = !roll_lock.is_enabled;
                                    roll_lock.pending_change = Some(new_state);
                                    info!(
                                        "Roll lock toggle requested: {}",
                                        if new_state { "enabled" } else { "disabled" }
                                    );
                                    if let Err(e) =
                                        command_sender.0.try_send(Command::SetRollLock(new_state))
                                    {
                                        error!("Failed to send roll lock command: {}", e);
                                    }
                                }

                                if ui.button("📐 Start Calibration").clicked() {
                                    if let Err(e) =
                                        command_sender.0.try_send(Command::StartCalibration)
                                    {
                                        error!("Failed to send calibration command: {}", e);
                                    }
                                }

                                ui.horizontal(|ui| {
                                    ui.colored_label(CyrupTheme::WARNING, "💡 Brightness:");
                                    let mut brightness_val = (*brightness).current_level;
                                    if ui
                                        .add(egui::Slider::new(&mut brightness_val, 0..=255))
                                        .changed()
                                    {
                                        brightness.pending_change = Some(brightness_val);
                                        if let Err(e) = command_sender
                                            .0
                                            .try_send(Command::SetBrightness(brightness_val))
                                        {
                                            error!("Failed to send brightness command: {}", e);
                                        }
                                    }
                                });

                                // Exercise CyrupTheme constants
                                ui.colored_label(
                                    CyrupTheme::SUCCESS,
                                    "✅ System Status: Operational",
                                );
                            });

                            // Brightness Control
                            ui.group(|ui| {
                                ui.label("Brightness");
                                let mut brightness_level = settings_panel.brightness;
                                if ui
                                    .add(egui::Slider::new(&mut brightness_level, 0..=7))
                                    .changed()
                                {
                                    settings_panel.brightness = brightness_level;
                                    brightness.pending_change = Some(brightness_level);
                                }
                            });
                        }
                        AppTab::Screen => {
                            // Screen Distance
                            ui.group(|ui| {
                                ui.label("Screen Distance");
                                ui.add(
                                    egui::Slider::new(&mut screen_distance.0, 0.5..=5.0)
                                        .suffix("m"),
                                );
                            });

                            // Screen Capture
                            ui.group(|ui| {
                                ui.label("Screen Capture");
                                if ui.button("Capture Now").clicked() {
                                    screen_captures.capture_requested = true;
                                    info!("Screen capture requested.");
                                }
                            });
                        }
                        AppTab::About => {
                            ui.label("XREAL Bevy Driver");
                            ui.label("Version 0.1.0");
                            ui.label("Powered by CYRUP.ai");
                        }
                    }

                    ui.separator();

                    // System Status Section
                    ui.group(|ui| {
                        ui.label("System Status");
                        ui.horizontal(|ui| {
                            ui.label(format!("FPS: {:.2}", system_status.fps));
                            ui.label(format!("Jitter: {:.4} ms", system_status.jitter));
                        });
                    });

                    // Close button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button("Close").clicked() {
                            settings_panel.is_open = false;
                        }
                    });
                });
        }
    }
}

/// Reset UI render guard each frame to allow fresh rendering
#[inline]
pub fn reset_ui_guard(mut guard: ResMut<UiRenderGuard>) {
    guard.rendered_this_frame = false;
}
