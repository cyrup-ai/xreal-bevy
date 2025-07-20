use anyhow::Result;
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::{ExtractedView, ViewTarget},
    },
};
use std::collections::HashMap;
use wgpu::{CompositeAlphaMode, PresentMode, Surface, SurfaceConfiguration, TextureFormat};

use super::PluginError;

/// Multi-surface manager for coordinating plugin rendering
/// Integrates with Bevy's render pipeline and existing stereo rendering
#[derive(Resource)]
pub struct SurfaceManager {
    /// Plugin surfaces mapped by plugin ID
    plugin_surfaces: HashMap<String, PluginSurface>,
    /// Shared surface configuration
    base_config: SurfaceConfiguration,
    /// Surface format for compatibility
    surface_format: TextureFormat,
    /// Maximum number of concurrent surfaces
    max_surfaces: usize,
}

/// Individual plugin surface with its own configuration
#[allow(dead_code)]
pub struct PluginSurface {
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub size: (u32, u32),
    pub plugin_id: String,
    pub is_visible: bool,
    pub z_order: i32,
    pub position_3d: Vec3,
    pub created_at: std::time::Instant,
}

impl SurfaceManager {
    pub fn new() -> Result<Self> {
        // Get surface format from system capabilities
        let surface_format = TextureFormat::Bgra8UnormSrgb; // Standard format for compatibility

        let base_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 1920, // Default size
            height: 1080,
            present_mode: PresentMode::Fifo, // VSync for smooth rendering
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![surface_format],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            plugin_surfaces: HashMap::new(),
            base_config,
            surface_format,
            max_surfaces: 16, // Reasonable limit for AR environment
        })
    }

    /// Create surface for plugin
    pub fn create_surface(&mut self, plugin_id: String, size: (u32, u32)) -> Result<String> {
        if self.plugin_surfaces.len() >= self.max_surfaces {
            return Err(
                PluginError::SurfaceError("Maximum surface limit reached".to_string()).into(),
            );
        }

        // Create surface configuration for this plugin
        let mut config = self.base_config.clone();
        config.width = size.0;
        config.height = size.1;

        // Note: In actual implementation, this would create a real WGPU surface
        // For now, showing the structure and integration pattern
        info!(
            "Creating surface for plugin: {} ({}x{})",
            plugin_id, size.0, size.1
        );

        let surface_id = format!("surface_{}", plugin_id);

        // In full implementation:
        // let surface = instance.create_surface(&window)?;
        // surface.configure(&device, &config);

        Ok(surface_id)
    }

    /// Destroy surface for plugin
    pub fn destroy_surface(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(_surface) = self.plugin_surfaces.remove(plugin_id) {
            info!("Destroyed surface for plugin: {}", plugin_id);
        }
        Ok(())
    }

    /// Get surface for plugin
    pub fn get_surface(&self, plugin_id: &str) -> Option<&PluginSurface> {
        self.plugin_surfaces.get(plugin_id)
    }

    /// Update surface visibility and 3D positioning
    pub fn update_surface_transform(
        &mut self,
        plugin_id: &str,
        position: Vec3,
        visible: bool,
    ) -> Result<()> {
        if let Some(surface) = self.plugin_surfaces.get_mut(plugin_id) {
            surface.position_3d = position;
            surface.is_visible = visible;
        }
        Ok(())
    }

    /// Resize surface for plugin
    pub fn resize_surface(&mut self, plugin_id: &str, new_size: (u32, u32)) -> Result<()> {
        if let Some(surface) = self.plugin_surfaces.get_mut(plugin_id) {
            surface.size = new_size;
            surface.config.width = new_size.0;
            surface.config.height = new_size.1;

            // In full implementation:
            // surface.surface.configure(&device, &surface.config);

            info!(
                "Resized surface for plugin: {} to {}x{}",
                plugin_id, new_size.0, new_size.1
            );
        }
        Ok(())
    }

    /// Get all visible surfaces ordered by z-order
    pub fn get_visible_surfaces(&self) -> Vec<&PluginSurface> {
        let mut surfaces: Vec<&PluginSurface> = self
            .plugin_surfaces
            .values()
            .filter(|s| s.is_visible)
            .collect();

        surfaces.sort_by_key(|s| s.z_order);
        surfaces
    }

    /// Get total GPU memory usage by all surfaces
    pub fn get_total_memory_usage(&self) -> u64 {
        self.plugin_surfaces
            .values()
            .map(|surface| {
                let (width, height) = surface.size;
                let bytes_per_pixel = surface
                    .config
                    .format
                    .block_copy_size(Some(wgpu::TextureAspect::All))
                    .unwrap_or(4) as u64;
                (width as u64) * (height as u64) * bytes_per_pixel
            })
            .sum()
    }
}

/// Bevy render node for compositing plugin surfaces
/// Integrates with existing render pipeline for stereo rendering
///
/// NOTE: Comprehensive surface compositor for future multi-plugin rendering.
/// Not yet integrated into render graph. Preserved for advanced compositing features.
#[allow(dead_code)]
pub struct PluginSurfaceCompositorNode;

impl ViewNode for PluginSurfaceCompositorNode {
    type ViewQuery = (&'static ExtractedView, &'static ViewTarget);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        (_view, view_target): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Get surface manager from world
        let Some(surface_manager) = world.get_resource::<SurfaceManager>() else {
            warn!("SurfaceManager resource not found");
            return Ok(());
        };

        // Get visible surfaces
        let visible_surfaces = surface_manager.get_visible_surfaces();

        if visible_surfaces.is_empty() {
            return Ok(());
        }

        // Get render device from world
        let render_device = world.resource::<RenderDevice>();

        // Create command encoder for compositing
        let mut encoder = render_device.wgpu_device().create_command_encoder(
            &bevy::render::render_resource::CommandEncoderDescriptor {
                label: Some("plugin_surface_compositor"),
            },
        );

        // Composite plugin surfaces into main view
        {
            let _render_pass =
                encoder.begin_render_pass(&bevy::render::render_resource::RenderPassDescriptor {
                    label: Some("plugin_surface_composite_pass"),
                    color_attachments: &[Some(
                        bevy::render::render_resource::RenderPassColorAttachment {
                            view: view_target.main_texture_view(),
                            resolve_target: None,
                            ops: bevy::render::render_resource::Operations {
                                load: bevy::render::render_resource::LoadOp::Load, // Preserve existing content
                                store: bevy::render::render_resource::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            // In full implementation, this would:
            // 1. Bind compositor shader
            // 2. For each visible surface:
            //    - Bind surface texture as input
            //    - Apply 3D transform based on position_3d
            //    - Render quad with appropriate blending
            // 3. Handle transparency and z-ordering
        }

        // Get render queue from world and submit commands
        let render_queue = world.resource::<RenderQueue>();
        render_queue.submit([encoder.finish()]);

        Ok(())
    }
}

/// System to manage plugin surface lifecycle
pub fn surface_management_system(
    mut surface_manager: ResMut<SurfaceManager>,
    plugin_registry: Res<super::FastPluginRegistry>,
    mut plugin_events: EventWriter<super::PluginSystemEvent>,
) {
    // Monitor active plugins and ensure they have surfaces
    let active_plugins: Vec<&str> = plugin_registry.list_active_plugins().collect();

    for plugin_id in &active_plugins {
        if surface_manager.get_surface(plugin_id).is_none() {
            // Create surface for newly active plugin
            match surface_manager.create_surface(plugin_id.to_string(), (1920, 1080)) {
                Ok(surface_id) => {
                    plugin_events.write(super::PluginSystemEvent::SurfaceCreated {
                        plugin_id: plugin_id.to_string(),
                        surface_id,
                    });
                }
                Err(e) => {
                    error!("Failed to create surface for plugin {}: {}", plugin_id, e);
                }
            }
        }
    }

    // Cleanup surfaces for inactive plugins
    let surface_plugin_ids: Vec<String> = surface_manager.plugin_surfaces.keys().cloned().collect();
    for plugin_id in surface_plugin_ids {
        if !active_plugins.contains(&plugin_id.as_str()) {
            if let Err(e) = surface_manager.destroy_surface(&plugin_id) {
                error!("Failed to destroy surface for plugin {}: {}", plugin_id, e);
            } else {
                plugin_events.write(super::PluginSystemEvent::SurfaceDestroyed {
                    surface_id: format!("surface_{}", plugin_id),
                });
            }
        }
    }
}

/// System to render active plugins to their surfaces
pub fn plugin_render_system(
    surface_manager: ResMut<SurfaceManager>,
    mut plugin_registry: ResMut<super::FastPluginRegistry>,
    _render_device: Res<RenderDevice>,
    _render_queue: Res<RenderQueue>,
    time: Res<Time>,
    mut performance_tracker: ResMut<super::context::PluginPerformanceTracker>,
) {
    let _delta_time = time.delta_secs();
    let _frame_count = time.elapsed_secs() as u64 * 60; // Approximate frame count

    // Render each active plugin to its surface
    let active_plugins: Vec<String> = plugin_registry
        .list_active_plugins()
        .map(|s| s.to_string())
        .collect();

    for plugin_id in &active_plugins {
        if let Some(surface) = surface_manager.get_surface(plugin_id) {
            if !surface.is_visible {
                continue;
            }

            // FastPluginRegistry doesn't expose mutable app access for safety
            // Instead, we use the performance recording API directly
            if let Some(_entry) = plugin_registry.get_plugin(plugin_id) {
                let render_start = std::time::Instant::now();

                // Create render context for this plugin
                // Note: In full implementation, this would create actual WGPU resources
                // For now, showing the integration pattern

                // Simulate plugin rendering (FastPluginRegistry doesn't expose app directly)
                // The actual rendering is managed internally by the FastPluginRegistry
                let render_time = render_start.elapsed().as_secs_f32() * 1000.0; // Convert to ms

                // Record performance metrics using the FastPluginRegistry API
                if let Err(e) =
                    plugin_registry.record_performance(plugin_id, (render_time * 1000.0) as u32)
                {
                    error!(
                        "Failed to record performance for plugin {}: {}",
                        plugin_id, e
                    );
                }

                // Also record with the performance tracker
                performance_tracker
                    .record_frame_time_for_plugin(plugin_id.to_string(), render_time);
            }
        }
    }
}

/// System to update plugin surface positions based on 3D scene
pub fn update_plugin_surface_positions(
    mut surface_manager: ResMut<SurfaceManager>,
    orientation: Res<crate::tracking::Orientation>,
    screen_distance: Res<crate::ScreenDistance>,
) {
    // Update surface positions based on head tracking and configured distance
    let head_rotation = orientation.quat;
    let base_distance = screen_distance.0;

    // In full implementation, this would:
    // 1. Calculate 3D positions for each plugin window
    // 2. Apply head tracking rotation
    // 3. Handle window arrangement and focus management
    // 4. Update surface transforms accordingly

    for (_plugin_id, surface) in &mut surface_manager.plugin_surfaces {
        if surface.is_visible {
            // Calculate position based on window arrangement
            // This is a simplified example - full implementation would have proper window management
            let offset = Vec3::new(0.0, 0.0, base_distance);
            let rotated_position = head_rotation * offset;

            surface.position_3d = rotated_position;
        }
    }
}

/// Resource for tracking plugin window focus
#[derive(Resource, Default)]
pub struct PluginWindowManager {
    pub focused_plugin: Option<String>,
    pub window_arrangement: WindowArrangement,
    pub focus_history: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WindowArrangement {
    Tiled,
    Floating,
    Stacked,
}

impl Default for WindowArrangement {
    fn default() -> Self {
        WindowArrangement::Floating
    }
}

impl PluginWindowManager {
    pub fn focus_plugin(&mut self, plugin_id: String) {
        // Update focus history
        if let Some(current) = &self.focused_plugin {
            if current != &plugin_id {
                self.focus_history.push(current.clone());
                if self.focus_history.len() > 10 {
                    self.focus_history.remove(0);
                }
            }
        }

        self.focused_plugin = Some(plugin_id);
    }

    pub fn get_focused_plugin(&self) -> Option<&String> {
        self.focused_plugin.as_ref()
    }

    pub fn unfocus_plugin(&mut self, plugin_id: &str) {
        if self.focused_plugin.as_ref() == Some(&plugin_id.to_string()) {
            // Restore previous focus if available
            self.focused_plugin = self.focus_history.pop();
        }
    }
}

/// System to handle plugin window focus management
pub fn plugin_window_focus_system(
    _window_manager: ResMut<PluginWindowManager>,
    _surface_manager: ResMut<SurfaceManager>,
    input: Res<ButtonInput<MouseButton>>,
    _windows: Query<&Window>,
) {
    // Handle window focus changes based on user interaction
    if input.just_pressed(MouseButton::Left) {
        // In full implementation, this would:
        // 1. Ray cast from mouse position into 3D scene
        // 2. Determine which plugin surface was clicked
        // 3. Update focus accordingly
        // 4. Send focus events to plugins
    }
}
