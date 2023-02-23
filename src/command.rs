use crate::State;
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
        let first = s.chars().nth(0);
        if  first == Some('/') {
            match &s as &str{
                "/place" =>{

                }
                "/draw" =>{
                    
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

                "/test"=>{
                    s = String::from("Welcome to ASYMPTOTE Industries (TM) !");
                }

                "/tab"=>{
                    s = String::from("Psiprobe joined the game");

                }
                _=>{
                    s = String::from("Fail to parse command");
                }
                
            }
            state.iced_state.queue_message(ServerLog(s));
        }
        else {
            state.iced_state.queue_message(ChatMessage);
        }
    }

}