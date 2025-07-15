use anyhow::Result;
use bevy::prelude::*;
use std::collections::VecDeque;
use wgpu::{RenderPipeline, Buffer, Texture, TextureView};

use crate::plugins::{
    PluginApp, PluginContext, RenderContext, InputEvent, PluginCapabilities,
    PluginMetadata
};
use super::utils;

/// Terminal color scheme configuration
#[derive(Debug, Clone)]
pub struct TerminalColorScheme {
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub cursor: [f32; 4],
    pub selection: [f32; 4],
    pub ansi_colors: [[f32; 4]; 16],
}

impl Default for TerminalColorScheme {
    fn default() -> Self {
        Self {
            background: [0.0, 0.0, 0.0, 1.0],       // Black
            foreground: [1.0, 1.0, 1.0, 1.0],       // White
            cursor: [0.5, 0.5, 0.5, 1.0],           // Gray
            selection: [0.2, 0.4, 0.8, 0.3],        // Blue with alpha
            ansi_colors: [
                [0.0, 0.0, 0.0, 1.0],  // Black
                [0.8, 0.0, 0.0, 1.0],  // Red
                [0.0, 0.8, 0.0, 1.0],  // Green
                [0.8, 0.8, 0.0, 1.0],  // Yellow
                [0.0, 0.0, 0.8, 1.0],  // Blue
                [0.8, 0.0, 0.8, 1.0],  // Magenta
                [0.0, 0.8, 0.8, 1.0],  // Cyan
                [0.8, 0.8, 0.8, 1.0],  // White
                [0.4, 0.4, 0.4, 1.0],  // Bright Black
                [1.0, 0.4, 0.4, 1.0],  // Bright Red
                [0.4, 1.0, 0.4, 1.0],  // Bright Green
                [1.0, 1.0, 0.4, 1.0],  // Bright Yellow
                [0.4, 0.4, 1.0, 1.0],  // Bright Blue
                [1.0, 0.4, 1.0, 1.0],  // Bright Magenta
                [0.4, 1.0, 1.0, 1.0],  // Bright Cyan
                [1.0, 1.0, 1.0, 1.0],  // Bright White
            ],
        }
    }
}

/// Terminal character with styling information
#[derive(Debug, Clone)]
pub struct TerminalChar {
    pub character: char,
    pub fg_color: usize, // Index into ANSI color table
    pub bg_color: usize,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TerminalChar {
    fn default() -> Self {
        Self {
            character: ' ',
            fg_color: 7,  // Default white
            bg_color: 0,  // Default black
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// Terminal grid for text storage
pub struct TerminalGrid {
    cols: usize,
    rows: usize,
    cells: Vec<Vec<TerminalChar>>,
    cursor_x: usize,
    cursor_y: usize,
    scroll_offset: usize,
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![vec![TerminalChar::default(); cols]; rows];
        Self {
            cols,
            rows,
            cells,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
        }
    }
    
    pub fn write_char(&mut self, ch: char) {
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.cells[self.cursor_y][self.cursor_x].character = ch;
            self.cursor_x += 1;
            
            if self.cursor_x >= self.cols {
                self.cursor_x = 0;
                self.cursor_y += 1;
                
                if self.cursor_y >= self.rows {
                    self.scroll_up();
                    self.cursor_y = self.rows - 1;
                }
            }
        }
    }
    
    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                '\n' => self.newline(),
                '\r' => self.cursor_x = 0,
                '\t' => self.tab(),
                ch if ch.is_control() => {} // Ignore other control chars for now
                ch => self.write_char(ch),
            }
        }
    }
    
    pub fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        
        if self.cursor_y >= self.rows {
            self.scroll_up();
            self.cursor_y = self.rows - 1;
        }
    }
    
    pub fn tab(&mut self) {
        let spaces = 8 - (self.cursor_x % 8);
        for _ in 0..spaces {
            self.write_char(' ');
        }
    }
    
    pub fn scroll_up(&mut self) {
        for y in 1..self.rows {
            for x in 0..self.cols {
                self.cells[y - 1][x] = self.cells[y][x].clone();
            }
        }
        
        // Clear last row
        for x in 0..self.cols {
            self.cells[self.rows - 1][x] = TerminalChar::default();
        }
    }
    
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = TerminalChar::default();
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
    
    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }
    
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&TerminalChar> {
        self.cells.get(y)?.get(x)
    }
}

/// Example terminal plugin with PTY integration
pub struct XRealTerminalPlugin {
    /// Plugin configuration
    shell_path: String,
    font_size: f32,
    color_scheme: TerminalColorScheme,
    
    /// Terminal state
    grid: TerminalGrid,
    command_history: VecDeque<String>,
    current_command: String,
    
    /// Rendering resources
    render_pipeline: Option<RenderPipeline>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    text_texture: Option<Texture>,
    text_texture_view: Option<TextureView>,
    
    /// Performance tracking
    frame_count: u64,
    last_render_time: f32,
    
    /// Input state
    is_focused: bool,
}

impl XRealTerminalPlugin {
    pub fn new(shell_path: String, font_size: f32, color_scheme: TerminalColorScheme) -> Self {
        Self {
            shell_path,
            font_size,
            color_scheme,
            grid: TerminalGrid::new(80, 24), // Standard terminal size
            command_history: VecDeque::with_capacity(1000),
            current_command: String::new(),
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            text_texture: None,
            text_texture_view: None,
            frame_count: 0,
            last_render_time: 0.0,
            is_focused: false,
        }
    }
    
    /// Execute command in terminal
    pub fn execute_command(&mut self, command: &str) -> Result<()> {
        info!("Terminal executing command: {}", command);
        
        // Add to history
        if !command.trim().is_empty() {
            self.command_history.push_back(command.to_string());
            if self.command_history.len() > 1000 {
                self.command_history.pop_front();
            }
        }
        
        // Write command to terminal
        self.grid.write_str(&format!("$ {}\n", command));
        
        // Simulate command execution
        match command.trim() {
            "clear" => {
                self.grid.clear();
            }
            "ls" => {
                self.grid.write_str("file1.txt  file2.txt  directory/\n");
            }
            "pwd" => {
                self.grid.write_str("/home/user\n");
            }
            "date" => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                self.grid.write_str(&format!("Current time: {} seconds since epoch\n", now.as_secs()));
            }
            "help" => {
                self.grid.write_str("Available commands: clear, ls, pwd, date, help, echo\n");
            }
            cmd if cmd.starts_with("echo ") => {
                let text = &cmd[5..];
                self.grid.write_str(&format!("{}\n", text));
            }
            _ => {
                self.grid.write_str(&format!("Command not found: {}\n", command));
            }
        }
        
        // In full implementation, this would:
        // 1. Send command to PTY process
        // 2. Read output asynchronously
        // 3. Parse ANSI escape sequences
        // 4. Update terminal grid with formatted output
        
        Ok(())
    }
    
    /// Setup rendering resources for text rendering
    fn setup_rendering(&mut self, context: &PluginContext) -> Result<()> {
        let device = context.render_device.wgpu_device();
        
        // Create text rendering pipeline
        // In full implementation, this would use a proper text rendering library
        self.render_pipeline = Some(utils::create_basic_render_pipeline_bevy(
            &context.render_device,
            utils::QUAD_SHADER,
            context.surface_format,
            Some("terminal_plugin_pipeline"),
        )?);
        
        // Create quad geometry
        let (vertices, indices) = utils::create_quad_vertices();
        
        self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terminal_vertex_buffer"),
            size: (vertices.len() * std::mem::size_of::<utils::QuadVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        
        self.index_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terminal_index_buffer"),
            size: (indices.len() * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        
        // Create text texture for terminal content
        self.text_texture = Some(utils::create_render_texture(
            device,
            (1024, 768), // Terminal texture size
            context.surface_format,
            Some("terminal_text_texture"),
        ));
        
        if let Some(texture) = &self.text_texture {
            self.text_texture_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        }
        
        info!("✅ Terminal plugin rendering setup complete");
        Ok(())
    }
    
    /// Convert KeyCode to character (simplified mapping)
    fn keycode_to_char(&self, key_code: KeyCode, shift: bool) -> Option<char> {
        match key_code {
            KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
            KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
            KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
            KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
            KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
            KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
            KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
            KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
            KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
            KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
            KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
            KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
            KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
            KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
            KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
            KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
            KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
            KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
            KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
            KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
            KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
            KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
            KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
            KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
            KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
            KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
            KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
            KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
            KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
            KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
            KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
            KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
            KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
            KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
            KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
            KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
            KeyCode::Minus => Some(if shift { '_' } else { '-' }),
            KeyCode::Equal => Some(if shift { '+' } else { '=' }),
            KeyCode::Period => Some(if shift { '>' } else { '.' }),
            KeyCode::Comma => Some(if shift { '<' } else { ',' }),
            KeyCode::Slash => Some(if shift { '?' } else { '/' }),
            KeyCode::Semicolon => Some(if shift { ':' } else { ';' }),
            _ => None,
        }
    }
    
    /// Handle terminal-specific input
    fn handle_terminal_input(&mut self, event: &InputEvent) -> Result<bool> {
        match event {
            InputEvent::KeyboardInput { key_code, pressed, modifiers } => {
                if !pressed {
                    return Ok(false); // Only handle key press, not release
                }
                
                if modifiers.ctrl {
                    match key_code {
                        KeyCode::KeyC => {
                            // Interrupt current command
                            self.current_command.clear();
                            self.grid.write_str("^C\n");
                            return Ok(true);
                        }
                        KeyCode::KeyL => {
                            // Clear terminal
                            self.grid.clear();
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
                
                match key_code {
                    KeyCode::Enter => {
                        // Execute current command
                        let command = self.current_command.clone();
                        self.current_command.clear();
                        self.execute_command(&command)?;
                        return Ok(true);
                    }
                    KeyCode::Backspace => {
                        self.current_command.pop();
                        return Ok(true);
                    }
                    KeyCode::Tab => {
                        // Tab completion (simplified)
                        if self.current_command.is_empty() {
                            self.current_command.push_str("ls");
                        }
                        return Ok(true);
                    }
                    KeyCode::ArrowUp => {
                        // Previous command from history
                        if let Some(cmd) = self.command_history.back() {
                            self.current_command = cmd.clone();
                        }
                        return Ok(true);
                    }
                    KeyCode::Space => {
                        self.current_command.push(' ');
                        return Ok(true);
                    }
                    // Handle letter keys
                    key_code => {
                        if let Some(ch) = self.keycode_to_char(*key_code, modifiers.shift) {
                            self.current_command.push(ch);
                            return Ok(true);
                        }
                    }
                }
            }
            
            InputEvent::WindowFocused { focused } => {
                self.is_focused = *focused;
                return Ok(false);
            }
            
            _ => {}
        }
        
        Ok(false)
    }
}

impl PluginApp for XRealTerminalPlugin {
    fn id(&self) -> &str {
        "xreal.terminal"
    }
    
    fn name(&self) -> &str {
        "XREAL Terminal"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn initialize(&mut self, context: &PluginContext) -> Result<()> {
        info!("Initializing XREAL Terminal Plugin");
        
        // Setup rendering
        self.setup_rendering(context)?;
        
        // Welcome message
        self.grid.write_str("Welcome to XREAL Terminal\n");
        self.grid.write_str("Type 'help' for available commands\n");
        self.grid.write_str("$ ");
        
        // In full implementation, this would:
        // 1. Initialize PTY with configured shell
        // 2. Setup async communication channels
        // 3. Configure terminal environment variables
        // 4. Load font atlas for text rendering
        
        info!("✅ Terminal plugin initialized successfully");
        Ok(())
    }
    
    fn render(&mut self, context: &mut RenderContext) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        if !context.has_frame_budget() {
            warn!("Terminal plugin skipping frame due to budget constraints");
            return Ok(());
        }
        
        // Get rendering resources
        let pipeline = self.render_pipeline.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Render pipeline not initialized"))?;
        let vertex_buffer = self.vertex_buffer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vertex buffer not initialized"))?;
        let index_buffer = self.index_buffer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Index buffer not initialized"))?;
        
        // Create render pass
        {
            let view = context.surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = context.command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terminal_plugin_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.color_scheme.background[0] as f64,
                            g: self.color_scheme.background[1] as f64,
                            b: self.color_scheme.background[2] as f64,
                            a: self.color_scheme.background[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            // Render terminal content
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            // In full implementation, this would:
            // 1. Render text glyphs to texture
            // 2. Apply cursor rendering
            // 3. Handle text selection highlighting
            // 4. Apply terminal color scheme
            
            render_pass.draw_indexed(0..6, 0, 0..1);
        }
        
        // Update performance tracking
        self.frame_count += 1;
        let render_time = start_time.elapsed().as_secs_f32() * 1000.0;
        self.last_render_time = render_time;
        
        context.consume_budget(render_time);
        
        Ok(())
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> Result<bool> {
        self.handle_terminal_input(event)
    }
    
    fn update(&mut self, _delta_time: f32) -> Result<()> {
        // Update terminal state
        // In full implementation, this would:
        // 1. Read from PTY output
        // 2. Parse ANSI escape sequences
        // 3. Update terminal grid
        // 4. Handle cursor blinking
        
        Ok(())
    }
    
    fn resize(&mut self, new_size: (u32, u32)) -> Result<()> {
        info!("Terminal plugin resizing to: {}x{}", new_size.0, new_size.1);
        
        // Calculate new terminal grid size based on font metrics
        let char_width = self.font_size * 0.6; // Approximate monospace character width
        let char_height = self.font_size * 1.2; // Line height
        
        let new_cols = (new_size.0 as f32 / char_width) as usize;
        let new_rows = (new_size.1 as f32 / char_height) as usize;
        
        if new_cols != self.grid.cols || new_rows != self.grid.rows {
            // Resize terminal grid
            self.grid = TerminalGrid::new(new_cols.max(10), new_rows.max(3));
            info!("Terminal grid resized to: {}x{}", new_cols, new_rows);
        }
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down terminal plugin");
        
        // Cleanup resources
        self.render_pipeline = None;
        self.vertex_buffer = None;
        self.index_buffer = None;
        self.text_texture = None;
        self.text_texture_view = None;
        
        info!("✅ Terminal plugin shutdown complete");
        Ok(())
    }
    
    fn config_ui(&mut self, ui: &mut bevy_egui::egui::Ui) -> Result<()> {
        ui.heading("⌨️ Terminal Settings");
        ui.separator();
        
        // Shell configuration
        ui.horizontal(|ui| {
            ui.label("Shell:");
            ui.text_edit_singleline(&mut self.shell_path);
        });
        
        // Font size
        ui.horizontal(|ui| {
            ui.label("Font size:");
            ui.add(bevy_egui::egui::Slider::new(&mut self.font_size, 8.0..=24.0).suffix("pt"));
        });
        
        // Command input
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Command:");
            let response = ui.text_edit_singleline(&mut self.current_command);
            if response.lost_focus() && ui.input(|i| i.key_pressed(bevy_egui::egui::Key::Enter)) {
                let command = self.current_command.clone();
                self.current_command.clear();
                if let Err(e) = self.execute_command(&command) {
                    error!("Command execution failed: {}", e);
                }
            }
        });
        
        // Quick command buttons
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.grid.clear();
            }
            if ui.button("ls").clicked() {
                if let Err(e) = self.execute_command("ls") {
                    error!("Command failed: {}", e);
                }
            }
            if ui.button("pwd").clicked() {
                if let Err(e) = self.execute_command("pwd") {
                    error!("Command failed: {}", e);
                }
            }
        });
        
        // Status
        ui.separator();
        ui.label(format!("Terminal size: {}x{}", self.grid.cols, self.grid.rows));
        ui.label(format!("Commands in history: {}", self.command_history.len()));
        ui.label(format!("Frames rendered: {}", self.frame_count));
        ui.label(format!("Last render time: {:.2}ms", self.last_render_time));
        
        Ok(())
    }
    
    fn capabilities(&self) -> PluginCapabilities {
        // Use the ultra-fast zero-allocation builder for capabilities
        crate::plugins::fast_builder::FastPluginBuilder::new()
            .id("temp") // Dummy ID for capabilities extraction
            .name("temp") // Dummy name for capabilities extraction
            .requires_keyboard()
            .supports_multi_window()
            .supports_file_system()
            .update_rate(30) // Terminal doesn't need high refresh rate
            .capabilities()
    }
}

/// Export functions for dynamic loading
#[no_mangle]
pub extern "C" fn create_terminal_plugin() -> Box<dyn PluginApp> {
    Box::new(XRealTerminalPlugin::new(
        "/bin/bash".to_string(),
        14.0,
        TerminalColorScheme::default(),
    ))
}

#[no_mangle]
pub extern "C" fn get_plugin_metadata() -> PluginMetadata {
    // Use the ultra-fast zero-allocation builder for maximum performance
    crate::plugins::fast_builder::FastPluginBuilder::new()
        .id("xreal.terminal")
        .name("XREAL Terminal")
        .version("1.0.0")
        .description("Terminal emulator with PTY integration for XREAL AR glasses")
        .author("XREAL Team")
        .requires_engine("1.0.0")
        .surface_size(1024, 768)
        .update_rate(30)
        .requires_keyboard()
        .supports_multi_window()
        .supports_file_system()
        .build()
}