use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::pbr::NotShadowCaster;

/// Represents a virtual screen in 3D space
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct VirtualScreen {
    /// Size of the virtual screen in world units
    pub size: Vec2,
    /// Pixels per world unit
    pub pixels_per_unit: f32,
}

impl Default for VirtualScreen {
    fn default() -> Self {
        Self {
            size: Vec2::new(16.0, 9.0), // 16:9 aspect ratio by default
            pixels_per_unit: 100.0,     // 100 pixels per world unit
        }
    }
}

/// Plugin for virtual screen rendering
pub struct VirtualScreenPlugin;

impl Plugin for VirtualScreenPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VirtualScreen>()
            .add_systems(Startup, setup_virtual_screen);
    }
}

fn setup_virtual_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a simple quad for the virtual screen
    let mut quad_mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD);
    
    // Define vertex positions
    let positions: Vec<[f32; 3]> = vec![
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ];
    quad_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    
    // Define normals (all pointing forward)
    let normals: Vec<[f32; 3]> = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];
    quad_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    
    // Define UV coordinates
    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];
    quad_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    
    // Define indices (two triangles to form a quad)
    let quad_mesh = quad_mesh.with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    let mesh = meshes.add(quad_mesh);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, -2.0)
            .with_scale(Vec3::new(16.0, 9.0, 1.0)),
        VirtualScreen::default(),
        NotShadowCaster,
    ));
}
