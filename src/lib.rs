mod camera;
mod shell;
mod command;

use cgmath::Rotation3;

use shell::Controls;
use shell::Message::{FrameUpdate,Update,ServerLog};
use command::descriptor;
use std::iter;

use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{
    conversion, futures, program, renderer, winit, Clipboard, Color, Debug,
    Size, time::Instant,
};
use iced_wgpu::wgpu::util::DeviceExt;

use winit::{
    dpi::PhysicalSize,
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder,Fullscreen},
    
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
pub enum Message {
    TextChanged(String),
    FrameRate,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],

    _padding2: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0,  0.0,  25.0], color: [0.0, 0.03, 0.0] },
    Vertex { position: [0.0,  0.0, -25.0], color: [0.0, 0.03, 0.0] },
    Vertex { position: [25.0,  0.0, -0.0], color: [0.0, 0.03, 0.0] },
    Vertex { position: [-25.0, 0.0, -0.0], color: [0.0, 0.03, 0.0] },
];

const VERTICES_CUBE: &[Vertex] = &[
    Vertex { position: [0.0,  0.0,  5.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.0,  0.0, -5.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.0,  50.0, -0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.0, -10.0, -0.0], color: [0.0, 1.0,0.0] },
];

const VERTICES_CENTER: &[Vertex] = &[
    Vertex { position: [10.0,  -100.0,  50.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [10.0,  -100.0, -50.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [10.0,  200.0, 50.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [10.0,  -100.0,  -50.0], color: [1.0, 0.0,0.0] },
    Vertex { position: [10.0, 200.0,  50.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [10.0,  200.0, -50.0], color: [1.0, 0.0, 0.0] },

    Vertex { position: [-10.0,  -100.0,  50.0], color: [-1.0, 0.0, 0.0] },
    Vertex { position: [-10.0,  -100.0, -50.0], color: [-1.0, 0.0, 0.0] },
    Vertex { position: [-10.0,  200.0, 50.0],   color: [-1.0, 0.0, 0.0] },
    Vertex { position: [-10.0,  -100.0,  -50.0],color: [-1.0, 0.0,0.0] },
    Vertex { position: [-10.0, 200.0,  50.0],   color: [-1.0, 0.0, 0.0] },
    Vertex { position: [-10.0,  200.0, -50.0],  color: [-1.0, 0.0, 0.0] },

    Vertex { position: [-10.0,  -100.0, 50.0], color: [0.0, 0.0, 1.0] },
    Vertex { position: [10.0,  -100.0,  50.0], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-10.0,  200.0,  50.0],   color: [0.0, 0.0, 1.0] },
    Vertex { position: [10.0,  -100.0, 50.0],color: [0.0, 0.0, 1.0] },
    Vertex { position: [-10.0, 200.0,   50.0],   color: [0.0, 0.0, 1.0] },
    Vertex { position: [10.0,  200.0,  50.0],  color: [0.0, 0.0, 1.0] },

    Vertex { position: [-10.0,  -100.0, -50.0], color: [0.0, 0.0, -1.0] },
    Vertex { position: [10.0,  -100.0,  -50.0], color: [0.0, 0.0, -1.0] },
    Vertex { position: [-10.0,  200.0,  -50.0],   color: [0.0, 0.0, -1.0] },
    Vertex { position: [10.0,  -100.0, -50.0],color: [0.0, 0.0, -1.0] },
    Vertex { position: [-10.0, 200.0,   -50.0],   color: [0.0, 0.0, -1.0] },
    Vertex { position: [10.0,  200.0,  -50.0],  color: [0.0, 0.0, -1.0] },

    Vertex { position: [-10.0,  200.0, 50.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [10.0,   200.0,  50.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [-10.0,  200.0,  -50.0],   color: [0.0, 1.0, 0.0] },
    Vertex { position: [10.0,  200.0, 50.0],color: [0.0, 1.0, 0.0] },
    Vertex { position: [-10.0, 200.0,   -50.0],   color: [0.0, 1.0, 0.0] },
    Vertex { position: [10.0,  200.0,  -50.0],  color: [0.0, 1.0, 0.0] },

    Vertex { position: [-10.0,  -100.0, 50.0], color: [0.0, -1.0, -0.0] },
    Vertex { position: [10.0,  -100.0,  50.0], color: [0.0, -1.0, -0.0] },
    Vertex { position: [-10.0,  -100.0,  -50.0],   color: [0.0, -1.0, -0.0] },
    Vertex { position: [10.0,  -100.0, 50.0],color: [0.0, -1.0, -0.0] },
    Vertex { position: [-10.0, -100.0,   -50.0],   color: [0.0, -1.0, -0.0] },
    Vertex { position: [10.0,  -100.0,  -50.0],  color: [0.0, -1.0, -0.0] },
    
];

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex_tex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}


const VERTICES_TEX: &[Vertex_tex] = &[

    Vertex_tex { position: [1.0,  1.0, 0.0], tex_coords: [1.0 ,0.0] },
    Vertex_tex { position: [1.0, -1.0, 0.0], tex_coords: [1.0 ,1.0] },
    Vertex_tex { position: [-1.0,-1.0, 0.0], tex_coords: [0.0 ,1.0] },
    Vertex_tex { position: [-1.0, -1.0, 0.0], tex_coords:[0.0 ,1.0] },
    Vertex_tex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0 ,0.0] },
    Vertex_tex { position: [1.0, 1.0, 0.0], tex_coords:  [1.0 ,0.0] },

];

impl Vertex_tex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex_tex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}



const NUM_INSTANCES_PER_ROW: i32 = 50;


struct Instance {
    position: cgmath::Vector3<f32>,
}

// NEW!
impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)).into(),
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

























struct State {

    command_parser: descriptor,

    iced_state: program::State<shell::Controls>,
    clipboard: Clipboard,
    modifiers: ModifiersState,
    staging_belt: wgpu::util::StagingBelt,
    size: winit::dpi::PhysicalSize<u32>,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    viewport: Viewport,
    debug: Debug,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,
    render_quad_pipeline: wgpu::RenderPipeline,
    render_light_pipeline: wgpu::RenderPipeline,
    render_center_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    vertex_tex_buffer: wgpu::Buffer,
    vertex_cube_buffer: wgpu::Buffer,
    vertex_center_buffer: wgpu::Buffer,

    num_vertices: u32,
    num_cube_vertices: u32,
    num_center_vertices: u32,

    camera: camera::Camera,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    uniform: camera::Uniform,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    //texture_size:wgpu::Extent3d,
    diffuse_texture:wgpu::Texture,
    diffuse_bind_group: wgpu::BindGroup,

    depth_texture_view:wgpu::TextureView,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    renderer: Renderer,

    cli_status: bool,
    cli_flag: bool,

    framerate_timer: f32,
    framerate_count: i32,

}
impl State {
    async fn new(window: &Window,scr_width:u32,scr_height:u32) -> Self {

        let command_parser = descriptor::new();

        let framerate_timer = 0.0;
        let framerate_count = 1;
        // Initialize staging belt
        
        let cursor_position = PhysicalPosition::new(-1.0, -1.0);
        let cli_status = false;
        let cli_flag = false;
        #[cfg(target_arch = "wasm32")]
        let default_backend = wgpu::Backends::GL;
        #[cfg(not(target_arch = "wasm32"))]
        let default_backend = wgpu::Backends::PRIMARY;
        let backend =wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let (format, (device, queue)) = futures::executor::block_on(async {
            let adapter = wgpu::util::initialize_adapter_from_env_or_default(
                &instance,
                backend,
                Some(&surface),
            )
            .await
            .expect("No suitable GPU adapters found on the system!");
        
            let adapter_features = adapter.features();
        
            #[cfg(target_arch = "wasm32")]
            let needed_limits = wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter.limits());
        
            #[cfg(not(target_arch = "wasm32"))]
            let needed_limits = wgpu::Limits::default();
        
            (
                surface
                    .get_supported_formats(&adapter)
                    .first()
                    .copied()
                    .expect("Get preferred format"),
                adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            features: adapter_features & wgpu::Features::default(),
                            limits: needed_limits,
                        },
                        None,
                    )
                    .await
                    .expect("Request device"),
            )
        });

        let mut renderer =Renderer::new(Backend::new(&device, Settings::default(), format));

        let mut debug = Debug::new();

        let staging_belt = wgpu::util::StagingBelt::new(5 * 1024);  

        let size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(scr_width,scr_height),
            //todos
            window.scale_factor(),
        );


        let mut modifiers = ModifiersState::default();
        let mut clipboard = Clipboard::connect(&window);

        // Initialize scene and GUI controls
        let control = Controls::new();
        // Initialize iced
        let mut iced_state = program::State::new(
            control,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );

        let texture_size = wgpu::Extent3d {
            width: scr_width/2,
            height: scr_height/2,
            depth_or_array_layers: 1,
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: texture_size.width,
            height: texture_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        


        ///////////////////////////Camera////////////////////////////
        let camera = camera::Camera {
            eye: (0.0, 0.0, 0.0).into(),
            position: (0.0, 0.0, 0.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: scr_width as f32 / scr_height as f32,
            fovy: scr_height as f32 / 4.0 as f32,
            znear: -10000.0,
            zfar: 10000.0,
        };

        let camera_controller = camera::CameraController::new(scr_width as f32 , scr_height as f32,300.0,0.002);
        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });







        let uniform = camera::Uniform::new();

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });
        
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });



        
        let light_uniform = LightUniform {
            position: [200.0, 300.0, 200.0],
            _padding: 0,
            color: [0.012, 0.02, 0.014],
            _padding2: 0,
        };
        
         // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });







        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,//Bgra8UnormSrgb? Rgba8UnormSrgb!
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("diffuse_texture"),
            }
        );

        let depth_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,//Bgra8UnormSrgb?
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
                label: Some("depth_texture"),
            }
        );


        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let depth_sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );


        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

            let diffuse_bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                        }
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );
            









































        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("QuadShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/quad_shader.wgsl").into()),
        });

        let light_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LightShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light_shader.wgsl").into()),
        });

        let center_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CenterShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/center_shader.wgsl").into()),
        });




        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_tex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Texture Buffer"),
            contents: bytemuck::cast_slice(VERTICES_TEX),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_cube_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Cube Buffer"),
            contents: bytemuck::cast_slice(VERTICES_CUBE),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_center_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Center Buffer"),
            contents: bytemuck::cast_slice(VERTICES_CENTER),
            usage: wgpu::BufferUsages::VERTEX,
        });










        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_quad_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_light_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_center_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });






        





        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),


            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(),InstanceRaw::desc()],
            },


            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),


            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },


            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },

            
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });






        let render_quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: Some("Render Texture Pipeline"),
            layout: Some(&render_quad_pipeline_layout),

            vertex: wgpu::VertexState {
                module: &quad_shader,
                entry_point: "vs_main",
                buffers: &[Vertex_tex::desc()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &quad_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),


            primitive: wgpu::PrimitiveState {

                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,

            },

            depth_stencil: None,

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let render_light_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: Some("Render Light Pipeline"),
            layout: Some(&render_light_pipeline_layout),

            vertex: wgpu::VertexState {
                module: &light_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &light_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),


            primitive: wgpu::PrimitiveState {

                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,

            },


            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });


        let render_center_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: Some("Render Light Pipeline"),
            layout: Some(&render_center_pipeline_layout),

            vertex: wgpu::VertexState {
                module: &center_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &center_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),


            primitive: wgpu::PrimitiveState {

                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,

            },


            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });




















        

        let instances = (-NUM_INSTANCES_PER_ROW..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (-NUM_INSTANCES_PER_ROW..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { x:(x*50) as f32, y: 0.0, z: (z*50) as f32 } ;
                Instance {
                    position,
                }
            })
        }).collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = VERTICES.len() as u32;
        let num_cube_vertices = VERTICES_CUBE.len() as u32;
        let num_center_vertices = VERTICES_CENTER.len() as u32;

        //let view_sensitivity = 0.3;

        Self {
            
            command_parser,
            iced_state,
            modifiers,
            clipboard,
            staging_belt,
            size,
            cursor_position,
            viewport,
            debug,

            surface,
            device,
            queue,
            config,

            render_pipeline,
            render_quad_pipeline,
            render_light_pipeline,
            render_center_pipeline,

            vertex_buffer,
            vertex_tex_buffer,
            vertex_cube_buffer,
            vertex_center_buffer,

            num_vertices,
            num_cube_vertices,
            num_center_vertices,

            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,

            uniform,
            uniform_buffer,
            uniform_bind_group,

            light_uniform,
            light_buffer,
            light_bind_group,

            //texture_size,
            diffuse_texture,
            diffuse_bind_group,
            depth_texture_view,

            instances,
            instance_buffer,

            renderer,

            cli_status,
            cli_flag,

            framerate_timer,
            framerate_count,

        }

    }

    pub fn cursormoved(&mut self,new_pos:winit::dpi::PhysicalPosition<f64>){
        self.cursor_position = new_pos;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {

            self.config.width = new_size.width;
            self.config.height = new_size.height;
            
            self.surface.configure(&self.device, &self.config);
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;

        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event);
        if self.camera_controller.is_slash_pressed && !self.cli_flag{
            self.cli_flag = !self.cli_flag;
            self.cli_status = !self.cli_status;
        }
        if self.camera_controller.is_slash_released{
            self.cli_flag = false;
        }
        true
    }

    fn update(&mut self, dt: std::time::Duration) {

        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_controller.process_mouse_position(self.cursor_position.x, self.cursor_position.y);
        
        //self.uniform.update((960.0 - self.cursor_position.x as f32)/960.0 * self.view_sensitivity , ( self.cursor_position.y as f32 - 540.0 )/540.0 * self.view_sensitivity);
        //texture offset disabled

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );

        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position =
                (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));

    }








    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_view = self.diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default()); 
        let mut texture_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass_texture = texture_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            }
                        ),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            
            
            render_pass_texture.set_pipeline(&self.render_pipeline);
            render_pass_texture.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass_texture.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass_texture.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass_texture.draw(0..self.num_vertices, 0..self.instances.len() as _);


            render_pass_texture.set_pipeline(&self.render_light_pipeline);
            render_pass_texture.set_bind_group(0, &self.camera_bind_group, &[]); 
            render_pass_texture.set_bind_group(1, &self.light_bind_group, &[]);

            render_pass_texture.set_vertex_buffer(0, self.vertex_cube_buffer.slice(..));

            render_pass_texture.draw(0..self.num_cube_vertices ,0..1);

            render_pass_texture.set_pipeline(&self.render_center_pipeline);
            render_pass_texture.set_bind_group(0, &self.camera_bind_group, &[]); 
            render_pass_texture.set_bind_group(1, &self.light_bind_group, &[]);

            render_pass_texture.set_vertex_buffer(0, self.vertex_center_buffer.slice(..));

            render_pass_texture.draw(0..self.num_center_vertices ,0..1);
            

        }

        let mut surface_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Surface Render Encoder"),
            });

        {


            let mut render_pass = surface_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.01,
                                g: 0.01,
                                b: 0.01,
                                a: 0.01,
                            }
                        ),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        
            render_pass.set_pipeline(&self.render_quad_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); 
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_tex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
            
        }
        
        if self.cli_status{
            self.renderer.with_primitives(|backend, primitive| {
                backend.present(
                    &self.device,
                    &mut self.staging_belt,
                    &mut surface_encoder,
                    //&mut texture_encoder,
                    &view,
                    primitive,
                    &self.viewport,
                    &self.debug.overlay(),
                );
            });
        }

        self.staging_belt.finish();
        self.queue.submit(iter::once(texture_encoder.finish()));
        self.queue.submit(iter::once(surface_encoder.finish()));
        output.present();


       // And recall staging buffers
       self.staging_belt.recall();

        Ok(())
    }
}










#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let monitor = event_loop.primary_monitor().unwrap();
    let video_mode = monitor.video_modes().next();
    //let size = video_mode.clone().map_or(PhysicalSize::new(1920, 1080), |vm| vm.size());
    let window = WindowBuilder::new()
        .with_visible(true)
        .with_title("?")
        .with_fullscreen(video_mode.map(|vm| Fullscreen::Exclusive(vm)))
        .build(&event_loop)
        .unwrap();
    //window.set_cursor_visible(false);

    let mut scr_width = window.inner_size().width;
    let mut scr_height = window.inner_size().height;

    #[cfg(target_arch = "wasm32")]
    {

        use wasm_bindgen::prelude::*;

        #[wasm_bindgen(module = "/tab.js")]
        extern "C" {
            fn get_width() -> u32;
            fn get_height() -> u32;
        }

        scr_width = get_width();
        scr_height = get_height();

        use winit::platform::web::WindowExtWebSys;
        use web_sys::console;
        web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("window")?;
            let canvas = web_sys::Element::from(window.canvas());
            dst.append_child(&canvas).ok()?;
            // Request fullscreen, if denied, continue as normal
            match canvas.request_fullscreen() {
                Ok(_) => {},
                Err(_) => ()
            }
            console::log_1(&"Hello using web-sys".into());
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
    }
    let mut state = State::new(&window,scr_width,scr_height).await;
    
    

    // State::new uses async code, so we're going to wait for it to finish

    let mut a = 0;
    let mut last_render_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {

        //use web_sys::console;
        //use winit::platform::web::WindowExtWebSys;
        //let win = web_sys::window().expect("0");
        //let doc = win.document().expect("0");
        
        match event {

            winit::event::Event::MainEventsCleared => {
                // If there are events pending
                if !state.iced_state.is_queue_empty() {
                    // We update iced
                    
                    let _ = state.iced_state.update(
                        state.viewport.logical_size(),
                        conversion::cursor_position(
                            state.cursor_position,
                            state.viewport.scale_factor(),
                        ),
                        &mut state.renderer,
                        &iced_wgpu::Theme::Matrix,
                        &renderer::Style { text_color:  Color::from_rgb(
                                0x01 as f32 / 255.0,
                                0x01 as f32 / 255.0,
                                0x01 as f32 / 255.0,
                            ) 
                        },
                        &mut state.clipboard,
                        &mut state.debug,
                    );  
                }
                window.request_redraw();
            }

            winit::event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. 
            } => if state.camera_controller.mouse_right_pressed 
            {
                state.camera_controller.process_mouse_motion(delta.0, delta.1)
            }

            winit::event::Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                {
                    match event {

                        #[cfg(not(target_arch="wasm32"))]
                        WindowEvent::CloseRequested
                        |   WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,

                        WindowEvent::ModifiersChanged(new_modifiers) => {
                            state.modifiers = *new_modifiers;
                        }

                        WindowEvent::CursorMoved {
                            position,
                            ..
                        } => {
                            state.cursormoved(*position);
                        }

                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }

                    state.input(event);

                    if state.cli_status{

                        if let Some(event) = iced_winit::conversion::window_event(
                            &event,
                            window.scale_factor(),
                            state.modifiers,
                        ) {
                            state.iced_state.queue_event(event);
                        }

                    }

                    
                }
            }

            winit::event::Event::RedrawRequested(window_id) if window_id == window.id() => {


                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);

                state.iced_state.queue_message(Update);
                
                if state.framerate_timer<1.0 {
                    state.framerate_timer += dt.as_secs_f32();
                    state.framerate_count += 1;
                }
                else {
                    state.iced_state.queue_message(FrameUpdate(state.framerate_count));
                    state.framerate_timer = 0.0;
                    state.framerate_count = 1;
                }
                
                
                #[cfg(target_arch = "wasm32")]{
                    use web_sys::console;

                   //console::log_1(&t.into());
                }

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }

                window.set_cursor_icon(
                    iced_winit::conversion::mouse_interaction(
                        state.iced_state.mouse_interaction(),//state.iced_state note
                    ),
                );
            }
            _ => {}
        }

        
        a = a+1;
        if window.fullscreen().is_none() {
            if a>333 {
                window.set_inner_size(PhysicalSize::new(1, 1));
                *control_flow = ControlFlow::Exit
            }
        }else{
            if a == 33 {
                state.cli_status = true;
                state.iced_state.queue_message(ServerLog("Welcome to ASYMPTOTE Industries (TM) !".to_string()));
                state.iced_state.queue_message(ServerLog("Current Version: 1.0.0".to_string()));
                


                //iced enabled && welcome message
            }
            if a>777 {
                
                a = 777;
            }
            
        }
        
        
    });
}
