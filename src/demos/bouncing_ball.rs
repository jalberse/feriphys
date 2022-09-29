use crate::camera::CameraBundle;
use crate::forms;
use crate::gpu_interface::GPUInterface;
use crate::gui;
use crate::instance::{Instance, InstanceRaw};
use crate::light;
use crate::model::{ColoredMesh, DrawColoredMesh, Model, ModelVertex, Vertex};
use crate::rendering;
use crate::resources;
use crate::simulation;
use crate::texture;
use crate::utilities;
use cgmath::prelude::*;
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

struct State {
    gpu: GPUInterface,
    time_accumulator: std::time::Duration,
    #[allow(dead_code)]
    render_pipeline: wgpu::RenderPipeline,
    obj_model: Model,
    camera_bundle: CameraBundle,
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
    bounding_box_mesh: ColoredMesh,
    sphere_mesh: ColoredMesh,
    simulation_state: simulation::bounce::State,
}

impl State {
    // Creating some of the wgpu types requires async types
    fn new(window: &Window) -> Self {
        let gpu: GPUInterface = GPUInterface::new(&window);

        let texture_bind_group_layout = rendering::create_texture_bind_group_layout(&gpu);

        let camera_bundle = CameraBundle::new(&gpu);

        let light_uniform = light::LightUniform::new([6.0, 2.0, 6.0], [1.0, 1.0, 1.0]);
        let (light_bind_group_layout, light_bind_group) =
            light::create_light_bind_group(&gpu, light_uniform);

        let depth_texture =
            texture::Texture::create_depth_texture(&gpu.device, &gpu.config, "depth texture");

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &camera_bundle.camera_bind_group_layout,
                        &light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        // Render pipeline for textured models
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
            };
            rendering::create_render_pipeline(
                &gpu.device,
                &render_pipeline_layout,
                gpu.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[ModelVertex::desc(), InstanceRaw::desc::<5>()],
                shader,
            )
        };

        // Render pipeline for our physical light object in the scene.
        let light_render_pipeline = {
            let layout = gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Light Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_bundle.camera_bind_group_layout,
                        &light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light.wgsl").into()),
            };
            rendering::create_render_pipeline(
                &gpu.device,
                &layout,
                gpu.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[ModelVertex::desc()],
                shader,
            )
        };

        // Render pipeline for colored meshes without any textures.
        let colored_render_pipeline = rendering::create_colored_mesh_render_pipeline(
            &gpu,
            &camera_bundle,
            &light_bind_group_layout,
        );

        let lightbulb_model = resources::load_model(
            "cube.obj",
            &gpu.device,
            &gpu.queue,
            &texture_bind_group_layout,
        )
        .unwrap();

        let bounding_box_mesh = forms::get_cube_interior_normals(&gpu.device, [0.5, 0.0, 0.5]);
        let sphere_mesh = forms::generate_sphere(&gpu.device, [0.2, 0.8, 0.2], 1.0, 32, 32);

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
        let static_instance_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Dynamic Instance Buffer"),
                    contents: bytemuck::cast_slice(&dynamic_instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        let simulation_state = simulation::bounce::State::new();

        Self {
            gpu,
            time_accumulator: std::time::Duration::from_millis(0),
            render_pipeline,
            obj_model: lightbulb_model,
            camera_bundle,
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
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        utilities::resize(
            new_size,
            &mut self.gpu,
            &mut self.depth_texture,
            &mut self.camera_bundle.projection,
        );
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
            } => self
                .camera_bundle
                .camera_controller
                .process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_bundle.camera_controller.process_scroll(delta);
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

        self.camera_bundle.update_gpu(&self.gpu, frame_time);

        // SIMULATE until our simulation has "consumed" the accumulated time in discrete, fixed timesteps.
        while self.time_accumulator >= self.simulation_state.get_timestep() {
            // Note that our elapsed simulation time might be less than SIMULATION_DT if a collision occured.
            // That's OK, just continue simulating the next time step from the collision next iteration.
            let elapsed_sim_time = self.simulation_state.step();
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
        self.gpu.queue.write_buffer(
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
            .gpu
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
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );

            render_pass.set_pipeline(&self.colored_render_pipeline);
            render_pass.draw_colored_mesh_instanced(
                &self.bounding_box_mesh,
                STATIC_INSTANCE_INDEX_BOUNDING_BOX..STATIC_INSTANCE_INDEX_BOUNDING_BOX + 1,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );

            // TODO we should build a more robust system for correlating models with the instance buffer,
            //      and their index(s) in the instance buffers. For now, since we have only 3 objects,
            //      I'll juggle them in code.
            render_pass.set_vertex_buffer(1, self.dynamic_instance_buffer.slice(..));
            render_pass.draw_colored_mesh_instanced(
                &self.sphere_mesh,
                DYNAMIC_INSTANCE_INDEX_BALL..DYNAMIC_INSTANCE_INDEX_BALL + 1,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );
        }

        // Finish up the command buffer in finish(), and submit to the gpu's queue!
        encoder.finish()
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Our game loop follows the famous "fix your timestep!" model:
    // https://gafferongames.com/post/fix_your_timestep/
    // The state holds the accumulator.
    let mut state = State::new(&window);

    let mut gui = gui::Gui::new(&state.gpu.device, &state.gpu.config, &window);
    let mut bouncing_ball_ui = gui::bounce_gui::BouncingBallUi::new();

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
                state.simulation_state.sync_state_from_ui(&mut bouncing_ball_ui);
                let output = state.gpu.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                let gui_render_command_buffer = gui.render(
                    &mut bouncing_ball_ui,
                    frame_time,
                    &state.gpu.device,
                    &state.gpu.config,
                    &state.gpu.queue,
                    &window,
                    &output
                );

                state.gpu.queue.submit([simulation_render_command_buffer, gui_render_command_buffer]);
                output.present();
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.mouse_pressed {
                state.camera_bundle.camera_controller.process_mouse(delta.0, delta.1)
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
