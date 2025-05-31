#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct PixelateUniform {
    texel : vec2<f32>,
    threshold : f32
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var normal_texture: texture_2d<f32>;
@group(0) @binding(2) var depth_texture: texture_depth_2d;
@group(0) @binding(3) var texture_sampler: sampler;
@group(0) @binding(4) var<uniform> pixelate_data: PixelateUniform;
@group(0) @binding(5) var<uniform> projection: mat4x4<f32>;

fn perspective_camera_near() -> f32 {
    return projection[3][2];
}

fn linear_depth(ndc_depth: f32) -> f32 {
#ifdef VIEW_PROJECTION_PERSPECTIVE
    return -perspective_camera_near() / ndc_depth;
#else ifdef VIEW_PROJECTION_ORTHOGRAPHIC
    return -(projection[3][2] - ndc_depth) / projection[2][2];
#else
    let view_pos = projection * vec4(0.0, 0.0, ndc_depth, 1.0);
    return view_pos.z / view_pos.w;
#endif
}

fn clamp(t : f32, a : f32, b : f32) -> f32 {
    return min(max(t, a), b);
}

fn get_normal(uv : vec2<f32>) -> vec3<f32> {
    let normal = textureSample(normal_texture, texture_sampler, uv);
    return normalize(vec3(normal.r, normal.g, normal.b));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let offsets = array( 
        in.uv + (vec2(-1.0, 0.0) * pixelate_data.texel),
        in.uv + (vec2(1.0, 0.0) * pixelate_data.texel),
        in.uv + (vec2(0.0, -1.0) * pixelate_data.texel), 
        in.uv + (vec2(0.0, 1.0) * pixelate_data.texel)
    );
    
    //Calculate depth difference
    var depth_diff = 0.0;
    let depth = linear_depth(textureSample(depth_texture, texture_sampler, in.uv));
    
    for(var i: i32 = 0; i < 4; i++) {
        let offset = offsets[i];
        let depth_offset = linear_depth(textureSample(depth_texture, texture_sampler, offset));
        depth_diff += clamp(depth_offset - depth, 0.0, 1.0);
    }
    
    let smooth_diff = smoothstep(0.35, 0.4, depth_diff);
    
    // Calculate normal difference
    var normal_diff = 0.0;
    let normal = textureSample(normal_texture, texture_sampler, in.uv);
    
    for(var i: i32 = 0; i < 4; i++) {
        let offset = offsets[i];
        let depth_offset = linear_depth(textureSample(depth_texture, texture_sampler, offset));
        depth_diff += clamp(depth_offset - depth, 0.0, 1.0);
    }
    
    if(smooth_diff > pixelate_data.threshold) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        let base_color = textureSample(screen_texture, texture_sampler, in.uv);
        return vec4<f32>(base_color);
    }
}