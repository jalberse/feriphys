use std::collections::BTreeSet;

use cgmath::{InnerSpace, Vector3};
use itertools::Itertools;

pub struct Vertex {
    position: Vector3<f32>,
}

impl Vertex {
    pub fn new(position: Vector3<f32>) -> Vertex {
        Vertex { position }
    }
}

#[derive(Debug, PartialEq)]
pub struct Edge {
    v0: Vector3<f32>,
    v1: Vector3<f32>,
}

impl Edge {
    pub fn new(v0: Vector3<f32>, v1: Vector3<f32>) -> Edge {
        Edge { v0, v1 }
    }
}

#[derive(Debug, PartialEq)]
pub struct Face {
    v0: Vector3<f32>,
    v1: Vector3<f32>,
    v2: Vector3<f32>,
}

impl Face {
    pub fn normal(&self) -> Vector3<f32> {
        (self.v1 - self.v0).cross(self.v2 - self.v0).normalize()
    }

    pub fn distance(&self, point: cgmath::Vector3<f32>) -> f32 {
        (point - self.v1).dot(self.normal())
    }
}

pub struct Obstacle {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}

impl Obstacle {
    pub fn new(vertex_positions: Vec<Vector3<f32>>, vertex_indices: Vec<usize>) -> Obstacle {
        let vertices = vertex_positions
            .iter()
            .map(|v| Vertex { position: *v })
            .collect_vec();

        let mut edges_set = BTreeSet::default();
        for (v0, v1, v2) in vertex_indices.iter().tuples() {
            let mut edge0 = BTreeSet::new();
            edge0.insert(v0);
            edge0.insert(v1);

            let mut edge1 = BTreeSet::new();
            edge1.insert(v1);
            edge1.insert(v2);

            let mut edge2 = BTreeSet::new();
            edge2.insert(v2);
            edge2.insert(v0);

            edges_set.insert(edge0);
            edges_set.insert(edge1);
            edges_set.insert(edge2);
        }
        let edges = edges_set.iter().fold(Vec::new(), |mut array, x| {
            let verts_indices = x.iter().collect_vec();

            array.push(Edge {
                v0: vertex_positions[**verts_indices[0]],
                v1: vertex_positions[**verts_indices[1]],
            });
            array
        });

        let mut faces = Vec::with_capacity(vertex_indices.len() / 3);
        for (v0, v1, v2) in vertex_indices.iter().tuples() {
            faces.push(Face {
                v0: vertex_positions[*v0],
                v1: vertex_positions[*v1],
                v2: vertex_positions[*v2],
            });
        }

        Obstacle {
            vertices,
            edges,
            faces,
        }
    }

    // TODO This doesn't efficiently use indices, we repeat each vertex. We should properly use indexing,
    //  which will require more bookkeeping in Obstacle.
    /// Gets vertices to render
    pub fn get_vertices_to_render(&self) -> (Vec<Vector3<f32>>, Vec<usize>) {
        let vertex_positions = self.faces.iter().fold(Vec::new(), |mut array, f| {
            array.push(f.v0);
            array.push(f.v1);
            array.push(f.v2);
            array
        });
        let vertex_indices = 0..self.faces.len() * 3;
        (vertex_positions, vertex_indices.collect_vec())
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    pub fn get_faces(&self) -> &Vec<Face> {
        &self.faces
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Vector3, Zero};
    use itertools::Itertools;

    use super::Edge;
    use super::Face;
    use super::Obstacle;

    fn get_strip() -> Obstacle {
        let vertex_positions = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
        ];
        let vertex_indices = vec![0, 3, 2, 0, 1, 3];
        Obstacle::new(vertex_positions, vertex_indices)
    }

    #[test]
    fn ctor() {
        let obstacle = get_strip();
        assert_eq!(4, obstacle.vertices.len());
        assert_eq!(5, obstacle.edges.len());
        assert_eq!(2, obstacle.faces.len());

        let expected_points = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
        ];
        assert_eq!(
            expected_points,
            obstacle.vertices.iter().map(|v| v.position).collect_vec()
        );

        assert!(obstacle
            .edges
            .contains(&Edge::new(Vector3::<f32>::zero(), Vector3::<f32>::unit_y())));
        assert!(obstacle.edges.contains(&Edge::new(
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
        )));
        assert!(obstacle.edges.contains(&Edge::new(
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
        )));
        assert!(obstacle
            .edges
            .contains(&Edge::new(Vector3::<f32>::zero(), Vector3::<f32>::unit_x())));
        assert!(obstacle.edges.contains(&Edge::new(
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y()
        )));

        let expected_faces = vec![
            Face {
                v0: Vector3::<f32>::zero(),
                v1: Vector3::<f32>::unit_y(),
                v2: Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            },
            Face {
                v0: Vector3::<f32>::zero(),
                v1: Vector3::<f32>::unit_x(),
                v2: Vector3::<f32>::unit_y(),
            },
        ];
        assert_eq!(expected_faces, obstacle.faces);
    }
}
