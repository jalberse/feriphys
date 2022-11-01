pub mod bounce;
pub mod flocking;
pub mod particles;
pub mod spring_mass_damper;

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{event::Event, window::Window};

pub trait Ui {
    fn ui(&mut self, ctx: &egui::Context);
}

pub struct Gui {
    platform: Platform,
    render_pass: RenderPass,
}

impl Gui {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, window: &Window) -> Gui {
        let size = window.inner_size();
        let platform = Platform::new(PlatformDescriptor {
            physical_width: (size.width as u32) / 2,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let egui_rpass = RenderPass::new(device, config.format, 1);

        Gui {
            platform,
            render_pass: egui_rpass,
        }
    }

    pub fn handle_events(&mut self, event: &Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn render<T: Ui>(
        &mut self,
        ui: &mut T,
        dt: std::time::Duration,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        window: &Window,
        output: &wgpu::SurfaceTexture,
    ) -> wgpu::CommandBuffer {
        self.platform.update_time(dt.as_secs_f64());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Gui Render Encoder"),
        });
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        self.platform.begin_frame();

        // Draw the UI.
        ui.ui(&self.platform.context());

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let full_output = self.platform.end_frame(Some(window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: config.width,
            physical_height: config.height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        self.render_pass
            .add_textures(device, queue, &tdelta)
            .expect("add texture ok");
        self.render_pass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.render_pass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();
        encoder.finish()
    }
}
