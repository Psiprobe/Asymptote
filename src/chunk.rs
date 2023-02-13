use std::time::Duration;

use cgmath::num_traits::abs;
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use crate::{Instance, InstanceRaw, camera};
use crate::camera::*;

const RADIUS_CHUNK:i32 = 32;
const RADIUS_VOXEL:i32 = 64;
pub struct ChunkManager{
     
    pub chunk_list:Vec<Chunk>,
    pub w:f32,
    

}

impl ChunkManager{
    pub fn new(device:&wgpu::Device) -> Self{

        let mut chunk_list:Vec<Chunk> = Default::default();
        let w = 1.0;

        for x in 0..RADIUS_CHUNK{
            for y in 0..1{
                for z in 0..RADIUS_CHUNK{

                    let chunk_pos_x = x - RADIUS_CHUNK/2;
                    let chunk_pos_z = z - RADIUS_CHUNK/2;

                    if((chunk_pos_x*chunk_pos_x + chunk_pos_z*chunk_pos_z )as f32).sqrt()> (RADIUS_CHUNK/2) as f32{
                        continue
                    }

                    if chunk_pos_x == 0 && chunk_pos_z == 0 {
                        chunk_list.push(Chunk::quad(chunk_pos_x, y, chunk_pos_z, true,device));
                    }
                    else{
                        chunk_list.push(Chunk::default(chunk_pos_x, y, chunk_pos_z, true,device));
                    }

                }
            }
        }
        Self{
            chunk_list,
            w
        }
    }


    pub fn update(&mut self,device:&wgpu::Device,dt:Duration,camera:&Camera,mouse_pos_x:f64,mouse_pos_y:f64){

        let mouse_x = mouse_pos_x /2.0 - 480.0;

        let mouse_y = mouse_pos_y / 2.0 * 3.0 - 900.0;
        

        if self.w > 1.0{
            self.w =  -0.2;
        }
        else{
            self.w = self.w + (dt.as_secs_f32()*2.0);
        }

        let camera_mouse_eye = camera.eye + (camera.forward * mouse_y as f32)  + (camera.left * mouse_x as f32);
        let camera_mouse_target = camera.target + (camera.forward * mouse_y as f32) + (camera.left * mouse_x as f32);



        let camera_target_z = (31.0 - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.z - camera_mouse_target.z) + camera_mouse_eye.z;
        let camera_target_x = (31.0 - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.x - camera_mouse_target.x) + camera_mouse_eye.x;



        self.chunk_list.iter_mut().for_each(|c|{

            let v_range_x_min = c.position[0] * RADIUS_VOXEL - RADIUS_VOXEL/2;
            let v_range_x_max = c.position[0] * RADIUS_VOXEL + RADIUS_VOXEL/2;
            let v_range_z_min = c.position[2] * RADIUS_VOXEL - RADIUS_VOXEL/2;
            let v_range_z_max = c.position[2] * RADIUS_VOXEL + RADIUS_VOXEL/2;

            if camera_target_x > v_range_x_min as f32&& camera_target_x < v_range_x_max as f32{

                if camera_target_z > v_range_z_min as f32&& camera_target_z < v_range_z_max as f32{

                let instance_data = c.voxel_data.iter_mut().map(|v|{
                    if abs(v.position[0]) <=30.0 && abs(v.position[2]) <=30.0 {
                        v.color.y = self.w;
                        v.depth_strength = 0.0;
                        Instance::to_raw(v)
                    }
                    else{
                        Instance::to_raw(v)
                    }
                    
                }
                ).collect::<Vec<_>>();

                let buffer_data = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
                });

                c.buffer_data = buffer_data;

            }
            }
        })
    }
}


pub struct Chunk{
    pub position:[i32;3],
    pub voxel_data:Vec<Instance>,
    pub instance_len:u32,
    pub buffer_data:wgpu::Buffer,
    pub is_active: bool,
    pub is_selected: bool,
}

impl Chunk{

    pub fn default(x:i32,y:i32,z:i32,is_active:bool,device:&wgpu::Device)->Self{

        let chunk_position = [x,y,z];
        let mut voxel_data:Vec<Instance> = Default::default();
        let instance_data:Vec<InstanceRaw>;

        

        for x in 0 ..RADIUS_VOXEL{
            for y in 0 ..1{
                for z in 0 ..RADIUS_VOXEL{

                
                    let position= cgmath::Vector3 { x:(x as i32 - RADIUS_VOXEL/2 + chunk_position[0] * RADIUS_VOXEL) as f32, y:31.0 as f32, z:(z as i32 - RADIUS_VOXEL/2 + chunk_position[2] * RADIUS_VOXEL) as f32} ;
                    let color= cgmath::Vector4 {x:0.0,y:0.5,z:0.0,w:1.0};
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
            instance_len,
            buffer_data,
            is_active,
            is_selected:false,
        }

    }
    pub fn quad(x:i32,y:i32,z:i32,is_active:bool,device:&wgpu::Device)->Self{
        let chunk_position = [x,y,z];
        let mut voxel_data:Vec<Instance> = Default::default();
        let instance_data:Vec<InstanceRaw>;

        

        for x in 0 ..RADIUS_VOXEL{
            for y in 0 ..1{
                for z in 0 ..RADIUS_VOXEL{

                
                    let position= cgmath::Vector3 { x:(x as i32 - RADIUS_VOXEL/2 + chunk_position[0] * RADIUS_VOXEL) as f32, y:31.0 as f32, z:(z as i32 - RADIUS_VOXEL/2 + chunk_position[2] * RADIUS_VOXEL) as f32} ;
                    let color= cgmath::Vector4 {x:0.0,y:0.5,z:0.0,w:1.0};
                    let mut is_active = false;
                    if position.y == 31.0 {
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
            instance_len,
            buffer_data,
            is_active,
            is_selected:true,
        }
    }
}