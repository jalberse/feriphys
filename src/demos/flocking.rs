use crate::{
    graphics::{
        self, camera::CameraBundle, entity::Entity, gpu_interface::GPUInterface,
        instance::Instance, light, resources, scene::Scene, texture,
    },
    gui,
    simulation::{
        self,
        flocking::{flocking, obstacle::Obstacle},
    },
};

use cgmath::{Rotation3, Vector3, Zero};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use super::utils;

struct State {
    gpu: GPUInterface,
    model_render_pipeline: wgpu::RenderPipeline,
    colored_mesh_render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_bundle: CameraBundle,
    light_bind_group: wgpu::BindGroup,
    // TODO use a vec of simulations instead of this.
    simulation: flocking::Simulation,
    simulation_2: flocking::Simulation,
    scene: Scene,
    mouse_pressed: bool,
    time_accumulator: std::time::Duration,
    // TODO this is used for accumulating simulations for the second simulation.
    //   The time accumulator should likely be associated with a simulation.
    //   Simulation could possibly be a trait to share this kind of thing.
    time_accumulator_2: std::time::Duration,
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

        let texture_bind_group_layout = graphics::util::create_texture_bind_group_layout(&gpu);

        // Set up the environment.
        let seafloor_tile_model = resources::load_model(
            "seafloor.obj",
            &gpu.device,
            &gpu.queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        let seafloor_tile_instances = vec![Instance {
            position: Vector3::<f32>::zero(),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 30.0,
        }];
        let seafloor_entity = Entity::new(&gpu, seafloor_tile_model, seafloor_tile_instances);

        let ship_model = resources::load_model(
            "pirate_ship.obj",
            &gpu.device,
            &gpu.queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        let ship_instances = vec![Instance {
            position: Vector3::<f32>::new(-5.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        }];
        let ship_entity = Entity::new(&gpu, ship_model, ship_instances);
        let obstacles = Obstacle::from_entity(&ship_entity, 4.0);
        let obstacles_2 = obstacles.clone();

        // Set up the first simulation
        let lead_boid = simulation::flocking::boid::LeadBoid::new(|t| -> Vector3<f32> {
            Vector3::<f32>::new(25.0 * f32::cos(t / 12.0), 0.5, 0.0)
        });
        let lead_boids = Some(vec![lead_boid]);

        let initial_boids_position = Vector3::<f32>::new(25.0, 0.5, 0.0);

        let num_boids = if cfg!(debug_assertions) { 30 } else { 110 };

        let simulation = flocking::Simulation::new(
            vec![initial_boids_position],
            num_boids,
            None,
            lead_boids,
            Some(obstacles),
            None,
        );

        // Add the first simulation info to the scene
        let fish_model = resources::load_model(
            "blue_fish.obj",
            &gpu.device,
            &gpu.queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        let instances = simulation.get_boid_instances();

        let boids_entity = Entity::new(&gpu, fish_model, instances);

        // Set up the second simulation that we'll display alongside the first
        let initial_boids_position_2_0 = Vector3::<f32>::new(15.0, 10.0, 0.0);
        let initial_boids_position_2_1 = Vector3::<f32>::new(25.0, 0.5, 0.0);
        let lead_boid = simulation::flocking::boid::LeadBoid::new(|t| -> Vector3<f32> {
            Vector3::<f32>::new(
                15.0 * f32::cos(t / 12.0),
                6.0 + 5.0 * f32::cos(t / 12.0),
                15.0 * f32::sin(t / 12.0),
            )
        });
        let lead_boid_2 = simulation::flocking::boid::LeadBoid::new(|t| -> Vector3<f32> {
            Vector3::<f32>::new(25.0 * f32::cos(t / 10.0), 1.0, 10.0 * f32::sin(t / 9.0))
        });
        let lead_boids = Some(vec![lead_boid, lead_boid_2]);
        let simulation_2 = flocking::Simulation::new(
            vec![initial_boids_position_2_0, initial_boids_position_2_1],
            num_boids,
            None,
            lead_boids,
            Some(obstacles_2),
            None,
        );

        // Add the second simulation info to the scene
        let fish_model_2 = resources::load_model(
            "yellow_fish.obj",
            &gpu.device,
            &gpu.queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        let instances = simulation_2.get_boid_instances();

        let boids_entity_2 = Entity::new(&gpu, fish_model_2, instances);

        let scene = Scene::new(
            Some(vec![
                boids_entity,
                boids_entity_2,
                seafloor_entity,
                ship_entity,
            ]),
            None,
            None,
        );

        Self {
            gpu,
            model_render_pipeline,
            colored_mesh_render_pipeline,
            depth_texture,
            camera_bundle,
            light_bind_group,
            simulation,
            simulation_2,
            scene,
            mouse_pressed: false,
            time_accumulator: std::time::Duration::from_millis(0),
            time_accumulator_2: std::time::Duration::from_millis(0),
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
        self.time_accumulator_2 = self.time_accumulator_2 + frame_time;
        self.camera_bundle.update_gpu(&self.gpu, frame_time);

        while self.time_accumulator >= self.simulation.get_timestep() {
            let elapsed_sim_time = self.simulation.step();
            self.time_accumulator = self.time_accumulator - elapsed_sim_time;
        }

        while self.time_accumulator_2 >= self.simulation_2.get_timestep() {
            let elapsed_sim_time = self.simulation_2.step();
            self.time_accumulator_2 = self.time_accumulator_2 - elapsed_sim_time;
        }

        let new_instances = self.simulation.get_boid_instances();
        self.scene
            .update_entity_instances(&self.gpu, 0, new_instances);

        let new_instances = self.simulation_2.get_boid_instances();
        self.scene
            .update_entity_instances(&self.gpu, 1, new_instances);
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
                state.simulation_2.sync_sim_config_from_ui(&mut flocking_ui);
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
