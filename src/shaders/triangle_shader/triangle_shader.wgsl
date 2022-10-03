struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    flag: vec3<f32>,
}
@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    if (light.flag[0] == 0.0){

        let light_dir = normalize(light.position - model.position);
        let diffuse_strength = max(dot(model.normal, light_dir), 0.0);
        let rgb_color = vec3<f32>(model.color[0],model.color[1],model.color[2]);
        let diffuse_color = rgb_color * diffuse_strength;

        out.color = vec4<f32>(diffuse_color[0],diffuse_color[1],diffuse_color[2],model.color[3]);

    }
    else{
        out.color = vec4<f32>((model.normal[0]+1.0)/2.0,(model.normal[1]+1.0)/2.0,(model.normal[2]+1.0)/2.0,1.0);
    }
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}