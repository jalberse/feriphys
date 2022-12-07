/// A demo of the spring-mass-damper simulation.
use crate::{
    graphics::{
        self, camera::CameraBundle, entity::ColoredMeshEntity, forms, gpu_interface::GPUInterface,
        instance::Instance, light, model::ColoredMesh, texture,
    },
    gui,
    simulation::sph::Simulation,
    simulation::{collidable_mesh::CollidableMesh, particles_cpu::particle},
};

use cgmath::Rotation3;
use itertools::Itertools;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use super::utils;

struct State {
    gpu: GPUInterface,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_bundle: CameraBundle,
    light_bind_group: wgpu::BindGroup,
    mouse_pressed: bool,
    time_accumulator: std::time::Duration,
    obstacle: CollidableMesh,
    simulation: Simulation,
}

impl State {
    fn new(window: &Window) -> Self {
        let gpu: GPUInterface = GPUInterface::new(&window);
        let camera_bundle =
            CameraBundle::new(&gpu, (0.0, 0.0, 9.0), cgmath::Deg(-90.0), cgmath::Deg(0.0));
        let depth_texture =
            texture::Texture::create_depth_texture(&gpu.device, &gpu.config, "depth texture");

        let light_uniform = light::LightUniform::new([6.0, 2.0, 6.0], [1.0, 1.0, 1.0]);
        let (light_bind_group_layout, light_bind_group) =
            light::create_light_bind_group(&gpu, light_uniform);

        let render_pipeline = graphics::util::create_colored_mesh_render_pipeline(
            &gpu,
            &camera_bundle,
            &light_bind_group_layout,
        );

        let obstacle = get_obstacle();
        let simulation = Simulation::new();

        Self {
            gpu,
            render_pipeline,
            depth_texture,
            camera_bundle,
            light_bind_group,
            mouse_pressed: false,
            time_accumulator: std::time::Duration::from_millis(0),
            obstacle,
            simulation,
        }
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
        utils::handle_input_default(event, &mut self.camera_bundle, &mut self.mouse_pressed)
    }

    fn update(&mut self, frame_time: std::time::Duration) {
        self.time_accumulator = self.time_accumulator + frame_time;
        self.camera_bundle.update_gpu(&self.gpu, frame_time);

        // TODO call simulation once we actually have it
        // while self.time_accumulator >= self.simulation.get_timestep() {
        //     let elapsed_sim_time = self.simulation.step();
        //     self.time_accumulator = self.time_accumulator - elapsed_sim_time;
        // }
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

        let obstacle_mesh = ColoredMesh::from_collidable_mesh(
            &self.gpu.device,
            "floor".to_string(),
            &self.obstacle,
            [0.1, 0.9, 0.1],
        );
        let obstacle_instances = vec![Instance::default()];
        let obstacle_entity = ColoredMeshEntity::new(&self.gpu, obstacle_mesh, obstacle_instances);

        // TODO maybe cache the sphere lol
        let sphere = forms::generate_sphere(&self.gpu.device, [0.9, 0.1, 0.1], 0.05, 16, 16);
        let particles = self.simulation.get_particles();
        let particle_instances = particles
            .iter()
            .map(|p| Instance {
                position: *p.position(),
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 1.0,
            })
            .collect_vec();
        let particles_entity = ColoredMeshEntity::new(&self.gpu, sphere, particle_instances);

        // TODO get other data from simulation to update Instance data to e.g. color by density, pressure, velocity, curl, etc.
        //         That might be a function that takes an Enum for DataRequest and returns a color for it in the simulation, or something.

        {
            let mut render_pass =
                utils::begin_default_render_pass(&mut encoder, &view, &self.depth_texture.view);

            render_pass.set_pipeline(&self.render_pipeline);
            obstacle_entity.draw(
                &mut render_pass,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );
            particles_entity.draw(
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
    // TODO get sph UI once made

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
                // TODO sync sim from UI
                let output = state.gpu.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                // TODO get gui_render_command_buffer

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

fn get_obstacle() -> CollidableMesh {
    let (vertex_positions, indices) = graphics::forms::get_cube_interior_normals_vertices();
    let vertex_positions = vertex_positions.iter().map(|v| v * 2.0).collect_vec();
    CollidableMesh::new(vertex_positions, indices)
}
