use crate::camera::{Camera, CameraBundle, CameraController, CameraUniform, Projection};
use crate::gpu_interface::GPUInterface;
use crate::instance::{Instance, InstanceRaw};
use crate::light;
use crate::model::{ColoredMesh, ColoredVertex, DrawColoredMesh, Model, ModelVertex, Vertex};
use crate::rendering;
use crate::texture;
use crate::{gui, utilities};

use cgmath::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

struct State {
    gpu: GPUInterface,
    time_accumulator: std::time::Duration,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_bundle: CameraBundle,
    mouse_pressed: bool,
}

impl State {
    fn new(window: &Window) -> Self {
        let gpu: GPUInterface = GPUInterface::new(&window);

        let camera_bundle = CameraBundle::new(&gpu);
        let depth_texture =
            texture::Texture::create_depth_texture(&gpu.device, &gpu.config, "depth texture");

        let light_uniform = light::LightUniform::new([6.0, 2.0, 6.0], [1.0, 1.0, 1.0]);
        let (light_bind_group_layout, light_bind_group) =
            light::create_light_bind_group(&gpu, light_uniform);

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Colored Mesh Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("color_shader.wgsl").into()),
            };
            let layout = gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Colored Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_bundle.camera_bind_group_layout,
                        &light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            rendering::create_render_pipeline(
                &gpu.device,
                &layout,
                gpu.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[ModelVertex::desc(), InstanceRaw::desc::<5>()],
                shader,
            )
        };

        Self {
            gpu,
            time_accumulator: std::time::Duration::from_millis(0),
            render_pipeline,
            depth_texture,
            camera_bundle,
            mouse_pressed: false,
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

    fn update(&mut self, frame_time: std::time::Duration) {}

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

            // TODO We want to define an instance buffer and a quad mesh and draw them here.
            //      Either here or earlier, the instances should be modified to ensure the quads are
            //      facing the camera.
            // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            // render_pass.draw_colored_mesh_instanced(
            //     &self.sphere_mesh,
            //     DYNAMIC_INSTANCE_INDEX_BALL..DYNAMIC_INSTANCE_INDEX_BALL + 1,
            //     &self.camera_bundle.camera_bind_group,
            //     &self.light_bind_group,
            // );
        }

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
    // TODO we'll want to define some particles UI and create it here to pass in.

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
                // TODO sync state from UI.
                let output = state.gpu.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                // TODO get some gui_render_command_buffer once we add UI.

                state.gpu.queue.submit([simulation_render_command_buffer]);
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
