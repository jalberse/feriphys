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
