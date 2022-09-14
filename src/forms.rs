/// The forms module provides basic forms (planes, spheres, cubes...) for rendering.
use super::model;
use cgmath::prelude::*;
use itertools::Itertools;
use wgpu::util::DeviceExt;

fn get_normals(
    vertex_positions: &Vec<cgmath::Vector3<f32>>,
    indices: &Vec<u16>,
) -> Vec<cgmath::Vector3<f32>> {
    // Calculate the normals of each vertex by averaging the normals of all adjacent faces.
    let mut normals = Vec::new();
    for _ in 0..vertex_positions.len() {
        normals.push(cgmath::Vector3::new(0.0, 0.0, 0.0));
    }
    for (a, b, c) in indices.iter().tuples() {
        let edge1 = vertex_positions[*a as usize] - vertex_positions[*b as usize];
        let edge2 = vertex_positions[*a as usize] - vertex_positions[*c as usize];
        let face_normal = edge1.cross(edge2);
        // Add this face's normal to each vertex's normal.
        normals[*a as usize] += face_normal;
        normals[*b as usize] += face_normal;
        normals[*c as usize] += face_normal;
    }
    normals
        .iter()
        .map(|n| n.normalize())
        .collect::<Vec<cgmath::Vector3<f32>>>()
}

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
    let vertex_positions: Vec<cgmath::Vector3<f32>> = indices
        .iter()
        .map(|i| -> cgmath::Vector3<f32> { vertex_positions[*i as usize] })
        .collect();
    let indices = Vec::from_iter(0..vertex_positions.len() as u16);

    let num_indices = indices.len() as u32;
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

    model::ColoredMesh {
        name: "Colored Mesh".to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    }
}

/// Generates a sphere mesh with the specified color, radius, and number of sectors and stacks.
/// The vertices have their normals averaged across adjacent faces.
#[allow(dead_code)]
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
    let mut indices: Vec<u16> = Vec::new();

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for _j in 0..sectors {
            // First and last stacks do not need quads, just tris.
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }
            if i != (stacks - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }
            k1 = k1 + 1;
            k2 = k2 + 1;
        }
    }

    let num_indices = indices.len() as u32;
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

    model::ColoredMesh {
        name: "Colored sphere Mesh".to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
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

    // Cubes with averaged vertex normals look bad withoutholding edges. So we'll use non-averaged
    // vertexes. That means generating the duplicate ones, and using 0..n as indices.
    let vertex_positions: Vec<cgmath::Vector3<f32>> = indices
        .iter()
        .map(|i| -> cgmath::Vector3<f32> { vertex_positions[*i as usize] })
        .collect();
    let indices = Vec::from_iter(0..vertex_positions.len() as u16);

    let num_indices = indices.len() as u32;
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

    model::ColoredMesh {
        name: "Colored Mesh".to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
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

    let indices: Vec<u16> = vec![0, 1, 4, 1, 2, 4, 2, 3, 4];
    let num_indices = indices.len() as u32;
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

    model::ColoredMesh {
        name: "Colored Mesh".to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: num_indices,
    }
}
