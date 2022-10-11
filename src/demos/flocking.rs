use crate::{
    graphics::{
        self,
        camera::CameraBundle,
        entity::{ColoredMeshEntity, Entity},
        forms,
        gpu_interface::GPUInterface,
        instance::Instance,
        light, resources,
        scene::Scene,
        texture,
    },
    gui,
    simulation::{
        self,
        flocking::{flocking, obstacle::Obstacle},
    },
};

use cgmath::{Rotation3, Vector3};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

struct State {
    gpu: GPUInterface,
    model_render_pipeline: wgpu::RenderPipeline,
    colored_mesh_render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_bundle: CameraBundle,
    light_bind_group: wgpu::BindGroup,
    simulation: flocking::Simulation,
    scene: Scene,
    mouse_pressed: bool,
    time_accumulator: std::time::Duration,
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

        let model_render_pipeline = graphics::util::create_model_render_pipeline(
            &gpu,
            &camera_bundle,
            &light_bind_group_layout,
        );
        let colored_mesh_render_pipeline = graphics::util::create_colored_mesh_render_pipeline(
            &gpu,
            &camera_bundle,
            &light_bind_group_layout,
        );

        let cube = forms::get_cube(&gpu.device, [0.9, 0.1, 0.1]);
        let cube_instances = vec![Instance {
            position: Vector3::<f32> {
                x: -6.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        }];
        let cube_entity = ColoredMeshEntity::new(&gpu, cube, cube_instances);

        let simulation = Self::create_sim(&cube_entity);

        let texture_bind_group_layout = graphics::util::create_texture_bind_group_layout(&gpu);
        let fish_model = resources::load_model(
            "blue_fish.obj",
            &gpu.device,
            &gpu.queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        let instances = simulation.get_boid_instances();

        let boids_entity = Entity::new(&gpu, fish_model, instances);

        let scene = Scene::new(Some(vec![boids_entity]), Some(vec![cube_entity]), None);

        Self {
            gpu,
            model_render_pipeline,
            colored_mesh_render_pipeline,
            depth_texture,
            camera_bundle,
            light_bind_group,
            simulation,
            scene,
            mouse_pressed: false,
            time_accumulator: std::time::Duration::from_millis(0),
        }
    }

    fn create_sim(obstacle_entity: &ColoredMeshEntity) -> flocking::Simulation {
        let bounding_box = simulation::bounding_box::BoundingBox {
            x_range: (-10.0..10.0),
            y_range: (-10.0..10.0),
            z_range: (-10.0..10.0),
        };

        let lead_boid = simulation::flocking::boid::LeadBoid::new(|t| -> Vector3<f32> {
            Vector3::<f32> {
                x: 6.0 * f32::cos(t / 2.0),
                y: 0.0,
                z: 6.0 * f32::sin(t / 2.0),
            }
        });
        let lead_boids = Some(vec![lead_boid]);

        let obstacles = Obstacle::from_entity(obstacle_entity, 1.75);

        flocking::Simulation::new(bounding_box, lead_boids, Some(obstacles), None)
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        graphics::util::resize(
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
        self.time_accumulator = self.time_accumulator + frame_time;
        self.camera_bundle.update_gpu(&self.gpu, frame_time);

        while self.time_accumulator >= self.simulation.get_timestep() {
            let elapsed_sim_time = self.simulation.step();
            self.time_accumulator = self.time_accumulator - elapsed_sim_time;
        }

        let new_instances = self.simulation.get_boid_instances();
        self.scene
            .update_entity_instances(&self.gpu, 0, new_instances);
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

            render_pass.set_pipeline(&self.model_render_pipeline);
            self.scene.draw_entities(
                &mut render_pass,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );
            render_pass.set_pipeline(&self.colored_mesh_render_pipeline);
            self.scene.draw_colored_mesh_entities(
                &mut render_pass,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );
        }

        encoder.finish()
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window);

    let mut gui = gui::Gui::new(&state.gpu.device, &state.gpu.config, &window);
    let mut flocking_ui = gui::flocking::FlockingUi::new();

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
                state.simulation.sync_sim_config_from_ui(&mut flocking_ui);
                let output = state.gpu.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                let gui_render_command_buffer = gui.render(
                    &mut flocking_ui,
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
