use super::utils;
use crate::graphics;
use crate::graphics::camera::CameraBundle;
use crate::graphics::entity::ColoredMeshEntity;
use crate::graphics::forms;
use crate::graphics::gpu_interface::GPUInterface;
use crate::graphics::instance::Instance;
use crate::graphics::light;
use crate::graphics::scene::Scene;
use crate::graphics::texture;
use crate::gui;
use crate::simulation;

use cgmath::Rotation3;
use cgmath::Vector3;
use cgmath::Zero;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

struct State {
    gpu: GPUInterface,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_bundle: CameraBundle,
    light_bind_group: wgpu::BindGroup,
    simulation_state: simulation::particles_cpu::particles::Simulation,
    scene: Scene,
    mouse_pressed: bool,
    time_accumulator: std::time::Duration,
}

impl State {
    fn new(window: &Window) -> Self {
        let gpu: GPUInterface = GPUInterface::new(&window);

        let camera_bundle =
            CameraBundle::new(&gpu, (0.0, 1.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(0.0));
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

        let obstacle = forms::get_cube_kilter(&gpu.device, [0.9, 0.1, 0.1]);

        let simulation_state = simulation::particles_cpu::particles::Simulation::new(&obstacle);

        let instances = vec![Instance {
            position: Vector3::<f32>::zero(),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        }];
        let obstacle_entity = ColoredMeshEntity::new(&gpu, obstacle, instances, None);

        let particles_entity = simulation_state.get_particles_entity(&gpu);
        let scene = Scene::new(
            None,
            Some(vec![obstacle_entity]),
            Some(vec![particles_entity]),
        );

        Self {
            gpu,
            render_pipeline,
            depth_texture,
            camera_bundle,
            light_bind_group,
            simulation_state,
            scene,
            mouse_pressed: false,
            time_accumulator: std::time::Duration::from_millis(0),
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

        // Simulate until our simulation has "consumed" the accumulated time in discrete, fixed timesteps.
        while self.time_accumulator >= self.simulation_state.get_timestep() {
            let elapsed_sim_time = self.simulation_state.step();
            self.time_accumulator = self.time_accumulator - elapsed_sim_time;
        }

        let particle_instances = self.simulation_state.get_particles_instances();
        self.scene.update_particle_instances(
            &self.gpu,
            0,
            particle_instances,
            self.camera_bundle.camera.position,
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

        {
            let mut render_pass =
                utils::begin_default_render_pass(&mut encoder, &view, &self.depth_texture.view);

            render_pass.set_pipeline(&self.render_pipeline);
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

    // Our game loop follows the famous "fix your timestep!" model:
    // https://gafferongames.com/post/fix_your_timestep/
    // The state holds the accumulator.
    let mut state = State::new(&window);

    let mut gui = gui::Gui::new(&state.gpu.device, &state.gpu.config, &window);
    let mut particles_ui = gui::particles::ParticlesUi::new();

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
                state.simulation_state.sync_sim_config_from_ui(&mut particles_ui);
                let output = state.gpu.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                let gui_render_command_buffer = gui.render(
                    &mut particles_ui,
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
