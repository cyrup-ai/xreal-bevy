use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::{CommandChannel, ScreenDistance};
use crate::tracking::{CalibrationState, Command};

#[derive(Resource, Default)]
pub struct UiRenderGuard {
    rendered_this_frame: bool,
    frame_count: u32,
}

/// Professional dark theme colors for XREAL AR/VR context
struct XrealTheme;

impl XrealTheme {
    const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(15, 15, 20);
    const SURFACE: egui::Color32 = egui::Color32::from_rgb(25, 25, 35);
    const SURFACE_HOVER: egui::Color32 = egui::Color32::from_rgb(35, 35, 50);
    const ACCENT: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);
    const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(120, 200, 255);
    const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(240, 240, 250);
    const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(180, 180, 200);
    const SUCCESS: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
    const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 180, 80);
    const BORDER: egui::Color32 = egui::Color32::from_rgb(50, 50, 70);
    
    fn apply_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Window styling
        style.visuals.window_fill = Self::BACKGROUND;
        style.visuals.panel_fill = Self::SURFACE;
        style.visuals.window_stroke = egui::Stroke::new(1.0, Self::BORDER);
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 8],
            blur: 16,
            spread: 0,
            color: egui::Color32::from_black_alpha(80),
        };
        
        // Widget styling
        style.visuals.widgets.noninteractive.bg_fill = Self::SURFACE;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);
        
        style.visuals.widgets.inactive.bg_fill = Self::SURFACE;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_SECONDARY);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Self::BORDER);
        
        style.visuals.widgets.hovered.bg_fill = Self::SURFACE_HOVER;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Self::ACCENT);
        
        style.visuals.widgets.active.bg_fill = Self::ACCENT;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, Self::BACKGROUND);
        style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, Self::ACCENT_HOVER);
        
        // Button styling
        style.visuals.button_frame = true;
        
        // Spacing
        style.spacing.button_padding = egui::Vec2::new(12.0, 8.0);
        style.spacing.item_spacing = egui::Vec2::new(12.0, 8.0);
        style.spacing.indent = 16.0;
        
        ctx.set_style(style);
    }
}

#[inline]
pub fn settings_ui(
    mut contexts: EguiContexts, 
    sender: Res<CommandChannel>, 
    cal_state: Res<CalibrationState>, 
    mut distance: ResMut<ScreenDistance>,
    mut guard: ResMut<UiRenderGuard>,
) {
    // Prevent duplicate UI renders within the same frame
    if guard.rendered_this_frame {
        return;
    }
    guard.rendered_this_frame = true;
    guard.frame_count += 1;

    // Skip first few frames to allow egui to fully initialize
    if guard.frame_count < 10 {
        return;
    }

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => {
            // Egui context not ready yet, skip this frame
            return;
        }
    };
    
    // Apply professional XREAL theme
    XrealTheme::apply_style(ctx);
    
    // Compact desktop widget control panel
    egui::Window::new("XREAL Control Center")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
        .fixed_size([380.0, 280.0])
        .show(ctx, |ui| {
            // Compact header with branding
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("🥽 XREAL Virtual Desktop")
                        .size(16.0)
                        .color(XrealTheme::ACCENT)
                        .strong()
                );
            });
            
            ui.add_space(10.0);
            
            // Main control sections in vertical layout for compact widget
            ui.vertical(|ui| {
                // Display Configuration
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("📺 Display")
                                .size(14.0)
                                .color(XrealTheme::ACCENT)
                                .strong()
                        );
                        ui.add_space(20.0);
                        
                        // Distance control
                        ui.label("Depth:");
                        ui.add(
                            egui::Slider::new(&mut distance.0, -10.0..=-1.0)
                                .suffix("m")
                                .custom_formatter(|n, _| format!("{:.1}m", n))
                        );
                    });
                });
                
                ui.add_space(8.0);
                
                // Mode controls in horizontal layout
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let mut is_3d = true;
                        if ui.checkbox(&mut is_3d, "🌐 3D").changed() {
                            if let Err(_) = sender.0.try_send(Command::SetDisplayMode(is_3d)) {
                                warn!("Failed to send display mode command");
                            }
                        }
                        
                        let mut locked = false;
                        if ui.checkbox(&mut locked, "🔒 Lock").changed() {
                            if let Err(_) = sender.0.try_send(Command::SetRollLock(locked)) {
                                warn!("Failed to send roll lock command");
                            }
                        }
                        
                        ui.add_space(10.0);
                        
                        // Brightness control
                        ui.label("☀️");
                        let mut brightness = 4u8;
                        ui.add(egui::Slider::new(&mut brightness, 0..=7)
                            .show_value(false));
                        if ui.small_button("Set").clicked() {
                            if let Err(_) = sender.0.try_send(Command::SetBrightness(brightness)) {
                                warn!("Failed to send brightness command");
                            }
                        }
                    });
                });
                
                ui.add_space(8.0);
                
                // Tracking section
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("🎯 Tracking")
                                .size(14.0)
                                .color(XrealTheme::ACCENT)
                                .strong()
                        );
                        ui.add_space(20.0);
                        
                        match cal_state.as_ref() {
                            CalibrationState::Calibrating { start_time, .. } => {
                                let elapsed = start_time.elapsed().as_secs();
                                let progress = (elapsed as f32 / 5.0).min(1.0);
                                
                                ui.label(
                                    egui::RichText::new("🔄 Calibrating...")
                                        .color(XrealTheme::WARNING)
                                );
                                ui.add(egui::ProgressBar::new(progress).text(format!("{}s", elapsed)));
                            }
                            CalibrationState::Calibrated { .. } => {
                                ui.label(
                                    egui::RichText::new("✅ Active")
                                        .color(XrealTheme::SUCCESS)
                                );
                                if ui.small_button("🔄 Recal").clicked() {
                                    if let Err(_) = sender.0.try_send(Command::StartCalibration) {
                                        warn!("Failed to send calibration command");
                                    }
                                }
                            }
                            CalibrationState::Idle => {
                                ui.label(
                                    egui::RichText::new("⚠️ Idle")
                                        .color(XrealTheme::WARNING)
                                );
                                if ui.small_button("🎯 Start").clicked() {
                                    if let Err(_) = sender.0.try_send(Command::StartCalibration) {
                                        warn!("Failed to send calibration command");
                                    }
                                }
                            }
                        }
                    });
                });
                
                ui.add_space(8.0);
                
                // System status
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚡ System")
                                .size(14.0)
                                .color(XrealTheme::ACCENT)
                                .strong()
                        );
                        ui.add_space(20.0);
                        
                        ui.label("🔗");
                        ui.label(
                            egui::RichText::new("Connected")
                                .size(11.0)
                                .color(XrealTheme::SUCCESS)
                        );
                        ui.add_space(10.0);
                        
                        ui.label("📊");
                        ui.label(
                            egui::RichText::new("60 FPS")
                                .size(11.0)
                                .color(XrealTheme::SUCCESS)
                        );
                        ui.add_space(10.0);
                        
                        if ui.small_button("⚙️").clicked() {
                            // Future: Open detailed settings
                        }
                    });
                });
            });
        });
    
}

/// Reset UI render guard each frame to allow fresh rendering
#[inline]
pub fn reset_ui_guard(mut guard: ResMut<UiRenderGuard>) {
    guard.rendered_this_frame = false;
}