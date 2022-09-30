use crate::State;
use crate::shell;
use shell::Message::{FrameUpdate,Update,ServerLog};
pub struct descriptor{
    pub text: String,
}
impl descriptor{
    pub fn new() -> Self{
        Self { 
            text: "".to_string()
         }
    }
    pub fn update(&mut self,t: String){

        self.text = t;
        

    }
    pub fn match_string(&self, state: State){
        let s = String::from(&self.text);
        match &s as &str{
            _=>{
                state.iced_state.queue_message(ServerLog("Fail to parse command".to_string()));
            }
        }

    }
}