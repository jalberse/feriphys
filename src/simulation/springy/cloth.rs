/// Cloth is simulated as a spring-mass-damper mesh.
// TODO we should have a boolean to say, dont' make any torsional springs for struts for this mesh
use super::springy_mesh::{SpringConfig, SpringyMesh, StrutKey};

use cgmath::Vector3;
use itertools::Itertools;
use rustc_hash::FxHashMap;

pub struct Cloth {
    pub mesh: SpringyMesh,
}

impl Cloth {
    // TODO this can further be improved by allowing tensile and bending spring strength by weft and warp.
    // TODO this can be improved by passing in a point and a normal, and placing the mesh at that point, oriented towards that normal.
    // TODO We can further add a from_mesh() function to start the simulation from an arbitrary mesh.
    /// Construct a new cloth.
    /// rows and cols refer to the number of vertices, not quads/tris.
    /// Spring constants are for a strut of NOMINAL_STRUT_LENGTH.
    pub fn new(
        rows: usize,
        cols: usize,
        spacing: f32,
        position: Vector3<f32>,
        point_mass: f32,
        tensile_stiffness: f32,
        tensile_damping: f32,
        shear_stiffness: f32,
        shear_damping: f32,
        binding_spring_stiffness: f32,
        binding_spring_damping: f32,
        pinned_vertices: Vec<usize>,
    ) -> Cloth {
        let mut vertex_positions = Vec::new();
        for row in 0..rows {
            for col in 0..cols {
                vertex_positions.push(Vector3::<f32>::new(
                    col as f32 * spacing,
                    row as f32 * spacing,
                    0.0,
                ));
            }
        }
        // Transform the cloth so the center is at the origin, and then transform to the specified position
        let vertex_positions = vertex_positions
            .iter()
            .map(|v| {
                v - Vector3::<f32> {
                    x: cols as f32 / 2.0 * spacing,
                    y: rows as f32 / 2.0 * spacing,
                    z: 0.0,
                }
            })
            .map(|v| v + position)
            .collect_vec();

        let mut shear_overrides = FxHashMap::default();
        let shear_cfg = SpringConfig {
            constant: shear_stiffness,
            damping: shear_damping,
        };

        // Generate the top left tri of each "quad" formed by the grid.
        let mut indices = Vec::new();
        for row in 0..(rows - 1) {
            for col in 0..(cols - 1) {
                let i = (row * cols) + col;
                indices.push(i + 1);
                indices.push(i + cols);
                indices.push(i);

                shear_overrides.insert(StrutKey::new(i + cols, i + 1), shear_cfg.clone());
            }
        }
        // Generate the bottom right tri of each quad formed by the grid.
        for row in 1..(rows) {
            for col in 0..(cols - 1) {
                let i = (row * cols) + col;
                indices.push(i - cols + 1);
                indices.push(i + 1);
                indices.push(i);
            }
        }
        let indices = indices;

        let mut mesh = SpringyMesh::new(
            vertex_positions,
            indices,
            point_mass * rows as f32 * cols as f32,
            tensile_stiffness,
            tensile_damping,
            None,
            &Some(shear_overrides),
        );
        for pin_index in pinned_vertices.iter() {
            mesh.add_pin(*pin_index)
        }

        // Binding springs resist bending of cloth as a whole.
        let mut binding_spring_index_pairs = Vec::new();
        for row in 0..(rows - 2) {
            for col in 0..(cols - 2) {
                let i = (row * cols) + col;
                binding_spring_index_pairs.push((i, i + 2));
                binding_spring_index_pairs.push((i, i + 2 * cols));
            }
        }
        for pair in binding_spring_index_pairs.iter() {
            mesh.add_strut(*pair, binding_spring_stiffness, binding_spring_damping);
        }

        Cloth { mesh }
    }
}
