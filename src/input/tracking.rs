use bevy::math::Quat;
use bevy::prelude::*;

/// Tracks the orientation of the XREAL glasses
#[derive(Resource, Default, Debug)]
pub struct Orientation {
    /// Current rotation quaternion
    pub rotation: Quat,
    /// Whether the orientation tracking is active
    pub is_tracking: bool,
    /// Last update timestamp
    pub last_update: f64,
}

impl Orientation {
    /// Create a new Orientation with default values
    pub fn new() -> Self {
        Self {
            rotation: Quat::IDENTITY,
            is_tracking: false,
            last_update: 0.0,
        }
    }

    /// Update the current orientation
    pub fn update(&mut self, rotation: Quat, timestamp: f64) {
        self.rotation = rotation;
        self.last_update = timestamp;
        self.is_tracking = true;
    }

    /// Get the forward direction vector
    pub fn forward(&self) -> Vec3 {
        (self.rotation * -Vec3::Z).normalize()
    }

    /// Get the up direction vector
    pub fn up(&self) -> Vec3 {
        (self.rotation * Vec3::Y).normalize()
    }

    /// Get the right direction vector
    pub fn right(&self) -> Vec3 {
        (self.rotation * Vec3::X).normalize()
    }

    /// Get the euler angles (pitch, yaw, roll) in radians
    pub fn to_euler(&self) -> (f32, f32, f32) {
        let (pitch, yaw, roll) = self.rotation.to_euler(EulerRot::YXZ);
        (pitch, yaw, roll)
    }
}

/// Plugin for orientation tracking
pub struct OrientationPlugin;

impl Plugin for OrientationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Orientation>()
            .add_systems(Update, update_orientation);
    }
}

fn update_orientation(
    time: Res<Time>,
    mut orientation: ResMut<Orientation>,
    // In a real implementation, this would read from the XREAL glasses
    // For now, we'll just simulate some movement
    mut rotation: Local<f32>,
) {
    // Simulate rotation for testing
    *rotation += time.delta_secs() * 0.5;
    orientation.rotation = Quat::from_rotation_y(*rotation);
    orientation.last_update = time.elapsed().as_secs_f64();
    orientation.is_tracking = true;
}
