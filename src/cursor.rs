use crate::render::VirtualScreen;
use crate::tracking::Orientation;
use bevy::prelude::*;

/// Head-tracked cursor component for AR interaction
#[derive(Component)]
pub struct HeadCursor {
    pub size: f32,
    pub color: Color,
    pub hit_screen: Option<usize>,
    pub hit_position: Option<Vec2>,
}

impl Default for HeadCursor {
    fn default() -> Self {
        Self {
            size: 0.02,
            color: Color::srgb(0.0, 1.0, 0.0), // Green cursor
            hit_screen: None,
            hit_position: None,
        }
    }
}

/// Resource to track cursor state
#[derive(Resource)]
pub struct CursorState {
    pub is_active: bool,
    pub dwell_time: f32,
    pub dwell_threshold: f32,
    pub last_hit_screen: Option<usize>,
    pub last_hit_position: Option<Vec2>,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            is_active: true,
            dwell_time: 0.0,
            dwell_threshold: 2.0, // 2 seconds for dwell selection
            last_hit_screen: None,
            last_hit_position: None,
        }
    }
}

/// Spawn head-tracked cursor in the 3D scene
pub fn spawn_head_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a small sphere for the cursor
    let cursor_mesh = meshes.add(Sphere::new(0.01));
    let cursor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        emissive: Color::srgb(0.0, 0.3, 0.0).into(),
        unlit: true,
        ..default()
    });

    commands.spawn((
        MeshMaterial3d(cursor_material),
        Mesh3d(cursor_mesh),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
        HeadCursor::default(),
        Name::new("Head Cursor"),
    ));

    // Initialize cursor state resource
    commands.insert_resource(CursorState::default());
}

/// Update cursor position based on head tracking
pub fn update_head_cursor(
    mut cursor_query: Query<(&mut Transform, &mut HeadCursor)>,
    mut cursor_state: ResMut<CursorState>,
    orientation: Res<Orientation>,
    virtual_screens: Query<(&Transform, &VirtualScreen)>,
    time: Res<Time>,
) {
    if !cursor_state.is_active {
        return;
    }

    let Ok((mut cursor_transform, mut cursor)) = cursor_query.single_mut() else {
        return;
    };

    // Use real head tracking data for cursor positioning
    let head_rotation = orientation.quat;
    let head_position = Vec3::ZERO;

    // Cast ray from head position in head direction
    let ray_origin = head_position;
    let ray_dir = head_rotation * Vec3::NEG_Z;

    // Find intersection with virtual screens
    let mut closest_hit = None;
    let mut closest_distance = f32::MAX;

    for (screen_transform, virtual_screen) in virtual_screens.iter() {
        let plane_normal = *screen_transform.forward();
        let denom = ray_dir.dot(plane_normal);

        if denom.abs() <= 0.0001 {
            continue;
        }

        let t = (screen_transform.translation - ray_origin).dot(plane_normal) / denom;
        if t <= 0.0 {
            continue;
        }

        let hit_point = ray_origin + t * ray_dir;
        let local_hit = screen_transform.compute_matrix().inverse() * hit_point.extend(1.0);
        let u = (local_hit.x + 1.0) * 0.5;
        let v = (local_hit.y + 1.0) * 0.5;

        if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 && t < closest_distance {
            closest_distance = t;
            closest_hit = Some((virtual_screen.0, hit_point, Vec2::new(u, v)));
        }
    }

    // Update cursor position and state
    if let Some((screen_id, hit_point, hit_uv)) = closest_hit {
        // Position cursor at hit point
        cursor_transform.translation = hit_point;
        cursor.hit_screen = Some(screen_id);
        cursor.hit_position = Some(hit_uv);

        // Update dwell time for gaze selection
        if cursor_state.last_hit_screen == Some(screen_id) {
            cursor_state.dwell_time += time.delta_secs();

            // Change cursor color based on dwell progress
            let progress = cursor_state.dwell_time / cursor_state.dwell_threshold;
            if progress >= 1.0 {
                cursor.color = Color::srgb(1.0, 0.0, 0.0); // Red when ready to select
            } else {
                cursor.color = Color::srgb(progress, 1.0 - progress, 0.0); // Green to yellow
            }
        } else {
            cursor_state.dwell_time = 0.0;
            cursor.color = Color::srgb(0.0, 1.0, 0.0); // Reset to green
        }

        cursor_state.last_hit_screen = Some(screen_id);
        cursor_state.last_hit_position = Some(hit_uv);
    } else {
        // No hit - position cursor at default distance
        cursor_transform.translation = ray_origin + ray_dir * 2.0;
        cursor.hit_screen = None;
        cursor.hit_position = None;
        cursor_state.dwell_time = 0.0;
        cursor.color = Color::srgb(0.5, 0.5, 0.5); // Gray when not targeting
        cursor_state.last_hit_screen = None;
        cursor_state.last_hit_position = None;
    }
}

/// Update cursor material color based on state
pub fn update_cursor_material(
    cursor_query: Query<Entity, (With<HeadCursor>, Changed<HeadCursor>)>,
    cursor_data: Query<&HeadCursor>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in cursor_query.iter() {
        if let (Ok(cursor), Ok(material_handle)) =
            (cursor_data.get(entity), material_query.get(entity))
        {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color = cursor.color;
                material.emissive = LinearRgba::from(cursor.color) * 0.3;
            }
        }
    }
}
