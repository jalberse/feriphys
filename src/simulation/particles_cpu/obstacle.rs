use crate::graphics::model::ColoredMesh;

use cgmath::{InnerSpace, Vector3};
use itertools::Itertools;

pub struct Tri {
    v1: Vector3<f32>,
    v2: Vector3<f32>,
    v3: Vector3<f32>,
}

impl Tri {
    pub fn normal(&self) -> Vector3<f32> {
        (self.v2 - self.v1).cross(self.v3 - self.v1).normalize()
    }

    pub fn distance_from_plane(&self, point: cgmath::Vector3<f32>) -> f32 {
        (point - self.v1).dot(self.normal())
    }
}

pub struct Obstacle {
    tris: Vec<Tri>,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl Obstacle {
    pub fn new(mesh: &ColoredMesh) -> Obstacle {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        for vertex_index in mesh.vertex_indices.iter() {
            let v = mesh.vertex_positions[*vertex_index as usize];
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
            min_z = min_z.min(v.z);
            max_z = max_z.max(v.z);
        }

        let mut tris = vec![];
        for (i1, i2, i3) in mesh.vertex_indices.iter().tuple_windows() {
            let v1 = mesh.vertex_positions[*i1 as usize];
            let v2 = mesh.vertex_positions[*i2 as usize];
            let v3 = mesh.vertex_positions[*i3 as usize];
            tris.push(Tri { v1, v2, v3 });
        }
        Obstacle {
            tris,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    /// True if the position is in the bounds of the box.
    /// Useful for quick preliminary checks.
    /// Should call with the NEW position, not the old position.
    pub fn in_bounds(&self, position: &Vector3<f32>) -> bool {
        position.x >= self.min_x
            && position.x <= self.max_x
            && position.y >= self.min_y
            && position.y <= self.max_y
            && position.z >= self.min_z
            && position.z <= self.max_z
    }

    /// Returns None if the particle did not collide with the tri.
    /// Otherwise, returns the first polygon it finds that it did collide with.
    pub fn get_collided_tri(
        &self,
        old_position: Vector3<f32>,
        old_velocity: Vector3<f32>,
        new_position: Vector3<f32>,
        dt: f32,
    ) -> Option<&Tri> {
        self.tris.iter().find(|tri| -> bool {
            let old_distance_to_plane = tri.distance_from_plane(old_position);
            let new_distance_to_plane = tri.distance_from_plane(new_position);
            // If the signs are different, the point has crossed the plane
            let crossed_plane = old_distance_to_plane.is_sign_positive()
                != new_distance_to_plane.is_sign_positive();
            if !crossed_plane {
                false
            } else {
                // Get the point in the plane of the tri
                let fraction_timestep =
                    old_distance_to_plane / old_distance_to_plane - new_distance_to_plane;

                let collision_point = old_position + dt * fraction_timestep * old_velocity;

                // Flatten the tri and the point into 2D to check containment.
                let (v1_flat, v2_flat, v3_flat, point_flat) = if tri.normal().x >= tri.normal().y
                    && tri.normal().x >= tri.normal().z
                {
                    // Eliminate the x component of all the elements
                    let v1_flat = Vector3::<f32>::new(0.0, tri.v1.y, tri.v1.z);
                    let v2_flat = Vector3::<f32>::new(0.0, tri.v2.y, tri.v2.z);
                    let v3_flat = Vector3::<f32>::new(0.0, tri.v3.y, tri.v3.z);
                    let point_flat = Vector3::<f32>::new(0.0, collision_point.y, collision_point.z);
                    (v1_flat, v2_flat, v3_flat, point_flat)
                } else if tri.normal().y >= tri.normal().x && tri.normal().y >= tri.normal().z {
                    // Eliminate the y component of all the elements
                    let v1_flat = Vector3::<f32>::new(tri.v1.x, 0.0, tri.v1.z);
                    let v2_flat = Vector3::<f32>::new(tri.v2.x, 0.0, tri.v2.z);
                    let v3_flat = Vector3::<f32>::new(tri.v3.x, 0.0, tri.v3.z);
                    let point_flat = Vector3::<f32>::new(collision_point.x, 0.0, collision_point.z);
                    (v1_flat, v2_flat, v3_flat, point_flat)
                } else {
                    // Eliminate the z component of all the elements
                    let v1_flat = Vector3::<f32>::new(tri.v1.x, tri.v1.y, 0.0);
                    let v2_flat = Vector3::<f32>::new(tri.v2.x, tri.v2.y, 0.0);
                    let v3_flat = Vector3::<f32>::new(tri.v3.x, tri.v3.y, 0.0);
                    let point_flat = Vector3::<f32>::new(collision_point.x, collision_point.y, 0.0);
                    (v1_flat, v2_flat, v3_flat, point_flat)
                };

                // Then check the point by comparing the orientation of the cross products
                let cross1 = (v2_flat - v1_flat).cross(point_flat - v1_flat);
                let cross2 = (v3_flat - v2_flat).cross(point_flat - v2_flat);
                let cross3 = (v1_flat - v3_flat).cross(point_flat - v3_flat);

                let cross1_orientation = cross1.dot(tri.normal()).is_sign_positive();
                let cross2_orientation = cross2.dot(tri.normal()).is_sign_positive();
                let cross3_orientation = cross3.dot(tri.normal()).is_sign_positive();

                // The point is in the polygon iff the orientation for all three cross products are equal.
                cross1_orientation == cross2_orientation && cross2_orientation == cross3_orientation
            }
        })
    }
}
