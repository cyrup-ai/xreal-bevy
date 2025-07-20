use crate::capture::CaptureTask;
use crate::{Orientation, ScreenCaptures, ScreenDistance};
use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Component)]
pub struct VirtualScreen(pub usize);

#[derive(Component)]
pub struct ScreenMaterial(pub Handle<StandardMaterial>);

#[inline]
pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    captures: Option<Res<ScreenCaptures>>,
) {
    let num_screens = captures.as_ref().map(|c| c.num_streams).unwrap_or(1);

    // Create virtual screens with optimized spacing
    for i in 0..num_screens {
        let x = (i as f32 - (num_screens - 1) as f32 * 0.5) * 3.0;
        // Create properly sized placeholder texture for screen capture
        let placeholder_image = Image::new_fill(
            Extent3d {
                width: 512,
                height: 512,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255], // Black with full alpha
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );

        let material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(placeholder_image)),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        });

        let mesh_handle = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(2.0)));
        commands
            .spawn_empty()
            .insert(Mesh3d(mesh_handle))
            .insert(MeshMaterial3d(material_handle.clone()))
            .insert(Transform::from_xyz(x, 0.0, -5.0))
            .insert(Visibility::default())
            .insert(VirtualScreen(i))
            .insert(ScreenMaterial(material_handle));
    }

    // Camera setup
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::NEG_Z, Vec3::Y),
    ));

    // Lighting setup
    commands.spawn((
        PointLight {
            intensity: 2000.0,
            range: 20.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 5.0),
    ));

    // Ambient light for better visibility
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
        affects_lightmapped_meshes: false,
    });
}

#[inline]
pub fn update_camera_from_orientation(
    mut query: Query<&mut Transform, With<Camera>>,
    orientation: Res<Orientation>,
) {
    if let Ok(mut transform) = query.single_mut() {
        transform.rotation = orientation.quat;
    }
}

/// Spawn screen capture tasks non-blocking
#[inline]
pub fn spawn_capture_tasks(
    mut commands: Commands,
    captures: Option<Res<ScreenCaptures>>,
    query: Query<Entity, (With<VirtualScreen>, Without<CaptureTask>)>,
) {
    // Only spawn capture tasks if ScreenCaptures resource is available
    if let Some(captures) = captures {
        // Spawn capture tasks for screens that don't have one
        for entity in &query {
            if let Some(task) = captures.spawn_capture_task(entity) {
                commands.entity(entity).insert(task);
            }
        }
    }
}

/// Handle completed capture tasks using CommandQueue pattern with jitter measurement
#[inline]
pub fn handle_capture_tasks(
    mut commands: Commands,
    mut tasks: Query<&mut CaptureTask>,
    mut jitter_metrics: ResMut<crate::JitterMetrics>,
    time: Res<Time>,
) {
    use bevy::tasks::{block_on, futures_lite::future};

    // Use high-precision timing for capture interval measurement
    let current_time = time.elapsed_secs_f64() * 1000.0;

    for mut task in &mut tasks {
        // Poll the task non-blocking - this is the only acceptable use of block_on for polling
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            // Measure screen capture timing for jitter analysis
            if jitter_metrics.last_capture_time > 0.0 {
                let capture_interval = current_time - jitter_metrics.last_capture_time;
                jitter_metrics.add_capture_measurement(capture_interval);
            }
            jitter_metrics.last_capture_time = current_time;

            // Apply the command queue to execute deferred world modifications
            commands.append(&mut command_queue);
        }
    }
}

#[inline]
pub fn update_screen_positions(
    mut query: Query<&mut Transform, With<VirtualScreen>>,
    distance: Res<ScreenDistance>,
) {
    let dist = distance.0;
    for mut transform in &mut query {
        transform.translation.z = dist;
    }
}
