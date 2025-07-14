use bevy::prelude::*;
use enigo::{Enigo, Settings, Mouse, Button, Direction, Coordinate};
use crate::render::VirtualScreen;

#[inline]
pub fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    query_camera: Query<&Transform, With<Camera>>,
    query_plane: Query<(&Transform, &VirtualScreen)>,
    windows: Query<&Window>,
) {
    if !keys.just_pressed(KeyCode::Space) { return; }
    
    let camera_transform = match query_camera.single() {
        Ok(transform) => transform,
        Err(_) => return,
    };
    
    let mut hit = None;
    let ray_origin = camera_transform.translation;
    let ray_dir = camera_transform.forward();
    
    // Optimized raycasting with early exit
    for (plane_transform, virtual_screen) in &query_plane {
        let plane_normal = *plane_transform.forward();
        let denom = ray_dir.dot(plane_normal);
        
        if denom.abs() <= 0.0001 { continue; }
        
        let t = (plane_transform.translation - ray_origin).dot(plane_normal) / denom;
        if t <= 0.0 { continue; }
        
        let hit_point = ray_origin + t * ray_dir;
        let local_hit = plane_transform.compute_matrix().inverse() * hit_point.extend(1.0);
        let u = (local_hit.x + 1.0) * 0.5;
        let v = (local_hit.y + 1.0) * 0.5;
        
        if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 {
            hit = Some((virtual_screen.0, u, v));
            break;
        }
    }

    if let Some((_screen_index, u, v)) = hit {
        let window = match windows.single() {
            Ok(window) => window,
            Err(_) => return,
        };
        
        let screen_x = (window.resolution.width() * u) as i32;
        let screen_y = (window.resolution.height() * (1.0 - v)) as i32;

        // Optimized mouse control with error handling
        match Enigo::new(&Settings::default()) {
            Ok(mut enigo) => {
                if let Err(e) = enigo.move_mouse(screen_x, screen_y, Coordinate::Abs) {
                    warn!("Failed to move mouse: {:?}", e);
                } else if let Err(e) = enigo.button(Button::Left, Direction::Press) {
                    warn!("Failed to press mouse button: {:?}", e);
                } else if let Err(e) = enigo.button(Button::Left, Direction::Release) {
                    warn!("Failed to release mouse button: {:?}", e);
                }
            }
            Err(e) => {
                warn!("Failed to initialize enigo for mouse control: {:?}", e);
            }
        }
    }
}