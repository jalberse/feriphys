/// The forms module provides basic forms (planes, spheres, cubes...) for rendering.
use super::{model, util::get_normals};

use cgmath::Vector3;
use wgpu::util::DeviceExt;

/// Zips the vertex positions with their normals, and adds the color,
/// to get the ColoredVertex. Normals can be gotten from vertex positions
/// and their indices using get_normals().
///
/// Panics if vertex_position and normals are of different lengths.
fn get_colored_vertices(
    vertex_positions: &Vec<cgmath::Vector3<f32>>,
    normals: &Vec<cgmath::Vector3<f32>>,
    color: [f32; 3],
) -> Vec<model::ColoredVertex> {
    vertex_positions
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| -> model::ColoredVertex {
            model::ColoredVertex {
                position: [v.x, v.y, v.z],
                color,
                normal: [n.x, n.y, n.z],
            }
        })
        .collect::<Vec<_>>()
}

/// Gets the vertex buffer and index buffer, respectively.
fn get_buffers(
    device: &wgpu::Device,
    vertex_positions: &Vec<Vector3<f32>>,
    indices: &Vec<u16>,
    color: [f32; 3],
) -> (wgpu::Buffer, wgpu::Buffer) {
    let normals = get_normals(&vertex_positions, &indices);
    let vertices = get_colored_vertices(&vertex_positions, &normals, color);

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

    (vertex_buffer, index_buffer)
}

#[allow(dead_code)]
pub fn get_cube_interior_normals(device: &wgpu::Device, color: [f32; 3]) -> model::ColoredMesh {
    let vertex_positions = vec![
        // front
        cgmath::Vector3 {
            x: -1.0,
            y: -1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: -1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: 1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: -1.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: -1.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: 1.0,
            z: -1.0,
        },
    ];

    let indices: Vec<u16> = vec![
        2, 1, 0, 0, 3, 2, // front
        6, 5, 1, 1, 2, 6, // right
        5, 6, 7, 7, 4, 5, // back
        3, 0, 4, 4, 7, 3, // left
        1, 5, 4, 4, 0, 1, // bottom
        6, 2, 3, 3, 7, 6, // top
    ];

    // Cubes with averaged vertex normals look bad withoutholding edges. So we'll use non-averaged
    // vertexes. That means generating the duplicate ones, and using 0..n as indices.
    let vertex_positions: Vec<Vector3<f32>> = indices
        .iter()
        .map(|i| -> Vector3<f32> { vertex_positions[*i as usize] })
        .collect();
    let vertex_indices = Vec::from_iter(0..vertex_positions.len() as u16);

    let num_elements = vertex_indices.len() as u32;

    let (vertex_buffer, index_buffer) =
        get_buffers(device, &vertex_positions, &vertex_indices, color);

    model::ColoredMesh {
        name: "Colored Cube".to_string(),
        vertex_positions,
        vertex_indices,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}

/// Generates a sphere mesh with the specified color, radius, and number of sectors and stacks.
/// The vertices have their normals averaged across adjacent faces.
pub fn generate_sphere(
    device: &wgpu::Device,
    color: [f32; 3],
    radius: f32,
    sectors: u16,
    stacks: u16,
) -> model::ColoredMesh {
    let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
    let stack_step = std::f32::consts::PI / stacks as f32;

    let mut vertex_positions = Vec::new();
    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
        let xy = radius * f32::cos(stack_angle);
        let z = radius * f32::sin(stack_angle);

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;
            let x = xy * f32::cos(sector_angle);
            let y = xy * f32::sin(sector_angle);
            vertex_positions.push(cgmath::Vector3 { x, y, z });
        }
    }

    // generate CCW index list of sphere triangles
    // k1--k1+1
    // |  / |
    // | /  |
    // k2--k2+1
    let mut vertex_indices: Vec<u16> = Vec::new();

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for _j in 0..sectors {
            // First and last stacks do not need quads, just tris.
            if i != 0 {
                vertex_indices.push(k1);
                vertex_indices.push(k2);
                vertex_indices.push(k1 + 1);
            }
            if i != (stacks - 1) {
                vertex_indices.push(k1 + 1);
                vertex_indices.push(k2);
                vertex_indices.push(k2 + 1);
            }
            k1 = k1 + 1;
            k2 = k2 + 1;
        }
    }

    let num_elements = vertex_indices.len() as u32;

    let (vertex_buffer, index_buffer) =
        get_buffers(device, &vertex_positions, &vertex_indices, color);

    model::ColoredMesh {
        name: "Colored Sphere".to_string(),
        vertex_positions,
        vertex_indices,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}

#[allow(dead_code)]
pub fn get_cube(device: &wgpu::Device, color: [f32; 3]) -> model::ColoredMesh {
    let vertex_positions = vec![
        // front
        cgmath::Vector3 {
            x: -1.0,
            y: -1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: -1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: 1.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: -1.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: -1.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: 1.0,
            z: -1.0,
        },
    ];

    let indices: Vec<u16> = vec![
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        7, 6, 5, 5, 4, 7, // back
        4, 0, 3, 3, 7, 4, // left
        4, 5, 1, 1, 0, 4, // bottom
        3, 2, 6, 6, 7, 3, // top
    ];

    // Cubes with averaged vertex normals look bad without holding edges. So we'll use non-averaged
    // vertexes. That means generating the duplicate ones, and using 0..n as indices.
    let vertex_positions: Vec<cgmath::Vector3<f32>> = indices
        .iter()
        .map(|i| -> cgmath::Vector3<f32> { vertex_positions[*i as usize] })
        .collect();
    let vertex_indices = Vec::from_iter(0..vertex_positions.len() as u16);

    let num_elements = vertex_indices.len() as u32;

    let (vertex_buffer, index_buffer) =
        get_buffers(device, &vertex_positions, &vertex_indices, color);

    model::ColoredMesh {
        name: "Colored Cube".to_string(),
        vertex_positions,
        vertex_indices,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}

pub fn get_cube_kilter(device: &wgpu::Device, color: [f32; 3]) -> model::ColoredMesh {
    let vertex_positions = vec![
        // front
        cgmath::Vector3 {
            x: -0.5,
            y: -0.7,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: 0.8,
            y: -0.8,
            z: 0.9,
        },
        cgmath::Vector3 {
            x: 0.75,
            y: 0.5,
            z: 0.6,
        },
        cgmath::Vector3 {
            x: -0.8,
            y: 0.5,
            z: 0.55,
        },
        cgmath::Vector3 {
            x: -0.6,
            y: -0.6,
            z: -0.6,
        },
        cgmath::Vector3 {
            x: 0.7,
            y: -0.7,
            z: -0.7,
        },
        cgmath::Vector3 {
            x: 0.75,
            y: 0.75,
            z: -0.75,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: 1.0,
            z: -1.0,
        },
    ];

    let indices: Vec<u16> = vec![
        0, 1, 2, 2, 3, 0, // front
        1, 5, 6, 6, 2, 1, // right
        7, 6, 5, 5, 4, 7, // back
        4, 0, 3, 3, 7, 4, // left
        4, 5, 1, 1, 0, 4, // bottom
        3, 2, 6, 6, 7, 3, // top
    ];

    // Cubes with averaged vertex normals look bad withoutholding edges. So we'll use non-averaged
    // vertexes. That means generating the duplicate ones, and using 0..n as indices.
    let vertex_positions: Vec<cgmath::Vector3<f32>> = indices
        .iter()
        .map(|i| -> cgmath::Vector3<f32> { vertex_positions[*i as usize] })
        .collect();
    let vertex_indices = Vec::from_iter(0..vertex_positions.len() as u16);

    let num_elements = vertex_indices.len() as u32;

    let (vertex_buffer, index_buffer) =
        get_buffers(device, &vertex_positions, &vertex_indices, color);

    model::ColoredMesh {
        name: "Colored Cube".to_string(),
        vertex_positions,
        vertex_indices,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}

/// Returns a 1x1 quad in the y plane centered on the origin, with normals
/// in the positive y direction.
pub fn get_quad(device: &wgpu::Device, color: [f32; 3]) -> model::ColoredMesh {
    let vertex_positions = vec![
        cgmath::Vector3 {
            x: -0.5,
            y: 0.0,
            z: 0.5,
        },
        cgmath::Vector3 {
            x: 0.5,
            y: 0.0,
            z: 0.5,
        },
        cgmath::Vector3 {
            x: -0.5,
            y: 0.0,
            z: -0.5,
        },
        cgmath::Vector3 {
            x: 0.5,
            y: 0.0,
            z: -0.5,
        },
    ];

    let vertex_indices: Vec<u16> = vec![1, 3, 2, 2, 0, 1];
    let num_elements = vertex_indices.len() as u32;

    let (vertex_buffer, index_buffer) =
        get_buffers(device, &vertex_positions, &vertex_indices, color);

    model::ColoredMesh {
        name: "Colored Quad".to_string(),
        vertex_positions,
        vertex_indices,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}

#[allow(dead_code)]
pub fn get_hexagon(device: &wgpu::Device, color: [f32; 3]) -> model::ColoredMesh {
    let vertex_positions = vec![
        cgmath::Vector3 {
            x: -0.0868241,
            y: 0.49240386,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: -0.49513406,
            y: 0.06958647,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: -0.21918549,
            y: -0.44939706,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: 0.35966998,
            y: -0.3473291,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: 0.44147372,
            y: 0.2347359,
            z: 0.0,
        },
    ];

    let vertex_indices: Vec<u16> = vec![0, 1, 4, 1, 2, 4, 2, 3, 4];
    let num_elements = vertex_indices.len() as u32;
    let normals = get_normals(&vertex_positions, &vertex_indices);
    let vertices = get_colored_vertices(&vertex_positions, &normals, color);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mesh colored vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mesh colored index buffer"),
        contents: bytemuck::cast_slice(&vertex_indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    model::ColoredMesh {
        name: "Colored Hexagon".to_string(),
        vertex_positions,
        vertex_indices,
        vertex_buffer,
        index_buffer,
        num_elements,
    }
}
