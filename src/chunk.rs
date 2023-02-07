use cgmath::num_traits::*;

pub struct ChunkManager{
    pub chunk_list:Vec<Chunk>,
}

impl ChunkManager{
    pub fn new() -> Self{
        let chunk_list = (-128..128).flat_map(|z| {
            //(-0..1).flat_map(move |y| {
                (-128..128).map(move |x| {

                    if ((x*x + z*z) as f32).sqrt() > 128.0 {
                        Chunk::default(x,0,z,false)

                    }
                    else{
                        Chunk::default(x,0,z,true)
                    }

                }).filter(|x| x.is_active)
            })
            .collect::<Vec<_>>();

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
    pub voxeldata:[[[Voxel;8];8];8],
    pub is_active: bool,
    pub is_selected: bool,
}


#[derive(Copy)]
#[derive(Clone)]
pub struct Voxel{
    pub position:[i32;3],
    pub color:[f32;4],
    pub normal:[f32;3],
    pub is_active: bool,
    pub depth_strength:f32,
    pub normal_strength:f32,
}

impl Voxel{
    pub fn new(x:i32,y:i32,z:i32)->Self{
        Self{
            position:[x,y,z],
            is_active:false,
            normal:[0.0,1.0,0.0],
            color:[1.0,1.0,1.0,1.0],
            depth_strength: 0.5,
            normal_strength: 1.0,

        }
    }
}

impl Chunk{

    

    pub fn default(x:i32,y:i32,z:i32,is_active:bool)->Self{

        let position = [x,y,z];
        let mut voxeldata= [[[Voxel::new(0, 0, 0);8];8];8];

        

        for x in 0..8{
            for y in 0..8{
                for z in 0..8{

                    let mut chunk_xline = false;
                    let mut chunk_zline = false;

                    if position[0]<0{
                        if (position[0]-1)%8 == 0{
                            chunk_xline = true;
                        }
                    }
                    else{
                        if position[0]%8 == 0 {
                            chunk_xline = true;
                        }
                    }

                    if position[2]<0{
                        if (position[2]-1)%8 == 0{
                            chunk_zline = true;
                        }
                    }
                    else{
                        if position[2]%8 == 0 {
                            chunk_zline = true;
                        }
                    }



                    if y == 0{

                        if (x == 0 && chunk_xline) || (z == 0 && chunk_zline){

                            voxeldata[x][y][z] = Voxel::new(x as i32, y as i32, z as i32);
                            voxeldata[x][y][z].color = [0.0,1.0,0.0,1.0];
                            voxeldata[x][y][z].depth_strength = 0.0;
                            voxeldata[x][y][z].is_active = true;

                        }
                        //cross symbol

                    }
                    
                    else{
                        voxeldata[x][y][z] = Voxel::new(x as i32, y as i32, z as i32);
                        voxeldata[x][y][z].is_active = false;
                    }
                   

                }
            }
        }

        Self{
            position,
            voxeldata,
            is_active,
            is_selected:false,
        }

    }
}