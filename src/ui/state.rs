use bevy::prelude::Resource;

#[derive(Debug, Default, Resource)]
pub struct SystemStatus {
    pub fps: f64,
    pub jitter: f64,
}

#[derive(Debug, Default, Resource)]
pub struct SettingsPanelState {
    pub display_preset: DisplayPreset,
    pub sbs_enabled: bool,
    pub head_locked: bool,
    pub brightness: u8,
    pub is_open: bool,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum DisplayPreset {
    #[default]
    Standard,
    Cinema,
    Gaming,
}

#[derive(Debug, Default, Resource)]
pub struct TopMenuState {
    pub selected_tab: AppTab,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum AppTab {
    #[default]
    Settings,
    Screen,
    About,
}

#[derive(Resource, Debug)]
pub struct JitterMetrics {
    pub frame_times: Vec<f64>,
    pub last_capture_time: f64,
    pub capture_intervals: Vec<f64>,
}

impl Default for JitterMetrics {
    fn default() -> Self {
        Self {
            frame_times: Vec::with_capacity(100),
            last_capture_time: 0.0,
            capture_intervals: Vec::with_capacity(100),
        }
    }
}

impl JitterMetrics {
    pub fn add_capture_measurement(&mut self, interval: f64) {
        self.capture_intervals.push(interval);
        if self.capture_intervals.len() > 100 {
            self.capture_intervals.remove(0);
        }
    }
}
