//! Browser plugin components for Bevy ECS
//!
//! This module defines all components used by the browser plugin for entity management,
//! rendering, and input handling within the Bevy ECS architecture.

use bevy::prelude::*;
use bevy::render::render_resource::{BindGroup, Buffer, RenderPipeline, Texture, TextureFormat};
use serde::{Deserialize, Serialize};

/// Component marking an entity as a browser instance
#[derive(Component, Debug, Clone, Reflect)]
pub struct BrowserEntity {
    /// Unique identifier for this browser instance
    pub id: String,
    /// Current URL being displayed
    pub current_url: String,
    /// Whether this browser instance is currently active/focused
    pub is_active: bool,
    /// Browser viewport size
    pub viewport_size: (u32, u32),
}

impl BrowserEntity {
    /// Create a new browser entity with default settings
    #[inline(always)]
    pub fn new(id: String, initial_url: String) -> Self {
        Self {
            id,
            current_url: initial_url,
            is_active: false,
            viewport_size: (1920, 1080), // Default AR glasses resolution
        }
    }

    /// Set the active state of this browser instance
    #[inline(always)]
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Update the viewport size
    #[inline(always)]
    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_size = (width, height);
    }

    /// Navigate to a new URL
    #[inline(always)]
    pub fn navigate_to(&mut self, url: String) {
        self.current_url = url;
    }
}

/// Component for browser rendering surface
#[derive(Component, Debug)]
pub struct BrowserSurface {
    /// WGPU render pipeline for browser content
    pub render_pipeline: Option<RenderPipeline>,
    /// Vertex buffer for rendering
    pub vertex_buffer: Option<Buffer>,
    /// Index buffer for rendering
    pub index_buffer: Option<Buffer>,
    /// Bind group for shader resources
    pub bind_group: Option<BindGroup>,
    /// Texture for browser content
    pub texture: Option<Texture>,
    /// Surface format
    pub format: TextureFormat,
    /// Whether the surface needs updating
    pub needs_update: bool,
}

impl BrowserSurface {
    /// Create a new browser surface with default settings
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            bind_group: None,
            texture: None,
            format: TextureFormat::Bgra8UnormSrgb,
            needs_update: true,
        }
    }

    /// Mark surface as needing update
    #[inline(always)]
    pub fn mark_dirty(&mut self) {
        self.needs_update = true;
    }

    /// Clear the dirty flag
    #[inline(always)]
    pub fn clear_dirty(&mut self) {
        self.needs_update = false;
    }
}

impl Default for BrowserSurface {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Component for browser input handling
#[derive(Component, Debug, Clone)]
pub struct BrowserInput {
    /// Whether this browser can receive keyboard input
    pub accepts_keyboard: bool,
    /// Whether this browser can receive mouse input
    pub accepts_mouse: bool,
    /// Whether this browser can receive touch input
    pub accepts_touch: bool,
    /// Last mouse position within the browser
    pub last_mouse_position: (f32, f32),
    /// Currently pressed keys
    pub pressed_keys: Vec<KeyCode>,
    /// Mouse button states
    pub mouse_buttons: MouseButtonState,
}

/// Mouse button state tracking
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MouseButtonState {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl MouseButtonState {
    /// Create new mouse button state with all buttons released
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
        }
    }

    /// Check if any mouse button is pressed
    #[inline(always)]
    pub const fn any_pressed(self) -> bool {
        self.left || self.right || self.middle
    }
}

impl Default for MouseButtonState {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserInput {
    /// Create a new browser input component with default settings
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            accepts_keyboard: true,
            accepts_mouse: true,
            accepts_touch: true,
            last_mouse_position: (0.0, 0.0),
            pressed_keys: Vec::new(),
            mouse_buttons: MouseButtonState::new(),
        }
    }

    /// Update mouse position
    #[inline(always)]
    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.last_mouse_position = (x, y);
    }

    /// Set mouse button state
    #[inline(always)]
    pub fn set_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        match button {
            MouseButton::Left => self.mouse_buttons.left = pressed,
            MouseButton::Right => self.mouse_buttons.right = pressed,
            MouseButton::Middle => self.mouse_buttons.middle = pressed,
            _ => {} // Other buttons not tracked
        }
    }

    /// Add pressed key
    #[inline(always)]
    pub fn add_key(&mut self, key: KeyCode) {
        if !self.pressed_keys.contains(&key) {
            self.pressed_keys.push(key);
        }
    }

    /// Remove pressed key
    #[inline(always)]
    pub fn remove_key(&mut self, key: KeyCode) {
        self.pressed_keys.retain(|&k| k != key);
    }

    /// Clear all pressed keys
    #[inline(always)]
    pub fn clear_keys(&mut self) {
        self.pressed_keys.clear();
    }

    /// Check if a specific key is pressed
    #[inline(always)]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
}

impl Default for BrowserInput {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Component for browser navigation state
#[derive(Component, Debug, Clone)]
pub struct BrowserNavigation {
    /// Whether the browser is currently loading
    pub is_loading: bool,
    /// Loading progress (0.0 to 1.0)
    pub loading_progress: f32,
    /// Whether navigation can go back
    pub can_go_back: bool,
    /// Whether navigation can go forward
    pub can_go_forward: bool,
    /// Current page title
    pub page_title: String,
    /// Current favicon URL
    pub favicon_url: Option<String>,
}

impl BrowserNavigation {
    /// Create a new browser navigation component
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            is_loading: false,
            loading_progress: 0.0,
            can_go_back: false,
            can_go_forward: false,
            page_title: String::new(),
            favicon_url: None,
        }
    }

    /// Start loading
    #[inline(always)]
    pub fn start_loading(&mut self) {
        self.is_loading = true;
        self.loading_progress = 0.0;
    }

    /// Update loading progress
    #[inline(always)]
    pub fn update_progress(&mut self, progress: f32) {
        self.loading_progress = progress.clamp(0.0, 1.0);
        if self.loading_progress >= 1.0 {
            self.is_loading = false;
        }
    }

    /// Complete loading
    #[inline(always)]
    pub fn complete_loading(&mut self) {
        self.is_loading = false;
        self.loading_progress = 1.0;
    }

    /// Set navigation capabilities
    #[inline(always)]
    pub fn set_navigation_state(&mut self, can_back: bool, can_forward: bool) {
        self.can_go_back = can_back;
        self.can_go_forward = can_forward;
    }

    /// Update page information
    #[inline(always)]
    pub fn update_page_info(&mut self, title: String, favicon: Option<String>) {
        self.page_title = title;
        self.favicon_url = favicon;
    }
}

impl Default for BrowserNavigation {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Component bundle for creating a complete browser entity
#[derive(Bundle)]
pub struct BrowserBundle {
    pub entity: BrowserEntity,
    pub surface: BrowserSurface,
    pub input: BrowserInput,
    pub navigation: BrowserNavigation,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl BrowserBundle {
    /// Create a new browser bundle with the given ID and initial URL
    #[inline(always)]
    pub fn new(id: String, initial_url: String) -> Self {
        Self {
            entity: BrowserEntity::new(id, initial_url),
            surface: BrowserSurface::new(),
            input: BrowserInput::new(),
            navigation: BrowserNavigation::new(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }
}