use std::time::Duration;

use cgmath::Vector3;
use cgmath::num_traits::abs;
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use crate::{Instance, InstanceRaw};
use crate::camera::*;

const RADIUS_CHUNK:i32 = 32;
const RADIUS_VOXEL:i32 = 64;

pub struct ChunkManager{

    pub chunk_list:Vec<Chunk>,
    pub debug_mode:bool,
    pub w:f32,

    pub previous_indicator_first:[i32;3],
    pub previous_indicator_last:[i32;3],

}
#[derive(Copy, Clone,PartialEq)]
pub enum ChunkType {
    TerrainIndicator,
    UsrIndicator,
    Default,
}

impl ChunkManager{
    pub fn new(device:&wgpu::Device) -> Self{

        let mut chunk_list:Vec<Chunk> = Default::default();
        let mut debug_mode = true;

        let w = 1.0;

        for x in 0..RADIUS_CHUNK{
            for y in 0..1{
                for z in 0..RADIUS_CHUNK{

                    let chunk_pos_x = x - RADIUS_CHUNK/2;
                    let chunk_pos_z = z - RADIUS_CHUNK/2;

                    if((chunk_pos_x*chunk_pos_x + chunk_pos_z*chunk_pos_z )as f32).sqrt()> (RADIUS_CHUNK/2) as f32{
                        continue
                    }
                    
                    chunk_list.push(Chunk::indicator_cross(chunk_pos_x, y, chunk_pos_z, true,device));

                }
            }
        }

        Self{
            chunk_list,
            debug_mode,
            w,

            previous_indicator_first:Default::default(),
            previous_indicator_last:Default::default()
        }
    }
    pub fn draw(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],device:&wgpu::Device){

        let chunk_pos_x_first = ((first[0] + 32) as f32/64.0).floor() as i32;
        let chunk_pos_x_last= ((last[0] + 32) as f32/64.0).floor() as i32;

        let chunk_pos_y_first = ((first[1] + 32) as f32/64.0).floor() as i32;
        let chunk_pos_y_last= ((last[1] + 32) as f32/64.0).floor() as i32;
        
        let chunk_pos_z_first = ((first[2] + 32) as f32/64.0).floor() as i32;
        let chunk_pos_z_last= ((last[2] + 32) as f32/64.0).floor() as i32;

        for xx in chunk_pos_x_first..chunk_pos_x_last + 1{
            for yy in chunk_pos_y_first..chunk_pos_y_last + 1{
                for zz in chunk_pos_z_first..chunk_pos_z_last + 1{

                    self.chunk_list.iter_mut().for_each(|c|{
                        if c.position[0] == xx && c.position[1] == yy && c.position[2] == zz{

                            let mut x_first = xx * 64 - 32;
                            let mut x_last = xx * 64 + 31;

                            let mut y_first = yy * 64 - 32;
                            let mut y_last = yy * 64 + 31;

                            let mut z_first = zz * 64 - 32;
                            let mut z_last = zz * 64 + 31;

                            if x_first < first[0]   {x_first = first[0]}
                            if x_last  > last[0]    {x_last = last[0]}
                            if y_first < first[1]   {y_first = first[1]}
                            if y_last  > last[1]    {y_last = last[1]}
                            if z_first < first[2]   {z_first = first[2]}
                            if z_last  > last[2]    {z_last = last[2]}

                            let c_first = [x_first,y_first,z_first];
                            let c_last = [x_last,y_last,z_last];

                            c.draw(c_first, c_last, color, device)
                        }
                    })
            

                }
            }
        }
    }
    pub fn place(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],delete:bool,device:&wgpu::Device,chunk_type: ChunkType){

        let chunk_pos_x_first = ((first[0] + 32) as f32/64.0).floor() as i32;
        let chunk_pos_x_last= ((last[0] + 32) as f32/64.0).floor() as i32;

        let chunk_pos_y_first = ((first[1] + 32) as f32/64.0).floor() as i32;
        let chunk_pos_y_last= ((last[1] + 32) as f32/64.0).floor() as i32;
        
        let chunk_pos_z_first = ((first[2] + 32) as f32/64.0).floor() as i32;
        let chunk_pos_z_last= ((last[2] + 32) as f32/64.0).floor() as i32;

        for xx in chunk_pos_x_first..chunk_pos_x_last + 1{
            for yy in chunk_pos_y_first..chunk_pos_y_last + 1{
                for zz in chunk_pos_z_first..chunk_pos_z_last + 1{

                    let mut x_first = xx * 64 - 32;
                    let mut x_last = xx * 64 + 31;

                    let mut y_first = yy * 64 - 32;
                    let mut y_last = yy * 64 + 31;

                    let mut z_first = zz * 64 - 32;
                    let mut z_last = zz * 64 + 31;

                    if x_first < first[0]   {x_first = first[0]}
                    if x_last  > last[0]    {x_last = last[0]}
                    if y_first < first[1]   {y_first = first[1]}
                    if y_last  > last[1]    {y_last = last[1]}
                    if z_first < first[2]   {z_first = first[2]}
                    if z_last  > last[2]    {z_last = last[2]}

                    let c_first = [x_first,y_first,z_first];
                    let c_last = [x_last,y_last,z_last];

                    let mut chunk_modified_flag = false;

                    self.chunk_list.iter_mut().filter(|c| c.current_type == chunk_type).for_each(|c|{
                        if c.position[0] == xx && c.position[1] == yy && c.position[2] == zz{
                            c.place(c_first, c_last, color, delete , device);
                            chunk_modified_flag = true;
                        }
                    });
                    
                    if !chunk_modified_flag{
                        let mut chunk = Chunk::empty(xx, yy, zz, true, device, chunk_type);
                        chunk.place(c_first, c_last, color, delete, device);
                        self.chunk_list.push(chunk);
                    }
            

                }
            }
        }

    }
    pub fn on_click(){

    }
    pub fn temp_draw(){
        
    }
    pub fn update(&mut self,device:&wgpu::Device,queue: &wgpu::Queue,dt:Duration,camera:&Camera,camera_controller:&CameraController,mouse_pos_x:f64,mouse_pos_y:f64,texture_size: wgpu::Extent3d){

        let mouse_x = mouse_pos_x / 2.0 - texture_size.width as f64 / 4.0;

        let mouse_y = mouse_pos_y / 2.0 * 3.0 - texture_size.height as f64 / 4.0 * 3.0;
        

        if self.w < -0.5{
            self.w =  1.5;
        }
        else{
            self.w = self.w - (dt.as_secs_f32()*4.0);
        }

        let camera_mouse_eye = camera.eye + (camera.forward * mouse_y as f32)  + (camera.left * mouse_x as f32);
        let camera_mouse_target = camera.target + (camera.forward * mouse_y as f32) + (camera.left * mouse_x as f32);

        let mut camera_target_z = (31.0 - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.z - camera_mouse_target.z) + camera_mouse_eye.z;
        let mut camera_target_x = (31.0 - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.x - camera_mouse_target.x) + camera_mouse_eye.x;

        
        self.chunk_list.iter_mut().for_each(|c|{
            let v_range_x_min = c.position[0] * RADIUS_VOXEL - RADIUS_VOXEL/2;
            let v_range_x_max = c.position[0] * RADIUS_VOXEL + RADIUS_VOXEL/2;
            let v_range_z_min = c.position[2] * RADIUS_VOXEL - RADIUS_VOXEL/2;
            let v_range_z_max = c.position[2] * RADIUS_VOXEL + RADIUS_VOXEL/2;
            
            if camera_target_x > v_range_x_min as f32&& camera_target_x < v_range_x_max as f32 && camera_target_z > v_range_z_min as f32&& camera_target_z < v_range_z_max as f32{
                c.is_selected = true;
                if camera_controller.is_down_pressed{
                    camera_target_x = (c.position[0] * RADIUS_VOXEL) as f32;
                    camera_target_z = (c.position[2] * RADIUS_VOXEL) as f32;
                }
            }
            else {
                c.is_selected = false;
            }
        });
        





        let indicator_first = [camera_target_x as i32 - RADIUS_VOXEL/2, 31 ,camera_target_z as i32 - RADIUS_VOXEL/2];
        let indicator_last = [camera_target_x as i32 + RADIUS_VOXEL/2, 31 ,camera_target_z as i32 + RADIUS_VOXEL/2];
        
        let mut delete = false;
        let mut color = [0.0,self.w,0.0,self.w];

        if camera_controller.is_control_pressed{
            delete = true;
            color = [self.w,0.0,0.0,self.w];
        }

        if camera_controller.mouse_left_pressed{
            self.place(
            indicator_first,
            indicator_last,
            [1.0,1.0,1.0,1.0],
            delete,
            device,
            ChunkType::Default
            );
        }

        self.place(
            self.previous_indicator_first,
            self.previous_indicator_last,
            color,
            true,
            device,
            ChunkType::UsrIndicator
        );
        
        self.place(
            [indicator_first[0],indicator_first[1]+1,indicator_first[2]],
            [indicator_last[0],indicator_last[1]+1,indicator_last[2]],
            color,
            false,
            device,
            ChunkType::UsrIndicator
        );

        self.previous_indicator_first = [indicator_first[0],indicator_first[1]+1,indicator_first[2]];
        self.previous_indicator_last = [indicator_last[0],indicator_last[1]+1,indicator_last[2]];

        
    }
}


pub struct Chunk{
    pub position:[i32;3],
    pub voxel_data:Vec<Instance>,
    pub instance_data:Vec<InstanceRaw>,
    pub instance_len:u32,
    pub buffer_data:wgpu::Buffer,
    pub is_active: bool,
    pub is_selected: bool,
    pub need_update: bool,
    pub current_type :ChunkType,
}

impl Chunk{

    pub fn indicator_cross(x:i32,y:i32,z:i32,is_active:bool,device:&wgpu::Device)->Self{

        let chunk_position = [x,y,z];
        let mut voxel_data:Vec<Instance> = Default::default();
        let instance_data:Vec<InstanceRaw>;

        

        for x in 0 ..RADIUS_VOXEL{
            for y in 0 ..1{
                for z in 0 ..RADIUS_VOXEL{

                
                    let position= cgmath::Vector3 { x:(x as i32 - RADIUS_VOXEL/2 + chunk_position[0] * RADIUS_VOXEL) as f32, y:31.0 as f32, z:(z as i32 - RADIUS_VOXEL/2 + chunk_position[2] * RADIUS_VOXEL) as f32} ;
                    let color= cgmath::Vector4 {x:0.0,y:0.05,z:0.0,w:1.0};
                    let mut is_active = false;
                    if position.y == 31.0 && (abs(position.x % 64.0) == 31.0|| abs(position.z % 64.0) == 31.0){
                        is_active = true;
                    }
                    voxel_data.push(Instance {
                        is_active,
                        position,
                        color,
                        depth_strength:0.0,
                        normal_strength:1.0,
                    });           
                }
            }  
        }
        
        instance_data = voxel_data.iter().filter(|v| v.is_active).map(Instance::to_raw).collect::<Vec<_>>();
        let buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });
        let instance_len = instance_data.len() as u32;

        Self{
            position:chunk_position,
            voxel_data,
            instance_data,
            instance_len,
            buffer_data,
            is_active,
            is_selected:false,
            need_update:false,
            current_type:ChunkType::TerrainIndicator,
        }

    }
    pub fn empty(x:i32,y:i32,z:i32,is_active:bool,device:&wgpu::Device,chunk_type:ChunkType)->Self{

        let chunk_position = [x,y,z];
        let mut voxel_data:Vec<Instance> = Default::default();
        let instance_data:Vec<InstanceRaw>;
        
        instance_data = voxel_data.iter().filter(|v| v.is_active).map(Instance::to_raw).collect::<Vec<_>>();
        let buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });
        let instance_len = instance_data.len() as u32;

        Self{
            position:chunk_position,
            voxel_data,
            instance_data,
            instance_len,
            buffer_data,
            is_active,
            is_selected:true,
            need_update:false,
            current_type:chunk_type,
        }
    }

    pub fn draw(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],device:&wgpu::Device){

        self.voxel_data.iter_mut().filter(|v| v.is_active).for_each(|v|{

            if v.position[0] >= first[0] as f32&& v.position[0] <= last[0] as f32{
                if v.position[1] >= first[1] as f32&& v.position[1] <= last[1] as f32{
                    if v.position[2] >= first[2] as f32&& v.position[2] <= last[2] as f32{

                        v.color = cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};

                    }
                }
            }
        });

        self.instance_data = self.voxel_data.iter().filter(|v| v.is_active).map(Instance::to_raw).collect::<Vec<_>>();
        self.buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });
        self.current_type = ChunkType::Default;
        self.instance_len = self.instance_data.len() as u32;

    }


    pub fn place(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],delete:bool,device:&wgpu::Device){


        self.voxel_data
        .retain(|v|

            (v.position[0] < first[0] as f32 || v.position[0] > last[0] as f32) ||
            (v.position[1] < first[1] as f32 || v.position[1] > last[1] as f32) ||
            v.position[2] < first[2] as f32 || v.position[2] > last[2] as f32

        );
        

        if !delete{

            for x in first[0] ..last[0] + 1{
                for y in first[1] ..last[1] + 1{
                    for z in first[2] ..last[2] + 1{
                    
                        let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
                        let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
                        let is_active = true;
    
                        self.voxel_data.push(Instance {
                            is_active,
                            position,
                            color,
                            depth_strength:0.0,
                            normal_strength:1.0,
                        });          
    
                    }
                }  
            }
        }


        self.instance_data = self.voxel_data.iter().filter(|v| v.is_active).map(Instance::to_raw).collect::<Vec<_>>();

        self.buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });

        self.instance_len = self.instance_data.len() as u32;


    }

}