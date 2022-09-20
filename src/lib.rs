mod camera;
mod forms;
mod model;
mod resources;
mod simulation;
mod texture;
use crate::model::DrawColoredMesh;
mod gui;

use cgmath::prelude::*;
use model::Vertex;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

// The indices of the models in the scene in their respective instance buffers.
// This practice should be abstracted away in the future, but since we have only 3
// objects right now, we'll manually keep track of indices.
const STATIC_INSTANCE_INDEX_LIGHT: u32 = 0;
const STATIC_INSTANCE_INDEX_BOUNDING_BOX: u32 = 1;
const DYNAMIC_INSTANCE_INDEX_BALL: u32 = 0;

const SIMULATION_DT_DEFAULT: std::time::Duration = std::time::Duration::from_millis(1);
const SIMULATION_DT_ADJUSTMENT_SIZE: std::time::Duration = std::time::Duration::from_micros(100);
const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

struct Instance {
    pub position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    scale: f32,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let model = cgmath::Matrix4::from_translation(self.position)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_scale(self.scale);
        InstanceRaw {
            model: model.into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

/// Reduced matrix from an Instance to be placed in the buffer for shaders.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}
impl model::Vertex for InstanceRaw {
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// This is a uniform to store the camera's view projection matrix for
/// the vertex shader.
/// A uniform is a blob of data that's available for a set of shaders.
/// We derive bytemuck::Pod and Zeroable so that it can be stored in a buffer.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

struct State {
    time_accumulator: std::time::Duration,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    #[allow(dead_code)]
    render_pipeline: wgpu::RenderPipeline,
    obj_model: model::Model,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: camera::CameraController,
    /// Models which do not require updates each frame will have their own instance buffer
    #[allow(dead_code)]
    static_instances: Vec<Instance>,
    static_instance_buffer: wgpu::Buffer,
    /// Instances which do require updates each frame (for animation, etc) will have their
    /// instance information (i.e. transformations!) stored in their own buffer.
    #[allow(dead_code)]
    dynamic_instances: Vec<Instance>,
    dynamic_instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    mouse_pressed: bool,
    colored_render_pipeline: wgpu::RenderPipeline,
    bounding_box_mesh: model::ColoredMesh,
    sphere_mesh: model::ColoredMesh,
    simulation_state: simulation::bounce::State,
    simulation_dt: std::time::Duration,
}

impl State {
    // Creating some of the wgpu types requires async types
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU.
        // Its main purpose is to create Adapters and Surfaces.
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        // The surface is the part of the window that we draw to.
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),

                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    // This project isn't built for web at the time of writing, though.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

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

        let camera = camera::Camera::new((0.0, 0.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(0.0));
        let projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let light_uniform = LightUniform {
            position: [6.0, 2.0, 6.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        // Render pipeline for textured models
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        // Render pipeline for our physical light object in the scene.
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        // Render pipeline for colored meshes without any textures.
        let colored_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Colored Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Colored Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("color_shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ColoredVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        let lightbulb_model =
            resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).unwrap();

        let bounding_box_mesh = forms::get_cube_interior_normals(&device, [0.5, 0.0, 0.5]);
        let sphere_mesh = forms::generate_sphere(&device, [0.2, 0.8, 0.2], 1.0, 32, 32);

        // Create the static instances and its buffer. We'll use this for the bounding box, which won't move.
        let static_instances = vec![
            // STATIC_INSTANCE_INDEX_LIGHT
            Instance {
                position: cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 1.0,
            },
            // STATIC_INSTANCE_INDEX_BOUNDING_BOX
            Instance {
                position: cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 2.0,
            },
        ];
        let static_instance_data = static_instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        let static_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&static_instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create the dynamic instance buffer, which we'll update each frame with the new position for the sphere.
        let dynamic_instances = vec![
            // DYNAMIC_INSTANCE_INDEX_BALL
            Instance {
                position: cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 1.0,
            },
        ];
        let dynamic_instance_data = dynamic_instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        let dynamic_instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Dynamic Instance Buffer"),
                contents: bytemuck::cast_slice(&dynamic_instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let simulation_state = simulation::bounce::State::new();

        Self {
            time_accumulator: std::time::Duration::from_millis(0),
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            obj_model: lightbulb_model,
            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            static_instances,
            static_instance_buffer,
            dynamic_instances,
            dynamic_instance_buffer,
            depth_texture,
            light_bind_group,
            light_render_pipeline,
            mouse_pressed: false,
            colored_render_pipeline,
            bounding_box_mesh,
            sphere_mesh,
            simulation_state,
            simulation_dt: SIMULATION_DT_DEFAULT,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // depth_texture must be udpated *after* the config, to get new width and height.
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.projection.resize(new_size.width, new_size.height)
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => {
                let camera_triggered = self.camera_controller.process_keyboard(*key, *state);
                let dt_adjustment_triggered = match key {
                    VirtualKeyCode::E => {
                        let new_dt_maybe = self
                            .simulation_dt
                            .checked_add(SIMULATION_DT_ADJUSTMENT_SIZE);
                        match new_dt_maybe {
                            Some(dt) => self.simulation_dt = std::cmp::min(dt, SIMULATION_DT_MAX),
                            None => self.simulation_dt = SIMULATION_DT_MAX,
                        }
                        println!("h: {:?}", &self.simulation_dt);
                        true
                    }
                    VirtualKeyCode::Q => {
                        let new_dt_maybe = self
                            .simulation_dt
                            .checked_sub(SIMULATION_DT_ADJUSTMENT_SIZE);
                        match new_dt_maybe {
                            Some(dt) => self.simulation_dt = std::cmp::max(dt, SIMULATION_DT_MIN),
                            None => self.simulation_dt = SIMULATION_DT_MIN,
                        }
                        println!("h: {:?}", &self.simulation_dt);
                        true
                    }
                    _ => false,
                };
                let param_adjustment_triggered = match key {
                    VirtualKeyCode::N => {
                        self.simulation_state.decrease_sphere_mass();
                        true
                    }
                    VirtualKeyCode::M => {
                        self.simulation_state.increase_sphere_mass();
                        true
                    }
                    VirtualKeyCode::R => {
                        self.simulation_state.increase_gravity();
                        true
                    }
                    VirtualKeyCode::F => {
                        self.simulation_state.decrease_gravity();
                        true
                    }
                    VirtualKeyCode::T => {
                        self.simulation_state.increase_drag();
                        true
                    }
                    VirtualKeyCode::G => {
                        self.simulation_state.decrease_drag();
                        true
                    }
                    VirtualKeyCode::Y => {
                        self.simulation_state.increase_wind_x();
                        true
                    }
                    VirtualKeyCode::H => {
                        self.simulation_state.decrease_wind_x();
                        true
                    }
                    VirtualKeyCode::U => {
                        self.simulation_state.increase_wind_y();
                        true
                    }
                    VirtualKeyCode::J => {
                        self.simulation_state.decrease_wind_y();
                        true
                    }
                    VirtualKeyCode::I => {
                        self.simulation_state.increase_wind_z();
                        true
                    }
                    VirtualKeyCode::K => {
                        self.simulation_state.decrease_wind_z();
                        true
                    }
                    VirtualKeyCode::O => {
                        self.simulation_state.increase_coefficient_of_restitution();
                        true
                    }
                    VirtualKeyCode::L => {
                        self.simulation_state.decrease_coefficient_of_restitution();
                        true
                    }
                    VirtualKeyCode::Z => {
                        self.simulation_state.decrease_coefficient_of_friciton();
                        true
                    }
                    VirtualKeyCode::X => {
                        self.simulation_state.increase_coefficient_of_friction();
                        true
                    }
                    VirtualKeyCode::C => {
                        self.simulation_state
                            .decrease_static_coefficient_of_friciton();
                        true
                    }
                    VirtualKeyCode::V => {
                        self.simulation_state
                            .increase_static_coefficient_of_friction();
                        true
                    }
                    _ => false,
                };
                camera_triggered || dt_adjustment_triggered || param_adjustment_triggered
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, frame_time: std::time::Duration) {
        // Get the unsimulated time from the previous frame, so that we simulate it this time around.
        self.time_accumulator = self.time_accumulator + frame_time;

        self.camera_controller
            .update_camera(&mut self.camera, frame_time);
        // TODO It's more efficient to have a staging buffer. Possible future improvement.
        // See https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/#a-controller-for-our-camera
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // SIMULATE until our simulation has "consumed" the accumulated time in discrete, fixed timesteps.
        while self.time_accumulator >= self.simulation_dt {
            // Note that our elapsed simulation time might be less than SIMULATION_DT if a collision occured.
            // That's OK, just continue simulating the next time step from the collision next iteration.
            let elapsed_sim_time = self.simulation_state.step(self.simulation_dt);
            self.time_accumulator = self.time_accumulator - elapsed_sim_time;
        }

        // TODO we may want to add the last step of https://gafferongames.com/post/fix_your_timestep/
        //   to interpolate the state if the basic accumulator implementation is jumpy.

        // Update the sphere position for DISPLAY from the simulation state.
        self.dynamic_instances[DYNAMIC_INSTANCE_INDEX_BALL as usize].position =
            self.simulation_state.get_position();
        let new_ball_instance_data =
            self.dynamic_instances[DYNAMIC_INSTANCE_INDEX_BALL as usize].to_raw();

        // Note: The offset is 0 because the ball is the only instance in the dynamic instance buffer
        // In the future, we'd have to offset by the size of raw instance data multiplied by the index.
        self.queue.write_buffer(
            &self.dynamic_instance_buffer,
            0,
            bytemuck::cast_slice(&[new_ball_instance_data]),
        );
    }

    fn render(&mut self, output: &wgpu::SurfaceTexture) -> wgpu::CommandBuffer {
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // We'll use a CommandEncoder to create the commands to send to the GPU.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // begin_render_pass borrows encoder mutably, so we start a new block
        // so that we drop render_pass, so that we can use encoder later.
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    // texture to save the colors into
                    view: &view,
                    // The texture that will receive the resolved output; defaults to view.
                    resolve_target: None,
                    // Tells wgpu what to do with the colors on the screen (i.e. in view).
                    ops: wgpu::Operations {
                        // load tells wgpu how to handle colors from the previous screen.
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        // If we want to store the rendered results to the Texture behind out TextureView.
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(1, self.static_instance_buffer.slice(..));
            use crate::model::DrawLight;
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model_instanced(
                &self.obj_model,
                STATIC_INSTANCE_INDEX_LIGHT..STATIC_INSTANCE_INDEX_LIGHT + 1,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.colored_render_pipeline);
            render_pass.draw_colored_mesh_instanced(
                &self.bounding_box_mesh,
                STATIC_INSTANCE_INDEX_BOUNDING_BOX..STATIC_INSTANCE_INDEX_BOUNDING_BOX + 1,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            // TODO we should build a more robust system for correlating models with the instance buffer,
            //      and their index(s) in the instance buffers. For now, since we have only 3 objects,
            //      I'll juggle them in code.
            render_pass.set_vertex_buffer(1, self.dynamic_instance_buffer.slice(..));
            render_pass.draw_colored_mesh_instanced(
                &self.sphere_mesh,
                DYNAMIC_INSTANCE_INDEX_BALL..DYNAMIC_INSTANCE_INDEX_BALL + 1,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        // Finish up the command buffer in finish(), and submit to the gpu's queue!
        encoder.finish()
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Our game loop follows the famous "fix your timestep!" model:
    // https://gafferongames.com/post/fix_your_timestep/
    // The state holds the accumulator.
    let mut state = State::new(&window).await;

    let mut gui = gui::Gui::new(&state.device, &state.config, &window);

    let mut current_time = std::time::SystemTime::now();
    event_loop.run(move |event, _, control_flow| {
        gui.handle_events(&event);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                let new_time = std::time::SystemTime::now();
                let frame_time = new_time.duration_since(current_time).unwrap();
                current_time = new_time;
                state.update(frame_time);
                let output = state.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                let gui_render_command_buffer = gui.render(frame_time, &state.device, &state.config, &state.queue, &window, &output);

                state.queue.submit([simulation_render_command_buffer, gui_render_command_buffer]);
                output.present();
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.mouse_pressed {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
