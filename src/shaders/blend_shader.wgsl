struct Data {
    offset: vec3<f32>,
};

struct TexelData{

    normal :vec2<f32>,

    grad_up   :f32,
    grad_right:f32,
    grad_left :f32,
    grad_down :f32,

    contrast  :f32,
    is_horizontal:bool,

    diffuse_sample:vec4<f32>,
    
}




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

//fxaa function set
//see https://zhuanlan.zhihu.com/p/373379681

fn luma(tex_sample:vec4<f32>)-> f32{
    return (tex_sample.x) + (tex_sample.y ) + (tex_sample.z);
}

fn calc_normal(grad_up:f32,grad_down:f32,grad_left:f32,grad_right:f32,is_horizontal:bool) -> vec2<f32>{
    var normal = vec2<f32>(0.0 , 0.0);

    if(is_horizontal){
        normal.y = sign(abs(grad_up)    - abs(grad_down));
    }else{
        normal.x = sign(abs(grad_right) - abs(grad_left));
    }
    
    return normal;
}



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

    var FXAA_ABSOLUTE_LUMA_THRESHOLD = 0.01;
    var DEPTH_TEST_OFFSET = 0.01;

    let diffuse = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let diffuse_left = textureSample(t_diffuse, s_diffuse, in.tex_coords,vec2<i32>(-1,0));
    let diffuse_right = textureSample(t_diffuse, s_diffuse, in.tex_coords,vec2<i32>(1,0));
    let diffuse_up = textureSample(t_diffuse, s_diffuse, in.tex_coords,vec2<i32>(0,1));
    let diffuse_down = textureSample(t_diffuse, s_diffuse, in.tex_coords,vec2<i32>(0,-1));
    
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

    var tex_data:TexelData;

    let up    = luma(diffuse_up);
    let right = luma(diffuse_right);
    let left  = luma(diffuse_left);
    let down  = luma(diffuse_down);
    let middle= luma(diffuse);

    tex_data.contrast  = max(middle,max(max(up,down),max(left,right)))-min(middle,min(min(up,down),min(left,right)));

    if(tex_data.contrast > FXAA_ABSOLUTE_LUMA_THRESHOLD){

        tex_data.grad_up   = up    - middle;
        tex_data.grad_right= right - middle;
        tex_data.grad_left = left  - middle;
        tex_data.grad_down = down  - middle;

        let lumaGradV = abs(tex_data.grad_down + tex_data.grad_up);
        let lumaGradH = abs(tex_data.grad_right + tex_data.grad_left);

        tex_data.is_horizontal = lumaGradV  > lumaGradH;

        tex_data.normal = calc_normal(tex_data.grad_up,tex_data.grad_down,tex_data.grad_left,tex_data.grad_right,tex_data.is_horizontal);

        if (tex_data.is_horizontal){
            if(tex_data.normal.x > 0.0){
                tex_data.diffuse_sample = diffuse_down;
            } 
            else {
                tex_data.diffuse_sample = diffuse_up;
            }
        }else if(tex_data.normal.y > 0.0){
            tex_data.diffuse_sample = diffuse_left;
        } 
        else {
            tex_data.diffuse_sample = diffuse_right;
        }
        
        
    }

    let blend =( diffuse.xyz + tex_data.diffuse_sample.xyz) * 0.5;

    if(depth.x - depth_up.x > DEPTH_TEST_OFFSET||depth.x - depth_down.x > DEPTH_TEST_OFFSET ||depth.x - depth_right.x > DEPTH_TEST_OFFSET|| depth.x - depth_left.x > DEPTH_TEST_OFFSET ){

        return vec4<f32>((diffuse.xyz + vec3<f32>(1.0,1.0,1.0))*0.5,1.0);
        //return diffuse;

    }

    else if(tex_data.contrast > FXAA_ABSOLUTE_LUMA_THRESHOLD){

        
        //if (tex_data.is_horizontal){
        //    if(tex_data.normal.y > 0.0){
        //        return vec4<f32>(1.0,0.0,0.0,1.0);
        //    } 
        //    else{
        //        return vec4<f32>(0.0,1.0,0.0,1.0);
        //    }
        //}else if(tex_data.normal.x > 0.0){
        //    return vec4<f32>(0.0,0.0,1.0,1.0);
        //} 
        //else{
        //    return vec4<f32>(0.0,0.0,0.0,1.0);
        //}

        //return vec4<f32>(blend,1.0);
        return diffuse;
        
    }
    else if(normal.x != normal_left.x || normal.y != normal_left.y ||normal.z != normal_left.z){

        return diffuse;

    }
    
    else if(normal.x != normal_down.x|| normal.y != normal_down.y||normal.z != normal_down.z){

        return diffuse;

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

