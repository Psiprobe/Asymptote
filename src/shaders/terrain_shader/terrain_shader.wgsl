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
    @location(9) color: vec4<f32>,
    @location(10) depth_strength:f32,
    @location(11) normal_strength:f32,
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
    let rgb_color = vec3<f32>(instance.color[0],instance.color[1],instance.color[2]);
    let diffuse_color = rgb_color * diffuse_strength;

    if( (model_matrix *vec4<f32>(model.position, 1.0)).xyz.z >= light.position.z / 3.0 - 4.0 && (model_matrix *vec4<f32>(model.position, 1.0)).xyz.z <= light.position.z /3.0 + 4.0)
    {
        //out.color = vec4<f32>(0.5,1.0,0.25,1.0);
        out.color = vec4<f32>(diffuse_color[0],diffuse_color[1],diffuse_color[2],instance.color[3]);
    }
    else if ((model_matrix *vec4<f32>(model.position, 1.0)).xyz.x >= light.position.z / 3.0 - 4.0 && (model_matrix *vec4<f32>(model.position, 1.0)).xyz.x <= light.position.z /3.0 + 4.0){
        //out.color = vec4<f32>(0.5,1.0,0.25,1.0);
        out.color = vec4<f32>(diffuse_color[0],diffuse_color[1],diffuse_color[2],instance.color[3]);
    }else{
        out.color = vec4<f32>(diffuse_color[0],diffuse_color[1],diffuse_color[2],instance.color[3]);
    }
    return out;
}

// Fragment

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { 

    return vec4<f32>(in.color);
}