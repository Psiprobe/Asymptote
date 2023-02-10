use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use crate::{Instance, InstanceRaw};

const RADIUS_CHUNK:i32 = 32;
const RADIUS_VOXEL:i32 = 64;
pub struct ChunkManager{
    pub chunk_list:Vec<Chunk>,
}

impl ChunkManager{
    pub fn new(device:&wgpu::Device) -> Self{

        let mut chunk_list:Vec<Chunk> = Default::default();

        for x in 0..RADIUS_CHUNK{
            for y in 0..1{
                for z in 0..RADIUS_CHUNK{

                    let chunk_pos_x = x - RADIUS_CHUNK/2;
                    let chunk_pos_z = z - RADIUS_CHUNK/2;

                    if((chunk_pos_x*chunk_pos_x + chunk_pos_z*chunk_pos_z )as f32).sqrt()> (RADIUS_CHUNK/2) as f32{
                        continue
                    }

                    chunk_list.push(Chunk::default(chunk_pos_x, y, chunk_pos_z, true,device));

                }
            }
        }
        Self{
            chunk_list
        }
    }


    pub fn update()
    {
  
    }
}


pub struct Chunk{
    //pub position:[i32;3],
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

                
                    let position= cgmath::Vector3 { x:(x as i32 - RADIUS_VOXEL/2 + chunk_position[0] * RADIUS_VOXEL) as f32, y:0.0 as f32, z:(z as i32 - RADIUS_VOXEL/2 + chunk_position[2] * RADIUS_VOXEL) as f32} ;
                    let color= cgmath::Vector4 {x:0.0,y:1.0,z:0.0,w:1.0};
                    let mut is_active = false;
                    if y == 0 && (position.x % 64.0 == 0.0 || position.z % 64.0 == 0.0){
                        is_active = true;
                    }
                    voxel_data.resize((x * RADIUS_VOXEL + y + z ) as usize,Instance {
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
            //position:chunk_position,
            voxel_data,
            instance_len,
            buffer_data,
            is_active,
            is_selected:false,
        }

    }
}