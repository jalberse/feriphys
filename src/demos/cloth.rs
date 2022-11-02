/// A demo of the spring-mass-damper simulation.
use crate::{
    graphics::{
        self, camera::CameraBundle, entity::ColoredMeshEntity, gpu_interface::GPUInterface,
        instance::Instance, light, model::ColoredMesh, scene::Scene, texture,
    },
    gui,
    simulation::springy::cloth::Cloth,
    simulation::springy::{obstacle::Obstacle, simulation::Simulation},
};

use cgmath::{Vector3, Zero};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use super::utils;

struct State {
    simulation: Simulation,
    gpu: GPUInterface,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    camera_bundle: CameraBundle,
    light_bind_group: wgpu::BindGroup,
    scene: Scene,
    mouse_pressed: bool,
    time_accumulator: std::time::Duration,
}

impl State {
    fn new(window: &Window) -> Self {
        let gpu: GPUInterface = GPUInterface::new(&window);
        let camera_bundle =
            CameraBundle::new(&gpu, (0.0, 0.0, 5.0), cgmath::Deg(-90.0), cgmath::Deg(0.0));
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

        let rows = 20 as usize;
        let cols = 20 as usize;
        let tablecloth = Cloth::new(
            rows,
            cols,
            0.1,
            Vector3::<f32>::zero(),
            10.0,
            2000.0,
            200.0,
            500.0,
            20.0,
            5.0,
            2.0,
            vec![
                rows * cols - 1,
                (rows * cols) - cols,
                (rows * cols) - (cols / 2),
            ],
        );
        let tablecloth_mesh = tablecloth.mesh;
        let obstacles = get_obstacles();
        let simulation = Simulation::new(vec![tablecloth_mesh], obstacles);

        // Note we're keeping the scene around since we'll probably have some static obstacles that we'd like to draw
        // for the springy mesh to interact with.
        let scene = Scene::new(None, None, None);

        Self {
            simulation,
            gpu,
            render_pipeline,
            depth_texture,
            camera_bundle,
            light_bind_group,
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

        while self.time_accumulator >= self.simulation.get_timestep() {
            let elapsed_sim_time = self.simulation.step();
            self.time_accumulator = self.time_accumulator - elapsed_sim_time;
        }
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

        // TODO handle rendering *all* springy meshes in simulation
        let cube_mesh = ColoredMesh::from_springy_mesh(
            &self.gpu.device,
            "springy cube".to_string(),
            &self.simulation.get_meshes()[0],
            [0.9, 0.1, 0.1],
        );
        let cube_instances = vec![Instance::default()];
        let cube_entity = ColoredMeshEntity::new(&self.gpu, cube_mesh, cube_instances);

        // TODO handle rendering *all* obstacles in simulation
        let obstacle_mesh = ColoredMesh::from_obstacle(
            &self.gpu.device,
            "floor".to_string(),
            &self.simulation.get_obstacles()[0],
            [0.1, 0.9, 0.1],
        );
        let obstacle_instances = vec![Instance::default()];
        let obstacle_entity = ColoredMeshEntity::new(&self.gpu, obstacle_mesh, obstacle_instances);

        {
            let mut render_pass =
                utils::begin_default_render_pass(&mut encoder, &view, &self.depth_texture.view);

            render_pass.set_pipeline(&self.render_pipeline);
            self.scene.draw_colored_mesh_entities(
                &mut render_pass,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );
            cube_entity.draw(
                &mut render_pass,
                &self.camera_bundle.camera_bind_group,
                &self.light_bind_group,
            );
            obstacle_entity.draw(
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
    let mut ui = gui::spring_mass_damper::SpringMassDamperUi::new();

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
                state.simulation.sync_sim_config_from_ui(&mut ui);
                let output = state.gpu.surface.get_current_texture().unwrap();
                let simulation_render_command_buffer = state.render(&output);
                let gui_render_command_buffer = gui.render(
                    &mut ui,
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

fn get_obstacles() -> Vec<Obstacle> {
    let vertex_positions = vec![
        -Vector3::<f32>::unit_x() + Vector3::<f32>::unit_z() - Vector3::<f32>::unit_y() * 2.0,
        Vector3::<f32>::unit_x() + Vector3::<f32>::unit_z() - Vector3::<f32>::unit_y() * 2.0,
        Vector3::<f32>::unit_x() - Vector3::<f32>::unit_z() - Vector3::<f32>::unit_y() * 2.0,
        -Vector3::<f32>::unit_x() - Vector3::<f32>::unit_z() - Vector3::<f32>::unit_y() * 2.0,
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    vec![Obstacle::new(vertex_positions, indices)]
}
