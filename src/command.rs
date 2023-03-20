use crate::State;
use crate::chunk::ChunkType;
use crate::shell;
use shell::Message::{ServerLog,ChatMessage};
pub struct Descriptor{
    pub text: String,
}
impl Descriptor{
    
    pub fn new() -> Self{
        Self { 
            text: Default::default()
        }
    }

    pub fn parse_command(&self, state: &mut State){
        
        let mut s = String::from(&self.text);

        let v: Vec<&str> = s.split(" ").collect();

        let ch = v[0].chars().nth(0);

        if ch == Some('/') {
            let mut i = 1;

            match v[0] as &str{

                "/place" =>{

                    let mut first = [0,0,0];
                    let mut last = [0,0,0];
                    let mut color = [0.0,0.0,0.0,0.0];
                    let mut id = 0;
                    let delete = false;

                    if v.len() < 12 {
                        s = String::from("Insufficient args");
                    }
                    else{
                        while i < 12{

                            if i >= 1 && i <=3{
                                first[i-1] = v[i].parse::<i32>().unwrap();
                            }

                            if i >= 4 && i <=6{
                                last[i-4] = v[i].parse::<i32>().unwrap();
                            }

                            if i >= 7 && i <= 10{
                                color[i-7] = v[i].parse::<f32>().unwrap();
                            }

                            if i == 11{
                                id = v[i].parse::<i32>().unwrap();
                            }

                            i = i + 1;

                        }

                        s = "Model placed at ".to_owned() 

                        + &first[0].to_string()+ &' '.to_string()
                        + &first[1].to_string()+ &' '.to_string()
                        + &first[2].to_string()+ &' '.to_string()

                        + &"; ".to_owned() 
            
                        + &last[0].to_string()+ &' '.to_string()
                        + &last[1].to_string()+ &' '.to_string()
                        + &last[2].to_string()+ &' '.to_string()

                        + &" ID =".to_owned() 
                        + &id.to_string()+ &' '.to_string()
                        
                    }

                    state.chunk_manager.place(first, last, color, delete, &state.device, ChunkType::Default,id, &mut state.iced_state);

                }

                "/delete" =>{

                    let mut first = [0,0,0];
                    let mut last = [0,0,0];
                    let mut color = [0.0,0.0,0.0,0.0];
                    let mut id = 0;
                    let delete = true;

                    if v.len() < 12 {
                        s = String::from("Insufficient args");
                    }
                    else{
                        while i < 12{

                            if i >= 1 && i <=3{
                                first[i-1] = v[i].parse::<i32>().unwrap();
                            }

                            if i >= 4 && i <=6{
                                last[i-4] = v[i].parse::<i32>().unwrap();
                            }

                            if i >= 7 && i <= 10{
                                color[i-7] = v[i].parse::<f32>().unwrap();
                            }

                            if i == 11{
                                id = v[i].parse::<i32>().unwrap();
                            }

                            i = i + 1;

                        }

                        s = "Model deleted at ".to_owned() 

                        + &first[0].to_string()+ &' '.to_string()
                        + &first[1].to_string()+ &' '.to_string()
                        + &first[2].to_string()+ &' '.to_string()

                        + &"; ".to_owned() 
            
                        + &last[0].to_string()+ &' '.to_string()
                        + &last[1].to_string()+ &' '.to_string()
                        + &last[2].to_string()+ &' '.to_string()

                        + &" ID =".to_owned() 
                        + &id.to_string()+ &' '.to_string()
                        
                    }

                    state.chunk_manager.place(first, last, color, delete, &state.device, ChunkType::Default, id,&mut state.iced_state);

                }

                "/draw" =>{

                    let mut first = [0,0,0];
                    let mut last = [0,0,0];
                    let mut color = [0.0,0.0,0.0,0.0];
                    let mut id = 0;

                    if v.len() < 12 {
                        s = String::from("Insufficient args");
                    }
                    else{
                        while i < 12{

                            if i >= 1 && i <=3{
                                first[i-1] = v[i].parse::<i32>().unwrap();
                            }

                            if i >= 4 && i <=6{
                                last[i-4] = v[i].parse::<i32>().unwrap();
                            }

                            if i >= 7 && i <= 10{
                                color[i-7] = v[i].parse::<f32>().unwrap();
                            }

                            if i == 11{
                                id = v[i].parse::<i32>().unwrap();
                            }

                            i = i + 1;

                        }

                        s = "Model drawn at ".to_owned() 

                        + &first[0].to_string()+ &' '.to_string()
                        + &first[1].to_string()+ &' '.to_string()
                        + &first[2].to_string()+ &' '.to_string()

                        + &"; ".to_owned() 
            
                        + &last[0].to_string()+ &' '.to_string()
                        + &last[1].to_string()+ &' '.to_string()
                        + &last[2].to_string()+ &' '.to_string()

                        + &" ID =".to_owned() 
                        + &id.to_string()+ &' '.to_string()

                    }
                    state.chunk_manager.draw(first, last, color,id,&state.device);
                }
                "/get"=>{
                    s = String::from("");
                }

                "/diffuse"=>{
                    state.diffuse_texture_flag = true;
                    state.normal_texture_flag = false;
                    state.depth_texture_flag = false;
                    state.output_texture_flag = false;
                    s = String::from("Texture changed");

                }

                "/normal"=>{
                    state.diffuse_texture_flag = false;
                    state.normal_texture_flag = true;
                    state.depth_texture_flag = false;
                    state.output_texture_flag = false;
                    s = String::from("Texture changed");
                }

                "/depth"=>{
                    state.diffuse_texture_flag = false;
                    state.normal_texture_flag = false;
                    state.depth_texture_flag = true;
                    state.output_texture_flag = false;
                    s = String::from("Texture changed");
                }

                "/output"=>{
                    state.diffuse_texture_flag = false;
                    state.normal_texture_flag = false;
                    state.depth_texture_flag = false;
                    state.output_texture_flag = true;
                    s = String::from("Texture changed");
                }

                _=>{
                    s = String::from("Fail to parse command");
                }
                
            }
            //state.iced_state.queue_message(ServerLog(s));
        }
        else {
            state.iced_state.queue_message(ChatMessage);
        }
    }

}