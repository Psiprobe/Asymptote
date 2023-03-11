use crate::{chunk::*, Instance};

pub fn parse_place(x:i32,y:i32,z:i32,first:[i32;3],last:[i32;3],color:[f32;4],model_state: &mut ModelState) -> Option<Instance>{
        match model_state.id {

        0 =>{

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let mut color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
                
            let xx:i32;
            let zz:i32;
                
            if x<0{
                xx = x-32;
            }
            else{
                xx = x;
            }
            if z<0{
                zz = z-32;
            }
            else{
                zz = z;
            }
            if (xx.abs() % 64 > 31 && zz.abs() % 64 > 31)||(xx.abs() % 64 <= 31 && zz.abs() % 64 <= 31){
                color[0] = 0.05;
                color[1] = 0.65;
                color[2] = 0.05;
            }
        
            let mut normal = cgmath::Vector3 { x:0.0, y:0.0, z:0.0};
        
            if y == first[1]{
                normal[1] = -1.0;
            }
            if y == last[1]{
                normal[1] = 1.0;
            }
            
        
            
        
            Some(Instance {
                position,
                color,
                normal,
                depth_strength:0.5,
                normal_strength:1.0,
            })
        }

        1 =>{
            
            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let mut normal = cgmath::Vector3 { x:0.0, y:0.0, z:0.0};
        
            if y == first[1]{
                normal[1] = -1.0;
            }
            if y == last[1]{
                normal[1] = 1.0;
            }

            
        
            
        
            Some(Instance {
                position,
                color,
                normal,
                depth_strength:0.5,
                normal_strength:1.0,
            })
        }
        2 =>{

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
        
            let mut normal = cgmath::Vector3 { x:0.0, y:0.0, z:0.0};
        
            if x == first[0]{
                normal[0] = -1.0;
            }
            if x == last[0] {
                normal[0] = 1.0;
            }
            if z == first[2] {
                normal[2] = -1.0;
            }
            if z == last[2]{
                normal[2] = 1.0;
            }

            if normal[0] == 0.0 &&normal[2] == 0.0{
                if y == first[1]{
                    normal[1] = -1.0;
                }
                if y == last[1]{
                    normal[1] = 1.0;
                }
            }
            
        
            Some(Instance {
                position,
                color,
                normal,
                depth_strength:0.5,
                normal_strength:1.0,
            })
        }
        _ => None
    }
}

