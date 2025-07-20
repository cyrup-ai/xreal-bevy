//! Terminal plugin resources for Bevy ECS
//!
//! This module defines all resources used by the terminal plugin for global state management,
//! configuration, and shared data within the Bevy ECS architecture.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::color_scheme::{TerminalColorScheme, ColorSchemeVariant, AnsiColor};
use crate::error::{TerminalError, TerminalResult};

/// Global terminal plugin configuration resource
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Default shell command to execute
    pub default_shell: String,
    /// Default font size in pixels
    pub default_font_size: f32,
    /// Default terminal grid size (columns, rows)
    pub default_grid_size: (usize, usize),
    /// Color scheme to use
    pub color_scheme: TerminalColorScheme,
    /// Maximum number of concurrent terminal instances
    pub max_instances: usize,
    /// Scrollback buffer size (lines)
    pub scrollback_lines: usize,
    /// Whether to enable bell/beep sounds
    pub bell_enabled: bool,
    /// Whether to enable cursor blinking
    pub cursor_blink: bool,
    /// Font family name
    pub font_family: String,
    /// Whether to enable bold text
    pub bold_enabled: bool,
    /// Whether to enable italic text
    pub italic_enabled: bool,
}

impl TerminalConfig {
    /// Create a new terminal configuration with sensible defaults
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            default_shell: "/bin/zsh".to_string(), // Default to zsh on macOS
            default_font_size: 14.0,
            default_grid_size: (80, 24), // Standard terminal size
            color_scheme: TerminalColorScheme::new(),
            max_instances: 8, // Reasonable limit for AR environment
            scrollback_lines: 1000,
            bell_enabled: false, // Disabled by default for AR
            cursor_blink: true,
            font_family: "Monaco".to_string(), // Good monospace font on macOS
            bold_enabled: true,
            italic_enabled: true,
        }
    }

    /// Create configuration for development with enhanced features
    #[inline(always)]
    pub fn development() -> Self {
        let mut config = Self::new();
        config.default_shell = "/bin/bash".to_string();
        config.scrollback_lines = 5000; // More scrollback for development
        config.color_scheme = TerminalColorScheme::dark_theme();
        config
    }

    /// Create configuration optimized for performance
    #[inline(always)]
    pub fn performance_optimized() -> Self {
        let mut config = Self::new();
        config.scrollback_lines = 500; // Less scrollback for performance
        config.bell_enabled = false;
        config.cursor_blink = false; // Disable blinking for performance
        config.bold_enabled = false; // Disable bold for performance
        config.italic_enabled = false; // Disable italic for performance
        config
    }

    /// Set color scheme variant
    #[inline(always)]
    pub fn with_color_scheme(mut self, variant: ColorSchemeVariant) -> Self {
        self.color_scheme = variant.to_color_scheme();
        self
    }

    /// Set default shell
    #[inline(always)]
    pub fn with_shell(mut self, shell: String) -> Self {
        self.default_shell = shell;
        self
    }

    /// Set font size
    #[inline(always)]
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.default_font_size = size.max(8.0).min(72.0);
        self
    }

    /// Set grid size
    #[inline(always)]
    pub fn with_grid_size(mut self, cols: usize, rows: usize) -> Self {
        self.default_grid_size = (cols.max(20), rows.max(5));
        self
    }

    /// Validate configuration settings
    #[inline(always)]
    pub fn validate(&self) -> TerminalResult<()> {
        if self.default_font_size < 8.0 || self.default_font_size > 72.0 {
            return Err(TerminalError::ConfigError("Font size must be between 8 and 72".to_string()));
        }
        if self.default_grid_size.0 < 20 || self.default_grid_size.1 < 5 {
            return Err(TerminalError::ConfigError("Grid size too small (minimum 20x5)".to_string()));
        }
        if self.max_instances == 0 {
            return Err(TerminalError::ConfigError("Max instances cannot be zero".to_string()));
        }
        if self.scrollback_lines > 10000 {
            return Err(TerminalError::ConfigError("Scrollback lines too large (maximum 10000)".to_string()));
        }
        Ok(())
    }
}

impl Default for TerminalConfig {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Global terminal state resource
#[derive(Resource, Debug)]
pub struct TerminalState {
    /// Number of active terminal instances
    pub active_instances: usize,
    /// Total memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Performance metrics
    pub performance_metrics: TerminalPerformanceMetrics,
    /// Whether the terminal system is initialized
    pub is_initialized: bool,
    /// Global command history (shared across instances)
    pub global_history: CommandHistory,
}

impl TerminalState {
    /// Create a new terminal state with default values
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            active_instances: 0,
            memory_usage_bytes: 0,
            performance_metrics: TerminalPerformanceMetrics::new(),
            is_initialized: false,
            global_history: CommandHistory::new(),
        }
    }

    /// Register a new terminal instance
    #[inline(always)]
    pub fn register_instance(&mut self) {
        self.active_instances = self.active_instances.saturating_add(1);
    }

    /// Unregister a terminal instance
    #[inline(always)]
    pub fn unregister_instance(&mut self) {
        self.active_instances = self.active_instances.saturating_sub(1);
    }

    /// Update memory usage
    #[inline(always)]
    pub fn update_memory_usage(&mut self, bytes: u64) {
        self.memory_usage_bytes = bytes;
    }

    /// Mark as initialized
    #[inline(always)]
    pub fn set_initialized(&mut self, initialized: bool) {
        self.is_initialized = initialized;
    }

    /// Get memory usage in megabytes
    #[inline(always)]
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

impl Default for TerminalState {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Terminal grid for storing character data
#[derive(Component, Debug, Clone)]
pub struct TerminalGrid {
    /// Grid dimensions (columns, rows)
    pub cols: usize,
    pub rows: usize,
    /// Character data stored as flat vector
    cells: Vec<TerminalCell>,
    /// Scrollback buffer
    scrollback: VecDeque<Vec<TerminalCell>>,
    /// Maximum scrollback lines
    max_scrollback: usize,
}

/// Single terminal cell containing character and styling
#[derive(Debug, Clone, Copy)]
pub struct TerminalCell {
    /// Character to display
    pub ch: char,
    /// Foreground color (ANSI color index or RGB)
    pub fg_color: CellColor,
    /// Background color (ANSI color index or RGB)
    pub bg_color: CellColor,
    /// Text attributes (bold, italic, underline, etc.)
    pub attributes: CellAttributes,
}

/// Color representation for terminal cells
#[derive(Debug, Clone, Copy)]
pub enum CellColor {
    /// ANSI color index (0-15)
    Ansi(AnsiColor),
    /// RGB color
    Rgb(u8, u8, u8),
    /// Default color (use scheme default)
    Default,
}

/// Text attributes for terminal cells
#[derive(Debug, Clone, Copy)]
pub struct CellAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub reverse: bool,
    pub blink: bool,
}

impl TerminalCell {
    /// Create a new empty terminal cell
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            ch: ' ',
            fg_color: CellColor::Default,
            bg_color: CellColor::Default,
            attributes: CellAttributes::new(),
        }
    }

    /// Create a terminal cell with character
    #[inline(always)]
    pub const fn with_char(ch: char) -> Self {
        Self {
            ch,
            fg_color: CellColor::Default,
            bg_color: CellColor::Default,
            attributes: CellAttributes::new(),
        }
    }

    /// Set character
    #[inline(always)]
    pub fn set_char(&mut self, ch: char) {
        self.ch = ch;
    }

    /// Set foreground color
    #[inline(always)]
    pub fn set_fg_color(&mut self, color: CellColor) {
        self.fg_color = color;
    }

    /// Set background color
    #[inline(always)]
    pub fn set_bg_color(&mut self, color: CellColor) {
        self.bg_color = color;
    }

    /// Set attributes
    #[inline(always)]
    pub fn set_attributes(&mut self, attributes: CellAttributes) {
        self.attributes = attributes;
    }

    /// Check if cell is empty (space with default colors)
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.ch == ' ' && 
        matches!(self.fg_color, CellColor::Default) &&
        matches!(self.bg_color, CellColor::Default) &&
        !self.attributes.has_any()
    }
}

impl Default for TerminalCell {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl CellAttributes {
    /// Create new attributes with all flags false
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            dim: false,
            reverse: false,
            blink: false,
        }
    }

    /// Check if any attribute is set
    #[inline(always)]
    pub const fn has_any(&self) -> bool {
        self.bold || self.italic || self.underline || self.strikethrough || 
        self.dim || self.reverse || self.blink
    }

    /// Reset all attributes
    #[inline(always)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for CellAttributes {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalGrid {
    /// Create a new terminal grid
    #[inline(always)]
    pub fn new(cols: usize, rows: usize) -> Self {
        let cell_count = cols * rows;
        let cells = vec![TerminalCell::new(); cell_count];
        
        Self {
            cols,
            rows,
            cells,
            scrollback: VecDeque::new(),
            max_scrollback: 1000,
        }
    }

    /// Get cell at position
    #[inline(always)]
    pub fn get_cell(&self, col: usize, row: usize) -> Option<&TerminalCell> {
        if col < self.cols && row < self.rows {
            let index = row * self.cols + col;
            self.cells.get(index)
        } else {
            None
        }
    }

    /// Get mutable cell at position
    #[inline(always)]
    pub fn get_cell_mut(&mut self, col: usize, row: usize) -> Option<&mut TerminalCell> {
        if col < self.cols && row < self.rows {
            let index = row * self.cols + col;
            self.cells.get_mut(index)
        } else {
            None
        }
    }

    /// Set cell at position
    #[inline(always)]
    pub fn set_cell(&mut self, col: usize, row: usize, cell: TerminalCell) {
        if let Some(target_cell) = self.get_cell_mut(col, row) {
            *target_cell = cell;
        }
    }

    /// Clear the entire grid
    #[inline(always)]
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = TerminalCell::new();
        }
    }

    /// Clear a specific row
    #[inline(always)]
    pub fn clear_row(&mut self, row: usize) {
        if row < self.rows {
            let start_index = row * self.cols;
            let end_index = start_index + self.cols;
            for cell in &mut self.cells[start_index..end_index] {
                *cell = TerminalCell::new();
            }
        }
    }

    /// Scroll up by one line (move content up, add new line at bottom)
    #[inline(always)]
    pub fn scroll_up(&mut self) {
        // Save top line to scrollback
        let top_line: Vec<TerminalCell> = self.cells[0..self.cols].to_vec();
        self.scrollback.push_back(top_line);
        
        // Trim scrollback if needed
        while self.scrollback.len() > self.max_scrollback {
            self.scrollback.pop_front();
        }
        
        // Move all lines up
        for row in 0..(self.rows - 1) {
            let src_start = (row + 1) * self.cols;
            let src_end = src_start + self.cols;
            let dst_start = row * self.cols;
            let _dst_end = dst_start + self.cols;
            
            self.cells.copy_within(src_start..src_end, dst_start);
        }
        
        // Clear bottom line
        self.clear_row(self.rows - 1);
    }

    /// Insert character at position, handling line wrapping
    #[inline(always)]
    pub fn insert_char(&mut self, col: usize, row: usize, ch: char) {
        if let Some(cell) = self.get_cell_mut(col, row) {
            cell.set_char(ch);
        }
    }

    /// Get a row as a string
    #[inline(always)]
    pub fn get_row_string(&self, row: usize) -> String {
        if row < self.rows {
            let start_index = row * self.cols;
            let end_index = start_index + self.cols;
            self.cells[start_index..end_index]
                .iter()
                .map(|cell| cell.ch)
                .collect()
        } else {
            String::new()
        }
    }

    /// Resize the grid
    #[inline(always)]
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        let new_cell_count = new_cols * new_rows;
        let mut new_cells = vec![TerminalCell::new(); new_cell_count];
        
        // Copy existing content
        let copy_cols = self.cols.min(new_cols);
        let copy_rows = self.rows.min(new_rows);
        
        for row in 0..copy_rows {
            let old_start = row * self.cols;
            let old_end = old_start + copy_cols;
            let new_start = row * new_cols;
            let new_end = new_start + copy_cols;
            
            new_cells[new_start..new_end].copy_from_slice(&self.cells[old_start..old_end]);
        }
        
        self.cols = new_cols;
        self.rows = new_rows;
        self.cells = new_cells;
    }

    /// Get scrollback line
    #[inline(always)]
    pub fn get_scrollback_line(&self, index: usize) -> Option<&Vec<TerminalCell>> {
        self.scrollback.get(index)
    }

    /// Get scrollback line count
    #[inline(always)]
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }
}

/// Command history management
#[derive(Debug, Clone)]
pub struct CommandHistory {
    /// History entries
    entries: VecDeque<String>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Current position in history
    current_position: usize,
}

impl CommandHistory {
    /// Create a new command history
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 1000,
            current_position: 0,
        }
    }

    /// Add a command to history
    #[inline(always)]
    pub fn add_command(&mut self, command: String) {
        if !command.trim().is_empty() {
            // Don't add duplicate consecutive commands
            if self.entries.back() != Some(&command) {
                self.entries.push_back(command);
                
                // Trim to max entries
                while self.entries.len() > self.max_entries {
                    self.entries.pop_front();
                }
            }
            
            // Reset position to end
            self.current_position = self.entries.len();
        }
    }

    /// Get previous command in history
    #[inline(always)]
    pub fn previous(&mut self) -> Option<&String> {
        if self.current_position > 0 {
            self.current_position -= 1;
            self.entries.get(self.current_position)
        } else {
            None
        }
    }

    /// Get next command in history
    #[inline(always)]
    pub fn next(&mut self) -> Option<&String> {
        if self.current_position < self.entries.len() {
            self.current_position += 1;
            if self.current_position < self.entries.len() {
                self.entries.get(self.current_position)
            } else {
                None // At end of history
            }
        } else {
            None
        }
    }

    /// Search history for commands containing text
    #[inline(always)]
    pub fn search(&self, text: &str) -> Vec<&String> {
        self.entries
            .iter()
            .filter(|cmd| cmd.contains(text))
            .collect()
    }

    /// Clear all history
    #[inline(always)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_position = 0;
    }
}

impl Default for CommandHistory {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics for terminal plugin
#[derive(Debug, Clone)]
pub struct TerminalPerformanceMetrics {
    /// Total number of commands executed
    pub commands_executed: u64,
    /// Average command execution time in milliseconds
    pub average_execution_time_ms: f32,
    /// Total rendering time in milliseconds
    pub total_render_time_ms: f64,
    /// Number of render frames
    pub render_frames: u64,
    /// Last frame render time in milliseconds
    pub last_frame_time_ms: f32,
    /// Characters processed per second
    pub chars_per_second: f32,
}

impl TerminalPerformanceMetrics {
    /// Create new performance metrics
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            commands_executed: 0,
            average_execution_time_ms: 0.0,
            total_render_time_ms: 0.0,
            render_frames: 0,
            last_frame_time_ms: 0.0,
            chars_per_second: 0.0,
        }
    }

    /// Record a command execution
    #[inline(always)]
    pub fn record_command_execution(&mut self, execution_time_ms: f32) {
        self.commands_executed = self.commands_executed.saturating_add(1);
        // Update running average
        let total_time = self.average_execution_time_ms * (self.commands_executed - 1) as f32 + execution_time_ms;
        self.average_execution_time_ms = total_time / self.commands_executed as f32;
    }

    /// Record a render frame
    #[inline(always)]
    pub fn record_render_frame(&mut self, frame_time_ms: f32) {
        self.render_frames = self.render_frames.saturating_add(1);
        self.total_render_time_ms += frame_time_ms as f64;
        self.last_frame_time_ms = frame_time_ms;
    }

    /// Update character processing rate
    #[inline(always)]
    pub fn update_char_rate(&mut self, chars_processed: usize, time_delta: f32) {
        if time_delta > 0.0 {
            self.chars_per_second = chars_processed as f32 / time_delta;
        }
    }

    /// Get average frame time in milliseconds
    #[inline(always)]
    pub fn average_frame_time_ms(&self) -> f32 {
        if self.render_frames > 0 {
            (self.total_render_time_ms / self.render_frames as f64) as f32
        } else {
            0.0
        }
    }

    /// Get frames per second
    #[inline(always)]
    pub fn fps(&self) -> f32 {
        let avg_frame_time = self.average_frame_time_ms();
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }
}

impl Default for TerminalPerformanceMetrics {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}