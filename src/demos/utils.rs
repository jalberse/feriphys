use wgpu::{CommandEncoder, RenderPass, TextureView};
use winit::event::{ElementState, KeyboardInput, MouseButton, WindowEvent};

use crate::graphics::camera::CameraBundle;

pub fn begin_default_render_pass<'pass>(
    encoder: &'pass mut CommandEncoder,
    view: &'pass TextureView,
    depth_texture_view: &'pass TextureView,
) -> RenderPass<'pass> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            // texture to save the colors into
            view: view,
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
            view: depth_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
    })
}

pub fn handle_input_default(
    event: &WindowEvent,
    camera_bundle: &mut CameraBundle,
    mouse_pressed_state: &mut bool,
) -> bool {
    match event {
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },
            ..
        } => camera_bundle
            .camera_controller
            .process_keyboard(*key, *state),
        WindowEvent::MouseWheel { delta, .. } => {
            camera_bundle.camera_controller.process_scroll(delta);
            true
        }
        WindowEvent::MouseInput {
            button: MouseButton::Left,
            state,
            ..
        } => {
            *mouse_pressed_state = *state == ElementState::Pressed;
            true
        }
        _ => false,
    }
}
