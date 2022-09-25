use cgmath::*;
use iced_winit::winit::event::*;
use std::time::Duration;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub position: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::ortho(-self.aspect*self.fovy, self.aspect*self.fovy,-1.0*self.fovy ,1.0*self.fovy ,self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    offset:[f32; 3],
    _padding:u32,
}

impl Uniform {
    pub fn new() -> Self {
        Self {
            offset: cgmath::Vector3::new(0.0,0.0,0.0).into(),
            _padding: 0,
        }
    }

    pub fn update(&mut self,loc_x:f32,loc_y:f32){

        self.offset = cgmath::Vector3::new(loc_x,loc_y,0.0).into();

    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self,camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {

    scr_width: f32,
    scr_height: f32,
    aspect: f32,
    speed: f32,
    sensitivity: f32,
    
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,

    forward_count:f32,
    left_count:f32,

    rotate_horizontal: f32,
    rotate_vertical: f32,
    radius:f32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    yaw:f32,

    pub is_slash_released:bool,
    pub is_slash_pressed: bool,
    pub mouse_left_pressed: bool,
    pub mouse_right_pressed: bool,

    x_current: f32,
    y_current: f32,
    x_offset: f32,
    y_offset: f32,
}


impl CameraController {
    pub fn new(scr_width:f32, scr_height:f32, speed: f32, sensitivity: f32) -> Self {
        Self {

            scr_width,
            scr_height,
            aspect: 0.0,
            
            speed,
            sensitivity,

            is_slash_released:false,
            is_slash_pressed:false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,

            forward_count:0.0,
            left_count:0.0,

            is_left_pressed: false,
            is_right_pressed: false,
            
            mouse_left_pressed: false,
            mouse_right_pressed: false,

            x_current:0.0,
            y_current:0.0,
            x_offset: 0.0,
            y_offset: 0.0,
            
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            radius:2828.427125,
            pos_x: 0.0,
            pos_y: 1000.0,
            pos_z: 0.0,
            yaw:0.0,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {


            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_left_pressed = *state == ElementState::Pressed;
                true
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state,
                ..
            } => {
                self.mouse_right_pressed = *state == ElementState::Pressed;
                true
            }

            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                let is_released = *state == ElementState::Released;
                match keycode {

                    VirtualKeyCode::F3 => {
                        self.is_slash_pressed = is_pressed;
                        self.is_slash_released = is_released;
                        true
                    }
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn process_mouse_motion(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_mouse_position(&mut self, mouse_pos_x:f64,mouse_pos_y:f64){
        self.x_offset = mouse_pos_x as f32 - self.scr_width/2.0;
        self.y_offset = mouse_pos_y as f32 - self.scr_height/2.0;
    }

    pub fn update_camera(&mut self, camera: &mut Camera ,dt: Duration) {

        let dt = dt.as_secs_f32();
        
        self.yaw += self.rotate_horizontal;
        self.pos_x = Rad::sin(Rad(self.yaw*self.sensitivity))*self.radius;
        self.pos_z = Rad::cos(Rad(self.yaw*self.sensitivity))*self.radius;

        let forward = Vector3::new(self.pos_x, 0.0, self.pos_z).normalize();
        let left = camera.up.cross(forward).normalize();

        

        if self.is_forward_pressed{
            self.forward_count -= dt* self.speed;
        } 
        if self.is_backward_pressed{
            self.forward_count += dt* self.speed;
        } 

        if self.is_left_pressed{
            self.left_count -= dt* self.speed;
        }
        if self.is_right_pressed{
            self.left_count += dt* self.speed;
        }

        camera.position += (self.forward_count-self.forward_count%3.0) * forward;
        camera.position += (self.left_count-self.left_count%1.0) * left;

        self.forward_count %= 3.0;
        self.left_count %= 1.0;

        self.aspect = self.scr_width /self.scr_height;

        //camera accelerate calulate
        
        if  (self.x_offset / 2.0 - self.x_current).abs().sqrt() > 1.0 { //avoid glitching loop
            if self.x_current < self.x_offset / 2.0{
                self.x_current += (self.x_offset / 2.0 - self.x_current).sqrt()*dt*50.0;
            }
            else {
                self.x_current -= (self.x_current - self.x_offset / 2.0).sqrt()*dt*50.0;
            }
        }

        if (self.y_offset * self.aspect / 2.0 - self.y_current).abs().sqrt() > 1.0 {

            if self.y_current < self.y_offset * self.aspect / 2.0{
                self.y_current += (self.y_offset * self.aspect / 2.0 - self.y_current).sqrt()*dt*50.0;
            }
            else {
                self.y_current -= (self.y_current - self.y_offset * self.aspect / 2.0).sqrt()*dt*50.0;
            }
        }

        camera.target = camera.position + (self.x_current - self.x_current%1.0) * left + (self.y_current - self.y_current%3.0) * forward;

        camera.eye = cgmath::Point3::new(self.pos_x,self.pos_y,self.pos_z)+(
            camera.target-cgmath::Point3::new(0.0,0.0,0.0)
        );
        
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

    }
}
