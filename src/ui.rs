use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::{CommandChannel, ScreenDistance, DisplayModeState, RollLockState, BrightnessState, SystemStatus, SettingsPanelState, DisplayPreset, TopMenuState, AppTab};
use crate::tracking::{CalibrationState, Command};

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
        let mut style = (*ctx.style()).clone();
        
        // Window styling with CYRUP.ai blurred background aesthetic
        style.visuals.window_fill = Self::BACKGROUND;
        style.visuals.panel_fill = Self::SURFACE;
        style.visuals.window_stroke = egui::Stroke::new(1.0, Self::BORDER);
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 8],
            blur: 16,
            spread: 0,
            color: egui::Color32::from_rgba_unmultiplied(194, 97, 195, 17), // Purple shadow
        };
        
        // Widget styling with CYRUP.ai accent colors
        style.visuals.widgets.noninteractive.bg_fill = Self::SURFACE;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);
        
        style.visuals.widgets.inactive.bg_fill = Self::SURFACE;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_SECONDARY);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Self::BORDER);
        
        style.visuals.widgets.hovered.bg_fill = Self::SURFACE_HOVER;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);
        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Self::ACCENT);
        
        style.visuals.widgets.active.bg_fill = Self::ACCENT;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(14, 12, 20));
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
    mut display_mode: ResMut<DisplayModeState>,
    mut roll_lock: ResMut<RollLockState>,
    mut brightness: ResMut<BrightnessState>,
    system_status: Res<SystemStatus>,
    mut settings_panel: ResMut<SettingsPanelState>,
    mut top_menu: ResMut<TopMenuState>,
    time: Res<Time>,
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
    
    // Apply CYRUP.ai theme
    CyrupTheme::apply_style(ctx);
    
    // Top Menu Bar (invisible unless hovering, with debouncing)
    let screen_rect = ctx.screen_rect();
    let top_bar_height = 30.0;
    let hover_zone = egui::Rect::from_min_size(
        screen_rect.min,
        egui::Vec2::new(screen_rect.width(), top_bar_height)
    );
    
    // Check if mouse is in hover zone
    let mouse_pos = ctx.pointer_latest_pos();
    let is_mouse_in_zone = if let Some(pos) = mouse_pos {
        hover_zone.contains(pos)
    } else {
        false
    };
    
    // Update hover state with debouncing
    if is_mouse_in_zone {
        top_menu.hover_timer = (top_menu.hover_timer + time.delta_secs()).min(0.3);
        if top_menu.hover_timer >= 0.2 {
            top_menu.is_hovering = true;
        }
    } else {
        top_menu.hover_timer = (top_menu.hover_timer - time.delta_secs() * 2.0).max(0.0);
        if top_menu.hover_timer <= 0.0 {
            top_menu.is_hovering = false;
            top_menu.is_menu_open = false;
        }
    }
    
    // Show top menu bar only when hovering
    if top_menu.is_hovering {
        egui::TopBottomPanel::top("xreal_top_menu")
            .exact_height(top_bar_height)
            .frame(egui::Frame::NONE
                .fill(CyrupTheme::BACKGROUND)
                .stroke(egui::Stroke::new(1.0, CyrupTheme::BORDER)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    
                    // XREAL logo/title
                    ui.label(
                        egui::RichText::new("🥽 XREAL Desktop")
                            .size(14.0)
                            .color(CyrupTheme::ACCENT)
                            .strong()
                    );
                    
                    ui.add_space(20.0);
                    
                    // Application tabs
                    ui.horizontal(|ui| {
                        let tab_size = [80.0, 24.0];
                        
                        if ui.add_sized(tab_size, egui::SelectableLabel::new(
                            top_menu.selected_tab == AppTab::Browser, "🌐 Browser"
                        )).clicked() {
                            top_menu.selected_tab = AppTab::Browser;
                            top_menu.is_menu_open = !top_menu.is_menu_open;
                        }
                        
                        if ui.add_sized(tab_size, egui::SelectableLabel::new(
                            top_menu.selected_tab == AppTab::Terminal, "⌨️ Terminal"
                        )).clicked() {
                            top_menu.selected_tab = AppTab::Terminal;
                            top_menu.is_menu_open = !top_menu.is_menu_open;
                        }
                        
                        if ui.add_sized(tab_size, egui::SelectableLabel::new(
                            top_menu.selected_tab == AppTab::VSCode, "💻 VSCode"
                        )).clicked() {
                            top_menu.selected_tab = AppTab::VSCode;
                            top_menu.is_menu_open = !top_menu.is_menu_open;
                        }
                        
                        if ui.add_sized(tab_size, egui::SelectableLabel::new(
                            top_menu.selected_tab == AppTab::Files, "📁 Files"
                        )).clicked() {
                            top_menu.selected_tab = AppTab::Files;
                            top_menu.is_menu_open = !top_menu.is_menu_open;
                        }
                        
                        if ui.add_sized(tab_size, egui::SelectableLabel::new(
                            top_menu.selected_tab == AppTab::Media, "🎵 Media"
                        )).clicked() {
                            top_menu.selected_tab = AppTab::Media;
                            top_menu.is_menu_open = !top_menu.is_menu_open;
                        }
                        
                        if ui.add_sized(tab_size, egui::SelectableLabel::new(
                            top_menu.selected_tab == AppTab::Games, "🎮 Games"
                        )).clicked() {
                            top_menu.selected_tab = AppTab::Games;
                            top_menu.is_menu_open = !top_menu.is_menu_open;
                        }
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        
                        // System info in top bar
                        ui.label(
                            egui::RichText::new(&format!("{}fps", 
                                system_status.current_fps.map(|f| f as u32).unwrap_or(0)))
                                .size(10.0)
                                .color(CyrupTheme::TEXT_SECONDARY)
                        );
                    });
                });
            });
        
        // Show application launcher dropdown when menu is open
        if top_menu.is_menu_open {
            egui::Window::new("App Launcher")
                .title_bar(false)
                .resizable(false)
                .anchor(egui::Align2::LEFT_TOP, [10.0, top_bar_height + 5.0])
                .auto_sized()
                .frame(egui::Frame::window(&ctx.style())
                    .fill(CyrupTheme::SURFACE)
                    .stroke(egui::Stroke::new(1.0, CyrupTheme::ACCENT)))
                .show(ctx, |ui| {
                    ui.set_min_width(300.0);
                    
                    match top_menu.selected_tab {
                        AppTab::Browser => {
                            ui.label(egui::RichText::new("🌐 Web Browser").size(14.0).color(CyrupTheme::ACCENT).strong());
                            ui.separator();
                            if ui.button("🚀 Launch Chrome").clicked() {
                                info!("Launch Chrome requested");
                            }
                            if ui.button("🦊 Launch Firefox").clicked() {
                                info!("Launch Firefox requested");
                            }
                            if ui.button("🧭 Launch Safari").clicked() {
                                info!("Launch Safari requested");
                            }
                        }
                        AppTab::Terminal => {
                            ui.label(egui::RichText::new("⌨️ Terminal").size(14.0).color(CyrupTheme::ACCENT).strong());
                            ui.separator();
                            if ui.button("💻 Launch Terminal").clicked() {
                                info!("Launch Terminal requested");
                            }
                            if ui.button("⚡ Launch iTerm2").clicked() {
                                info!("Launch iTerm2 requested");
                            }
                            if ui.button("🔥 Launch Alacritty").clicked() {
                                info!("Launch Alacritty requested");
                            }
                        }
                        AppTab::VSCode => {
                            ui.label(egui::RichText::new("💻 Code Editor").size(14.0).color(CyrupTheme::ACCENT).strong());
                            ui.separator();
                            if ui.button("🆚 Launch VSCode").clicked() {
                                info!("Launch VSCode requested");
                            }
                            if ui.button("⚛️ Launch Cursor").clicked() {
                                info!("Launch Cursor requested");
                            }
                            if ui.button("🦀 Launch RustRover").clicked() {
                                info!("Launch RustRover requested");
                            }
                        }
                        AppTab::Files => {
                            ui.label(egui::RichText::new("📁 File Manager").size(14.0).color(CyrupTheme::ACCENT).strong());
                            ui.separator();
                            if ui.button("🗂️ Launch Finder").clicked() {
                                info!("Launch Finder requested");
                            }
                            if ui.button("⚡ Launch Path Finder").clicked() {
                                info!("Launch Path Finder requested");
                            }
                            if ui.button("📦 Launch Commander One").clicked() {
                                info!("Launch Commander One requested");
                            }
                        }
                        AppTab::Media => {
                            ui.label(egui::RichText::new("🎵 Media Player").size(14.0).color(CyrupTheme::ACCENT).strong());
                            ui.separator();
                            if ui.button("🎵 Launch Spotify").clicked() {
                                info!("Launch Spotify requested");
                            }
                            if ui.button("🎬 Launch VLC").clicked() {
                                info!("Launch VLC requested");
                            }
                            if ui.button("🎥 Launch IINA").clicked() {
                                info!("Launch IINA requested");
                            }
                        }
                        AppTab::Games => {
                            ui.label(egui::RichText::new("🎮 Gaming").size(14.0).color(CyrupTheme::ACCENT).strong());
                            ui.separator();
                            if ui.button("💨 Launch Steam").clicked() {
                                info!("Launch Steam requested");
                            }
                            if ui.button("🎯 Launch Epic Games").clicked() {
                                info!("Launch Epic Games requested");
                            }
                            if ui.button("🕹️ Launch Emulators").clicked() {
                                info!("Launch Emulators requested");
                            }
                        }
                    }
                });
        }
    }
    
    // Compact desktop widget control panel with dynamic sizing
    egui::Window::new("XREAL Control Center")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
        .auto_sized()
        .show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.set_max_width(450.0);
            // Compact header with branding
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("🥽 XREAL Virtual Desktop")
                        .size(16.0)
                        .color(CyrupTheme::ACCENT)
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
                                .color(CyrupTheme::ACCENT)
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
                        let mut is_3d = display_mode.is_3d_enabled;
                        if ui.checkbox(&mut is_3d, "🌐 3D").changed() {
                            // Set pending change for the display_mode_system to handle
                            display_mode.pending_change = Some(is_3d);
                        }
                        
                        let mut locked = roll_lock.is_enabled;
                        if ui.checkbox(&mut locked, "🔒 Lock").changed() {
                            // Set pending change for the roll_lock_system to handle
                            roll_lock.pending_change = Some(locked);
                        }
                        
                        ui.add_space(10.0);
                        
                        // Brightness control with actual state
                        ui.label("☀️");
                        let mut brightness_level = brightness.current_level;
                        if ui.add(egui::Slider::new(&mut brightness_level, 0..=7)
                            .show_value(true)
                            .custom_formatter(|n, _| format!("{}", n as u8))).changed() {
                            brightness.pending_change = Some(brightness_level);
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
                                .color(CyrupTheme::ACCENT)
                                .strong()
                        );
                        ui.add_space(20.0);
                        
                        match cal_state.as_ref() {
                            CalibrationState::Calibrating { start_time, .. } => {
                                let elapsed = start_time.elapsed().as_secs();
                                let progress = (elapsed as f32 / 5.0).min(1.0);
                                
                                ui.label(
                                    egui::RichText::new("🔄 Calibrating...")
                                        .color(CyrupTheme::WARNING)
                                );
                                ui.add(egui::ProgressBar::new(progress).text(format!("{}s", elapsed)));
                            }
                            CalibrationState::Calibrated { .. } => {
                                ui.label(
                                    egui::RichText::new("✅ Active")
                                        .color(CyrupTheme::SUCCESS)
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
                                        .color(CyrupTheme::WARNING)
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
                                .color(CyrupTheme::ACCENT)
                                .strong()
                        );
                        ui.add_space(20.0);
                        
                        ui.label("🔗");
                        let (connection_text, connection_color) = if system_status.connection_status {
                            ("Connected", CyrupTheme::SUCCESS)
                        } else {
                            ("Disconnected", CyrupTheme::WARNING)
                        };
                        ui.label(
                            egui::RichText::new(connection_text)
                                .size(11.0)
                                .color(connection_color)
                        );
                        ui.add_space(10.0);
                        
                        ui.label("📊");
                        let fps_text = match system_status.current_fps {
                            Some(fps) => format!("{:.0} FPS", fps),
                            None => "-- FPS".to_string(),
                        };
                        ui.label(
                            egui::RichText::new(&fps_text)
                                .size(11.0)
                                .color(CyrupTheme::SUCCESS)
                        );
                        ui.add_space(10.0);
                        
                        ui.label("📹");
                        let (capture_text, capture_color) = if system_status.capture_active {
                            ("Capturing", CyrupTheme::SUCCESS)
                        } else {
                            ("Inactive", CyrupTheme::WARNING)
                        };
                        ui.label(
                            egui::RichText::new(capture_text)
                                .size(11.0)
                                .color(capture_color)
                        );
                        ui.add_space(10.0);
                        
                        if ui.small_button("⚙️").clicked() {
                            settings_panel.is_open = !settings_panel.is_open;
                        }
                    });
                });
            });
        });
    
    // Settings panel - only show when open
    if settings_panel.is_open {
        egui::Window::new("Advanced Settings")
            .resizable(true)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .auto_sized()
            .constrain(true)
            .show(ctx, |ui| {
                ui.set_min_width(480.0);
                ui.set_max_width(600.0);
                
                ui.heading("🔧 XREAL Configuration");
                ui.add_space(10.0);
                
                // Display Presets Section
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new("📺 Display Presets")
                            .size(14.0)
                            .color(CyrupTheme::ACCENT)
                            .strong()
                    );
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut settings_panel.selected_preset, DisplayPreset::Gaming, "🎮 Gaming");
                        ui.selectable_value(&mut settings_panel.selected_preset, DisplayPreset::Productivity, "💼 Productivity");
                        ui.selectable_value(&mut settings_panel.selected_preset, DisplayPreset::Cinema, "🎬 Cinema");
                    });
                    
                    match settings_panel.selected_preset {
                        DisplayPreset::Gaming => {
                            ui.label("Optimized for low latency, high refresh rate");
                        }
                        DisplayPreset::Productivity => {
                            ui.label("Balanced settings for work and general use");
                        }
                        DisplayPreset::Cinema => {
                            ui.label("Enhanced visual quality for media consumption");
                        }
                    }
                });
                
                ui.add_space(10.0);
                
                // Advanced Calibration Section
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new("🎯 Advanced Calibration")
                            .size(14.0)
                            .color(CyrupTheme::ACCENT)
                            .strong()
                    );
                    ui.add_space(5.0);
                    
                    ui.checkbox(&mut settings_panel.advanced_calibration, "Enable advanced calibration mode");
                    
                    if settings_panel.advanced_calibration {
                        ui.horizontal(|ui| {
                            if ui.button("🔄 Quick Calibration").clicked() {
                                if let Err(_) = sender.0.try_send(Command::StartCalibration) {
                                    warn!("Failed to send calibration command");
                                }
                            }
                            if ui.button("⚡ Fast Calibration").clicked() {
                                if let Err(_) = sender.0.try_send(Command::StartCalibration) {
                                    warn!("Failed to send fast calibration command");
                                }
                            }
                        });
                    }
                });
                
                ui.add_space(10.0);
                
                // Performance Monitoring Section
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new("📊 Performance Monitoring")
                            .size(14.0)
                            .color(CyrupTheme::ACCENT)
                            .strong()
                    );
                    ui.add_space(5.0);
                    
                    ui.checkbox(&mut settings_panel.performance_monitoring, "Enable detailed performance metrics");
                    
                    if settings_panel.performance_monitoring {
                        ui.horizontal(|ui| {
                            ui.label("Current FPS:");
                            match system_status.current_fps {
                                Some(fps) => ui.label(format!("{:.1}", fps)),
                                None => ui.label("--"),
                            };
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Frame Time:");
                            match system_status.current_fps {
                                Some(fps) if fps > 0.0 => ui.label(format!("{:.2} ms", 1000.0 / fps)),
                                _ => ui.label("-- ms"),
                            };
                        });
                    }
                });
                
                ui.add_space(10.0);
                
                // Cache Management Section
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new("💾 Cache Management")
                            .size(14.0)
                            .color(CyrupTheme::ACCENT)
                            .strong()
                    );
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("🗑️ Clear Cache").clicked() {
                            // Future: Implement cache clearing
                            info!("Cache clearing requested");
                        }
                        if ui.button("🔄 Refresh Dependencies").clicked() {
                            // Future: Implement dependency refresh
                            info!("Dependency refresh requested");
                        }
                    });
                });
                
                ui.add_space(15.0);
                
                // Close button
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() - 80.0);
                    if ui.button("✅ Close").clicked() {
                        settings_panel.is_open = false;
                    }
                });
            });
    }
}

/// Reset UI render guard each frame to allow fresh rendering
#[inline]
pub fn reset_ui_guard(mut guard: ResMut<UiRenderGuard>) {
    guard.rendered_this_frame = false;
}