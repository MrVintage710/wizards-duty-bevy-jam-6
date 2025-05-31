#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(screen_texture, texture_sampler, in.uv);
    return vec4<f32>(base_color);
}