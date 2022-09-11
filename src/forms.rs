/// The forms module provides basic forms (planes, spheres, cubes...) for rendering.
use super::model;
use wgpu::util::DeviceExt;

pub fn get_hexagon(device: &wgpu::Device) -> model::ColoredMesh {
    // Make a colored mesh to draw.
    let vertices: &[model::ColoredVertex] = &[
        model::ColoredVertex {
            position: [-0.0868241, 0.49240386, 0.0],
            color: [0.5, 0.0, 0.5],
        }, // A
        model::ColoredVertex {
            position: [-0.49513406, 0.06958647, 0.0],
            color: [0.5, 0.0, 0.5],
        }, // B
        model::ColoredVertex {
            position: [-0.21918549, -0.44939706, 0.0],
            color: [0.5, 0.0, 0.5],
        }, // C
        model::ColoredVertex {
            position: [0.35966998, -0.3473291, 0.0],
            color: [0.5, 0.0, 0.5],
        }, // D
        model::ColoredVertex {
            position: [0.44147372, 0.2347359, 0.0],
            color: [0.5, 0.0, 0.5],
        }, // E
    ];

    let indices: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
    let num_indices = indices.len() as u32;

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mesh colored vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mesh colored index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    model::ColoredMesh {
        name: "Colored Mesh".to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    }
}
