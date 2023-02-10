mod camera;
mod shell;
mod command;
mod chunk;

use cgmath::*;

use shell::Controls;
use shell::Message::{FrameUpdate,Update,ServerLog,CommandParsed};
use command::Descriptor;
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
    flag: [f32; 3],
    _padding0: u32,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
}

const VERTICES: &[Vertex] = &[

    Vertex { position: [-0.0,  0.70,  0.0],        color: [1.0,1.0,1.0 ,1.0],       normal:[0.0, 1.0, 0.0],   },
    Vertex { position: [-0.0, -0.70,  0.0],        color: [1.0,1.0,1.0 ,1.0],       normal:[0.0, 1.0, 0.0],   },
    
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
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
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

const OFFSET_X: f32 = 0.0;
const OFFSET_Y: f32 = 0.0;
const SAMPLE_RATIO: f32 = 2.0;

const VERTICES_TEX: &[Vertex_tex] = &[

    Vertex_tex { position: [SAMPLE_RATIO  + OFFSET_X ,  SAMPLE_RATIO + OFFSET_Y, 0.0], tex_coords: [1.0 ,0.0] },
    Vertex_tex { position: [SAMPLE_RATIO  + OFFSET_X , -SAMPLE_RATIO + OFFSET_Y, 0.0], tex_coords: [1.0 ,1.0] },
    Vertex_tex { position: [-SAMPLE_RATIO  +OFFSET_X ,-SAMPLE_RATIO  + OFFSET_Y, 0.0], tex_coords: [0.0 ,1.0] },
    Vertex_tex { position: [-SAMPLE_RATIO  +OFFSET_X , -SAMPLE_RATIO + OFFSET_Y, 0.0], tex_coords:[0.0 ,1.0] },
    Vertex_tex { position: [-SAMPLE_RATIO  +OFFSET_X , SAMPLE_RATIO  + OFFSET_Y, 0.0], tex_coords: [0.0 ,0.0] },
    Vertex_tex { position: [SAMPLE_RATIO  + OFFSET_X , SAMPLE_RATIO  + OFFSET_Y, 0.0], tex_coords:  [1.0 ,0.0] },

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

#[derive(Clone)]
pub struct Instance {
    is_active: bool,
    position: cgmath::Vector3<f32>,
    color: cgmath::Vector4<f32>,
    depth_strength:f32,
    normal_strength:f32,
}

// NEW!
impl Instance {
    
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)).into(),
            color: [self.color.x,self.color.y,self.color.z,self.color.w],
            depth_strength: self.depth_strength,
            normal_strength: self.normal_strength,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    depth_strength: f32,
    normal_strength:f32,
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 21]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct State {

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
    
    render_blend_pipeline: wgpu::RenderPipeline,
    render_sample_pipeline: wgpu::RenderPipeline,

    render_terrain_pipeline: wgpu::RenderPipeline,
    render_terrain_normal_pipeline: wgpu::RenderPipeline,
    render_terrain_depth_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    vertex_tex_buffer: wgpu::Buffer,

    num_vertices: u32,

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
    
    diffuse_bind_group: wgpu::BindGroup,
    depth_bind_group: wgpu::BindGroup,
    normal_bind_group: wgpu::BindGroup,

    diffuse_texture_view:wgpu::TextureView,
    normal_texture_view:wgpu::TextureView,
    depth_texture_view:wgpu::TextureView,
    depth_test_texture_view:wgpu::TextureView,
    //msaa_texture_view:wgpu::TextureView,

    renderer: Renderer,

    cli_status: bool,

    framerate_timer: f32,
    framerate_count: i32,

    normal_texture_flag: bool,
    diffuse_texture_flag: bool,
    depth_texture_flag: bool,
    output_texture_flag: bool,
    cli_flag: bool,

    chunk_manager:chunk::ChunkManager,

    
}
impl State {
    async fn new(window: &Window,scr_width:u32,scr_height:u32) -> Self {

        
        let normal_texture_flag = false;
        let depth_texture_flag = false;
        let diffuse_texture_flag = false;
        let output_texture_flag = true;

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


        let modifiers = ModifiersState::default();
        let clipboard = Clipboard::connect(&window);

        // Initialize scene and GUI controls
        let control = Controls::new();
        // Initialize iced
        let iced_state = program::State::new(
            control,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );
        


        let texture_size = wgpu::Extent3d {
            width: scr_width,
            height: scr_height,
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
            forward: cgmath::Vector3::unit_y(),
            aspect: texture_size.width as f32 / texture_size.height as f32,
            fovy: texture_size.height as f32 / 2.0 as f32,
            znear: 1000.0,
            zfar: 5100.0,
            left: cgmath::Vector3::unit_y(),
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
            position: [150.0, 150.0, 0.0],
            _padding: 0,
            flag: [0.0,1.0,1.0],
            _padding0: 0,
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


        #[cfg(not(target_arch = "wasm32"))]
        let normal_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,//Bgra8UnormSrgb? Rgba8UnormSrgb
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("normal_texture"),
            }
        );



        #[cfg(not(target_arch = "wasm32"))]
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("diffuse_texture"),
            }
        );

        #[cfg(not(target_arch = "wasm32"))]
        let depth_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("depth_texture"),
            }
        );

        /* 
        #[cfg(not(target_arch = "wasm32"))]
        let msaa_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("depth_texture"),
            }
        );
        */

        #[cfg(target_arch = "wasm32")]
        let normal_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("normal_texture"),
            }
        );

        #[cfg(target_arch = "wasm32")]
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                    
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("diffuse_texture"),
            }
        );

        #[cfg(target_arch = "wasm32")]
        let depth_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                    
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("depth_texture"),
            }
        );
        /* 
        #[cfg(target_arch = "wasm32")]
        let msaa_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("depth_texture"),
            }
        );
        */
        let depth_test_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                
                size: texture_size,
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
                label: Some("depth_texture"),
            }
        );
        
        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_texture_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_test_texture_view = depth_test_texture.create_view(&wgpu::TextureViewDescriptor::default());
        //let msaa_texture_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let normal_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor { 
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


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

        let normal_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&normal_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&normal_sampler),
                    }
                ],
                label: Some("normal_bind_group"),
            }
        );

        let depth_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&depth_sampler),
                    }
                ],
                label: Some("normal_bind_group"),
            }
        );



        let terrain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/terrain_shader/terrain_shader.wgsl").into()),
        });
        
        let terrain_shader_normal = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain_shader_normal"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/terrain_shader/terrain_shader_normal.wgsl").into()),
        });

        let terrain_shader_depth = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain_shader_depth"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/terrain_shader/terrain_shader_depth.wgsl").into()),
        });


        /* 

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LineShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader/line_shader.wgsl").into()),
        });

        let triangle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TriangleShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle_shader/triangle_shader.wgsl").into()),
        });

        let line_shader_normal = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LineShader_normal"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader/line_shader_normal.wgsl").into()),
        });

        let triangle_shader_normal = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TriangleShader_normal"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle_shader/triangle_shader_normal.wgsl").into()),
        });

        let line_shader_depth = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LineShader_normal"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line_shader/line_shader_depth.wgsl").into()),
        });

        let triangle_shader_depth = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TriangleShader_normal"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle_shader/triangle_shader_depth.wgsl").into()),
        });

        */






        let sample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("QuadShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sample_shader.wgsl").into()),
        });
        
        let blend_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("BlendShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blend_shader.wgsl").into()),
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










        let render_terrain_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });


        let render_blend_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                    &texture_bind_group_layout,
                    &texture_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_sample_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[     
                    &uniform_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_terrain_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render_terrain_pipeline"),
            layout: Some(&render_terrain_pipeline_layout),


            vertex: wgpu::VertexState {
                module: &terrain_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(),InstanceRaw::desc()],
            },


            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader,
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
                cull_mode: None,
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



        let render_terrain_normal_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render_terrain_normal_pipeline"),
            layout: Some(&render_terrain_pipeline_layout),


            vertex: wgpu::VertexState {
                module: &terrain_shader_normal,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(),InstanceRaw::desc()],
            },


            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader_normal,
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
                cull_mode: None,
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

        let render_terrain_depth_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render_terrain_pipeline"),
            layout: Some(&render_terrain_pipeline_layout),


            vertex: wgpu::VertexState {
                module: &terrain_shader_depth,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(),InstanceRaw::desc()],
            },


            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader_depth,
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
                cull_mode: None,
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






        let render_blend_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: Some("Render Blend Texture Pipeline"),
            layout: Some(&render_blend_pipeline_layout),

            vertex: wgpu::VertexState {
                module: &blend_shader,
                entry_point: "vs_main",
                buffers: &[Vertex_tex::desc()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &blend_shader,
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

        let render_sample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {

            label: Some("Render Texture Pipeline"),
            layout: Some(&render_sample_pipeline_layout),

            vertex: wgpu::VertexState {
                module: &sample_shader,
                entry_point: "vs_main",
                buffers: &[Vertex_tex::desc()],
            },

            fragment: Some(wgpu::FragmentState {
                module: &sample_shader,
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


        let chunk_manager = chunk::ChunkManager::new(&device);

        let num_vertices = VERTICES.len() as u32;

        //let view_sensitivity = 0.3;

        Self {

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

            
            render_blend_pipeline,
            render_sample_pipeline,

            render_terrain_pipeline,
            render_terrain_normal_pipeline,
            render_terrain_depth_pipeline,

            vertex_buffer,
            vertex_tex_buffer,

            num_vertices,

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
            
            diffuse_bind_group,
            depth_bind_group,
            normal_bind_group,

            diffuse_texture_view,
            depth_texture_view,
            normal_texture_view,
            depth_test_texture_view,
            //msaa_texture_view,

            renderer,

            cli_status,

            framerate_timer,
            framerate_count,

            cli_flag,
            normal_texture_flag,
            diffuse_texture_flag,
            depth_texture_flag,
            output_texture_flag,

            chunk_manager,

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
        if self.camera_controller.is_cli_pressed && !self.cli_flag{
            self.cli_flag = !self.cli_flag;
            self.cli_status = !self.cli_status;
        }
        if self.camera_controller.is_cli_released{
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

        self.light_uniform.position[1] = 150.0 + 50.0*Rad::sin(Rad(self.light_uniform.position[0] /100.0 as f32));
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));


        
        //self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.instance_data));
 
        

        
        

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        let output = self.surface.get_current_texture()?;
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut normal_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Normal Encoder"),
            });

        {
            let mut render_pass_normal = normal_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Normal Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.normal_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.5,
                                g: 1.0,
                                b: 0.5,
                                a: 0.0,
                            }
                        ),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_test_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass_normal.set_pipeline(&self.render_terrain_normal_pipeline);

            render_pass_normal.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass_normal.set_bind_group(1, &self.light_bind_group, &[]);

            render_pass_normal.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            self.chunk_manager.chunk_list.iter().for_each(|c|{
                render_pass_normal.set_vertex_buffer(1, c.buffer_data.slice(..));
                render_pass_normal.draw(0..self.num_vertices, 0..c.instance_len as _);
            });

            //render_pass_normal.set_pipeline(&self.render_line_normal_pipeline);
            //render_pass_normal.set_bind_group(0, &self.camera_bind_group, &[]); 
            //render_pass_normal.set_bind_group(1, &self.light_bind_group, &[]);

            //render_pass_normal.set_vertex_buffer(0, self.vertex_cube_buffer.slice(..));

            //render_pass_normal.draw(0..self.num_cube_vertices ,0..1);

            //render_pass_normal.set_pipeline(&self.render_triangle_normal_pipeline);
            //render_pass_normal.set_bind_group(0, &self.camera_bind_group, &[]); 
            //render_pass_normal.set_bind_group(1, &self.light_bind_group, &[]);

            //render_pass_normal.set_vertex_buffer(0, self.vertex_center_buffer.slice(..));

            //render_pass_normal.draw(0..self.num_center_vertices ,0..1);
            

        }

        let mut depth_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Normal Encoder"),
            });

        {
            let mut render_pass_depth = depth_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Normal Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.depth_texture_view,
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
                    view: &self.depth_test_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass_depth.set_pipeline(&self.render_terrain_depth_pipeline);

            render_pass_depth.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass_depth.set_bind_group(1, &self.light_bind_group, &[]);

            render_pass_depth.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            self.chunk_manager.chunk_list.iter().for_each(|c|{
                render_pass_depth.set_vertex_buffer(1, c.buffer_data.slice(..));
                render_pass_depth.draw(0..self.num_vertices, 0..c.instance_len as _);
            });
                
            
            

            //render_pass_depth.set_pipeline(&self.render_line_depth_pipeline);
            //render_pass_depth.set_bind_group(0, &self.camera_bind_group, &[]); 
            //render_pass_depth.set_bind_group(1, &self.light_bind_group, &[]);

            //render_pass_depth.set_vertex_buffer(0, self.vertex_cube_buffer.slice(..));

            //render_pass_depth.draw(0..self.num_cube_vertices ,0..1);

            //render_pass_depth.set_pipeline(&self.render_triangle_depth_pipeline);
            //render_pass_depth.set_bind_group(0, &self.camera_bind_group, &[]); 
            //render_pass_depth.set_bind_group(1, &self.light_bind_group, &[]);

            //render_pass_depth.set_vertex_buffer(0, self.vertex_center_buffer.slice(..));

            //render_pass_depth.draw(0..self.num_center_vertices ,0..1);
            

        }

        let mut diffuse_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass_diffuse = diffuse_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.diffuse_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }
                        ),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_test_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            
            
            render_pass_diffuse.set_pipeline(&self.render_terrain_pipeline);

            render_pass_diffuse.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass_diffuse.set_bind_group(1, &self.light_bind_group, &[]);

            render_pass_diffuse.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            self.chunk_manager.chunk_list.iter().for_each(|c|{
                render_pass_diffuse.set_vertex_buffer(1, c.buffer_data.slice(..));
                render_pass_diffuse.draw(0..self.num_vertices, 0..c.instance_len as _);
            });

            //render_pass_diffuse.set_pipeline(&self.render_line_pipeline);
            //render_pass_diffuse.set_bind_group(0, &self.camera_bind_group, &[]); 
            //render_pass_diffuse.set_bind_group(1, &self.light_bind_group, &[]);

            //render_pass_diffuse.set_vertex_buffer(0, self.vertex_cube_buffer.slice(..));

            //render_pass_diffuse.draw(0..self.num_cube_vertices ,0..1);

            //render_pass_diffuse.set_pipeline(&self.render_triangle_pipeline);
            //render_pass_diffuse.set_bind_group(0, &self.camera_bind_group, &[]); 
            //render_pass_diffuse.set_bind_group(1, &self.light_bind_group, &[]);

            //render_pass_diffuse.set_vertex_buffer(0, self.vertex_center_buffer.slice(..));

            //render_pass_diffuse.draw(0..self.num_center_vertices ,0..1);
            

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
                    //view: &self.msaa_texture_view,
                    //resolve_target: Some(&view),
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

        
            if self.output_texture_flag {
                render_pass.set_pipeline(&self.render_blend_pipeline);
                render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(2, &self.normal_bind_group, &[]);
                render_pass.set_bind_group(3, &self.depth_bind_group, &[]);
            }

            else if self.normal_texture_flag {
                render_pass.set_pipeline(&self.render_sample_pipeline);
                render_pass.set_bind_group(1, &self.normal_bind_group, &[]);
            } 
            else if self.diffuse_texture_flag{
                render_pass.set_pipeline(&self.render_sample_pipeline);
                render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            }
            else if self.depth_texture_flag {
                render_pass.set_pipeline(&self.render_sample_pipeline);
                render_pass.set_bind_group(1, &self.depth_bind_group, &[]);
            }
            
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

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
        self.queue.submit(iter::once(normal_encoder.finish()));
        self.queue.submit(iter::once(depth_encoder.finish()));
        self.queue.submit(iter::once(diffuse_encoder.finish()));
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
    #[cfg(target_arch = "wasm32")]
    let scr_width;
    #[cfg(target_arch = "wasm32")]
    let scr_height;
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


    #[cfg(not(target_arch = "wasm32"))]
    let scr_width = window.inner_size().width;
    #[cfg(not(target_arch = "wasm32"))]
    let scr_height = window.inner_size().height;

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window,scr_width,scr_height).await;

    let mut command_parser = Descriptor::new();
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
            } => 
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
                
                if state.iced_state.program().parse_flag {

                    let t = &state.iced_state.program().text;
                    command_parser.text = t.to_string();
                    command_parser.parse_command(&mut state);

                    state.iced_state.queue_message(CommandParsed);
                    
                }

                #[cfg(target_arch = "wasm32")]{
                    //use web_sys::console;
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
