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
var s_shadow: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let near = 100.0;
    let far = 5000.0;
    let depthSample = textureSample(t_shadow, s_shadow, in.tex_coords);
    //let depthSample = (depthSample + 1.0) / 2.0;
    let r = (2.0 * near) / (far + near - depthSample * (far - near));
    return vec4<f32>(vec3<f32>(depthSample),1.0);
}