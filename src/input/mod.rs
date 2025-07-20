//! Input handling system for XREAL Bevy integration
//!
//! This module provides input handling for XREAL glasses, including:
//! - Head tracking input
//! - Gaze-based interaction
//! - Mouse and keyboard emulation
//! - Input configuration and state management

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy::ecs::schedule::SystemSet;
use bevy::log::warn;

// Re-export commonly used types
pub use cursor::CursorState;
// pub use error::InputError; // Currently unused
pub use plugins::input::InputSystem;
pub use render::VirtualScreen;
pub use tracking::Orientation;

// Internal modules
mod cursor;
mod error;
pub mod plugins;
mod render;
mod tracking;

/// Configuration for input system
#[derive(Resource, Clone, Debug)]
pub struct InputConfig {
    /// Minimum time between input events in milliseconds (for rate limiting)
    pub min_event_interval_ms: u64,
    /// Enable/disable input processing
    pub enabled: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            min_event_interval_ms: 16, // ~60fps by default
            enabled: true,
        }
    }
}

/// System set for input-related systems
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct InputSystemSet;

/// Plugin that sets up all input-related systems and resources
pub struct InputSystemPlugin;

impl Plugin for InputSystemPlugin {
    fn build(&self, app: &mut App) {
        // Configure the input system set to run in the Update schedule
        app.configure_sets(
            Update,
            InputSystemSet.after(bevy::transform::TransformSystem::TransformPropagate), // After transforms are updated
        );

        app
            // Add our plugin modules
            .add_plugins((
                cursor::CursorPlugin,
                render::VirtualScreenPlugin,
                tracking::OrientationPlugin,
                plugins::input::InputPlugin,
            ))
            // Add our input configuration
            .init_resource::<InputConfig>()
            // Add our main input handling system with proper error handling
            .add_systems(Update, handle_input);
    }
}

/// Ray structure for raycasting
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Find the intersection point with a plane
    pub fn intersect_plane(&self, plane_origin: Vec3, plane_normal: Vec3) -> Option<Vec3> {
        let denom = plane_normal.dot(self.direction);
        if denom.abs() > 1e-6 {
            let t = (plane_origin - self.origin).dot(plane_normal) / denom;
            if t >= 0.0 {
                return Some(self.origin + self.direction * t);
            }
        }
        None
    }
}

/// Main input handling system
///
/// This system processes input from various sources (head tracking, gaze, keyboard, etc.)
/// and converts it into appropriate input events and state changes.
pub fn handle_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    _orientation: Res<Orientation>,
    mut cursor_state: ResMut<CursorState>,
    query_plane: Query<(&GlobalTransform, &VirtualScreen), With<VirtualScreen>>,
    window: Query<&Window, With<PrimaryWindow>>,
    // Use a system parameter that allows mutable access to the non-send resource
    mut input_system: NonSendMut<InputSystem>,
) {
    // Update cursor state based on time
    cursor_state.update(time.delta().as_secs_f32());

    // Check if we should trigger a click (space bar or dwell time complete)
    let should_trigger = keys.just_pressed(KeyCode::Space) || cursor_state.is_dwell_complete();

    // If we have a virtual screen, update cursor position based on head tracking
    if let Ok((transform, screen)) = query_plane.single() {
        // Create a ray from the head position in the direction of the head orientation
        let ray = Ray {
            origin: transform.translation(),
            direction: transform.forward().normalize(),
        };

        // Check for intersection with the virtual screen
        let plane_normal = transform.forward().normalize();
        if let Some(intersection) = ray.intersect_plane(transform.translation(), plane_normal) {
            // Convert 3D intersection to 2D screen coordinates
            let local_pos = transform
                .compute_matrix()
                .inverse()
                .transform_point3(intersection);
            if let Ok(window) = window.single() {
                let screen_pos = Vec2::new(
                    (local_pos.x / screen.size.x + 0.5) * window.resolution.width(),
                    (0.5 - local_pos.y / screen.size.y) * window.resolution.height(),
                );

                // Update cursor position
                cursor_state.position = screen_pos;

                // Move the system cursor to match
                // Move the system cursor to match
                // We can directly use the mutable reference from NonSendMut
                if let Err(e) = input_system.move_mouse(screen_pos.x as i32, screen_pos.y as i32) {
                    warn!("Failed to move cursor: {}", e);
                }

                // Start or continue dwell timer if cursor is over a target
                if should_trigger {
                    cursor_state.start_dwell();

                    // If we're triggering a click, do it now
                    if cursor_state.is_dwell_complete() {
                        if let Err(e) = input_system.click(enigo::Button::Left) {
                            warn!("Failed to trigger click: {}", e);
                        }

                        cursor_state.reset_dwell();
                    }
                } else {
                    cursor_state.reset_dwell();
                }
            }
        }
    }
}

// Re-export for external use
pub mod prelude {
    // All imports are currently unused and commented out
    // pub use super::{
    //     cursor::CursorState,
    //     error::InputError,
    //     plugins::input::InputSystem,
    // };
}
