// Terminal shader for XREAL virtual desktop
// Renders terminal text with proper color support and anti-aliasing

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct TerminalUniforms {
    transform: mat4x4<f32>,
    grid_size: vec2<f32>,
    cell_size: vec2<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: TerminalUniforms;

@group(0) @binding(1)
var terminal_texture: texture_2d<f32>;

@group(0) @binding(2)
var terminal_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.transform * vec4<f32>(input.position, 1.0);
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the terminal texture
    let tex_color = textureSample(terminal_texture, terminal_sampler, input.tex_coords);
    
    // Apply vertex color modulation for terminal text colors
    let final_color = tex_color * input.color;
    
    // Simple alpha test for text rendering
    if (final_color.a < 0.1) {
        discard;
    }
    
    return final_color;
}