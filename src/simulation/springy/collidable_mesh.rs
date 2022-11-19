use std::{collections::BTreeSet, time::Duration};

use cgmath::{InnerSpace, Vector3};
use itertools::Itertools;
pub struct Vertex {
    position: Vector3<f32>,
}

impl Vertex {
    pub fn position(&self) -> Vector3<f32> {
        self.position
    }
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
    pub v0: Vector3<f32>,
    pub v1: Vector3<f32>,
    pub v2: Vector3<f32>,
}

impl Face {
    pub fn normal(&self) -> Vector3<f32> {
        (self.v1 - self.v0).cross(self.v2 - self.v0).normalize()
    }

    pub fn distance_from_plane(&self, point: &cgmath::Vector3<f32>) -> f32 {
        (point - self.v0).dot(self.normal())
    }
}

pub struct CollidableMesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
}

impl CollidableMesh {
    pub fn new(vertex_positions: Vec<Vector3<f32>>, vertex_indices: Vec<usize>) -> CollidableMesh {
        let vertices = vertex_positions
            .iter()
            .map(|v| Vertex::new(*v))
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

            array.push(Edge::new(
                vertex_positions[**verts_indices[0]],
                vertex_positions[**verts_indices[1]],
            ));
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

        CollidableMesh {
            vertices,
            edges,
            faces,
        }
    }

    pub fn get_collided_face_from_list<'a>(
        faces: &'a Vec<&Face>,
        old_position: Vector3<f32>,
        new_position: Vector3<f32>,
        dt: Duration,
    ) -> Option<&'a Face> {
        let result = faces.iter().find(|face| -> bool {
            let old_velocity = (new_position - old_position) / dt.as_secs_f32();

            let old_distance_to_plane = face.distance_from_plane(&old_position);
            let new_distance_to_plane = face.distance_from_plane(&new_position);

            let crossed_plane = old_distance_to_plane.is_sign_positive()
                != new_distance_to_plane.is_sign_positive();

            if !crossed_plane {
                return false;
            }
            // Get the point in the plane of the tri
            let fraction_timestep =
                old_distance_to_plane / (old_distance_to_plane - new_distance_to_plane);
            let collision_point =
                old_position + dt.as_secs_f32() * fraction_timestep * old_velocity;
            let face_normal = face.normal();
            // Flatten the tri and the point into 2D to check containment.
            let (v0_flat, v1_flat, v2_flat, point_flat) =
                if face_normal.x >= face_normal.y && face_normal.x >= face_normal.z {
                    // Eliminate the x component of all the elements
                    let v0_flat = Vector3::<f32>::new(0.0, face.v0.y, face.v0.z);
                    let v1_flat = Vector3::<f32>::new(0.0, face.v1.y, face.v1.z);
                    let v2_flat = Vector3::<f32>::new(0.0, face.v2.y, face.v2.z);
                    let point_flat = Vector3::<f32>::new(0.0, collision_point.y, collision_point.z);
                    (v0_flat, v1_flat, v2_flat, point_flat)
                } else if face_normal.y >= face_normal.x && face_normal.y >= face_normal.z {
                    // Eliminate the y component of all the elements
                    let v0_flat = Vector3::<f32>::new(face.v0.x, 0.0, face.v0.z);
                    let v1_flat = Vector3::<f32>::new(face.v1.x, 0.0, face.v1.z);
                    let v2_flat = Vector3::<f32>::new(face.v2.x, 0.0, face.v2.z);
                    let point_flat = Vector3::<f32>::new(collision_point.x, 0.0, collision_point.z);
                    (v0_flat, v1_flat, v2_flat, point_flat)
                } else {
                    // Eliminate the z component of all the elements
                    let v0_flat = Vector3::<f32>::new(face.v0.x, face.v0.y, 0.0);
                    let v1_flat = Vector3::<f32>::new(face.v1.x, face.v1.y, 0.0);
                    let v2_flat = Vector3::<f32>::new(face.v2.x, face.v2.y, 0.0);
                    let point_flat = Vector3::<f32>::new(collision_point.x, collision_point.y, 0.0);
                    (v0_flat, v1_flat, v2_flat, point_flat)
                };
            // Then check the point by comparing the orientation of the cross products
            let cross0 = (v1_flat - v0_flat).cross(point_flat - v0_flat);
            let cross1 = (v2_flat - v1_flat).cross(point_flat - v1_flat);
            let cross2 = (v0_flat - v2_flat).cross(point_flat - v2_flat);
            let cross0_orientation = cross0.dot(face.normal()).is_sign_positive();
            let cross1_orientation = cross1.dot(face.normal()).is_sign_positive();
            let cross2_orientation = cross2.dot(face.normal()).is_sign_positive();
            // The point is in the polygon iff the orientation for all three cross products are equal.
            cross0_orientation == cross1_orientation && cross1_orientation == cross2_orientation
        });
        match result {
            Some(face) => Some(*face),
            None => None,
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

    #[allow(dead_code)]
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

    use super::CollidableMesh;
    use super::Edge;
    use super::Face;

    fn get_strip() -> CollidableMesh {
        let vertex_positions = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
        ];
        let vertex_indices = vec![0, 3, 2, 0, 1, 3];
        CollidableMesh::new(vertex_positions, vertex_indices)
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
