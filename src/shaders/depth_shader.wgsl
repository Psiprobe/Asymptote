struct Data {
    offset: vec3<f32>,
};
@group(1)@binding(0)
var<uniform> data: Data;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_shadow: texture_depth_2d;
@group(0)@binding(1)
var s_shadow: sampler_comparison;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = textureSampleCompare(t_shadow, s_shadow, in.tex_coords,0.3);
    return vec4<f32>(vec3<f32>(depth), 1.0);
    //return textureSample(t_shadow, s_shadow, in.tex_coords);
}