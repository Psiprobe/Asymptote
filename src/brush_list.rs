use crate::chunk::*;
pub fn parse_draw(x:i32,y:i32,z:i32,first:[i32;3],last:[i32;3],color:[f32;4],brush_state: &BrushState) -> Option<[f32;4]>{
    match brush_state.id{

        0 => {
            Some(color)
        }
        
        1 => {
            Some(color)
        }

        _ => {None}
    }
}

