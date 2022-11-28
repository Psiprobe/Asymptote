struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0)@binding(0)
var<uniform> camera: CameraUniform;
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

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;

    out.clip_position =  camera.view_proj * model_matrix *vec4<f32>(model.position, 1.0);

    let light_dir = normalize(light.position - (model_matrix *vec4<f32>(model.position, 1.0)).xyz);
    var diffuse_strength = max(dot(model.normal, light_dir),0.0);
    let rgb_color = vec3<f32>(model.color[0],model.color[1],model.color[2]);
    let diffuse_color = rgb_color *  diffuse_strength * diffuse_strength;

    if( (model_matrix *vec4<f32>(model.position, 1.0)).xyz.y == 1.0 || (model_matrix *vec4<f32>(model.position, 1.0)).xyz.y == 2.0){
        out.color = vec4<f32>(0.0,1.0,0.0,1.0);
    }
    else{
        out.color = vec4<f32>(diffuse_color[0],diffuse_color[1],diffuse_color[2],model.color[3]);
    }
    return out;
}

// Fragment

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { 
    return vec4<f32>(in.color);
}