use bevy::prelude::*;
use enigo::*;
use crate::error::{InputError, InputResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

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

/// Main resource for handling input operations
#[derive(Resource)]
pub struct InputSystem {
    enigo: Enigo,
    last_event: AtomicU64,
    config: InputConfig,
}

impl InputSystem {
    /// Create a new input system with default settings
    pub fn new() -> InputResult<Self> {
        let settings = Settings::default();
        let enigo = Enigo::new(&settings).map_err(InputError::Initialization)?;
        
        Ok(Self {
            enigo,
            last_event: AtomicU64::new(0),
            config: InputConfig::default(),
        })
    }
    
    /// Move the mouse to the specified coordinates
    #[inline]
    pub fn move_mouse(&self, x: i32, y: i32) -> InputResult<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Rate limiting check
        let now = Instant::now();
        let last = self.last_event.load(Ordering::Acquire);
        let elapsed = now.duration_since(Instant::from_millis(last));
        
        if elapsed.as_millis() < self.config.min_event_interval_ms as u128 {
            return Err(InputError::RateLimited);
        }
        
        // Perform the mouse movement
        self.enigo.move_mouse(x, y, Coordinate::Abs)?;
        
        // Update last event time
        self.last_event.store(now.as_millis() as u64, Ordering::Release);
        
        Ok(())
    }
    
    /// Click the specified mouse button
    #[inline]
    pub fn click(&self, button: MouseButton) -> InputResult<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        self.enigo.button(button, Direction::Press)?;
        self.enigo.button(button, Direction::Release)?;
        
        Ok(())
    }
    
    /// Update configuration
    pub fn update_config<F>(&mut self, f: F) -> InputResult<()>
    where
        F: FnOnce(&mut InputConfig) -> InputResult<()>,
    {
        f(&mut self.config)
    }
}

/// Plugin for the input system
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        match InputSystem::new() {
            Ok(input_system) => {
                app.insert_resource(input_system);
                info!("Input system initialized successfully");
            }
            Err(e) => {
                error!("Failed to initialize input system: {}", e);
                // We could add a fallback implementation here if needed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: Actual tests will be in the tests/ directory per requirements
    // These are just compile-time checks
    
    #[test]
    fn test_input_system_compiles() {
        // This is just a compile-time check
        let _ = InputSystem::new();
    }
}
