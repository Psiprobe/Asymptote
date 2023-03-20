use std::{cmp::max};


pub fn parse_draw(x:i32,y:i32,z:i32,first:[i32;3],last:[i32;3],mut color:[f32;4],id: i32) -> Option<[f32;4]>{
    match id{

        0 => {
            Some(color)
        }
        
        1 => {        
            Some([color[0],color[1],color[2],0.5])
        }

        _ => {None}
    }
}

