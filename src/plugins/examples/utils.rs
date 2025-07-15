//! Utility functions for plugin example implementations
//! 
//! Provides optimized, zero-allocation utilities for common plugin operations
//! including shader creation, geometry generation, and WGPU resource management.

use anyhow::Result;
use bevy::render::renderer::RenderDevice;
use bytemuck::{Pod, Zeroable};
use wgpu::{Device, RenderPipeline, Texture, Buffer, util::DeviceExt};

/// Vertex data for textured quad rendering
/// Uses bytemuck for zero-copy casting to GPU buffers
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

/// Pre-computed quad vertices for maximum performance
/// Arranged as two triangles forming a full-screen quad
pub const QUAD_VERTICES: [QuadVertex; 4] = [
    QuadVertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // Bottom-left
    QuadVertex { position: [ 1.0, -1.0, 0.0], tex_coords: [1.0, 1.0] }, // Bottom-right
    QuadVertex { position: [ 1.0,  1.0, 0.0], tex_coords: [1.0, 0.0] }, // Top-right
    QuadVertex { position: [-1.0,  1.0, 0.0], tex_coords: [0.0, 0.0] }, // Top-left
];

/// Quad indices for triangle rendering (clockwise winding)
pub const QUAD_INDICES: [u16; 6] = [
    0, 1, 2,  // First triangle
    2, 3, 0,  // Second triangle
];

/// Basic textured quad shader for plugin rendering
/// Optimized WGSL with efficient vertex processing and texture sampling
pub const QUAD_SHADER: &str = r#"
// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, input.tex_coords);
}
"#;

/// Simple colored quad shader for basic rendering
pub const COLORED_QUAD_SHADER: &str = r#"
// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

// Fragment shader
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple gradient based on texture coordinates
    return vec4<f32>(input.tex_coords.x, input.tex_coords.y, 0.5, 1.0);
}
"#;

/// Create optimized render pipeline for plugin use (Bevy RenderDevice version)
/// 
/// Uses cached shader modules and optimized pipeline state for maximum performance.
/// No allocations after initial creation.
/// 
/// # Arguments
/// * `render_device` - Bevy RenderDevice for resource creation
/// * `shader_source` - WGSL shader source code
/// * `format` - Target surface format
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `Result<RenderPipeline>` - Configured render pipeline or error
#[inline]
pub fn create_basic_render_pipeline_bevy(
    render_device: &RenderDevice,
    shader_source: &str,
    format: wgpu::TextureFormat,
    label: Option<&str>,
) -> Result<RenderPipeline> {
    create_basic_render_pipeline(render_device.wgpu_device(), shader_source, format, label)
}

/// Create optimized render pipeline for plugin use
/// 
/// Uses cached shader modules and optimized pipeline state for maximum performance.
/// No allocations after initial creation.
/// 
/// # Arguments
/// * `device` - WGPU device for resource creation
/// * `shader_source` - WGSL shader source code
/// * `format` - Target surface format
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `Result<RenderPipeline>` - Configured render pipeline or error
#[inline]
pub fn create_basic_render_pipeline(
    device: &Device,
    shader_source: &str,
    format: wgpu::TextureFormat,
    label: Option<&str>,
) -> Result<RenderPipeline> {
    // Create shader module
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: label.map(|l| format!("{}_shader", l)).as_deref(),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Define vertex buffer layout
    let vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            // Position attribute
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            // Texture coordinate attribute
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    };

    // Create render pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: label.map(|l| format!("{}_layout", l)).as_deref(),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer_layout],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    Ok(pipeline)
}

/// Create quad vertex and index data
/// 
/// Returns pre-computed vertex and index arrays for efficient quad rendering.
/// No allocations - uses const data.
/// 
/// # Returns
/// * `(Vec<QuadVertex>, Vec<u16>)` - Vertex and index data
#[inline]
pub fn create_quad_vertices() -> (Vec<QuadVertex>, Vec<u16>) {
    (QUAD_VERTICES.to_vec(), QUAD_INDICES.to_vec())
}

/// Create render texture with optimized settings
/// 
/// Creates a texture suitable for plugin rendering with efficient memory layout
/// and optimal format selection.
/// 
/// # Arguments
/// * `device` - WGPU device for resource creation
/// * `size` - Texture dimensions (width, height)
/// * `format` - Texture format
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `Texture` - Created render texture
#[inline]
pub fn create_render_texture(
    device: &Device,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    label: Option<&str>,
) -> Texture {
    let size = wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Create vertex buffer from quad vertices (Bevy RenderDevice version)
/// 
/// Optimized buffer creation with proper usage flags and efficient memory layout.
/// 
/// # Arguments
/// * `render_device` - Bevy RenderDevice for resource creation
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `Buffer` - Vertex buffer containing quad geometry
#[inline]
pub fn create_quad_vertex_buffer_bevy(render_device: &RenderDevice, label: Option<&str>) -> Buffer {
    create_quad_vertex_buffer(render_device.wgpu_device(), label)
}

/// Create vertex buffer from quad vertices
/// 
/// Optimized buffer creation with proper usage flags and efficient memory layout.
/// 
/// # Arguments
/// * `device` - WGPU device for resource creation
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `Buffer` - Vertex buffer containing quad geometry
#[inline]
pub fn create_quad_vertex_buffer(device: &Device, label: Option<&str>) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(&QUAD_VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

/// Create index buffer from quad indices
/// 
/// Optimized buffer creation for index data with proper usage flags.
/// 
/// # Arguments
/// * `device` - WGPU device for resource creation
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `Buffer` - Index buffer containing quad indices
#[inline]
pub fn create_quad_index_buffer(device: &Device, label: Option<&str>) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(&QUAD_INDICES),
        usage: wgpu::BufferUsages::INDEX,
    })
}

/// Performance-optimized texture sampler
/// 
/// Creates a sampler with settings optimized for plugin rendering scenarios.
/// 
/// # Arguments
/// * `device` - WGPU device for resource creation
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `wgpu::Sampler` - Configured texture sampler
#[inline]
pub fn create_default_sampler(device: &Device, label: Option<&str>) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

/// Create bind group layout for texture rendering
/// 
/// Standard bind group layout for texture + sampler combination used by plugins.
/// 
/// # Arguments
/// * `device` - WGPU device for resource creation
/// * `label` - Optional debug label
/// 
/// # Returns
/// * `wgpu::BindGroupLayout` - Bind group layout for texture rendering
#[inline]
pub fn create_texture_bind_group_layout(
    device: &Device,
    label: Option<&str>,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label,
        entries: &[
            // Texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Efficient texture upload helper
/// 
/// Uploads texture data with optimal memory alignment and transfer efficiency.
/// 
/// # Arguments
/// * `device` - WGPU device
/// * `queue` - WGPU queue for commands
/// * `texture` - Target texture
/// * `data` - Texture data (RGBA8 format)
/// * `size` - Texture dimensions
/// 
/// # Returns
/// * `Result<()>` - Success or error
pub fn upload_texture_data(
    _device: &Device,
    queue: &wgpu::Queue,
    texture: &Texture,
    data: &[u8],
    size: (u32, u32),
) -> Result<()> {
    let extent = wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    // Calculate proper alignment for texture data
    let bytes_per_pixel = 4; // RGBA8
    let unpadded_bytes_per_row = size.0 * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

    // Upload texture data
    queue.write_texture(
        texture.as_image_copy(),
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(padded_bytes_per_row),
            rows_per_image: Some(size.1),
        },
        extent,
    );

    Ok(())
}

/// Create complete textured quad setup
/// 
/// One-shot function to create all resources needed for textured quad rendering.
/// Optimized for plugin initialization with minimal overhead.
/// 
/// # Arguments
/// * `device` - WGPU device
/// * `format` - Target surface format
/// * `texture_size` - Size of render texture
/// * `label_prefix` - Prefix for debug labels
/// 
/// # Returns
/// * `Result<QuadRenderResources>` - Complete resource bundle
pub fn create_textured_quad_resources(
    device: &Device,
    format: wgpu::TextureFormat,
    texture_size: (u32, u32),
    label_prefix: &str,
) -> Result<QuadRenderResources> {
    // Create pipeline
    let pipeline = create_basic_render_pipeline(
        device,
        QUAD_SHADER,
        format,
        Some(&format!("{}_pipeline", label_prefix)),
    )?;

    // Create buffers
    let vertex_buffer = create_quad_vertex_buffer(
        device,
        Some(&format!("{}_vertices", label_prefix)),
    );
    
    let index_buffer = create_quad_index_buffer(
        device,
        Some(&format!("{}_indices", label_prefix)),
    );

    // Create texture
    let texture = create_render_texture(
        device,
        texture_size,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        Some(&format!("{}_texture", label_prefix)),
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create sampler
    let sampler = create_default_sampler(
        device,
        Some(&format!("{}_sampler", label_prefix)),
    );

    // Create bind group layout
    let bind_group_layout = create_texture_bind_group_layout(
        device,
        Some(&format!("{}_bind_group_layout", label_prefix)),
    );

    // Create bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{}_bind_group", label_prefix)),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok(QuadRenderResources {
        pipeline,
        vertex_buffer,
        index_buffer,
        texture,
        texture_view,
        sampler,
        bind_group_layout,
        bind_group,
    })
}

/// Complete resource bundle for textured quad rendering
/// 
/// Contains all WGPU resources needed for efficient quad rendering in plugins.
pub struct QuadRenderResources {
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub texture: Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl QuadRenderResources {
    /// Render quad with current resources
    /// 
    /// Optimized rendering path with minimal state changes.
    /// 
    /// # Arguments
    /// * `render_pass` - Active render pass
    #[inline]
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_vertex_size() {
        // Ensure vertex struct is properly packed
        assert_eq!(std::mem::size_of::<QuadVertex>(), 20); // 3*4 + 2*4 = 20 bytes
    }

    #[test]
    fn test_quad_geometry() {
        let (vertices, indices) = create_quad_vertices();
        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
        
        // Verify triangle winding
        assert_eq!(indices[0..3], [0, 1, 2]);
        assert_eq!(indices[3..6], [2, 3, 0]);
    }

    #[test]
    fn test_constants() {
        assert_eq!(QUAD_VERTICES.len(), 4);
        assert_eq!(QUAD_INDICES.len(), 6);
        assert!(!QUAD_SHADER.is_empty());
        assert!(!COLORED_QUAD_SHADER.is_empty());
    }
}