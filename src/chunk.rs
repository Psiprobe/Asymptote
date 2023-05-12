use std::collections::{HashMap};
use std::time::Duration;
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use iced_winit::{program, winit};

use winit::{
    dpi::PhysicalSize,
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder,Fullscreen},
};

use crate::{Instance,InstanceRaw, shell, VoxelType};
use crate::camera::*;
use crate::shell::Message::*;
use crate::brush_list;
use crate::model_list;

const RADIUS_CHUNK:i32 = 16;
const RADIUS_VOXEL:i32 = 256;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub position: [f32; 3],
    // uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: [f32; 4],
}


#[derive(Clone, Copy)]
pub struct BrushState {

    pub id:i32,
    pub radius:i32,
    pub color:[f32;4],
    pub max_id:i32,

}

impl BrushState {
    pub fn update(&mut self){
        match self.id {
            0 => {
            }

            1 => {
                self.radius = 1;
            }

            _=>{}
        }
    }

    pub fn new() -> Self{
        Self { id: 0, radius: RADIUS_VOXEL/4, color: [1.0,1.0,1.0,1.0] ,max_id:1}
    }
}
pub struct ModelState {
    pub id: i32,
    pub scale: i32,
    pub max_id: i32,
    pub height: i32,
    pub radius: i32,
    pub name: String,
    pub color: [f32;4],
}


impl ModelState {
    pub fn update(&mut self){
        match self.id {
            0|1|2|3|4|5|6|7|8|9 => {
                self.name = "NUM".to_string();
                self.height = RADIUS_VOXEL / self.scale * 2;
                self.radius = RADIUS_VOXEL / self.scale;
            }
            10 => {
                self.name = "CHESS".to_string();
                self.height = 1;
                self.radius = RADIUS_VOXEL / self.scale;
            }

            11 => {
                self.name = "PLANE".to_string();
                self.height = 1;
                self.radius = RADIUS_VOXEL / self.scale;
            }

            12 => {
                self.name = "BLOCK".to_string();
                self.height = RADIUS_VOXEL / self.scale;
                self.radius = RADIUS_VOXEL / self.scale;
            }

            13 => {
                self.name = "LAWN".to_string();
                self.height = 20;
                self.radius = RADIUS_VOXEL / self.scale;
            }
            14 => {
                self.name = "SPHERE".to_string();
                self.height = RADIUS_VOXEL / self.scale;
                self.radius = RADIUS_VOXEL / self.scale;
            }

            _=>{}
        }
    }

    pub fn new() -> Self{
        Self { id: 0, scale:1,max_id:14,height: 1,radius:Default::default(), name:"CHESS".to_owned(), color: [0.7,0.7,0.7,1.0]}
    }
}

#[derive(PartialEq)]
pub enum IndicatorState {
    Normal,
    Brush,
    Place,
}

impl IndicatorState {
    fn turn(&self) -> Self {
        match self {
            IndicatorState::Normal=> IndicatorState::Brush,
            IndicatorState::Brush => IndicatorState::Place,
            IndicatorState::Place => IndicatorState::Normal,
        }
    }

    fn to_str(&self) -> String{
        match self {
            IndicatorState::Normal=> "NORMAL_MODE".to_string(),
            IndicatorState::Brush => "BRUSH_MODE" .to_string(),
            IndicatorState::Place => "PLACE_MODE" .to_string(),
        }
    }
}

pub struct ChunkManager{

    pub chunk_list:Vec<Chunk>,

    pub point_light_list:Vec<Light>,

    pub debug_mode:bool,
    pub w:f32,

    pub chunk_overdose:bool,
    pub tab_overdose:bool,

    pub indicator_state:IndicatorState,
    pub model_state:ModelState,
    pub brush_state:BrushState,

    pub pervious_indicator_first:[i32;3],
    pub pervious_indicator_last:[i32;3],

    pub next_overdose:bool,
    pub prior_overdose:bool,

    pub pervious_mouse_left:bool,

    pub duplicate:bool,
    
}
#[derive(Copy, Clone, PartialEq)]
pub enum ChunkType {
    TerrainIndicator,
    UsrIndicator,
    Default,
}




impl ChunkManager{
    pub fn new(device:&wgpu::Device) -> Self{

        let mut chunk_list:Vec<Chunk> = Default::default();
        let mut light_vector:Vec<Light> = Default::default();



        for _x in 0..512{
            light_vector.push(
                Light {
                    position: [0.0, 0.0, 0.0],
                    _padding: 0,
                    color: [0.0,0.0,0.0,0.0],
                }
            );
        }

        let debug_mode = true;

        let w = 0.0;   

        Self{

            chunk_list,
            debug_mode,
            w,

            chunk_overdose:Default::default(),
            tab_overdose:Default::default(),

            indicator_state: IndicatorState::Normal,

            model_state: ModelState::new(),
            brush_state: BrushState::new(),

            pervious_indicator_first:Default::default(),
            pervious_indicator_last:Default::default(), 

            next_overdose:Default::default(),
            prior_overdose:Default::default(),

            duplicate: false,
            pervious_mouse_left: false,
            point_light_list: light_vector,

        }
    }


    pub fn update(&mut self,device:&wgpu::Device,dt:Duration,camera:&Camera,camera_controller:&mut CameraController,mouse_pos_x:f64,mouse_pos_y:f64,texture_size: wgpu::Extent3d, iced_state: &mut program::State<shell::Controls>, sample_ratio:&mut f32){
        
        if self.w < -0.5{
            self.w =  1.0;
        }
        else{
            self.w = self.w - (dt.as_secs_f32()*4.0);
        }

        if camera_controller.is_tab_pressed && !self.tab_overdose{
            self.indicator_state = self.indicator_state.turn();
        }

        let mut delete = false;
        
        let shell_color_config = iced_state.program().color;
        let mut color = [shell_color_config.r,shell_color_config.g,shell_color_config.b,self.w];

        self.point_light_list[0].color = [shell_color_config.r,shell_color_config.g,shell_color_config.b,1.0];

        if camera_controller.is_control_pressed{
            delete = true;
            color = [0.5,0.2,0.2,self.w];
        }



        if camera_controller.is_control_pressed{
            if camera_controller.scroll < 0.0{
                *sample_ratio += 0.1;
                if *sample_ratio > 4.0{
                    *sample_ratio = 4.0;
                }
            }
            else if camera_controller.scroll > 0.0{
                *sample_ratio -= 0.1;
                if *sample_ratio < 1.0{
                    *sample_ratio = 1.0;
                }
            }
        }
        
        else{

            match &self.indicator_state{

                IndicatorState::Normal => {
                    if camera_controller.scroll > 0.0{
                        self.brush_state.radius += 1;
                        if self.brush_state.radius > 64{
                            self.brush_state.radius = 64;
                        }
                    }
                    else if camera_controller.scroll < 0.0{
                        self.brush_state.radius -= 1;
                        if self.brush_state.radius < 1{
                            self.brush_state.radius = 1;
                        }
                    }
                    if !delete{
                        iced_state.queue_message(UsrIndicator(
                            self.indicator_state.to_str() + 
                            &self.brush_state.id.to_string() + 
                            &" pix:".to_string() +
                            &self.brush_state.radius.to_string()
                            , self.tab_overdose));
                        
                    }else{
                        iced_state.queue_message(UsrIndicator(self.indicator_state.to_str() + "_DELETE" , self.tab_overdose));
                    }
                },

                IndicatorState::Brush => {
                    if camera_controller.scroll > 0.0{
                        self.brush_state.radius += 1;
                        if self.brush_state.radius > 64{
                            self.brush_state.radius = 64;
                        }
                        self.brush_state.update();
                    }
                    else if camera_controller.scroll < 0.0{
                        self.brush_state.radius -= 1;
                        if self.brush_state.radius < 1{
                            self.brush_state.radius = 1;
                        }
                        self.brush_state.update();
                    }

                    if camera_controller.is_next_pressed && !self.next_overdose{

                        self.brush_state.id += 1;
                        if self.brush_state.id > self.brush_state.max_id{
                            self.brush_state.id = 0;
                        }
            
                    }
            
                    if camera_controller.is_prior_pressed && !self.prior_overdose{
            
                        self.brush_state.id -= 1;
                        if self.brush_state.id < 0{
                            self.brush_state.id = self.brush_state.max_id;
                        }
            
                    }

                    if !delete{
                        iced_state.queue_message(UsrIndicator(
                            self.indicator_state.to_str() + 
                            &self.brush_state.id.to_string() + 
                            &" pix:".to_string() +
                            &self.brush_state.radius.to_string()
                            , self.tab_overdose));
                        
                    }else{
                        iced_state.queue_message(UsrIndicator(self.indicator_state.to_str() + "_DELETE" , self.tab_overdose));
                    }
                },
                IndicatorState::Place => {

                    if camera_controller.scroll > 0.0{
                        self.model_state.scale *= 2;
                        if self.model_state.scale > 8{
                            self.model_state.scale = 8;
                        }
                    }
                    else if camera_controller.scroll < 0.0{
                        self.model_state.scale /= 2;
                        if self.model_state.scale < 1{
                            self.model_state.scale = 1;
                        }
                    }

                    if camera_controller.is_next_pressed && !self.next_overdose{

                        self.model_state.id += 1;
                        if self.model_state.id > self.model_state.max_id{
                            self.model_state.id = 0;
                        }
            
                    }
            
                    if camera_controller.is_prior_pressed && !self.prior_overdose{
            
                        self.model_state.id -= 1;
                        if self.model_state.id < 0{
                            self.model_state.id = self.model_state.max_id;
                        }
            
                    }
    
                    if !delete{
                        iced_state.queue_message(UsrIndicator(self.indicator_state.to_str() + &self.model_state.id.to_string(),self.tab_overdose));
                        
                    }else{
                        iced_state.queue_message(UsrIndicator(self.indicator_state.to_str() + "_DELETE" , self.tab_overdose));
                    }
                    
                },
            }
        }
        camera_controller.scroll = 0.0;

        self.model_state.update();
        self.brush_state.update();

        let mouse_x = mouse_pos_x / *sample_ratio as f64 - texture_size.width as f64 / 2.0 / *sample_ratio as f64;
        let mouse_y = mouse_pos_y / *sample_ratio as f64 * 3.0 - texture_size.height as f64 / 2.0 / *sample_ratio as f64 * 3.0;

        let camera_mouse_eye = camera.eye + (camera.forward * mouse_y as f32)  + (camera.left * mouse_x as f32);
        let camera_mouse_target = camera.target + (camera.forward * mouse_y as f32) + (camera.left * mouse_x as f32);

        let mut camera_target_y = (RADIUS_VOXEL/2 - 1 + (RADIUS_VOXEL * 8)) as f32;

        let mut camera_target_z = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.z - camera_mouse_target.z) + camera_mouse_eye.z;
        let mut camera_target_x = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.x - camera_mouse_target.x) + camera_mouse_eye.x;

        let mut voxel_founded = false;

        let snap_radius = RADIUS_VOXEL/self.model_state.scale;

        while camera_target_y >= (RADIUS_VOXEL/2 - 1 - (RADIUS_VOXEL * 8)) as f32&& !voxel_founded{

            camera_target_x = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.x - camera_mouse_target.x) + camera_mouse_eye.x;
            camera_target_z = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.z - camera_mouse_target.z) + camera_mouse_eye.z;
            
            camera_target_y = camera_target_y - 0.3 as f32;

            self.chunk_list.iter_mut().filter(|c|c.current_type != ChunkType::UsrIndicator).for_each(|c|{

                if !voxel_founded{

                    let v_range_x_min = c.position[0] * RADIUS_VOXEL - RADIUS_VOXEL/2;
                    let v_range_x_max = c.position[0] * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                    let v_range_y_min = c.position[1] * RADIUS_VOXEL - RADIUS_VOXEL/2;
                    let v_range_y_max = c.position[1] * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                    let v_range_z_min = c.position[2] * RADIUS_VOXEL - RADIUS_VOXEL/2;
                    let v_range_z_max = c.position[2] * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                    if camera_target_x >= v_range_x_min as f32 && camera_target_x <= v_range_x_max as f32 &&  
                        camera_target_y >= v_range_y_min as f32 && camera_target_y <= v_range_y_max as f32 &&  
                        camera_target_z >= v_range_z_min as f32 && camera_target_z <= v_range_z_max as f32
                    {

                        c.is_selected = true;
                        
                        camera_target_z = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.z - camera_mouse_target.z) + camera_mouse_eye.z;
                        camera_target_x = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.x - camera_mouse_target.x) + camera_mouse_eye.x;

                        let mut voxel_normal:[f32;3] = Default::default();

                        match c.position_hash.get(&[
                            camera_target_x as i32,
                            camera_target_y as i32,
                            camera_target_z as i32,
                        ])
                        {
                            Some(usize) => {
                                voxel_founded = true;
                                voxel_normal = c.voxel_data[*usize].normal.into();
                            },
                            None => {},
                        }

                        if voxel_founded && c.current_type == ChunkType::Default{

                            if self.indicator_state == IndicatorState::Place && !camera_controller.is_control_pressed{
                                camera_target_x = camera_target_x + voxel_normal[0];
                                camera_target_y = camera_target_y + voxel_normal[1];
                                camera_target_z = camera_target_z + voxel_normal[2];
                            }


                            if camera_controller.is_alt_pressed{
                                let chunk_pos_x = ((camera_target_x as i32 + snap_radius/2) as f32/ snap_radius as f32).floor() as i32;
                                let chunk_pos_y = ((camera_target_y as i32 + snap_radius/2) as f32/ snap_radius as f32).floor() as i32;
                                let chunk_pos_z = ((camera_target_z as i32 + snap_radius/2) as f32/ snap_radius as f32).floor() as i32;
                
                                camera_target_x = (chunk_pos_x * snap_radius) as f32;
                                camera_target_y = (chunk_pos_y * snap_radius) as f32;
                                camera_target_z = (chunk_pos_z * snap_radius) as f32;
                                
                            }
                        }
                    }
                }
            });
        }

        if !voxel_founded{

            camera_target_y = camera.target.y - RADIUS_VOXEL as f32;
        
            camera_target_x = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.x - camera_mouse_target.x) + camera_mouse_eye.x;
            camera_target_z = (camera_target_y - camera_mouse_eye.y) / (camera_mouse_eye.y - camera_mouse_target.y) * (camera_mouse_eye.z - camera_mouse_target.z) + camera_mouse_eye.z;

            if camera_controller.is_alt_pressed{

                let chunk_pos_x = ((camera_target_x as i32 + snap_radius/2) as f32/ snap_radius as f32).floor() as i32;
                let chunk_pos_z = ((camera_target_z as i32 + snap_radius/2) as f32/ snap_radius as f32).floor() as i32;

                camera_target_x = (chunk_pos_x * snap_radius) as f32;
                camera_target_z = (chunk_pos_z * snap_radius) as f32;

            }
        }
           

        //iced_state.queue_message(Coordinate([camera_target_x,camera_target_y,camera_target_z]));

        let indicator_first;

        let mut indicator_last ;

        let head:&str ;

        let id:i32;

        match &self.indicator_state{

            IndicatorState::Normal => {
                indicator_first = [
                    camera_target_x as i32 - self.model_state.radius/2, 
                    camera_target_y as i32 ,
                    camera_target_z as i32 - self.model_state.radius/2,
                ];

                indicator_last = [
                    camera_target_x as i32 + self.model_state.radius/2 - 1, 
                    camera_target_y as i32 + (self.model_state.height - 1) ,
                    camera_target_z as i32 + self.model_state.radius/2 - 1, 
                ];

                id = 0;
            },
            IndicatorState::Brush => {

                indicator_first = [
                    camera_target_x as i32 - self.brush_state.radius/2, 
                    camera_target_y as i32 - self.brush_state.radius/2,
                    camera_target_z as i32 - self.brush_state.radius/2,
                ];

                indicator_last = [
                    camera_target_x as i32 + self.brush_state.radius/2, 
                    camera_target_y as i32 + self.brush_state.radius/2, 
                    camera_target_z as i32 + self.brush_state.radius/2, 
                ];

                if delete{color = [1.0,1.0,1.0,1.0]}

                let indicator_brush_model_id = 1;

                self.place(
                    [indicator_first[0],indicator_first[1],indicator_first[2]],
                    [indicator_last[0],indicator_last[1],indicator_last[2]],
                    color,
                    false,
                    device,
                    ChunkType::UsrIndicator,
                    indicator_brush_model_id,
                    iced_state
                );

                id = self.brush_state.id;
            },
            IndicatorState::Place => {
                
                if delete{
                    indicator_first = [
                    camera_target_x as i32 - RADIUS_VOXEL/2, 
                    camera_target_y as i32,
                    camera_target_z as i32 - RADIUS_VOXEL/2
                ];

                indicator_last = [
                    camera_target_x as i32 + RADIUS_VOXEL/2 - 1, 
                    camera_target_y as i32,
                    camera_target_z as i32 + RADIUS_VOXEL/2 - 1, 
                ];
                }else{
                    indicator_first = [
                        camera_target_x as i32 - self.model_state.radius/2, 
                        camera_target_y as i32 ,
                        camera_target_z as i32 - self.model_state.radius/2
                    ];
    
                    indicator_last = [
                        camera_target_x as i32 + self.model_state.radius/2 - 1, 
                        camera_target_y as i32 + (self.model_state.height - 1) ,
                        camera_target_z as i32 + self.model_state.radius/2 - 1, 
                    ];
                }

                
                id = self.model_state.id;
                
                self.place(
                    [indicator_first[0],indicator_first[1]+1,indicator_first[2]],
                    [indicator_last[0],indicator_last[1]+1,indicator_last[2]],
                    color,
                    false,
                    device,
                    ChunkType::UsrIndicator,
                    id,
                    iced_state
                );

                if delete{
                    indicator_last[1] = RADIUS_VOXEL * 8;
                }
                
                
            },
        }


        if camera_controller.mouse_left_pressed  && !self.duplicate
        {

            color[3] = 1.0;
            match &self.indicator_state{

                IndicatorState::Normal => {
                    if !delete{
                        head = "";
                        
                    }else{
                        head = "";
                    }
                },
                IndicatorState::Brush => {
                    head = "/draw";
                },
                IndicatorState::Place => {
                    if !self.chunk_overdose{
                        if !delete{
                            head = "/place";
                        }else{
                            head = "/delete";
                        }
                    }
                    else{
                        head = "";
                    }
                    
                    
                },
            }
            
            if head != ""{

                let t = 
                head.to_owned() + &' '.to_string()
                + &indicator_first[0].to_string()+ &' '.to_string()
                + &indicator_first[1].to_string()+ &' '.to_string()
                + &indicator_first[2].to_string()+ &' '.to_string()
    
                + &indicator_last[0].to_string()+ &' '.to_string()
                + &indicator_last[1].to_string()+ &' '.to_string()
                + &indicator_last[2].to_string()+ &' '.to_string()
    
                + &color[0].to_string()+ &' '.to_string()
                + &color[1].to_string()+ &' '.to_string()
                + &color[2].to_string()+ &' '.to_string()
                + &color[3].to_string()+ &' '.to_string()

                + &id.to_string()+ &' '.to_string();

                if head == "/draw"{
                    iced_state.queue_message(shell::Message::CommandChanged(t.to_owned()));
                    iced_state.queue_message(shell::Message::Parse);
                }
            
                #[cfg(target_arch = "wasm32")]
                {
                
                    use wasm_bindgen::prelude::*;
                
                    #[wasm_bindgen(module = "/tab.js")]

                    extern "C" {

                        fn issue_command();
                        fn set_command_send_buffer(command:String);

                    }

                    set_command_send_buffer(t);
                    issue_command();
                }

                
    
                
            }
        }


        













        

        if camera_controller.mouse_left_pressed {
            self.chunk_overdose = true;
        }else {
            self.chunk_overdose = false;
        }

        if camera_controller.is_tab_pressed {
            self.tab_overdose = true;
        }else {
            self.tab_overdose = false;
        }

        if self.pervious_mouse_left && self.pervious_indicator_first == indicator_first{
            self.duplicate = true;
        }else{
            self.duplicate = false;
        }

        self.prior_overdose = camera_controller.is_prior_pressed;
        self.next_overdose = camera_controller.is_next_pressed;

        self.pervious_indicator_first = indicator_first;
        self.pervious_indicator_last = indicator_last;

        self.pervious_mouse_left = camera_controller.mouse_left_pressed;

        
    }

    pub fn draw(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],id:i32,device:&wgpu::Device){

        let chunk_pos_x_first = ((first[0] + RADIUS_VOXEL/2) as f32/ RADIUS_VOXEL as f32).floor() as i32;
        let chunk_pos_x_last= ((last[0] + RADIUS_VOXEL/2) as f32/ RADIUS_VOXEL as f32).floor() as i32;

        let chunk_pos_y_first = ((first[1] + RADIUS_VOXEL/2) as f32/ RADIUS_VOXEL as f32).floor() as i32;
        let chunk_pos_y_last= ((last[1] + RADIUS_VOXEL/2) as f32/ RADIUS_VOXEL as f32).floor() as i32;
        
        let chunk_pos_z_first = ((first[2] + RADIUS_VOXEL/2) as f32/ RADIUS_VOXEL as f32).floor() as i32;
        let chunk_pos_z_last= ((last[2] + RADIUS_VOXEL/2) as f32/ RADIUS_VOXEL as f32).floor() as i32;

        for xx in chunk_pos_x_first..chunk_pos_x_last + 1{
            for yy in chunk_pos_y_first..chunk_pos_y_last + 1{
                for zz in chunk_pos_z_first..chunk_pos_z_last + 1{

                    self.chunk_list.iter_mut().filter(|c|c.current_type == ChunkType::Default).for_each(|c|{
                        if c.position[0] == xx && c.position[1] == yy && c.position[2] == zz{

                            let mut x_first = xx * RADIUS_VOXEL - RADIUS_VOXEL/2;
                            let mut x_last =  xx * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                            let mut y_first = yy * RADIUS_VOXEL - RADIUS_VOXEL/2;
                            let mut y_last =  yy * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                            let mut z_first = zz * RADIUS_VOXEL - RADIUS_VOXEL/2;
                            let mut z_last =  zz * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                            if x_first < first[0]   {x_first = first[0]}
                            if x_last  > last[0]    {x_last = last[0]}
                            if y_first < first[1]   {y_first = first[1]}
                            if y_last  > last[1]    {y_last = last[1]}
                            if z_first < first[2]   {z_first = first[2]}
                            if z_last  > last[2]    {z_last = last[2]}

                            let c_first = [x_first,y_first,z_first];
                            let c_last = [x_last,y_last,z_last ];

                            c.draw(c_first, c_last, color, device, id , &mut self.point_light_list)
                        }
                    })
                }
            }
        }
    }
    pub fn place(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],delete:bool,device:&wgpu::Device,chunk_type: ChunkType,id:i32,iced_state: &mut program::State<shell::Controls>){

        //chunk offset
        let c_first = first;
        let c_last = last;

        
        let chunk_pos_x_first = ((c_first[0] + RADIUS_VOXEL / 2) as f32 / RADIUS_VOXEL as f32).floor() as i32;
        let chunk_pos_x_last= ((c_last[0] + RADIUS_VOXEL / 2) as f32 / RADIUS_VOXEL as f32).floor() as i32;

        let chunk_pos_y_first = ((c_first[1] + RADIUS_VOXEL / 2) as f32 / RADIUS_VOXEL as f32).floor() as i32;
        let chunk_pos_y_last= ((c_last[1] + RADIUS_VOXEL / 2) as f32 / RADIUS_VOXEL as f32).floor() as i32;
        
        let chunk_pos_z_first = ((c_first[2] + RADIUS_VOXEL / 2) as f32 / RADIUS_VOXEL as f32).floor() as i32;
        let chunk_pos_z_last= ((c_last[2] + RADIUS_VOXEL / 2) as f32 / RADIUS_VOXEL as f32).floor() as i32;

        for xx in chunk_pos_x_first..chunk_pos_x_last + 1{
            for yy in chunk_pos_y_first..chunk_pos_y_last + 1{
                for zz in chunk_pos_z_first..chunk_pos_z_last + 1{

                    let mut x_first = xx * RADIUS_VOXEL - RADIUS_VOXEL/2;
                    let mut x_last = xx * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                    let mut y_first = yy * RADIUS_VOXEL - RADIUS_VOXEL/2;
                    let mut y_last = yy * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                    let mut z_first = zz * RADIUS_VOXEL - RADIUS_VOXEL/2;
                    let mut z_last = zz * RADIUS_VOXEL + RADIUS_VOXEL/2 - 1;

                    if x_first < first[0]   {x_first = first[0]}
                    if x_last  > last[0]    {x_last = last[0]}
                    if y_first < first[1]   {y_first = first[1]}
                    if y_last  > last[1]    {y_last = last[1]}
                    if z_first < first[2]   {z_first = first[2]}
                    if z_last  > last[2]    {z_last = last[2]}

                    let c_first = [x_first,y_first,z_first];
                    let c_last = [x_last,y_last,z_last];

                    let mut chunk_modified_flag = false;

                    self.chunk_list.iter_mut().filter(|c| c.current_type == chunk_type).for_each(|c|{
                        if c.position[0] == xx && c.position[1] == yy && c.position[2] == zz{
                            c.place(c_first, c_last,first,last,color, delete , device, id);
                            chunk_modified_flag = true;
                        }
                    });
                    
                    if !chunk_modified_flag{
                        let mut chunk = Chunk::empty(xx, yy, zz, true, device, chunk_type);
                        chunk.place(c_first, c_last,first,last,color, delete, device, id);
                        self.chunk_list.push(chunk);
                    }
                }
            }
        }
    }
}


pub struct Chunk{
    pub position:[i32;3],
    pub voxel_data:Vec<Instance>,
    pub instance_data:Vec<InstanceRaw>,
    pub instance_len:u32,
    pub buffer_data:wgpu::Buffer,
    pub is_active: bool,
    pub is_selected: bool,
    pub need_update: bool,
    pub current_type :ChunkType,
    pub position_hash: HashMap<[i32;3],usize>,
}

impl Chunk{

    pub fn empty(x:i32,y:i32,z:i32,is_active:bool,device:&wgpu::Device,chunk_type:ChunkType)->Self{

        let chunk_position = [x,y,z];
        let voxel_data: Vec<Instance> = Default::default();
        let instance_data:Vec<InstanceRaw>;
        
        instance_data = voxel_data.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });
        let instance_len = instance_data.len() as u32;

        Self{
            position:chunk_position,
            voxel_data,
            instance_data,
            instance_len,
            buffer_data,
            is_active,
            is_selected:true,
            need_update:false,
            current_type:chunk_type,
            position_hash:Default::default(),
        }
    }

    pub fn draw(&mut self,first:[i32;3],last:[i32;3],color:[f32;4],device:&wgpu::Device,id: i32, point_light_list:&mut Vec<Light>){

        self.voxel_data.iter_mut().for_each(|v|{

            if v.position[0] >= first[0] as f32&& v.position[0] <= last[0] as f32{
                if v.position[1] >= first[1] as f32&& v.position[1] <= last[1] as f32{
                    if v.position[2] >= first[2] as f32&& v.position[2] <= last[2] as f32{

                        let color = brush_list::parse_draw(v.position[0] as i32,v.position[1] as i32,v.position[2] as i32,first,last,color,id);
                        match color{
                            Some(c) =>{
                                v.color = cgmath::vec4(c[0], c[1], c[2], c[3]);
                                if id == 1{
                                    v.current_type = VoxelType::Fire;
                                    for i in 2..512{
                                        if point_light_list[i].color[3] == 0.0{
                                            point_light_list[i].color = v.color.into();
                                            point_light_list[i].position = v.position.into();
                                            break;
                                        }
                                    }
                                }
                            }
                            _ =>{}
                        }
                    }
                }
            }
        });

        self.instance_data = self.voxel_data.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });
        self.current_type = ChunkType::Default;
        self.instance_len = self.instance_data.len() as u32;

    }


    pub fn place(&mut self,first:[i32;3],last:[i32;3],place_first:[i32;3],place_last:[i32;3],color:[f32;4],delete:bool,device:&wgpu::Device,id:i32){

        let len = self.voxel_data.len();
        self.voxel_data
        .retain(|v|

            (v.position[0] < first[0] as f32 || v.position[0] > last[0] as f32) ||
            (v.position[1] < first[1] as f32 || v.position[1] > last[1] as f32) ||
            v.position[2] < first[2] as f32 || v.position[2] > last[2] as f32

        );
        let retained_len = self.voxel_data.len();

        if retained_len != len{

            let mut hash_map:HashMap<[i32;3],usize> = Default::default();
            self.voxel_data.iter().for_each(|v|{
                hash_map.insert([v.position[0] as i32,v.position[1] as i32,v.position[2] as i32], self.voxel_data.len()-1);
            });
            self.position_hash = hash_map;
            //hash rebuilt
        }

        if !delete{

            for x in first[0] ..last[0] + 1{
                for y in first[1] ..last[1] + 1{
                    for z in first[2] ..last[2] + 1{

                        

                            let instance = model_list::parse_place(x, y, z, place_first,place_last, color, id);

                            match instance{
                                Some(ins) => {
                                    self.voxel_data.push(ins);
                                    self.position_hash.insert([x,y,z],self.voxel_data.len()-1);
                                }
                                _ => {}
                            }
                            
                           
                        
                    }
                }  
            }
        }

        self.instance_data = self.voxel_data.iter().map(Instance::to_raw).collect::<Vec<_>>();

        self.buffer_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instance_data),
            usage: wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
        });

        self.instance_len = self.instance_data.len() as u32;


    }
}