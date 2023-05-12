use cgmath::InnerSpace;
use cgmath::Point3;
use cgmath::Rad;
use cgmath::Angle;

use crate::{Instance, VoxelType};


pub fn parse_place(x:i32,y:i32,z:i32,first:[i32;3],last:[i32;3],color:[f32;4],id:i32) -> Option<Instance>{

        match id {

        0 =>{

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            if to_ellipsoid(first, last, [x,y,z]){
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else {
                None
            }
        }

        1 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let line_first= [(first[0]+last[0])/2 , first[1],(first[2]+last[2])/2];
            let line_last = [(first[0]+last[0])/2 , last[1],(first[2]+last[2])/2];

            

            if to_line(line_first,line_last,[x,y,z]){
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }

        }

        2 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let up_first= [first[0], (first[1] + last[1])/2,first[2]];
            let up_last = [last[0], last[1],last[2]];

            let down_first= [first[0],first[1] - (last[1] - first[1])/2 - 5,first[2]];
            let down_last = [last[0], (first[1] + last[1])/2 + 5,last[2]];

            let line_first = [first[0] , first[1],(first[2]+last[2])/2];
            let line_last = [last[0] , first[1],(first[2]+last[2])/2];

            if (to_ellipsoid(up_first,up_last,[x,y,z]) && (y>=((first[1] + last[1])/2 + last[1])/2||x>=(first[0] + last[0])/2))
            || (to_ellipsoid(down_first,down_last,[x,y,z]) && ( x < (first[0] + last[0])/2))
            ||to_line(line_first,line_last,[x,y,z]){
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }


        }

        3 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let up_first= [first[0], (first[1] + last[1])/2,first[2]];
            let up_last = [last[0], last[1],last[2]];

            let down_first= [first[0],first[1],first[2]];
            let down_last = [last[0], (first[1] + last[1])/2 + 5,last[2]];

            if (to_ellipsoid(up_first,up_last,[x,y,z]) && (y>=((first[1] + last[1])/2 + last[1])/2||x>=(first[0] + last[0])/2))
            || (to_ellipsoid(down_first,down_last,[x,y,z]) && ( y<=first[1] + (last[1] - first[1])/4||x>=(first[0] + last[0])/2))
            {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }


        }

        4 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let first_1 =[((first[0]+last[0])/2 + last[0])/2 , first[1],(first[2]+last[2])/2];
            let last_1 = [((first[0]+last[0])/2 + last[0])/2, last[1] ,(first[2]+last[2])/2];

            let first_2= last_1;
            let last_2 = [first[0],first[1]+(last[1]-first[1])/4,(first[2]+last[2])/2];

            let first_3= last_2;
            let last_3 = [last[0],first[1]+(last[1]-first[1])/4,(first[2]+last[2])/2];

            if to_line(first_1,last_1,[x,y,z])||to_line(first_2,last_2,[x,y,z])||to_line(first_3,last_3,[x,y,z])
            {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }
        }

        5 =>{

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let first_1 =[first[0] + (last[1] - first[1])/16 , last[1] - 2,(first[2]+last[2])/2];
            let last_1 = [last[0], last[1] - 2,(first[2]+last[2])/2];

            let first_2 = first_1;
            let last_2 = [first[0] + 2,last[1]-(last[1]-first[1])/2,(first[2]+last[2])/2];

            let first_3 = last_2;
            let last_3 = [(last[0]+first[0])/2,last[1]-(last[1]-first[1])/2 ,(first[2]+last[2])/2];

            let down_first= [first[0],first[1],first[2]];
            let down_last = [last[0], last[1] - (last[1]-first[1])/2 + 3,last[2]];

            if to_line(first_1,last_1,[x,y,z]) && x > first[0] + (last[1] - first[1])/15 && x < last[0] - (last[1] - first[1])/16
            || to_line(first_2,last_2,[x,y,z]) && y >= last[1]-(last[1]-first[1])/2
            || to_line(first_3,last_3,[x,y,z]) && x <= (last[0]+first[0])/2
            || (to_ellipsoid(down_first,down_last,[x,y,z]) && ( y<=(first[1] + last[1] - (last[1]-first[1])/2)/2||x>=(first[0] + last[0])/2))
            {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }


        }

        6 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};
            
            let up_first= [first[0], (first[1] + last[1])/2,first[2]];
            let up_last = [last[0], last[1],last[2]];

            let down_first= [first[0],first[1],first[2]];
            let down_last = [last[0], last[1] - (last[1]-first[1])/2 + 3,last[2]];

            let first_1 =[first[0] + 2,last[1],(first[2]+last[2])/2];
            let last_1 = [first[0] + 2,first[1],(first[2]+last[2])/2];

            if to_ellipsoid(down_first, down_last, [x,y,z])
            || to_ellipsoid(up_first, up_last, [x,y,z]) && y>=((first[1] + last[1])/2 + last[1])/2
            || to_line(first_1,last_1,[x,y,z]) && y<=((first[1] + last[1])/2 + last[1])/2 && y >= first[1]+(last[1]-first[1])/4
            {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }

        }

        7 => {
            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let first_1 =[first[0] , last[1] - 2,(first[2]+last[2])/2];
            let last_1 = [last[0], last[1] - 2,(first[2]+last[2])/2];

            let first_2 = last_1;
            let last_2 = [(first[0]+last[0])/2 , first[1] + 2,(first[2]+last[2])/2];

            if to_line(first_1,last_1,[x,y,z])||to_line(first_2,last_2,[x,y,z]){
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }

        }

        8 => {
            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};

            let up_first= [first[0], (first[1] + last[1])/2,first[2]];
            let up_last = [last[0], last[1],last[2]];

            let down_first= [first[0],first[1],first[2]];
            let down_last = [last[0], (first[1] + last[1])/2 + 5,last[2]];

            if to_ellipsoid(up_first,up_last,[x,y,z])
            || to_ellipsoid(down_first,down_last,[x,y,z])
            {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }
        }

        9 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.0, y:1.0, z:0.0};
            
            let up_first= [first[0], (first[1] + last[1])/2,first[2]];
            let up_last = [last[0], last[1],last[2]];

            let down_first= [first[0],first[1],first[2]];
            let down_last = [last[0], last[1] - (last[1]-first[1])/2 + 3,last[2]];

            let first_1 =[last[0] - 4, last[1],(first[2]+last[2])/2];
            let last_1 = [last[0] - 4,first[1],(first[2]+last[2])/2];

            if to_ellipsoid(down_first, down_last, [x,y,z]) && y<=(last[1] - first[1])/4 + first[1]
            || to_ellipsoid(up_first, up_last, [x,y,z])
            || to_line(first_1,last_1,[x,y,z]) && y<=((first[1] + last[1])/2 + last[1])/2 && y >= first[1]+(last[1]-first[1])/4
            {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }
        }


        10 =>{

            if x == first[0]||x == last[0]||y == first[1]||y == last[1]||z == first[2]||z == last[2]{
                let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
                let mut color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
                
                    
                let xx:i32;
                let zz:i32;
                    
                if x<0{
                    xx = x-64;
                }
                else{
                    xx = x;
                }
                if z<0{
                    zz = z-64;
                }
                else{
                    zz = z;
                }
                if (xx.abs() % 128 > 63 && zz.abs() % 128 > 63)||(xx.abs() % 128 <= 63 && zz.abs() % 128 <= 63){
                    color[0] = 1.0;
                    color[1] = 1.0;
                    color[2] = 1.0;
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
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }

            
        }

        11 =>{
            
            if x == first[0]||x == last[0]||y == first[1]||y == last[1]||z == first[2]||z == last[2]{
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
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }
        }
        12 =>{

            if x == first[0]||x == last[0]||y == first[1]||y == last[1]||z == first[2]||z == last[2]{

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
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })           
            }
            else{
                None
            }

            
        }

        13 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};
            let normal = cgmath::Vector3 { x:0.03 * Rad::sin(Rad(position.z*position.z*position.z*position.x* position.x)), y:1.0, z:0.03 * Rad::sin(Rad(position.z*position.z*position.x*position.x* position.x))};

            if position.y as i32 == first[1] {
                Some(Instance {
                    position,
                    color,
                    normal,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }

            else{
                None
            }

            
        }

        14 => {

            let position= cgmath::Vector3 { x:x as f32, y:y as f32, z:z as f32};
            let color= cgmath::Vector4 {x:color[0],y:color[1],z:color[2],w:color[3]};

            if to_ellipsoid(first, last, [x,y,z]){

                let normal = [x as f32 - (first[0]+last[0]) as f32 /2.0 ,y as f32 - (first[1]+last[1]) as f32/2.0 ,z as f32 - (first[2]+last[2]) as f32/2.0];
                let normalize= cgmath::Vector3 {x:normal[0],y:normal[1],z:normal[2]}.normalize();
                Some(Instance {
                    position,
                    color,
                    normal:normalize,
                    depth_strength:0.5,
                    normal_strength:1.0,
                    light_strength:0.0,
                    current_type: VoxelType::Object,
                })
            }
            else{
                None
            }
        }

        _ => None
    }
}

fn to_line(first:[i32;3],last:[i32;3],point:[i32;3]) -> bool{

    let first_point = cgmath::point3(first[0] as f32,first[1] as f32,first[2] as f32);
    let last_point = cgmath::point3(last[0] as f32,last[1] as f32,last[2] as f32);
    let location_point = cgmath::point3(point[0] as f32,point[1] as f32,point[2] as f32);

    let vector_1 = last_point - first_point;

    let vector_2 = last_point - location_point;

    let vector_3 = vector_1.cross(vector_2);

    let a = (vector_3.x.powf(2.0) + vector_3.y.powf(2.0) + vector_3.z.powf(2.0)).abs();
    let b = (vector_1.x.powf(2.0) + vector_1.y.powf(2.0) + vector_1.z.powf(2.0)).abs();

    let distance = a.sqrt() / b.sqrt();

    if distance < 3.0 && point[2] == (first[2] + last[2])/2{
        true
    }
    else{
        false
    }
}

fn to_ellipsoid(first:[i32;3],last:[i32;3],point:[i32;3]) -> bool{
    
    let a = ((last[0] - first[0])/2) as f32;
    let b = ((last[1] - first[1])/2) as f32;
    let c = ((last[2] - first[2])/2) as f32;

    let oriented_point0 = - a + point[0] as f32 - first[0] as f32;
    let oriented_point1 = - b + point[1] as f32 - first[1] as f32;
    let oriented_point2 = - c + point[2] as f32 - first[2] as f32;

    let inside_diameter = 5.0;

    let aa = a - inside_diameter;
    let bb = b - inside_diameter;
    let cc = c - inside_diameter;

    let point0 = (oriented_point0 * oriented_point0) as f32;
    let point1 = (oriented_point1 * oriented_point1) as f32;
    let point2 = (oriented_point2 * oriented_point2) as f32;

    let result_inside = point0 / (a * a) + point1 / (b * b) + point2 / (c * c);
    let result_outside =  point0 / (aa * aa) + point1 / (bb * bb) + point2 / (cc * cc);

    if result_outside > 1.0 && result_inside <1.0{
        true
    }
    else{
        false
    }
}
