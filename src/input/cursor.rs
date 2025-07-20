use bevy::prelude::*;

/// Tracks the state of the cursor for gaze-based interaction
#[derive(Resource, Default)]
pub struct CursorState {
    /// Current position of the cursor in normalized screen coordinates (0..1, 0..1)
    pub position: Vec2,
    /// Current dwell time (how long the cursor has been on a target)
    pub dwell_time: f32,
    /// Dwell threshold for activation (in seconds)
    pub dwell_threshold: f32,
    /// Whether the cursor is currently active
    pub is_active: bool,
    /// Whether the cursor is currently dwelling on a target
    is_dwelling: bool,
}

impl CursorState {
    /// Create a new CursorState with default values
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            dwell_time: 0.0,
            dwell_threshold: 1.0, // 1 second default threshold
            is_active: false,
            is_dwelling: false,
        }
    }
    
    /// Update the cursor state with the time since the last update
    pub fn update(&mut self, delta_seconds: f32) {
        if self.is_dwelling {
            self.dwell_time += delta_seconds;
        } else {
            self.dwell_time = 0.0;
        }
    }
    
    /// Check if the dwell time has been reached
    pub fn is_dwell_complete(&self) -> bool {
        self.dwell_time >= self.dwell_threshold
    }
    
    /// Reset the dwell timer
    pub fn reset_dwell(&mut self) {
        self.dwell_time = 0.0;
    }
    
    /// Start the dwell timer
    pub fn start_dwell(&mut self) {
        self.is_dwelling = true;
    }
    
    /// Stop the dwell timer
    pub fn stop_dwell(&mut self) {
        self.is_dwelling = false;
        self.dwell_time = 0.0;
    }
}

/// Plugin for cursor state management
pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorState>();
    }
}
