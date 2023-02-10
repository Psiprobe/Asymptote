use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use crate::State;
use crate::{Instance, InstanceRaw};

const RADIUS:i32 = 32;

pub struct ChunkManager{
    pub chunk_list:Vec<Chunk>,
}

impl ChunkManager{
    pub fn new(device:&wgpu::Device) -> Self{

        let mut chunk_list:Vec<Chunk> = Default::default();

        for x in 0..RADIUS{
            for y in 0..1{
                for z in 0..RADIUS{

                    chunk_list.push(Chunk::default(x - RADIUS/2, y, z - RADIUS/2, true,device));

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
    pub position:[i32;3],
    pub voxel_data:Vec<Instance>,
    pub instance_data:Vec<InstanceRaw>,
    pub buffer_data:wgpu::Buffer,
    pub is_active: bool,
    pub is_selected: bool,
}

impl Chunk{

    pub fn default(x:i32,y:i32,z:i32,is_active:bool,device:&wgpu::Device)->Self{

        let chunk_position = [x,y,z];
        let mut voxel_data:Vec<Instance> = Default::default();
        let mut instance_data:Vec<InstanceRaw> = Default::default();

        

        for x in 0 ..RADIUS{
            for y in 0 ..RADIUS{
                for z in 0 ..RADIUS{

                
                    let position= cgmath::Vector3 { x:(x as i32 - RADIUS/2 + chunk_position[0] * RADIUS) as f32, y:0.0 as f32, z:(z as i32 - RADIUS/2 + chunk_position[2] * RADIUS) as f32} ;
                    let color= cgmath::Vector4 {x:0.0,y:1.0,z:0.0,w:1.0};
                    let mut is_active = false;
                    if y == 0 && (position.x % 64.0 == 0.0 || position.z % 64.0 == 0.0){
                        is_active = true;
                    }
                    voxel_data.resize((x * RADIUS * RADIUS + y * RADIUS + z ) as usize,Instance {
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

        Self{
            position:chunk_position,
            voxel_data,
            instance_data,
            buffer_data,
            is_active,
            is_selected:false,
        }

    }
}