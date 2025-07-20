//! Terminal plugin components for Bevy ECS
//!
//! This module defines all components used by the terminal plugin for entity management,
//! rendering, and input handling within the Bevy ECS architecture.

use bevy::prelude::*;
use bevy::render::render_resource::{Buffer, RenderPipeline, Texture, TextureView};


/// Keyboard modifiers helper struct
#[derive(Debug, Clone, Copy)]
pub struct KeyboardModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Default for KeyboardModifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }
}

/// Component marking an entity as a terminal instance
#[derive(Component, Debug, Clone, Reflect)]
pub struct TerminalEntity {
    /// Unique identifier for this terminal instance
    pub id: String,
    /// Shell command being executed
    pub shell_command: String,
    /// Whether this terminal instance is currently active/focused
    pub is_active: bool,
    /// Terminal grid size (columns, rows)
    pub grid_size: (usize, usize),
    /// Font size in pixels
    pub font_size: f32,
    /// Whether the terminal is running
    pub is_running: bool,
}

impl TerminalEntity {
    /// Create a new terminal entity with default settings
    #[inline(always)]
    pub fn new(id: String, shell_command: String) -> Self {
        Self {
            id,
            shell_command,
            is_active: false,
            grid_size: (80, 24), // Standard terminal size
            font_size: 14.0,
            is_running: false,
        }
    }

    /// Set the active state of this terminal instance
    #[inline(always)]
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Update the grid size
    #[inline(always)]
    pub fn set_grid_size(&mut self, cols: usize, rows: usize) {
        self.grid_size = (cols, rows);
    }

    /// Update the font size
    #[inline(always)]
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size.max(8.0).min(72.0); // Reasonable bounds
    }

    /// Start the terminal
    #[inline(always)]
    pub fn start(&mut self) {
        self.is_running = true;
    }

    /// Stop the terminal
    #[inline(always)]
    pub fn stop(&mut self) {
        self.is_running = false;
    }

    /// Calculate viewport size based on grid and font
    #[inline(always)]
    pub fn calculate_viewport_size(&self) -> (u32, u32) {
        let char_width = self.font_size * 0.6; // Approximate monospace width
        let char_height = self.font_size * 1.2; // Line height with spacing
        
        let width = (self.grid_size.0 as f32 * char_width) as u32;
        let height = (self.grid_size.1 as f32 * char_height) as u32;
        
        (width.max(320), height.max(240)) // Minimum size
    }
}

/// Component for terminal rendering surface
#[derive(Component, Debug)]
pub struct TerminalSurface {
    /// WGPU render pipeline for terminal content
    pub render_pipeline: Option<RenderPipeline>,
    /// Vertex buffer for text rendering
    pub vertex_buffer: Option<Buffer>,
    /// Index buffer for text rendering
    pub index_buffer: Option<Buffer>,
    /// Texture for terminal text content
    pub text_texture: Option<Texture>,
    /// Texture view for rendering
    pub text_texture_view: Option<TextureView>,
    /// Surface format
    pub format: wgpu::TextureFormat,
    /// Whether the surface needs updating
    pub needs_update: bool,
    /// Last render time for performance tracking
    pub last_render_time: f32,
}

impl TerminalSurface {
    /// Create a new terminal surface with default settings
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            text_texture: None,
            text_texture_view: None,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            needs_update: true,
            last_render_time: 0.0,
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

    /// Update render time
    #[inline(always)]
    pub fn update_render_time(&mut self, time_ms: f32) {
        self.last_render_time = time_ms;
    }
}

impl Default for TerminalSurface {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Component for terminal input handling
#[derive(Component, Debug, Clone, Reflect)]
pub struct TerminalInput {
    /// Whether this terminal can receive keyboard input
    pub accepts_keyboard: bool,
    /// Whether this terminal can receive mouse input
    pub accepts_mouse: bool,
    /// Input buffer for pending characters
    pub input_buffer: Vec<char>,
    /// Currently pressed keys
    pub pressed_keys: Vec<KeyCode>,
    /// Mouse position within terminal grid
    pub mouse_grid_position: (usize, usize),
    /// Mouse button states
    pub mouse_buttons: MouseButtonState,
    /// Whether we're in paste mode
    pub paste_mode: bool,
}

/// Mouse button state tracking for terminal
#[derive(Debug, Clone, Copy, Reflect)]
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

impl TerminalInput {
    /// Create a new terminal input component with default settings
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            accepts_keyboard: true,
            accepts_mouse: true,
            input_buffer: Vec::new(),
            pressed_keys: Vec::new(),
            mouse_grid_position: (0, 0),
            mouse_buttons: MouseButtonState::new(),
            paste_mode: false,
        }
    }

    /// Add character to input buffer
    #[inline(always)]
    pub fn add_char(&mut self, ch: char) {
        self.input_buffer.push(ch);
    }

    /// Add string to input buffer
    #[inline(always)]
    pub fn add_string(&mut self, s: &str) {
        self.input_buffer.extend(s.chars());
    }

    /// Drain input buffer and return contents
    #[inline(always)]
    pub fn drain_input(&mut self) -> String {
        let result: String = self.input_buffer.iter().collect();
        self.input_buffer.clear();
        result
    }

    /// Update mouse grid position
    #[inline(always)]
    pub fn update_mouse_grid_position(&mut self, col: usize, row: usize) {
        self.mouse_grid_position = (col, row);
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

    /// Set paste mode
    #[inline(always)]
    pub fn set_paste_mode(&mut self, paste_mode: bool) {
        self.paste_mode = paste_mode;
    }

    /// Handle special key combinations
    #[inline(always)]
    pub fn handle_key_combination(&mut self, key: KeyCode, modifiers: &KeyboardModifiers) -> Option<String> {
        // Handle common terminal key combinations
        match (key, modifiers.ctrl, modifiers.shift, modifiers.alt) {
            (KeyCode::KeyC, true, false, false) => Some("\x03".to_string()), // Ctrl+C
            (KeyCode::KeyD, true, false, false) => Some("\x04".to_string()), // Ctrl+D
            (KeyCode::KeyZ, true, false, false) => Some("\x1a".to_string()), // Ctrl+Z
            (KeyCode::KeyL, true, false, false) => Some("\x0c".to_string()), // Ctrl+L
            (KeyCode::Enter, false, false, false) => Some("\r".to_string()), // Enter
            (KeyCode::Tab, false, false, false) => Some("\t".to_string()),   // Tab
            (KeyCode::Backspace, false, false, false) => Some("\x7f".to_string()), // Backspace
            (KeyCode::Escape, false, false, false) => Some("\x1b".to_string()), // Escape
            _ => None,
        }
    }
}

impl Default for TerminalInput {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Component for terminal cursor state
#[derive(Component, Debug, Clone, Reflect)]
pub struct TerminalCursor {
    /// Cursor position in grid (column, row)
    pub position: (usize, usize),
    /// Whether cursor is visible
    pub visible: bool,
    /// Cursor blink state
    pub blink_state: bool,
    /// Time since last blink toggle
    pub blink_timer: f32,
    /// Blink interval in seconds
    pub blink_interval: f32,
    /// Cursor style
    pub style: CursorStyle,
}

/// Terminal cursor styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

impl TerminalCursor {
    /// Create a new terminal cursor
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            position: (0, 0),
            visible: true,
            blink_state: true,
            blink_timer: 0.0,
            blink_interval: 0.5, // 500ms blink interval
            style: CursorStyle::Block,
        }
    }

    /// Update cursor position
    #[inline(always)]
    pub fn set_position(&mut self, col: usize, row: usize) {
        self.position = (col, row);
    }

    /// Set cursor visibility
    #[inline(always)]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Update blink animation
    #[inline(always)]
    pub fn update_blink(&mut self, delta_time: f32) {
        if self.visible {
            self.blink_timer += delta_time;
            if self.blink_timer >= self.blink_interval {
                self.blink_state = !self.blink_state;
                self.blink_timer = 0.0;
            }
        }
    }

    /// Check if cursor should be rendered
    #[inline(always)]
    pub fn should_render(&self) -> bool {
        self.visible && self.blink_state
    }

    /// Set cursor style
    #[inline(always)]
    pub fn set_style(&mut self, style: CursorStyle) {
        self.style = style;
    }
}

impl Default for TerminalCursor {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Component for terminal scrollback buffer
#[derive(Component, Debug, Clone, Reflect)]
pub struct TerminalScrollback {
    /// Maximum number of lines to keep in scrollback
    pub max_lines: usize,
    /// Current scroll position (0 = bottom, positive = scrolled up)
    pub scroll_position: usize,
    /// Whether scrollback is enabled
    pub enabled: bool,
}

impl TerminalScrollback {
    /// Create a new scrollback buffer
    #[inline(always)]
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            scroll_position: 0,
            enabled: true,
        }
    }

    /// Scroll up by the specified number of lines
    #[inline(always)]
    pub fn scroll_up(&mut self, lines: usize) {
        if self.enabled {
            self.scroll_position = (self.scroll_position + lines).min(self.max_lines);
        }
    }

    /// Scroll down by the specified number of lines
    #[inline(always)]
    pub fn scroll_down(&mut self, lines: usize) {
        if self.enabled {
            self.scroll_position = self.scroll_position.saturating_sub(lines);
        }
    }

    /// Reset scroll position to bottom
    #[inline(always)]
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = 0;
    }

    /// Check if scrolled away from bottom
    #[inline(always)]
    pub fn is_scrolled(&self) -> bool {
        self.scroll_position > 0
    }

    /// Set scrollback enabled state
    #[inline(always)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.scroll_position = 0;
        }
    }
}

impl Default for TerminalScrollback {
    #[inline(always)]
    fn default() -> Self {
        Self::new(1000) // Default 1000 lines of scrollback
    }
}

/// Component bundle for creating a complete terminal entity
#[derive(Bundle)]
pub struct TerminalBundle {
    pub entity: TerminalEntity,
    pub surface: TerminalSurface,
    pub input: TerminalInput,
    pub cursor: TerminalCursor,
    pub scrollback: TerminalScrollback,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl TerminalBundle {
    /// Create a new terminal bundle with the given ID and shell command
    #[inline(always)]
    pub fn new(id: String, shell_command: String) -> Self {
        Self {
            entity: TerminalEntity::new(id, shell_command),
            surface: TerminalSurface::new(),
            input: TerminalInput::new(),
            cursor: TerminalCursor::new(),
            scrollback: TerminalScrollback::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
        }
    }

    /// Create a terminal bundle with custom grid size
    #[inline(always)]
    pub fn with_grid_size(id: String, shell_command: String, cols: usize, rows: usize) -> Self {
        let mut bundle = Self::new(id, shell_command);
        bundle.entity.set_grid_size(cols, rows);
        bundle
    }

    /// Create a terminal bundle with custom font size
    #[inline(always)]
    pub fn with_font_size(id: String, shell_command: String, font_size: f32) -> Self {
        let mut bundle = Self::new(id, shell_command);
        bundle.entity.set_font_size(font_size);
        bundle
    }
}