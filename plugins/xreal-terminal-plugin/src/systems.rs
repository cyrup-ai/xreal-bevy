//! Terminal plugin systems for Bevy ECS
//!
//! This module implements all systems used by the terminal plugin for updating,
//! rendering, input handling, and lifecycle management within the Bevy ECS architecture.

use bevy::{
    prelude::*,
    input::keyboard::{KeyboardInput, Key},
    input::mouse::{MouseButtonInput, MouseWheel},
    window::CursorMoved,
    render::{
        render_resource::{BufferDescriptor, BufferUsages},
        renderer::{RenderDevice, RenderQueue},
    },
};
use tracing::{info, error};
use crate::{
    components::*,
    resources::*,
    color_scheme::*,
};

/// System to initialize terminal plugin resources
pub fn initialize_terminal_system(
    _commands: Commands,
    mut terminal_state: ResMut<TerminalState>,
    config: Res<TerminalConfig>,
) {
    if !terminal_state.is_initialized {
        info!("Initializing terminal plugin system");
        
        // Validate configuration
        if let Err(e) = config.validate() {
            error!("Terminal configuration validation failed: {}", e);
            return;
        }
        
        terminal_state.set_initialized(true);
        info!("Terminal plugin system initialized successfully");
    }
}

/// System to update terminal entities and their state
pub fn update_terminal_system(
    time: Res<Time>,
    mut query: Query<(&mut TerminalEntity, &mut TerminalCursor, &mut TerminalScrollback)>,
    mut terminal_state: ResMut<TerminalState>,
    config: Res<TerminalConfig>,
) {
    let delta_time = time.delta().as_secs_f32();
    
    for (entity, mut cursor, _) in query.iter_mut() {
        // Update cursor blinking if enabled
        if config.cursor_blink {
            cursor.update_blink(delta_time);
        }
        
        // Update performance metrics
        terminal_state.performance_metrics.record_render_frame(delta_time * 1000.0);
        
        // Handle terminal lifecycle
        if entity.is_running && !entity.is_active {
            // Terminal is running but not active, continue processing
            continue;
        }
    }
}

/// System to handle terminal input processing
pub fn process_terminal_input_system(
    mut keyboard_input: EventReader<KeyboardInput>,
    mut mouse_input: EventReader<MouseButtonInput>,
    mut cursor_moved: EventReader<CursorMoved>,
    mut query: Query<(&TerminalEntity, &mut TerminalInput, &mut TerminalGrid), With<TerminalEntity>>,
    _terminal_state: ResMut<TerminalState>,
    _config: Res<TerminalConfig>,
) {
    // Process keyboard input and character input (combined in Bevy 0.16.1)
    for input in keyboard_input.read() {
        if input.state.is_pressed() {
            for (entity, mut terminal_input, _grid) in query.iter_mut() {
                if !entity.is_active || !terminal_input.accepts_keyboard {
                    continue;
                }
                
                // Handle character input from logical key
                if let Key::Character(ref character) = input.logical_key {
                    for ch in character.chars() {
                        // Skip control characters except specific ones
                        if !ch.is_control() || matches!(ch, '\n' | '\r' | '\t' | '\x08') {
                            terminal_input.add_char(ch);
                        }
                    }
                }
                
                // Handle special key combinations
                let modifiers = KeyboardModifiers {
                    ctrl: false, // TODO: Get actual modifier state from input
                    shift: false,
                    alt: false,
                    meta: false,
                };
                
                if let Some(sequence) = terminal_input.handle_key_combination(input.key_code, &modifiers) {
                    terminal_input.add_string(&sequence);
                }
                
                terminal_input.add_key(input.key_code);
            }
        } else {
            for (_, mut terminal_input, _) in query.iter_mut() {
                terminal_input.remove_key(input.key_code);
            }
        }
    }
    
    // Process mouse input
    for mouse_event in mouse_input.read() {
        for (entity, mut terminal_input, _) in query.iter_mut() {
            if entity.is_active && terminal_input.accepts_mouse {
                terminal_input.set_mouse_button(mouse_event.button, mouse_event.state.is_pressed());
            }
        }
    }
    
    // Process cursor movement
    for cursor_event in cursor_moved.read() {
        for (entity, mut terminal_input, _) in query.iter_mut() {
            if entity.is_active && terminal_input.accepts_mouse {
                // Convert screen coordinates to grid coordinates
                let (_viewport_width, _viewport_height) = entity.calculate_viewport_size();
                let char_width = entity.font_size * 0.6;
                let char_height = entity.font_size * 1.2;
                
                let grid_col = (cursor_event.position.x / char_width) as usize;
                let grid_row = (cursor_event.position.y / char_height) as usize;
                
                terminal_input.update_mouse_grid_position(
                    grid_col.min(entity.grid_size.0.saturating_sub(1)),
                    grid_row.min(entity.grid_size.1.saturating_sub(1))
                );
            }
        }
    }
}

/// System to render terminal content
pub fn render_terminal_system(
    mut query: Query<(&TerminalEntity, &mut TerminalSurface, &TerminalGrid, &TerminalCursor)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    config: Res<TerminalConfig>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();
    
    for (entity, mut surface, _grid, _cursor) in query.iter_mut() {
        if !entity.is_active {
            continue;
        }
        
        // Skip rendering if surface doesn't need update
        if !surface.needs_update {
            continue;
        }
        
        // Create or update vertex buffer for text rendering
        if surface.vertex_buffer.is_none() {
            let vertex_data = create_terminal_vertices(entity, _grid, _cursor, &config.color_scheme);
            
            let buffer_descriptor = BufferDescriptor {
                label: Some("Terminal Vertex Buffer"),
                size: vertex_data.len() as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };
            
            let buffer = render_device.create_buffer(&buffer_descriptor);
            render_queue.write_buffer(&buffer, 0, &vertex_data);
            surface.vertex_buffer = Some(buffer);
        }
        
        // Update render time and clear dirty flag
        surface.update_render_time((time.elapsed().as_secs_f32() - current_time) * 1000.0);
        surface.clear_dirty();
    }
}

/// System to handle terminal scrolling
pub fn handle_terminal_scroll_system(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&TerminalEntity, &mut TerminalScrollback), With<TerminalEntity>>,
) {
    for scroll_event in scroll_events.read() {
        for (entity, mut scrollback) in query.iter_mut() {
            if !entity.is_active {
                continue;
            }
            
            // In Bevy 0.16.1, MouseWheel events provide scroll values directly
            // Treat as line-based scrolling by default
            if scroll_event.y > 0.0 {
                scrollback.scroll_up(scroll_event.y as usize);
            } else if scroll_event.y < 0.0 {
                scrollback.scroll_down((-scroll_event.y) as usize);
            }
        }
    }
}

/// System to manage terminal lifecycle (creation, destruction)
pub fn manage_terminal_lifecycle_system(
    mut commands: Commands,
    mut terminal_state: ResMut<TerminalState>,
    _config: Res<TerminalConfig>,
    query: Query<(Entity, &TerminalEntity), With<TerminalEntity>>,
) {
    // Count active instances
    let active_count = query.iter().filter(|(_, entity)| entity.is_running).count();
    terminal_state.active_instances = active_count;
    
    // Clean up inactive terminals
    for (entity_id, terminal_entity) in query.iter() {
        if !terminal_entity.is_running && !terminal_entity.is_active {
            info!("Cleaning up inactive terminal: {}", terminal_entity.id);
            commands.entity(entity_id).despawn();
            terminal_state.unregister_instance();
        }
    }
}

pub fn process_terminal_commands_system(
    mut query: Query<(&mut TerminalEntity, &mut TerminalInput, &mut TerminalGrid), With<TerminalEntity>>,
    mut terminal_state: ResMut<TerminalState>,
) {
    for (entity, mut input, mut grid) in query.iter_mut() {
        if !entity.is_running {
            continue;
        }
        
        // Process any pending input  
        let input_text = input.drain_input();
        if !input_text.is_empty() {
            // Add to command history
            terminal_state.global_history.add_command(input_text.clone());
            
            // Process the command (simplified for now)
            process_terminal_command(&mut grid, &input_text);
            
            // Record command execution
            terminal_state.performance_metrics.record_command_execution(1.0); // Placeholder timing
        }
    }
}

/// System to update terminal performance metrics
pub fn update_terminal_performance_system(
    mut terminal_state: ResMut<TerminalState>,
    query: Query<&TerminalEntity, With<TerminalEntity>>,
    _time: Res<Time>,
) {
    // Update memory usage estimation
    let memory_per_terminal = 1024 * 1024; // 1MB per terminal (rough estimate)
    let total_memory = query.iter().count() as u64 * memory_per_terminal;
    terminal_state.update_memory_usage(total_memory);
    
    // Update character processing rate (placeholder implementation)
    let chars_processed = 100; // Placeholder
    terminal_state.performance_metrics.update_char_rate(chars_processed, 1.0); // Using fixed delta for now
}

/// System to handle terminal cleanup on shutdown
pub fn cleanup_terminal_system(
    mut commands: Commands,
    query: Query<Entity, With<TerminalEntity>>,
    mut terminal_state: ResMut<TerminalState>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    
    terminal_state.active_instances = 0;
    terminal_state.set_initialized(false);
    info!("Terminal plugin cleanup completed");
}

// Helper functions

/// Create vertex data for terminal rendering
fn create_terminal_vertices(
    entity: &TerminalEntity,
    _grid: &TerminalGrid,
    _cursor: &TerminalCursor,
    _color_scheme: &TerminalColorScheme,
) -> Vec<u8> {
    // Simplified vertex creation - in a real implementation, this would
    // generate proper vertex data for text rendering
    let vertex_count = entity.grid_size.0 * entity.grid_size.1 * 6; // 6 vertices per character quad
    let vertex_size = 32; // Position (12) + UV (8) + Color (12) bytes per vertex
    
    vec![0u8; vertex_count * vertex_size]
}

/// Process a terminal command (simplified implementation)
fn process_terminal_command(grid: &mut TerminalGrid, command: &str) {
    // This is a simplified implementation - in reality, this would interface
    // with a PTY and handle ANSI escape sequences
    
    if command.trim().is_empty() {
        return;
    }
    
    // Find the next available row for output
    let output_row = 0; // Simplified - would track cursor position
    
    // Insert command output into grid
    for (i, ch) in command.chars().enumerate() {
        if i < grid.cols {
            grid.insert_char(i, output_row, ch);
        }
    }
    
    // Scroll if needed
    grid.scroll_up();
}

