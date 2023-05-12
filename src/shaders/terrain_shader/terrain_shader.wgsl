struct CameraUniform {
    view_proj: mat4x4<f32>,
    position:vec3<f32>,
    eye:vec3<f32>
};
@group(0)@binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    color: vec4<f32>,
}
struct Lights {
    data: array<Light,512>,
}
@group(1) @binding(0)
var<uniform> lights: Lights;

struct VertexInput {
    @location(0) position: vec3<f32>,
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
    @location(10) normal: vec3<f32>,
    @location(11) depth_strength:f32,
    @location(12) normal_strength:f32,
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

    let view = camera.eye - camera.position;
    let voxel_position = model_matrix *vec4<f32>(model.position, 1.0);

    let plane_a = view.x;
    let plane_b = view.y;
    let plane_c = view.z;
    let plane_d = - ( camera.eye.x * view.x + camera.eye.y * view.y + camera.eye.z * view.z );

    let distance_to_plane = abs(plane_a * voxel_position.x + plane_b * voxel_position.y + plane_c * voxel_position.z + plane_d)/ sqrt(plane_a * plane_a + plane_b * plane_b + plane_c * plane_c);

    let near = 1300.0;
    let far = 3750.0;

    
    //out.clip_position.z = distance_to_plane;
    //out.clip_position.w = 1.0 / distance_to_plane;

    out.clip_position =  camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
   
    var i = 0;

    var result:vec3<f32>;

    while i < 512 && lights.data[i].color.w > 0.0{

        
        let light_dir = normalize(lights.data[i].position - (model_matrix *vec4<f32>(model.position, 1.0)).xyz);
        let view_dir = normalize(camera.eye - (model_matrix *vec4<f32>(model.position, 1.0)).xyz);
        let reflect_dir = reflect(-light_dir, instance.normal);

        let ambient_strength = 0.1;
        var diffuse_strength = max(dot(instance.normal, light_dir),0.0);
        let specular_strength = 0.5;

        let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);

        let distance = length(lights.data[i].position - (model_matrix * vec4<f32>(model.position, 1.0)).xyz);

        var attenuation = 1.0 / (1.0 + 0.0014 * distance + 0.000007 * (distance * distance));


        if lights.data[i].color.w < 1.0{
            //point light

            attenuation *= lights.data[i].color.w;

            let ambient = ambient_strength * lights.data[i].color.xyz* attenuation;
            let diffuse = diffuse_strength * lights.data[i].color.xyz * attenuation;
            let specular = specular_strength * spec * lights.data[i].color.xyz * attenuation;

            result = result + (ambient + diffuse + specular) * lights.data[i].color.w;
        }
        else{
            //dir light
            let ambient = ambient_strength * lights.data[i].color.xyz;
            let diffuse = diffuse_strength * lights.data[i].color.xyz;
            let specular = specular_strength * spec * lights.data[i].color.xyz;

            result = result + ambient + diffuse;
            
        }
        i++;
    }

    let object_color = vec3<f32>(instance.color[0],instance.color[1],instance.color[2]);
    let color = result * object_color;

    //out.clip_position.w = (distance_to_plane - near) / (far - near);
    out.color = vec4<f32>(color[0],color[1],color[2],instance.color[3]);
    
    return out;
}

// Fragment

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { 

    return vec4<f32>(in.color);
}