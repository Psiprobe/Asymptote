struct Data {
    offset: vec3<f32>,
};
@group(0)@binding(0)
var<uniform> data: Data;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    
    var out: VertexOutput;

    let x = model.position + data.offset;

    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(x , 1.0);

    return out;
}


@group(1)@binding(0)
var t_diffuse: texture_2d<f32>;
@group(1)@binding(1)
var s_diffuse: sampler;

@group(2)@binding(0)
var t_normal: texture_2d<f32>;
@group(2)@binding(1)
var s_normal: sampler;

@group(3) @binding(0)
var t_depth: texture_2d<f32>;
@group(3)@binding(1)
var s_depth: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    
    let depth = textureSample(t_depth, s_depth, in.tex_coords);
    let depth_left = textureSample(t_depth, s_depth, in.tex_coords,vec2<i32>(0,1));
    let depth_right = textureSample(t_depth, s_depth, in.tex_coords,vec2<i32>(0,-1));
    let depth_up = textureSample(t_depth, s_depth, in.tex_coords,vec2<i32>(1,0));
    let depth_down = textureSample(t_depth, s_depth, in.tex_coords,vec2<i32>(-1,0));

    let normal = textureSample(t_normal, s_normal, in.tex_coords);
    let normal_left = textureSample(t_normal, s_normal, in.tex_coords,vec2<i32>(0,1));
    let normal_right = textureSample(t_normal, s_normal, in.tex_coords,vec2<i32>(0,-1));
    let normal_up = textureSample(t_normal, s_normal, in.tex_coords,vec2<i32>(-1,0));
    let normal_down = textureSample(t_normal, s_normal, in.tex_coords,vec2<i32>(-1,0));
    
    let diffuse = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    if(normal.x != normal_left.x || normal.y != normal_left.y ||normal.z != normal_left.z){
        return diffuse * 5.0;
    }
    else if(normal.x != normal_down.x|| normal.y != normal_down.y||normal.z != normal_down.z){
        return diffuse * 5.0;

    }
    else if((depth.x - depth_down.x) < -0.1|| (depth.x - depth_left.x) < -0.1|| (depth.x - depth_right.x) < -0.1|| (depth.x - depth_up.x) < -0.1){
        return diffuse * 5.0;
    }

    else {
        return diffuse;
    }
    
    //else if(normal.x != normal_down.x|| normal.y != normal_down.y||normal.z != normal_down.z){
       // return vec4<f32>(1.0,1.0,1.0,1.0);

    //}
    //else if(normal.x != normal_right.x|| normal.y != normal_right.y||normal.z != normal_right.z){
        //return vec4<f32>(1.0,1.0,1.0,1.0);

    //}

}