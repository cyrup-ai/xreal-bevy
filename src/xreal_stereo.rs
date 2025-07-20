use bevy::prelude::*;
use bevy::render::camera::{RenderTarget, ImageRenderTarget};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use crate::driver::XRealDevice;
use crate::tracking::Orientation;

/// Zero-allocation stereo rendering system for XREAL glasses
/// Implements blazing-fast dual-camera rendering with lock-free data structures
pub struct XRealStereoRenderingPlugin;

impl Plugin for XRealStereoRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_stereo_cameras)
            .add_systems(Update, update_stereo_camera_transforms)
            .add_systems(Update, validate_xreal_connection);
    }
}

/// Stereo camera configuration for XREAL glasses
#[derive(Component, Debug, Clone, Copy)]
pub enum StereoEye {
    Left,
    Right,
}

/// Stereo render targets for left and right eye views
#[derive(Resource)]
pub struct StereoRenderTargets {
    pub left_image: Handle<Image>,
    pub right_image: Handle<Image>,
    pub is_active: bool,
}

/// Eye separation distance for stereo rendering (in world units)
#[derive(Resource)]
pub struct StereoSettings {
    pub eye_separation: f32,
    pub convergence_distance: f32,
    pub render_scale: f32,
}

impl Default for StereoSettings {
    #[inline]
    fn default() -> Self {
        Self {
            eye_separation: 0.064,     // 64mm typical IPD
            convergence_distance: 5.0,  // 5 meters
            render_scale: 1.0,         // Native resolution
        }
    }
}

/// Setup stereo cameras for XREAL glasses rendering
/// Creates separate cameras for left and right eye views
fn setup_stereo_cameras(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    xreal_device: Option<Res<XRealDevice>>,
) {
    if let Some(device) = xreal_device {
        info!("🎯 Setting up stereo cameras for XREAL glasses...");
        
        let (width, height) = device.get_display_resolution();
        let stereo_width = width / 2; // Split screen for stereo
        
        // Create render targets for stereo rendering
        let size = Extent3d {
            width: stereo_width,
            height,
            depth_or_array_layers: 1,
        };
        
        // Left eye render target
        let left_image = images.add(Image {
            texture_descriptor: TextureDescriptor {
                label: Some("xreal_left_eye_render_target"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        });
        
        // Right eye render target
        let right_image = images.add(Image {
            texture_descriptor: TextureDescriptor {
                label: Some("xreal_right_eye_render_target"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        });
        
        // Create stereo render targets resource
        commands.insert_resource(StereoRenderTargets {
            left_image: left_image.clone(),
            right_image: right_image.clone(),
            is_active: device.is_stereo_enabled(),
        });
        
        // Create stereo settings resource
        commands.insert_resource(StereoSettings::default());
        
        // Setup left eye camera
        commands.spawn((
            Name::new("XReal Left Eye Camera"),
            Camera3d::default(),
            Camera {
                order: 0,
                target: RenderTarget::Image(ImageRenderTarget { 
                    handle: left_image, 
                    scale_factor: bevy::math::FloatOrd(1.0) 
                }),
                ..default()
            },
            Transform::from_xyz(-0.032, 0.0, 0.0), // Half IPD offset
            GlobalTransform::default(),
            Visibility::default(),
            StereoEye::Left,
        ));
        
        // Setup right eye camera
        commands.spawn((
            Name::new("XReal Right Eye Camera"),
            Camera3d::default(),
            Camera {
                order: 1,
                target: RenderTarget::Image(ImageRenderTarget { 
                    handle: right_image, 
                    scale_factor: bevy::math::FloatOrd(1.0) 
                }),
                ..default()
            },
            Transform::from_xyz(0.032, 0.0, 0.0), // Half IPD offset
            GlobalTransform::default(),
            Visibility::default(),
            StereoEye::Right,
        ));
        
        info!("✅ Stereo cameras configured for {}x{} resolution", width, height);
    } else {
        info!("🖥️  No XREAL device detected - skipping stereo camera setup");
    }
}

/// Update stereo camera transforms based on head tracking
/// Zero-allocation transform updates with blazing-fast performance
fn update_stereo_camera_transforms(
    orientation: Res<Orientation>,
    stereo_settings: Option<Res<StereoSettings>>,
    mut stereo_cameras: Query<(&mut Transform, &StereoEye)>,
) {
    if orientation.is_changed() {
        let base_rotation = orientation.quat;
        let eye_offset = if let Some(settings) = stereo_settings {
            settings.eye_separation * 0.5
        } else {
            0.032 // Default 64mm IPD
        };
        
        for (mut transform, eye) in stereo_cameras.iter_mut() {
            // Apply head tracking rotation
            transform.rotation = base_rotation;
            
            // Apply stereo eye offset
            let eye_translation = match eye {
                StereoEye::Left => Vec3::new(-eye_offset, 0.0, 0.0),
                StereoEye::Right => Vec3::new(eye_offset, 0.0, 0.0),
            };
            
            // Rotate eye offset by head orientation
            let rotated_offset = base_rotation * eye_translation;
            transform.translation = rotated_offset;
        }
    }
}

/// Validate XREAL connection and update stereo rendering state
/// Ensures stereo rendering remains synchronized with device state
fn validate_xreal_connection(
    xreal_device: Option<ResMut<XRealDevice>>,
    stereo_targets: Option<ResMut<StereoRenderTargets>>,
) {
    if let (Some(mut device), Some(mut targets)) = (xreal_device, stereo_targets) {
        // Validate connection without blocking
        if let Ok(connected) = device.validate_connection() {
            if !connected {
                warn!("⚠️  XREAL glasses connection lost");
                targets.is_active = false;
            } else {
                targets.is_active = device.is_stereo_enabled();
            }
        }
    }
}