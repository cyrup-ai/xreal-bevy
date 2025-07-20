//! State Schema Definitions for XREAL Virtual Desktop
//! 
//! Defines the complete state schema with versioned serialization support.
//! All state components are designed for atomic operations and validation.

use anyhow::Result;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema version for state migration support
pub const STATE_SCHEMA_VERSION: &str = "1.0.0";

/// Complete application state schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Schema version for migration support
    pub schema_version: String,
    /// Timestamp of last state update
    pub last_updated: u64,
    /// User preference settings
    pub user_preferences: UserPreferences,
    /// UI state and layout
    pub ui_state: UiState,
    /// IMU calibration data
    pub calibration_data: CalibrationData,
    /// Plugin system state
    pub plugin_state: PluginSystemState,
    /// Performance settings and thresholds
    pub performance_settings: PerformanceSettings,
    /// Window layout and positioning
    pub window_layout: WindowLayout,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION.to_string(),
            last_updated: 0,
            user_preferences: UserPreferences::default(),
            ui_state: UiState::default(),
            calibration_data: CalibrationData::default(),
            plugin_state: PluginSystemState::default(),
            performance_settings: PerformanceSettings::default(),
            window_layout: WindowLayout::default(),
        }
    }
}

/// User preference settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Virtual screen distance from user
    pub screen_distance: f32,
    /// 3D stereoscopic display mode enabled
    pub display_mode_3d: bool,
    /// Roll lock enabled for head tracking
    pub roll_lock_enabled: bool,
    /// Brightness level (0-7)
    pub brightness_level: u8,
    /// Auto-brightness enabled
    pub auto_brightness: bool,
    /// Comfort settings
    pub comfort_settings: ComfortSettings,
    /// Accessibility settings
    pub accessibility_settings: AccessibilitySettings,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            screen_distance: -5.0,
            display_mode_3d: true,
            roll_lock_enabled: false,
            brightness_level: 4,
            auto_brightness: false,
            comfort_settings: ComfortSettings::default(),
            accessibility_settings: AccessibilitySettings::default(),
        }
    }
}

/// Comfort settings for extended use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfortSettings {
    /// Eye strain reduction mode
    pub eye_strain_reduction: bool,
    /// Blue light filter intensity (0.0-1.0)
    pub blue_light_filter: f32,
    /// Motion comfort settings
    pub motion_comfort_level: MotionComfortLevel,
    /// Break reminders enabled
    pub break_reminders: bool,
    /// Break interval in minutes
    pub break_interval_minutes: u32,
}

impl Default for ComfortSettings {
    fn default() -> Self {
        Self {
            eye_strain_reduction: false,
            blue_light_filter: 0.0,
            motion_comfort_level: MotionComfortLevel::Normal,
            break_reminders: false,
            break_interval_minutes: 30,
        }
    }
}

/// Motion comfort levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MotionComfortLevel {
    High,
    Normal,
    Low,
}

/// Accessibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// High contrast mode enabled
    pub high_contrast: bool,
    /// Text scaling factor
    pub text_scaling: f32,
    /// Voice control enabled
    pub voice_control: bool,
    /// Reduced motion enabled
    pub reduced_motion: bool,
    /// Screen reader compatibility
    pub screen_reader_support: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            high_contrast: false,
            text_scaling: 1.0,
            voice_control: false,
            reduced_motion: false,
            screen_reader_support: false,
        }
    }
}

/// UI state and layout preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    /// Settings panel open state
    pub settings_panel_open: bool,
    /// Selected display preset
    pub selected_preset: DisplayPreset,
    /// Performance monitoring enabled
    pub performance_monitoring: bool,
    /// Advanced calibration mode enabled
    pub advanced_calibration: bool,
    /// Selected app tab
    pub selected_tab: AppTab,
    /// UI theme settings
    pub theme: UiTheme,
    /// Menu states
    pub menu_states: MenuStates,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            settings_panel_open: false,
            selected_preset: DisplayPreset::Productivity,
            performance_monitoring: false,
            advanced_calibration: false,
            selected_tab: AppTab::Browser,
            theme: UiTheme::default(),
            menu_states: MenuStates::default(),
        }
    }
}

/// Display presets for different use cases
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DisplayPreset {
    Gaming,
    Productivity,
    Cinema,
    Reading,
    Development,
}

impl Default for DisplayPreset {
    fn default() -> Self {
        DisplayPreset::Productivity
    }
}

/// Application tabs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AppTab {
    Browser,
    Terminal,
    VSCode,
    Files,
    Media,
    Games,
    Settings,
}

impl Default for AppTab {
    fn default() -> Self {
        AppTab::Browser
    }
}

/// UI theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTheme {
    /// Dark mode enabled
    pub dark_mode: bool,
    /// Primary color theme
    pub primary_color: ColorTheme,
    /// UI opacity (0.0-1.0)
    pub ui_opacity: f32,
    /// Font size scaling
    pub font_scale: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            dark_mode: true,
            primary_color: ColorTheme::Blue,
            ui_opacity: 0.9,
            font_scale: 1.0,
        }
    }
}

/// Color theme options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorTheme {
    Blue,
    Green,
    Purple,
    Orange,
    Red,
    Custom,
}

/// Menu states for different UI components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuStates {
    /// Top menu hover state
    pub top_menu_hover: bool,
    /// Top menu open state
    pub top_menu_open: bool,
    /// Context menu positions
    pub context_menu_positions: HashMap<String, (f32, f32)>,
    /// Collapsed menu sections
    pub collapsed_sections: Vec<String>,
}

impl Default for MenuStates {
    fn default() -> Self {
        Self {
            top_menu_hover: false,
            top_menu_open: false,
            context_menu_positions: HashMap::new(),
            collapsed_sections: Vec::new(),
        }
    }
}

/// IMU calibration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    /// Gyroscope bias correction
    pub gyro_bias: [f32; 3],
    /// Accelerometer bias correction
    pub accel_bias: [f32; 3],
    /// Magnetometer bias correction
    pub mag_bias: [f32; 3],
    /// Calibration quality scores (0.0-1.0)
    pub calibration_quality: CalibrationQuality,
    /// Calibration timestamp
    pub calibration_timestamp: u64,
    /// Calibration environment data
    pub calibration_environment: CalibrationEnvironment,
    /// Advanced calibration parameters
    pub advanced_params: AdvancedCalibrationParams,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            gyro_bias: [0.0; 3],
            accel_bias: [0.0; 3],
            mag_bias: [0.0; 3],
            calibration_quality: CalibrationQuality::default(),
            calibration_timestamp: 0,
            calibration_environment: CalibrationEnvironment::default(),
            advanced_params: AdvancedCalibrationParams::default(),
        }
    }
}

/// Calibration quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationQuality {
    /// Gyroscope calibration quality (0.0-1.0)
    pub gyro_quality: f32,
    /// Accelerometer calibration quality (0.0-1.0)
    pub accel_quality: f32,
    /// Magnetometer calibration quality (0.0-1.0)
    pub mag_quality: f32,
    /// Overall calibration confidence (0.0-1.0)
    pub overall_confidence: f32,
}

impl Default for CalibrationQuality {
    fn default() -> Self {
        Self {
            gyro_quality: 0.0,
            accel_quality: 0.0,
            mag_quality: 0.0,
            overall_confidence: 0.0,
        }
    }
}

/// Calibration environment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationEnvironment {
    /// Magnetic field strength
    pub magnetic_field_strength: f32,
    /// Temperature during calibration
    pub temperature: f32,
    /// Ambient light level
    pub ambient_light: f32,
    /// Motion stability during calibration
    pub motion_stability: f32,
}

impl Default for CalibrationEnvironment {
    fn default() -> Self {
        Self {
            magnetic_field_strength: 0.0,
            temperature: 20.0,
            ambient_light: 0.5,
            motion_stability: 0.0,
        }
    }
}

/// Advanced calibration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCalibrationParams {
    /// Adaptive filter coefficients
    pub filter_coefficients: [f32; 4],
    /// Noise covariance matrix
    pub noise_covariance: [f32; 9],
    /// Sensor fusion weights
    pub fusion_weights: [f32; 3],
    /// Drift compensation parameters
    pub drift_compensation: DriftCompensation,
}

impl Default for AdvancedCalibrationParams {
    fn default() -> Self {
        Self {
            filter_coefficients: [0.1, 0.1, 0.1, 0.1],
            noise_covariance: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            fusion_weights: [0.33, 0.33, 0.34],
            drift_compensation: DriftCompensation::default(),
        }
    }
}

/// Drift compensation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftCompensation {
    /// Drift correction enabled
    pub enabled: bool,
    /// Drift correction rate
    pub correction_rate: f32,
    /// Drift threshold
    pub drift_threshold: f32,
    /// Compensation algorithm
    pub algorithm: DriftCompensationAlgorithm,
}

impl Default for DriftCompensation {
    fn default() -> Self {
        Self {
            enabled: true,
            correction_rate: 0.01,
            drift_threshold: 0.1,
            algorithm: DriftCompensationAlgorithm::Kalman,
        }
    }
}

/// Drift compensation algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DriftCompensationAlgorithm {
    Kalman,
    Complementary,
    Madgwick,
    Mahony,
}

/// Plugin system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemState {
    /// Enabled plugins
    pub enabled_plugins: HashMap<String, PluginConfig>,
    /// Plugin loading order
    pub plugin_order: Vec<String>,
    /// Plugin system settings
    pub system_settings: PluginSystemSettings,
    /// Plugin resource limits
    pub resource_limits: PluginResourceLimits,
}

impl Default for PluginSystemState {
    fn default() -> Self {
        Self {
            enabled_plugins: HashMap::new(),
            plugin_order: Vec::new(),
            system_settings: PluginSystemSettings::default(),
            resource_limits: PluginResourceLimits::default(),
        }
    }
}

/// Individual plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin enabled state
    pub enabled: bool,
    /// Plugin-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Plugin window configuration
    pub window_config: PluginWindowConfig,
    /// Plugin permissions
    pub permissions: PluginPermissions,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
            window_config: PluginWindowConfig::default(),
            permissions: PluginPermissions::default(),
        }
    }
}

/// Plugin window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginWindowConfig {
    /// Window position
    pub position: (f32, f32),
    /// Window size
    pub size: (f32, f32),
    /// Window visibility
    pub visible: bool,
    /// Window z-order
    pub z_order: i32,
    /// Window transparency
    pub transparency: f32,
}

impl Default for PluginWindowConfig {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            size: (800.0, 600.0),
            visible: true,
            z_order: 0,
            transparency: 1.0,
        }
    }
}

/// Plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// Network access allowed
    pub network_access: bool,
    /// File system access allowed
    pub file_system_access: bool,
    /// Microphone access allowed
    pub microphone_access: bool,
    /// Camera access allowed
    pub camera_access: bool,
    /// System information access allowed
    pub system_info_access: bool,
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            network_access: false,
            file_system_access: false,
            microphone_access: false,
            camera_access: false,
            system_info_access: false,
        }
    }
}

/// Plugin system settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemSettings {
    /// Auto-load plugins on startup
    pub auto_load_plugins: bool,
    /// Hot reload enabled
    pub hot_reload_enabled: bool,
    /// Sandbox mode enabled
    pub sandbox_enabled: bool,
    /// Maximum concurrent plugins
    pub max_concurrent_plugins: u32,
    /// Plugin update check interval
    pub update_check_interval: u32,
}

impl Default for PluginSystemSettings {
    fn default() -> Self {
        Self {
            auto_load_plugins: true,
            hot_reload_enabled: cfg!(debug_assertions),
            sandbox_enabled: true,
            max_concurrent_plugins: 16,
            update_check_interval: 86400, // 24 hours
        }
    }
}

/// Plugin resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResourceLimits {
    /// Maximum memory per plugin (MB)
    pub max_memory_per_plugin: u64,
    /// Maximum total memory for all plugins (MB)
    pub max_total_memory: u64,
    /// Maximum CPU usage per plugin (%)
    pub max_cpu_per_plugin: f32,
    /// Maximum GPU memory per plugin (MB)
    pub max_gpu_memory_per_plugin: u64,
    /// Maximum file handles per plugin
    pub max_file_handles_per_plugin: u32,
}

impl Default for PluginResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_per_plugin: 512,
            max_total_memory: 2048,
            max_cpu_per_plugin: 25.0,
            max_gpu_memory_per_plugin: 256,
            max_file_handles_per_plugin: 100,
        }
    }
}

/// Performance settings and thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Target frame rate
    pub target_fps: u32,
    /// Frame time budget (ms)
    pub frame_time_budget: f32,
    /// Jitter tolerance threshold (ms)
    pub jitter_threshold: f32,
    /// Performance monitoring enabled
    pub monitoring_enabled: bool,
    /// Performance optimization settings
    pub optimization_settings: OptimizationSettings,
    /// Resource monitoring settings
    pub resource_monitoring: ResourceMonitoring,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            target_fps: 60,
            frame_time_budget: 16.0,
            jitter_threshold: 1.0,
            monitoring_enabled: true,
            optimization_settings: OptimizationSettings::default(),
            resource_monitoring: ResourceMonitoring::default(),
        }
    }
}

/// Performance optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    /// Aggressive optimization enabled
    pub aggressive_optimization: bool,
    /// Render scaling factor
    pub render_scaling: f32,
    /// Texture quality level
    pub texture_quality: TextureQuality,
    /// Shader optimization level
    pub shader_optimization: ShaderOptimization,
    /// Memory optimization enabled
    pub memory_optimization: bool,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            aggressive_optimization: false,
            render_scaling: 1.0,
            texture_quality: TextureQuality::High,
            shader_optimization: ShaderOptimization::Balanced,
            memory_optimization: true,
        }
    }
}

/// Texture quality levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
    Ultra,
}

/// Shader optimization levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShaderOptimization {
    Disabled,
    Basic,
    Balanced,
    Aggressive,
}

/// Resource monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMonitoring {
    /// CPU monitoring enabled
    pub cpu_monitoring: bool,
    /// Memory monitoring enabled
    pub memory_monitoring: bool,
    /// GPU monitoring enabled
    pub gpu_monitoring: bool,
    /// Network monitoring enabled
    pub network_monitoring: bool,
    /// Monitoring interval (seconds)
    pub monitoring_interval: f32,
}

impl Default for ResourceMonitoring {
    fn default() -> Self {
        Self {
            cpu_monitoring: true,
            memory_monitoring: true,
            gpu_monitoring: true,
            network_monitoring: false,
            monitoring_interval: 1.0,
        }
    }
}

/// Window layout and positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    /// Main window configuration
    pub main_window: WindowConfig,
    /// Plugin windows
    pub plugin_windows: HashMap<String, WindowConfig>,
    /// Virtual desktop layout
    pub virtual_desktop: VirtualDesktopLayout,
    /// Multi-monitor setup
    pub multi_monitor: MultiMonitorSetup,
}

impl Default for WindowLayout {
    fn default() -> Self {
        Self {
            main_window: WindowConfig::default(),
            plugin_windows: HashMap::new(),
            virtual_desktop: VirtualDesktopLayout::default(),
            multi_monitor: MultiMonitorSetup::default(),
        }
    }
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window position
    pub position: (f32, f32),
    /// Window size
    pub size: (f32, f32),
    /// Window minimized state
    pub minimized: bool,
    /// Window maximized state
    pub maximized: bool,
    /// Window fullscreen state
    pub fullscreen: bool,
    /// Window always on top
    pub always_on_top: bool,
    /// Window decorations enabled
    pub decorations: bool,
    /// Window transparency
    pub transparency: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            position: (100.0, 100.0),
            size: (450.0, 350.0),
            minimized: false,
            maximized: false,
            fullscreen: false,
            always_on_top: true,
            decorations: true,
            transparency: 1.0,
        }
    }
}

/// Virtual desktop layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualDesktopLayout {
    /// Desktop grid size
    pub grid_size: (u32, u32),
    /// Active desktop index
    pub active_desktop: u32,
    /// Desktop configurations
    pub desktop_configs: HashMap<u32, DesktopConfig>,
    /// Transition animation settings
    pub transition_settings: TransitionSettings,
}

impl Default for VirtualDesktopLayout {
    fn default() -> Self {
        Self {
            grid_size: (3, 2),
            active_desktop: 0,
            desktop_configs: HashMap::new(),
            transition_settings: TransitionSettings::default(),
        }
    }
}

/// Desktop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Desktop name
    pub name: String,
    /// Desktop background
    pub background: DesktopBackground,
    /// Desktop applications
    pub applications: Vec<String>,
    /// Desktop shortcuts
    pub shortcuts: HashMap<String, String>,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            name: "Desktop".to_string(),
            background: DesktopBackground::default(),
            applications: Vec::new(),
            shortcuts: HashMap::new(),
        }
    }
}

/// Desktop background settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopBackground {
    /// Background type
    pub background_type: BackgroundType,
    /// Background color
    pub color: [f32; 4],
    /// Background image path
    pub image_path: Option<String>,
    /// Background animation enabled
    pub animation_enabled: bool,
}

impl Default for DesktopBackground {
    fn default() -> Self {
        Self {
            background_type: BackgroundType::Color,
            color: [0.1, 0.1, 0.1, 1.0],
            image_path: None,
            animation_enabled: false,
        }
    }
}

/// Background types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BackgroundType {
    Color,
    Image,
    Video,
    Procedural,
}

/// Transition settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionSettings {
    /// Transition animation enabled
    pub animation_enabled: bool,
    /// Transition duration (seconds)
    pub duration: f32,
    /// Transition easing function
    pub easing: TransitionEasing,
    /// Transition effects
    pub effects: Vec<TransitionEffect>,
}

impl Default for TransitionSettings {
    fn default() -> Self {
        Self {
            animation_enabled: true,
            duration: 0.3,
            easing: TransitionEasing::EaseInOut,
            effects: vec![TransitionEffect::Slide],
        }
    }
}

/// Transition easing functions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransitionEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// Transition effects
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransitionEffect {
    Slide,
    Fade,
    Scale,
    Rotate,
    Flip,
}

/// Multi-monitor setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMonitorSetup {
    /// Primary monitor index
    pub primary_monitor: u32,
    /// Monitor configurations
    pub monitor_configs: HashMap<u32, MonitorConfig>,
    /// Span windows across monitors
    pub span_windows: bool,
    /// Mirror mode enabled
    pub mirror_mode: bool,
}

impl Default for MultiMonitorSetup {
    fn default() -> Self {
        Self {
            primary_monitor: 0,
            monitor_configs: HashMap::new(),
            span_windows: false,
            mirror_mode: false,
        }
    }
}

/// Monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Monitor name
    pub name: String,
    /// Monitor resolution
    pub resolution: (u32, u32),
    /// Monitor refresh rate
    pub refresh_rate: u32,
    /// Monitor position
    pub position: (i32, i32),
    /// Monitor rotation
    pub rotation: MonitorRotation,
    /// Monitor enabled
    pub enabled: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            name: "Primary Monitor".to_string(),
            resolution: (1920, 1080),
            refresh_rate: 60,
            position: (0, 0),
            rotation: MonitorRotation::Normal,
            enabled: true,
        }
    }
}

/// Monitor rotation options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MonitorRotation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl CalibrationData {
    /// Create CalibrationData from Bevy CalibrationState
    pub fn from_bevy_state(bevy_state: &crate::tracking::CalibrationState) -> Self {
        Self {
            gyro_bias: bevy_state.gyro_bias,
            accel_bias: bevy_state.accel_bias,
            mag_bias: bevy_state.mag_bias,
            calibration_quality: CalibrationQuality {
                gyro_quality: bevy_state.gyro_quality,
                accel_quality: bevy_state.accel_quality,
                mag_quality: bevy_state.mag_quality,
                overall_confidence: bevy_state.overall_confidence,
            },
            calibration_timestamp: bevy_state.sample_count as u64,
            calibration_environment: CalibrationEnvironment::default(),
            advanced_params: AdvancedCalibrationParams::default(),
        }
    }
    
    /// Apply CalibrationData to Bevy CalibrationState
    pub fn apply_to_bevy_state(&self, bevy_state: &mut crate::tracking::CalibrationState) {
        bevy_state.gyro_bias = self.gyro_bias;
        bevy_state.accel_bias = self.accel_bias;
        bevy_state.mag_bias = self.mag_bias;
        bevy_state.gyro_quality = self.calibration_quality.gyro_quality;
        bevy_state.accel_quality = self.calibration_quality.accel_quality;
        bevy_state.mag_quality = self.calibration_quality.mag_quality;
        bevy_state.overall_confidence = self.calibration_quality.overall_confidence;
        bevy_state.sample_count = self.calibration_timestamp as usize;
    }
}

/// State validation trait
pub trait StateValidation {
    /// Validate state component
    fn validate(&self) -> Result<()>;
}

impl StateValidation for AppState {
    fn validate(&self) -> Result<()> {
        // Validate schema version
        if self.schema_version != STATE_SCHEMA_VERSION {
            return Err(anyhow::anyhow!(
                "Schema version mismatch: expected {}, found {}",
                STATE_SCHEMA_VERSION,
                self.schema_version
            ));
        }
        
        // Validate all components
        self.user_preferences.validate()?;
        self.ui_state.validate()?;
        self.calibration_data.validate()?;
        self.plugin_state.validate()?;
        self.performance_settings.validate()?;
        self.window_layout.validate()?;
        
        Ok(())
    }
}

impl StateValidation for UserPreferences {
    fn validate(&self) -> Result<()> {
        // Validate screen distance
        if self.screen_distance < -20.0 || self.screen_distance > 20.0 {
            return Err(anyhow::anyhow!(
                "Screen distance out of range: {} (valid range: -20.0 to 20.0)",
                self.screen_distance
            ));
        }
        
        // Validate brightness level
        if self.brightness_level > 7 {
            return Err(anyhow::anyhow!(
                "Brightness level out of range: {} (valid range: 0-7)",
                self.brightness_level
            ));
        }
        
        // Validate comfort settings
        if self.comfort_settings.blue_light_filter < 0.0 || self.comfort_settings.blue_light_filter > 1.0 {
            return Err(anyhow::anyhow!(
                "Blue light filter out of range: {} (valid range: 0.0-1.0)",
                self.comfort_settings.blue_light_filter
            ));
        }
        
        // Validate accessibility settings
        if self.accessibility_settings.text_scaling < 0.5 || self.accessibility_settings.text_scaling > 3.0 {
            return Err(anyhow::anyhow!(
                "Text scaling out of range: {} (valid range: 0.5-3.0)",
                self.accessibility_settings.text_scaling
            ));
        }
        
        Ok(())
    }
}

impl StateValidation for UiState {
    fn validate(&self) -> Result<()> {
        // Validate theme settings
        if self.theme.ui_opacity < 0.0 || self.theme.ui_opacity > 1.0 {
            return Err(anyhow::anyhow!(
                "UI opacity out of range: {} (valid range: 0.0-1.0)",
                self.theme.ui_opacity
            ));
        }
        
        if self.theme.font_scale < 0.5 || self.theme.font_scale > 3.0 {
            return Err(anyhow::anyhow!(
                "Font scale out of range: {} (valid range: 0.5-3.0)",
                self.theme.font_scale
            ));
        }
        
        Ok(())
    }
}

impl StateValidation for CalibrationData {
    fn validate(&self) -> Result<()> {
        // Validate calibration quality values
        if self.calibration_quality.gyro_quality < 0.0 || self.calibration_quality.gyro_quality > 1.0 {
            return Err(anyhow::anyhow!(
                "Gyro quality out of range: {} (valid range: 0.0-1.0)",
                self.calibration_quality.gyro_quality
            ));
        }
        
        if self.calibration_quality.accel_quality < 0.0 || self.calibration_quality.accel_quality > 1.0 {
            return Err(anyhow::anyhow!(
                "Accel quality out of range: {} (valid range: 0.0-1.0)",
                self.calibration_quality.accel_quality
            ));
        }
        
        if self.calibration_quality.mag_quality < 0.0 || self.calibration_quality.mag_quality > 1.0 {
            return Err(anyhow::anyhow!(
                "Mag quality out of range: {} (valid range: 0.0-1.0)",
                self.calibration_quality.mag_quality
            ));
        }
        
        if self.calibration_quality.overall_confidence < 0.0 || self.calibration_quality.overall_confidence > 1.0 {
            return Err(anyhow::anyhow!(
                "Overall confidence out of range: {} (valid range: 0.0-1.0)",
                self.calibration_quality.overall_confidence
            ));
        }
        
        Ok(())
    }
}

impl StateValidation for PluginSystemState {
    fn validate(&self) -> Result<()> {
        // Validate plugin system settings
        if self.system_settings.max_concurrent_plugins == 0 {
            return Err(anyhow::anyhow!(
                "Max concurrent plugins must be greater than 0"
            ));
        }
        
        if self.system_settings.max_concurrent_plugins > 64 {
            return Err(anyhow::anyhow!(
                "Max concurrent plugins exceeds limit: {} (max: 64)",
                self.system_settings.max_concurrent_plugins
            ));
        }
        
        // Validate resource limits
        if self.resource_limits.max_memory_per_plugin == 0 {
            return Err(anyhow::anyhow!(
                "Max memory per plugin must be greater than 0"
            ));
        }
        
        if self.resource_limits.max_cpu_per_plugin < 0.0 || self.resource_limits.max_cpu_per_plugin > 100.0 {
            return Err(anyhow::anyhow!(
                "Max CPU per plugin out of range: {} (valid range: 0.0-100.0)",
                self.resource_limits.max_cpu_per_plugin
            ));
        }
        
        Ok(())
    }
}

impl StateValidation for PerformanceSettings {
    fn validate(&self) -> Result<()> {
        // Validate frame rate
        if self.target_fps == 0 {
            return Err(anyhow::anyhow!(
                "Target FPS must be greater than 0"
            ));
        }
        
        if self.target_fps > 240 {
            return Err(anyhow::anyhow!(
                "Target FPS exceeds limit: {} (max: 240)",
                self.target_fps
            ));
        }
        
        // Validate frame time budget
        if self.frame_time_budget <= 0.0 {
            return Err(anyhow::anyhow!(
                "Frame time budget must be greater than 0"
            ));
        }
        
        // Validate jitter threshold
        if self.jitter_threshold < 0.0 {
            return Err(anyhow::anyhow!(
                "Jitter threshold must be non-negative"
            ));
        }
        
        // Validate optimization settings
        if self.optimization_settings.render_scaling <= 0.0 || self.optimization_settings.render_scaling > 2.0 {
            return Err(anyhow::anyhow!(
                "Render scaling out of range: {} (valid range: 0.0-2.0)",
                self.optimization_settings.render_scaling
            ));
        }
        
        Ok(())
    }
}

impl StateValidation for WindowLayout {
    fn validate(&self) -> Result<()> {
        // Validate main window configuration
        if self.main_window.size.0 <= 0.0 || self.main_window.size.1 <= 0.0 {
            return Err(anyhow::anyhow!(
                "Main window size must be positive: {:?}",
                self.main_window.size
            ));
        }
        
        if self.main_window.transparency < 0.0 || self.main_window.transparency > 1.0 {
            return Err(anyhow::anyhow!(
                "Main window transparency out of range: {} (valid range: 0.0-1.0)",
                self.main_window.transparency
            ));
        }
        
        // Validate virtual desktop layout
        if self.virtual_desktop.grid_size.0 == 0 || self.virtual_desktop.grid_size.1 == 0 {
            return Err(anyhow::anyhow!(
                "Virtual desktop grid size must be positive: {:?}",
                self.virtual_desktop.grid_size
            ));
        }
        
        let max_desktop_index = (self.virtual_desktop.grid_size.0 * self.virtual_desktop.grid_size.1) - 1;
        if self.virtual_desktop.active_desktop > max_desktop_index {
            return Err(anyhow::anyhow!(
                "Active desktop index out of range: {} (max: {})",
                self.virtual_desktop.active_desktop,
                max_desktop_index
            ));
        }
        
        Ok(())
    }
}