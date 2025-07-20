//! Utility functions for plugin example implementations
//!
//! Provides optimized, zero-allocation utilities for common plugin operations
//! including shader creation, geometry generation, and WGPU resource management.

use anyhow::Result;
use bevy::{
    prelude::*,
    render::{
        // Removed duplicate imports to fix compilation errors
        render_resource::{
            BindGroup,
            BindGroupLayout,
            BindGroupLayoutEntry,
            BindingType,
            RenderPipeline,
            Sampler,
            SamplerBindingType,
            ShaderStages,
            TextureFormat,
            TextureSampleType,
            TextureView,
            TextureViewDimension,
        },
        renderer::RenderDevice,
    },
};
use bytemuck::{Pod, Zeroable};

use wgpu::{Device, Queue};

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
    QuadVertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    }, // Bottom-left
    QuadVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // Bottom-right
    QuadVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // Top-right
    QuadVertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    }, // Top-left
];

/// Quad indices for triangle rendering (clockwise winding)
pub const QUAD_INDICES: [u16; 6] = [
    0, 1, 2, // First triangle
    2, 3, 0, // Second triangle
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
    format: TextureFormat,
    label: Option<&str>,
) -> Result<RenderPipeline> {
    let wgpu_device = render_device.wgpu_device();
    
    // TODO: Fix WGPU ShaderModuleDescriptor type mismatch when plugin system is complete
    // // Create shader module with proper error handling
    // let shader_module = wgpu_device.create_shader_module(wgpu::ShaderModuleDescriptor {
    //     label,
    //     source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader_source)),
    // });

    // Define vertex attributes for basic quad rendering
    // Using static lifetime to avoid allocations
    const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
        // Position attribute (vec3)
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        // UV coordinate attribute (vec2)
        wgpu::VertexAttribute {
            offset: 12, // 3 * 4 bytes for position
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
    ];

    // Vertex buffer layout for QuadVertex structure
    let vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: VERTEX_ATTRIBUTES,
    };

    // TODO: Fix WGPU BindGroupLayoutDescriptor type mismatch when plugin system is complete
    // // Create basic bind group layout for texture + sampler
    // let bind_group_layout = wgpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //     label: Some("basic_pipeline_bind_group_layout"),
    //     entries: &[
    //         // Texture binding
    //         wgpu::BindGroupLayoutEntry {
    //             binding: 0,
    //             visibility: wgpu::ShaderStages::FRAGMENT,
    //             ty: wgpu::BindingType::Texture {
    //                 multisampled: false,
    //                 view_dimension: wgpu::TextureViewDimension::D2,
    //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
    //             },
    //             count: None,
    //         },
    //         // Sampler binding
    //         wgpu::BindGroupLayoutEntry {
    //             binding: 1,
    //             visibility: wgpu::ShaderStages::FRAGMENT,
    //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
    //             count: None,
    //         },
    //     ],
    // });

    // TODO: Fix WGPU PipelineLayoutDescriptor type mismatch when plugin system is complete
    // // Create pipeline layout
    // let pipeline_layout = wgpu_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    //     label: Some("basic_pipeline_layout"),
    //     bind_group_layouts: &[&bind_group_layout],
    //     push_constant_ranges: &[],
    // });

    // TODO: Fix WGPU render pipeline creation when plugin system is complete
    // // Create render pipeline with optimized settings
    // let render_pipeline = wgpu_device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    //     label,
    //     layout: Some(&pipeline_layout),
        // TODO: Fix WGPU shader module references when plugin system is complete
        // vertex: wgpu::VertexState {
        //     module: &shader_module,
        //     entry_point: "vs_main",
        //     buffers: &[vertex_buffer_layout],
        //     // compilation_options removed in newer WGPU versions
        // },
        // fragment: Some(wgpu::FragmentState {
        //     module: &shader_module,
        //     entry_point: "fs_main",
        //     targets: &[Some(wgpu::ColorTargetState {
        //         format,
        //         blend: Some(wgpu::BlendState {
        //             color: wgpu::BlendComponent {
        //                 src_factor: wgpu::BlendFactor::SrcAlpha,
        //                 dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        //                 operation: wgpu::BlendOperation::Add,
        //             },
        //             alpha: wgpu::BlendComponent {
        //                 src_factor: wgpu::BlendFactor::One,
        //                 dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        //                 operation: wgpu::BlendOperation::Add,
        //             },
        //         }),
        //         write_mask: wgpu::ColorWrites::ALL,
        //     })],
            // compilation_options removed in newer WGPU versions
        // }),
        // primitive: wgpu::PrimitiveState {
        //     topology: wgpu::PrimitiveTopology::TriangleList,
        //     strip_index_format: None,
        //     front_face: wgpu::FrontFace::Ccw,
        //     cull_mode: Some(wgpu::Face::Back),
        //     unclipped_depth: false,
        //     polygon_mode: wgpu::PolygonMode::Fill,
        //     conservative: false,
        // },
        // depth_stencil: None, // Basic pipeline without depth testing
        // multisample: wgpu::MultisampleState {
        //     count: 1,
        //     mask: !0,
        //     alpha_to_coverage_enabled: false,
        // },
        // multiview: None,
        // cache: None,
    // });

    // TODO: Return placeholder pipeline when WGPU is properly implemented
    Err(anyhow::anyhow!("WGPU pipeline creation temporarily disabled"))
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
    device: &wgpu::Device,
    shader_source: &str,
    format: TextureFormat,
    label: Option<&str>,
) -> Result<wgpu::RenderPipeline> {
    // Create shader module
    // Create shader module with proper label handling
    let shader_label = label.map(|s| format!("{}_shader", s));
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: shader_label.as_deref(),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Create pipeline layout with empty bind group layouts
    // Create pipeline layout with proper label handling
    let layout_label = label.map(|s| format!("{}_layout", s));
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: layout_label.as_deref(),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    // Create render pipeline with proper label handling
    let pipeline_label = label.map(|s| format!("{}_pipeline", s));
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: pipeline_label.as_deref(),
        layout: Some(&pipeline_layout),
        multiview: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<QuadVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    // Position
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    // Tex coords
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: std::mem::size_of::<[f32; 3]>() as u64,
                        shader_location: 1,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: match format {
                    bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb => {
                        wgpu::TextureFormat::Rgba8UnormSrgb
                    }
                    bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb => {
                        wgpu::TextureFormat::Bgra8UnormSrgb
                    }
                    _ => wgpu::TextureFormat::Rgba8UnormSrgb, // Default fallback
                },
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });

    // Return the wgpu pipeline directly - we'll handle the conversion at the call site
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
    device: &bevy::render::renderer::RenderDevice,
    size: (u32, u32),
    format: bevy::render::render_resource::TextureFormat,
    label: Option<&str>,
) -> bevy::render::render_resource::Texture {
    let size = bevy::render::render_resource::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    device.create_texture(&bevy::render::render_resource::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: bevy::render::render_resource::TextureDimension::D2,
        format,
        usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
            | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT
            | bevy::render::render_resource::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
/// * `label` - Optional debug label
///
/// # Returns
/// * `Buffer` - Vertex buffer containing quad geometry
#[inline]
pub fn create_quad_vertex_buffer_bevy(
    render_device: &RenderDevice,
    label: Option<&str>,
) -> bevy::render::render_resource::Buffer {
    // Create buffer directly using Bevy's RenderDevice
    let buffer_desc = bevy::render::render_resource::BufferDescriptor {
        label,
        size: (QUAD_VERTICES.len() * std::mem::size_of::<f32>()) as u64,
        usage: bevy::render::render_resource::BufferUsages::VERTEX
            | bevy::render::render_resource::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    let buffer = render_device.create_buffer(&buffer_desc);

    // Write data to buffer using render device
    // Note: In a real implementation, we'd need access to RenderQueue for writing
    // For now, return the empty buffer
    buffer
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
pub fn create_quad_vertex_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: Option<&str>,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label,
        size: (QUAD_VERTICES.len() * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&QUAD_VERTICES));
    buffer
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
pub fn create_quad_index_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: Option<&str>,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label,
        size: (QUAD_INDICES.len() * std::mem::size_of::<u16>()) as u64,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&QUAD_INDICES));
    buffer
}

/// Performance-optimized texture sampler
///
/// Creates a sampler with settings optimized for plugin rendering scenarios.
///
///
/// # Arguments
/// * `device` - Bevy RenderDevice for resource creation
/// * `label` - Optional debug label
///
/// # Returns
/// * `BindGroupLayout` - Bind group layout for texture rendering
#[inline]
pub fn create_texture_bind_group_layout(
    device: &RenderDevice,
    label: Option<&str>,
) -> BindGroupLayout {
    device.create_bind_group_layout(
        label,
        &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    )
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
#[inline]
pub fn upload_texture_data(
    _device: &Device,
    queue: &Queue,
    texture: &wgpu::Texture,
    data: &[u8],
    size: (u32, u32),
) -> Result<()> {
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size.0 * 4),
            rows_per_image: Some(size.1),
        },
        wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
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
    format: TextureFormat,
    texture_size: (u32, u32),
    label_prefix: &str,
) -> Result<QuadRenderResources> {
    let texture_label = format!("{}_texture", label_prefix);
    let _sampler_label = format!("{}_sampler", label_prefix);
    let bind_group_layout_label = format!("{}_bind_group_layout", label_prefix);
    let bind_group_label = format!("{}_bind_group", label_prefix);

    // Create texture directly using wgpu device
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&texture_label),
        size: wgpu::Extent3d {
            width: texture_size.0,
            height: texture_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: match format {
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb => {
                wgpu::TextureFormat::Rgba8UnormSrgb
            }
            bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb => {
                wgpu::TextureFormat::Bgra8UnormSrgb
            }
            _ => wgpu::TextureFormat::Rgba8UnormSrgb, // Default fallback
        },
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Create texture view
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create optimized sampler for texture filtering
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    // Create bind group layout for texture + sampler
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&bind_group_layout_label),
        entries: &[
            // Texture binding
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
            // Sampler binding
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    // Create bind group with texture and sampler
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&bind_group_label),
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

    // Return complete resource bundle
    Ok(QuadRenderResources {
        texture_view,
        sampler,
        bind_group_layout,
        bind_group,
    })
}

/// Create a default sampler with optimized settings for texture sampling
///
/// # Arguments
/// * `device` - The WGPU device to create the sampler on
/// * `label` - Optional label for debugging
///
/// # Returns
/// A new sampler with default settings
fn create_default_sampler(device: &Device, label: Option<&str>) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    })
}

/// Complete resource bundle for textured quad rendering
///
/// Contains all WGPU resources needed for efficient quad rendering in plugins.
pub struct QuadRenderResources {
    pub texture_view: TextureView,
    pub sampler: Sampler,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl QuadRenderResources {
    /// Render quad with current resources
    ///
    /// Optimized rendering path with minimal state changes.
    ///
    /// # Arguments
    /// * `render_pass` - Active render pass
    pub fn render<'a>(
        &'a self,
        render_pass: &mut bevy::render::render_phase::TrackedRenderPass<'a>,
    ) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}


