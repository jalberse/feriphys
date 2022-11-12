use std::{f32::consts::PI, time::Duration};

use crate::simulation::{
    consts, position::Position, springy::collidable_mesh, state::Stateful, velocity::Velocity,
};

use super::{collidable_mesh::CollidableMesh, config::Config};
use cgmath::{InnerSpace, Rad, Vector3, Zero};
use itertools::Itertools;
use rustc_hash::FxHashMap;

// TODO adjust these (or if these are "okay", we can use them and make them adjustable in UI)
pub const STRUT_STIFFNESS_DEFAULT: f32 = 999999.0;
pub const STRUT_DAMPING_DEFAULT: f32 = 700.0;
// Generally, torsional spring and damping parameters should be one or two orders of magnitude higher
// than the corresponding strut parameters.
pub const TORSIONAL_SPRING_STIFFNESS_DEFAULT: f32 = 7000.0;
pub const TORSIONAL_SPRING_DAMPING_DEFAULT: f32 = 7000.0;

// Spring and damper constants are chosen based on the desired strength of
// a some spring of length NOMINAL_STRUT_LENGTH. Each strut's spring and
// damping constants are then scaled by NOMINAL_STRUT_LENGTH / strut_length.
// This way, shorter edges have stiffer springs than longer edges, but any
// given length along any given edge will have the same springyness.
const NOMINAL_STRUT_LENGTH: f32 = 1.0;

#[derive(Debug, PartialEq)]
struct TorsionalSpring {
    spring_constant: f32,
    damping: f32,
    rest_angle: Rad<f32>,
}

pub struct TorsionalSpringConfig {
    spring_constant: f32,
    spring_damping: f32,
}

#[derive(Copy, Clone)]
pub struct SpringConfig {
    pub constant: f32,
    pub damping: f32,
}

impl Default for TorsionalSpringConfig {
    fn default() -> Self {
        TorsionalSpringConfig {
            spring_constant: TORSIONAL_SPRING_STIFFNESS_DEFAULT,
            spring_damping: TORSIONAL_SPRING_DAMPING_DEFAULT,
        }
    }
}

/// A strut is a 3D structural element for a Springy Mesh,
/// made up of a spring and a damper connecting two point masses.
/// The Strut also contains a torsional spring between the two adjacent
/// faces, if both exist, to give the mesh rigidity.
struct Strut {
    /// Spring stiffness
    stiffness: f32,
    /// Dampiing strength
    damping: f32,
    /// The resting length of the spring
    length: f32,
    /// The indices of the strut's vertices in the SpringyMesh.
    vertex_indices: (usize, usize),
    /// The indices of the strut's adjacent faces, if present
    face_indices: (Option<usize>, Option<usize>),
    /// A torsional spring connects the two faces across this strut.
    /// None if this strut does not act as a hinge for two faces.
    torsional_spring: Option<TorsionalSpring>,
}

impl Strut {
    pub fn new(
        nominal_stiffness: f32,
        nominal_damping: f32,
        length: f32,
        vertex_indices: (usize, usize),
        face_indices: (Option<usize>, Option<usize>),
        torsional_spring: Option<TorsionalSpring>,
    ) -> Strut {
        Strut {
            stiffness: NOMINAL_STRUT_LENGTH / length * nominal_stiffness,
            damping: NOMINAL_STRUT_LENGTH / length * nominal_damping,
            length,
            vertex_indices,
            face_indices,
            torsional_spring,
        }
    }
}

// TODO It probably makes sense to replace this key's usage by just using a BTreeSet for
//   the strut de-duplication.
/// Key for a Map between pairs of vertex indices to Struts
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct StrutKey {
    key: (usize, usize),
}

impl StrutKey {
    pub fn new(k1: usize, k2: usize) -> StrutKey {
        StrutKey {
            key: if k1 <= k2 { (k1, k2) } else { (k2, k1) },
        }
    }
}

/// A face of a SpringyMesh
struct Face {
    /// The indices of the struts comprising this Face's edges in the SpringyMesh
    strut_indices: (usize, usize, usize),
    vertex_indices: (usize, usize, usize),
}

impl Face {
    fn normal(&self, points: &Vec<Point>) -> Vector3<f32> {
        let v0 = &points[self.vertex_indices.0].position;
        let v1 = &points[self.vertex_indices.1].position;
        let v2 = &points[self.vertex_indices.2].position;
        // TODO I think this is... not correct for calculating the surface normal?
        (v1 - v0).cross(v2 - v0).normalize()
    }

    fn area(&self, points: &Vec<Point>) -> f32 {
        let v0 = &points[self.vertex_indices.0].position;
        let v1 = &points[self.vertex_indices.1].position;
        let v2 = &points[self.vertex_indices.2].position;
        (v1 - v0).cross(v2 - v0).magnitude() / 2.0
    }

    /// Returns the vertex angle of vertex_indices.0
    fn vertex_angle_0(&self, springy_mesh: &SpringyMesh) -> Rad<f32> {
        self.vertex_angle(
            self.vertex_indices.0,
            self.vertex_indices.1,
            self.vertex_indices.2,
            springy_mesh,
        )
    }

    /// Returns the vertex angle of vertex_indices.1
    fn vertex_angle_1(&self, springy_mesh: &SpringyMesh) -> Rad<f32> {
        self.vertex_angle(
            self.vertex_indices.1,
            self.vertex_indices.2,
            self.vertex_indices.0,
            springy_mesh,
        )
    }

    /// Returns the vertex angle of vertex_indices.2
    fn vertex_angle_2(&self, springy_mesh: &SpringyMesh) -> Rad<f32> {
        self.vertex_angle(
            self.vertex_indices.2,
            self.vertex_indices.0,
            self.vertex_indices.1,
            springy_mesh,
        )
    }

    fn vertex_angle(
        &self,
        v0_index: usize,
        v1_index: usize,
        v2_index: usize,
        springy_mesh: &SpringyMesh,
    ) -> Rad<f32> {
        (springy_mesh.points[v2_index].position - springy_mesh.points[v0_index].position)
            .angle(springy_mesh.points[v1_index].position - springy_mesh.points[v0_index].position)
    }
}

/// A point in a SpringyMesh
#[derive(Clone, Copy)]
pub struct Point {
    mass: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    accumulated_force: Vector3<f32>,
}

impl Position for Point {
    fn position(&self) -> Vector3<f32> {
        self.position
    }
}

impl Velocity for Point {
    fn velocity(&self) -> Vector3<f32> {
        self.velocity
    }
}

impl Point {
    fn new(mass: f32, position: Vector3<f32>) -> Point {
        Point {
            mass,
            position,
            velocity: Vector3::<f32>::zero(),
            accumulated_force: Vector3::<f32>::zero(),
        }
    }

    fn add_external_forces(&mut self, config: &Config) {
        self.accumulated_force += config.gravity;
    }
}

impl Stateful for Point {
    fn num_state_elements() -> usize {
        10
    }

    fn from_state_vector(state_data: Vec<f32>) -> Self {
        if state_data.len() != Self::num_state_elements() {
            panic!("State Vector incorrect size!")
        }
        let mass = state_data[0];
        let position = Vector3::<f32>::new(state_data[1], state_data[2], state_data[3]);
        let velocity = Vector3::<f32>::new(state_data[4], state_data[5], state_data[6]);
        // TODO I don't know if we actually need this in the state. Probably not? Try to remove. Same with mass, really.
        //      Issue is getting a point *from* a StateVector. I suppose that we could have a PointState
        //      that Point contains, that contains the truly stateful stuff. For now, this is ok, just a little inefficient.
        let accumulated_force = Vector3::<f32>::new(state_data[7], state_data[8], state_data[9]);
        Point {
            mass,
            position,
            velocity,
            accumulated_force,
        }
    }

    fn derivative(&self) -> Vec<f32> {
        vec![
            // Mass does not change
            0.0,
            // Position derivative
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
            // Velocity derivative (acceleration). F = ma.
            self.accumulated_force.x / self.mass,
            self.accumulated_force.y / self.mass,
            self.accumulated_force.z / self.mass,
            // Accumulated force does not change in derivative.
            0.0,
            0.0,
            0.0,
        ]
    }

    fn as_state(&self) -> Vec<f32> {
        vec![
            self.mass,
            self.position.x,
            self.position.y,
            self.position.z,
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
            self.accumulated_force.x,
            self.accumulated_force.y,
            self.accumulated_force.z,
        ]
    }
}

/// A springy, deformable mesh.
pub struct SpringyMesh {
    struts: Vec<Strut>,
    faces: Vec<Face>,
    points: Vec<Point>,
    pinned_points: Vec<usize>,
}

impl SpringyMesh {
    pub fn new(
        vertex_positions: Vec<Vector3<f32>>,
        vertex_indices: Vec<usize>,
        // Total mass of the mesh
        mass: f32,
        default_stiffness: f32,
        default_damping: f32,
        torsional_spring_config: Option<TorsionalSpringConfig>,
        strut_overrides: &Option<FxHashMap<StrutKey, SpringConfig>>,
    ) -> Self {
        let mass_per_vert = mass / vertex_positions.len() as f32;

        let points = vertex_positions
            .iter()
            .map(|p| Point::new(mass_per_vert, *p))
            .collect_vec();

        let mut struts = Vec::new();
        let mut strut_indices: FxHashMap<StrutKey, usize> = FxHashMap::default();
        for (v0i, v1i, v2i) in vertex_indices.iter().tuples() {
            let mut add_strut = |i1: usize, i2: usize| -> () {
                let strut_key = StrutKey::new(i1, i2);
                if strut_indices.get(&strut_key).is_none() {
                    strut_indices.insert(strut_key, struts.len());
                    let (stiffness, damping) = if let Some(override_map) = strut_overrides {
                        if let Some(override_cfg) = override_map.get(&strut_key) {
                            (override_cfg.constant, override_cfg.damping)
                        } else {
                            (default_stiffness, default_damping)
                        }
                    } else {
                        (default_stiffness, default_damping)
                    };
                    struts.push(Strut::new(
                        stiffness,
                        damping,
                        (points[strut_key.key.0].position - points[strut_key.key.1].position)
                            .magnitude(),
                        (strut_key.key.0, strut_key.key.1),
                        (None, None),
                        None,
                    ));
                }
            };
            add_strut(*v0i, *v1i);
            add_strut(*v1i, *v2i);
            add_strut(*v2i, *v0i);
        }

        let mut faces = Vec::new();
        for (v0i, v1i, v2i) in vertex_indices.iter().tuples() {
            // We expect a strut to be made for this vertex pair in the previous loop
            let strut_key = StrutKey::new(*v0i, *v1i);
            let strut_index_0 = strut_indices.get(&strut_key).unwrap();
            let strut_key = StrutKey::new(*v1i, *v2i);
            let strut_index_1 = strut_indices.get(&strut_key).unwrap();
            let strut_key = StrutKey::new(*v2i, *v0i);
            let strut_index_2 = strut_indices.get(&strut_key).unwrap();

            let face_index = faces.len();
            faces.push(Face {
                strut_indices: (*strut_index_0, *strut_index_1, *strut_index_2),
                vertex_indices: (*v0i, *v1i, *v2i),
            });

            let mut update_strut_faces = |strut_index: usize| -> () {
                let strut = &mut struts[strut_index];
                if strut.face_indices.0.is_none() {
                    strut.face_indices.0 = Some(face_index);
                } else if strut.face_indices.1.is_none() {
                    strut.face_indices.1 = Some(face_index);
                } else {
                    panic!("A strut shouldn't have more than two adjacent faces!");
                }
            };
            update_strut_faces(*strut_index_0);
            update_strut_faces(*strut_index_1);
            update_strut_faces(*strut_index_2);
        }

        // Now that struts are aware of their adjacent faces, add a torsional spring if necessary.
        if let Some(torsional_spring_config) = torsional_spring_config {
            let mut strut_rest_angles = FxHashMap::<usize, Rad<f32>>::default();
            for (strut_index, strut) in struts.iter().enumerate() {
                if let (Some(f1_index), Some(f2_index)) =
                    (strut.face_indices.0, strut.face_indices.1)
                {
                    // TODO try to share this code and torsional force finding code
                    let x_0_index = strut.vertex_indices.0;
                    let x_0 = &points[x_0_index];
                    let x_1_index = strut.vertex_indices.1;
                    let x_1 = &points[x_1_index];
                    let h = (x_1.position - x_0.position).normalize();
                    let f1 = &faces[f1_index];
                    let f2 = &faces[f2_index];
                    // x_2 lies on f_1, or the "left", i.e. _l triangle
                    let x_2_index =
                        crate::utils::tuple_difference(f1.vertex_indices, strut.vertex_indices);
                    let x_2 = &points[x_2_index];
                    // x_3 lies on f_2, or the "right", i.e. _r triangle
                    let x_3_index =
                        crate::utils::tuple_difference(f2.vertex_indices, strut.vertex_indices);
                    let x_3 = &points[x_3_index];

                    let normal_l = (x_2.position - x_0.position)
                        .cross(x_1.position - x_0.position)
                        .normalize();
                    let normal_r = (x_1.position - x_0.position)
                        .cross(x_3.position - x_0.position)
                        .normalize();

                    let theta = Rad(f32::atan2(
                        normal_l.cross(normal_r).dot(h),
                        normal_l.dot(normal_r),
                    ));

                    strut_rest_angles.insert(strut_index, theta);
                }
            }
            for (strut_index, angle) in strut_rest_angles.iter() {
                struts[*strut_index].torsional_spring = Some(TorsionalSpring {
                    spring_constant: torsional_spring_config.spring_constant,
                    damping: torsional_spring_config.spring_damping,
                    rest_angle: *angle,
                });
            }
        }

        SpringyMesh {
            struts,
            faces,
            points,
            pinned_points: vec![],
        }
    }

    pub fn add_strut(&mut self, vertex_indices: (usize, usize), stiffness: f32, damping: f32) {
        self.struts.push(Strut::new(
            stiffness,
            damping,
            (self.points[vertex_indices.0].position - self.points[vertex_indices.1].position)
                .magnitude(),
            vertex_indices,
            (None, None),
            None,
        ));
    }

    pub fn add_pin(&mut self, pin_index: usize) {
        self.pinned_points.push(pin_index);
    }

    pub fn get_points(&self) -> &Vec<Point> {
        &self.points
    }

    pub fn update_points(
        &mut self,
        mut new_points: Vec<Point>,
        obstacles: &Vec<collidable_mesh::CollidableMesh>,
        config: &Config,
    ) {
        let obstacle_faces = obstacles
            .iter()
            .map(|o| o.get_faces())
            .flatten()
            .collect_vec();
        let obstacle_edges = obstacles
            .iter()
            .map(|o| o.get_edges())
            .flatten()
            .collect_vec();
        let obstacle_vertices = obstacles
            .iter()
            .map(|o| o.get_vertices())
            .flatten()
            .collect_vec();

        // TODO collision detection can be more efficient with bounding box checks.

        // Vertex-Face collisions
        for (new_point, old_point) in new_points.iter_mut().zip(&self.points) {
            let collided_face_maybe = CollidableMesh::get_collided_face_from_list(
                &obstacle_faces,
                old_point,
                new_point,
                Duration::from_secs_f32(config.dt),
            );
            if let Some(face) = collided_face_maybe {
                let old_distance_to_plane = face.distance_from_plane(&old_point.position);
                let new_distance_to_plane = face.distance_from_plane(&new_point.position);

                let fraction_timestep =
                    old_distance_to_plane / (old_distance_to_plane - new_distance_to_plane);

                let collision_point =
                    old_point.position + config.dt * fraction_timestep * old_point.velocity;
                let velocity_collision = old_point.velocity
                    + config.dt * fraction_timestep * old_point.accumulated_force / old_point.mass;

                let new_position = collision_point + face.normal() * consts::EPSILON;

                let velocity_collision_normal =
                    velocity_collision.dot(face.normal()) * face.normal();
                let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                let velocity_response_normal =
                    -1.0 * velocity_collision_normal * config.coefficient_of_restitution;
                let velocity_response_tangent = velocity_collision_tangent
                    - velocity_collision_tangent.normalize()
                        * f32::min(
                            config.coefficient_of_friction * velocity_collision_normal.magnitude(),
                            velocity_collision_tangent.magnitude(),
                        );

                let velocity_response = velocity_response_normal + velocity_response_tangent;

                new_point.position = new_position;
                new_point.velocity = velocity_response;
            }
        }

        // TODO then, handle face-vertex collisions (obstacles' vertices against the mesh's faces)
        // For each vertex in obstacles
        //   Check for each face of the springy mesh
        //   the old face, and the new face.
        //   From there, the logic is equivalent - it's just the plane is moving instead of the position, but the logic is the same, it's all relative.

        // TODO then do edge-edge collisions (mesh's edge against environment edge)

        let original_points = self.points.clone();
        self.points = new_points;
        for pin_index in self.pinned_points.iter_mut() {
            self.points[*pin_index] = original_points[*pin_index];
        }
    }

    /// Returns the vertices and their indices. Useful for making a mesh for rendering
    pub fn get_vertices(&self) -> (Vec<Vector3<f32>>, Vec<usize>) {
        let vertex_positions = self.points.iter().map(|p| p.position).collect_vec();
        let vertex_indices =
            self.faces
                .iter()
                .map(|f| f.vertex_indices)
                .fold(Vec::new(), |mut array, c| {
                    array.push(c.0);
                    array.push(c.1);
                    array.push(c.2);
                    array
                });
        (vertex_positions, vertex_indices)
    }

    pub fn accumulate_forces(&mut self, config: &Config) {
        self.apply_external_point_forces(config);
        self.apply_strut_forces();
        // TODO unfortunately, torsional forces are broken, causing the mesh to explode. Try to fix them.
        // self.apply_torsional_forces();
        self.apply_face_forces(config);

        for pin_index in self.pinned_points.iter() {
            self.points[*pin_index].accumulated_force = Vector3::<f32>::zero();
        }
    }

    fn apply_external_point_forces(&mut self, config: &Config) {
        self.points
            .iter_mut()
            .for_each(|p| p.add_external_forces(config));
    }

    fn apply_strut_forces(&mut self) {
        self.struts.iter().for_each(|strut| {
            let p0 = &self.points[strut.vertex_indices.0].position;
            let p1 = &self.points[strut.vertex_indices.1].position;
            let u = (p1 - p0).normalize();

            let spring_force_p0 = strut.stiffness * ((p1 - p0).magnitude() - strut.length) * u;
            self.points[strut.vertex_indices.0].accumulated_force += spring_force_p0;
            let spring_force_p1 = -1.0 * spring_force_p0;
            self.points[strut.vertex_indices.1].accumulated_force += spring_force_p1;

            let v0 = &self.points[strut.vertex_indices.0].velocity;
            let v1 = &self.points[strut.vertex_indices.1].velocity;
            let damping_force_p0 = strut.damping * ((v1 - v0).dot(u)) * u;
            self.points[strut.vertex_indices.0].accumulated_force += damping_force_p0;
            let damping_force_p1 = -1.0 * damping_force_p0;
            self.points[strut.vertex_indices.1].accumulated_force += damping_force_p1;
        });
    }

    fn apply_torsional_forces(&mut self) {
        let mut vertex_forces: FxHashMap<usize, Vector3<f32>> = FxHashMap::default();
        self.struts.iter().for_each(|strut| {
            // See "Foundations of Physically Based Modeling and Animation" section 8.3.2: Computation of Torque from a torsional spring.
            if let (Some(f1_index), Some(f2_index)) = (strut.face_indices.0, strut.face_indices.1) {
                let x_0_index = strut.vertex_indices.0;
                let x_0 = &self.points[strut.vertex_indices.0];
                let x_1_index = strut.vertex_indices.1;
                let x_1 = &self.points[strut.vertex_indices.1];
                let f1 = &self.faces[f1_index];
                let f2 = &self.faces[f2_index];
                // x_2 lies on f_1, or the "left", i.e. _l triangle
                let x_2_index =
                    crate::utils::tuple_difference(f1.vertex_indices, strut.vertex_indices);
                let x_2 = &self.points[x_2_index];
                // x_3 lies on f_2, or the "right", i.e. _r triangle
                let x_3_index =
                    crate::utils::tuple_difference(f2.vertex_indices, strut.vertex_indices);
                let x_3 = &self.points[x_3_index];
                let l_01 = (x_1.position - x_0.position).magnitude();
                let h = (x_1.position - x_0.position).normalize();

                let d_02 = (x_2.position - x_0.position).dot(h);
                let d_03 = (x_3.position - x_0.position).dot(h);

                let r_l = (x_2.position - x_0.position) - d_02 * h;
                let r_r = (x_3.position - x_0.position) - d_03 * h;

                let normal_l = (x_1.position - x_0.position)
                    .cross(x_2.position - x_0.position)
                    .normalize();
                let normal_r = (x_3.position - x_0.position)
                    .cross(x_1.position - x_0.position)
                    .normalize();

                let theta = Rad(f32::atan2(
                    normal_l.cross(normal_r).dot(h),
                    normal_l.dot(normal_r),
                ));
                let theta_l_derivative = x_2.velocity.dot(normal_l) / r_l.magnitude();
                let theta_r_derivative = x_3.velocity.dot(normal_r) / r_r.magnitude();

                // Since there are two adjacent faces, we expect there to be a torsional spring, so unwrap safely.
                let torsional_spring = strut.torsional_spring.as_ref().unwrap();
                let spring_torque =
                    torsional_spring.spring_constant * (theta - torsional_spring.rest_angle).0 * h;
                let spring_damping_torque =
                    -1.0 * torsional_spring.damping * (theta_l_derivative + theta_r_derivative) * h;
                let torque = spring_torque + spring_damping_torque;

                let force_3 = torque.dot(h) / r_r.magnitude() * normal_r;
                let force_2 = torque.dot(h) / r_l.magnitude() * normal_l;
                let force_1 = (d_02 * force_2 + d_03 * force_3) / l_01;
                let force_0 = -1.0 * (force_1 + force_2 + force_3);

                *vertex_forces
                    .entry(x_0_index)
                    .or_insert(cgmath::Vector3::zero()) += force_0;
                *vertex_forces
                    .entry(x_1_index)
                    .or_insert(cgmath::Vector3::zero()) += force_1;
                *vertex_forces
                    .entry(x_2_index)
                    .or_insert(cgmath::Vector3::zero()) += force_2;
                *vertex_forces
                    .entry(x_3_index)
                    .or_insert(cgmath::Vector3::zero()) += force_3;
            }
        });

        for (vertex_index, force) in vertex_forces.iter() {
            self.points[*vertex_index].accumulated_force += *force;
        }
    }

    fn apply_face_forces(&mut self, config: &Config) {
        for face in self.faces.iter() {
            let v0 = self.points[face.vertex_indices.0];
            let v1 = self.points[face.vertex_indices.1];
            let v2 = self.points[face.vertex_indices.2];
            let average_vertex_velocity = (v0.velocity + v1.velocity + v2.velocity) / 3.0;
            let relative_velocity = average_vertex_velocity - config.wind;
            let effective_area =
                face.area(&self.points) * face.normal(&self.points).dot(relative_velocity).abs();
            let drag_force = -1.0 * config.drag_coefficient * effective_area * relative_velocity;
            let lift_force = -1.0
                * config.lift_coefficient
                * effective_area
                * (relative_velocity
                    * face
                        .normal(&self.points)
                        .cross(relative_velocity)
                        .magnitude());
            let v0_force = face.vertex_angle_0(&self) / Rad(PI) * (drag_force + lift_force);
            let v1_force = face.vertex_angle_1(&self) / Rad(PI) * (drag_force + lift_force);
            let v2_force = face.vertex_angle_2(&self) / Rad(PI) * (drag_force + lift_force);
            self.points[face.vertex_indices.0].accumulated_force += v0_force;
            self.points[face.vertex_indices.1].accumulated_force += v1_force;
            self.points[face.vertex_indices.2].accumulated_force += v2_force;
        }
    }

    pub fn clear_forces(&mut self) {
        self.points
            .iter_mut()
            .for_each(|p| p.accumulated_force = Vector3::<f32>::zero());
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use cgmath::{assert_relative_eq, Rad, Vector3, Zero};

    use crate::simulation::springy::springy_mesh::NOMINAL_STRUT_LENGTH;

    use super::{SpringyMesh, TorsionalSpringConfig};

    fn get_triangle() -> super::SpringyMesh {
        let vertex_positions = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x() * 2.0,
            Vector3::<f32>::unit_y(),
        ];
        let vertex_indices = vec![0, 1, 2];
        let tort_cfg = TorsionalSpringConfig {
            spring_constant: 4.0,
            spring_damping: 5.0,
        };
        super::SpringyMesh::new(
            vertex_positions,
            vertex_indices,
            1.0,
            2.0,
            3.0,
            Some(tort_cfg),
            &None,
        )
    }

    // A SpringyMesh made up of a strip of triangles. The last triangle is bent 90 degrees.
    fn get_strip() -> super::SpringyMesh {
        let vertex_positions = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_x() + Vector3::<f32>::unit_z(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
            Vector3::<f32>::unit_y() + Vector3::<f32>::unit_x(),
        ];
        let vertex_indices = vec![0, 4, 3, 0, 1, 4, 1, 5, 4, 1, 2, 5];
        let tort_cfg = TorsionalSpringConfig {
            spring_constant: 4.0,
            spring_damping: 5.0,
        };
        super::SpringyMesh::new(
            vertex_positions,
            vertex_indices,
            1.0,
            2.0,
            3.0,
            Some(tort_cfg),
            &None,
        )
    }

    #[test]
    fn ctor_triangle() {
        let springy_mesh = get_triangle();
        assert_eq!(3, springy_mesh.points.len());
        assert_eq!(3, springy_mesh.struts.len());
        assert_eq!(1, springy_mesh.faces.len());

        assert_eq!((0, 1, 2), springy_mesh.faces[0].strut_indices);
        assert_eq!((0, 1, 2), springy_mesh.faces[0].vertex_indices);

        assert_eq!(1.0 / 3.0, springy_mesh.points[0].mass);
        assert_eq!(Vector3::<f32>::zero(), springy_mesh.points[0].position);
        assert_eq!(Vector3::<f32>::zero(), springy_mesh.points[0].velocity);
        assert_eq!(
            Vector3::<f32>::zero(),
            springy_mesh.points[0].accumulated_force
        );

        assert_eq!(
            Vector3::<f32>::unit_x() * 2.0,
            springy_mesh.points[1].position
        );
        assert_eq!(Vector3::<f32>::zero(), springy_mesh.points[1].velocity);

        assert_eq!(
            2.0 * (NOMINAL_STRUT_LENGTH / 2.0),
            springy_mesh.struts[0].stiffness
        );
        assert_eq!(
            3.0 * (NOMINAL_STRUT_LENGTH / 2.0),
            springy_mesh.struts[0].damping
        );
        assert_eq!(2.0, springy_mesh.struts[0].length);
        // Since there's only one face, we expect no torsional spring rest angle (they connect two
        // adjacent faces across the strut as a hinge).
        assert_eq!(None, springy_mesh.struts[0].torsional_spring);
    }

    #[test]
    fn face_angles() {
        let springy_mesh = get_triangle();
        let face = &springy_mesh.faces[0];
        assert_eq!(Rad(PI / 2.0), face.vertex_angle_0(&springy_mesh));
        assert_eq!(Rad(0.4636476), face.vertex_angle_1(&springy_mesh));
        assert_eq!(Rad(1.1071488), face.vertex_angle_2(&springy_mesh));
    }

    // Tests the construction of a strip of 4 triangles as a springy mesh.
    // See "Foundations of Physically Based Modeling and Animation" figure 8.2
    // (though we are using slightly different indices here due to how we choose
    //  to construct the mesh)
    #[test]
    fn strip_ctor() {
        let springy_mesh = get_strip();
        assert_eq!(6, springy_mesh.points.len());
        assert_eq!(9, springy_mesh.struts.len());
        assert_eq!(4, springy_mesh.faces.len());

        // Test the faces have correct points
        assert_eq!((0, 4, 3), springy_mesh.faces[0].vertex_indices);
        assert_eq!((0, 1, 4), springy_mesh.faces[1].vertex_indices);
        assert_eq!((1, 5, 4), springy_mesh.faces[2].vertex_indices);
        assert_eq!((1, 2, 5), springy_mesh.faces[3].vertex_indices);

        // Test the faces have the right struts, and that those struts refer to
        // the points of the face.
        // Note that by convention, the struts' vertex indices are always strictly ordered,
        // rather than being ordered ccw like face vertex indices. This is because
        // struts are unique, and can "belong" to two faces where they might not be
        // ccw for both faces.
        assert_eq!((0, 1, 2), springy_mesh.faces[0].strut_indices);
        assert_eq!((0, 4), springy_mesh.struts[0].vertex_indices);
        assert_eq!((3, 4), springy_mesh.struts[1].vertex_indices);
        assert_eq!((0, 3), springy_mesh.struts[2].vertex_indices);

        // Note the third strut index is 0. That's because that strut is shared with face 0.
        assert_eq!((3, 4, 0), springy_mesh.faces[1].strut_indices);
        assert_eq!((0, 1), springy_mesh.struts[3].vertex_indices);
        assert_eq!((1, 4), springy_mesh.struts[4].vertex_indices);
        assert_eq!((0, 4), springy_mesh.struts[0].vertex_indices);

        assert_eq!((5, 6, 4), springy_mesh.faces[2].strut_indices);
        assert_eq!((1, 5), springy_mesh.struts[5].vertex_indices);
        assert_eq!((4, 5), springy_mesh.struts[6].vertex_indices);
        assert_eq!((1, 4), springy_mesh.struts[4].vertex_indices);

        assert_eq!((7, 8, 5), springy_mesh.faces[3].strut_indices);
        assert_eq!((1, 2), springy_mesh.struts[7].vertex_indices);
        assert_eq!((2, 5), springy_mesh.struts[8].vertex_indices);
        assert_eq!((1, 5), springy_mesh.struts[5].vertex_indices);

        // Check struts know what faces they border
        assert_eq!((Some(0), Some(1)), springy_mesh.struts[0].face_indices);
        assert_eq!((Some(0), None), springy_mesh.struts[1].face_indices);
        assert_eq!((Some(0), None), springy_mesh.struts[2].face_indices);
        assert_eq!((Some(1), None), springy_mesh.struts[3].face_indices);
        assert_eq!((Some(1), Some(2)), springy_mesh.struts[4].face_indices);
        assert_eq!((Some(2), Some(3)), springy_mesh.struts[5].face_indices);
        assert_eq!((Some(2), None), springy_mesh.struts[6].face_indices);
        assert_eq!((Some(3), None), springy_mesh.struts[7].face_indices);
        assert_eq!((Some(3), None), springy_mesh.struts[8].face_indices);

        // Only struts that are a hinge between faces should have torsional springs
        assert!(springy_mesh.struts[0].torsional_spring.is_some());
        assert!(springy_mesh.struts[1].torsional_spring.is_none());
        assert!(springy_mesh.struts[2].torsional_spring.is_none());
        assert!(springy_mesh.struts[3].torsional_spring.is_none());
        assert!(springy_mesh.struts[4].torsional_spring.is_some());
        assert!(springy_mesh.struts[5].torsional_spring.is_some());
        assert!(springy_mesh.struts[6].torsional_spring.is_none());
        assert!(springy_mesh.struts[7].torsional_spring.is_none());
        assert!(springy_mesh.struts[8].torsional_spring.is_none());

        let torsional_spring = springy_mesh.struts[0].torsional_spring.as_ref().unwrap();
        assert_eq!(4.0, torsional_spring.spring_constant);
        assert_eq!(5.0, torsional_spring.damping);
        assert_relative_eq!(Rad(0.0), torsional_spring.rest_angle);

        let torsional_spring = springy_mesh.struts[4].torsional_spring.as_ref().unwrap();
        assert_eq!(4.0, torsional_spring.spring_constant);
        assert_eq!(5.0, torsional_spring.damping);
        assert_relative_eq!(Rad(0.0), torsional_spring.rest_angle);

        let torsional_spring = springy_mesh.struts[5].torsional_spring.as_ref().unwrap();
        assert_eq!(4.0, torsional_spring.spring_constant);
        assert_eq!(5.0, torsional_spring.damping);
        assert_relative_eq!(-Rad(PI / 2.0), torsional_spring.rest_angle);

        // Check face normals
        assert_eq!(
            Vector3::<f32>::unit_z(),
            springy_mesh.faces[0].normal(&springy_mesh.points)
        );
        assert_eq!(
            Vector3::<f32>::unit_z(),
            springy_mesh.faces[1].normal(&springy_mesh.points)
        );
        assert_eq!(
            Vector3::<f32>::unit_z(),
            springy_mesh.faces[2].normal(&springy_mesh.points)
        );
        assert_eq!(
            Vector3::<f32>::unit_x() * -1.0,
            springy_mesh.faces[3].normal(&springy_mesh.points)
        );
    }

    #[test]
    fn get_vertices() {
        let springy_mesh = get_strip();
        let (vertex_positions, vertex_inidices) = springy_mesh.get_vertices();
        let expected_vertex_positions = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_x() + Vector3::<f32>::unit_z(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
            Vector3::<f32>::unit_y() + Vector3::<f32>::unit_x(),
        ];
        assert_eq!(expected_vertex_positions, vertex_positions);
        let expected_vertex_indices = vec![0, 4, 3, 0, 1, 4, 1, 5, 4, 1, 2, 5];
        assert_eq!(expected_vertex_indices, vertex_inidices);
    }

    // Tests torsional force for when angle of faces are acute
    // (the angles between the face normals will be obstuse, however)
    #[test]
    fn torsional_force_acute_45() {
        let vertices = vec![
            Vector3::<f32>::zero(),    // Hinge vertex 0
            -Vector3::<f32>::unit_z(), // Hinge vertex 1
            Vector3::<f32>::unit_x(),  // Face 1 other point
            Vector3::<f32>::unit_x() * f32::sqrt(2.0) / 2.0
                + Vector3::<f32>::unit_y() * f32::sqrt(2.0) / 2.0, // Other face, at a 45 degree angle
        ];
        let indices = vec![1, 2, 0, 3, 1, 0];
        let tort_cfg = TorsionalSpringConfig {
            spring_constant: 1.0,
            spring_damping: 1.0,
        };
        let mut mesh = SpringyMesh::new(vertices, indices, 2.0, 1.0, 1.0, Some(tort_cfg), &None);

        assert_relative_eq!(
            -Vector3::<f32>::unit_y(),
            mesh.faces[0].normal(&mesh.points)
        );
        assert_relative_eq!(
            Vector3::<f32>::unit_y() * f32::sqrt(2.0) / 2.0
                - Vector3::<f32>::unit_x() * f32::sqrt(2.0) / 2.0,
            mesh.faces[1].normal(&mesh.points)
        );

        assert!(mesh.struts[2].torsional_spring.is_some());
        assert_eq!(
            Rad(2.3561945), // 135 degrees
            mesh.struts[2].torsional_spring.as_ref().unwrap().rest_angle
        );
        mesh.apply_torsional_forces();
        // The faces are at the initial resting angle, so no torsional force should be applied.
        assert_eq!(Vector3::<f32>::zero(), mesh.points[0].accumulated_force);
        assert_eq!(Vector3::<f32>::zero(), mesh.points[1].accumulated_force);
        assert_eq!(Vector3::<f32>::zero(), mesh.points[2].accumulated_force);
        assert_eq!(Vector3::<f32>::zero(), mesh.points[3].accumulated_force);
        mesh.clear_forces();

        mesh.struts[2].torsional_spring.as_mut().unwrap().rest_angle = Rad(PI / 2.0);
        mesh.apply_torsional_forces();
        assert_relative_eq!(
            Vector3::<f32>::zero(),
            mesh.points[0].accumulated_force
                + mesh.points[1].accumulated_force
                + mesh.points[2].accumulated_force
                + mesh.points[3].accumulated_force
        );
        mesh.clear_forces();
    }

    #[test]
    fn disable_torsional_springs() {
        let vertex_positions = vec![
            Vector3::<f32>::zero(),
            Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_x() + Vector3::<f32>::unit_z(),
            Vector3::<f32>::unit_y() - Vector3::<f32>::unit_x(),
            Vector3::<f32>::unit_y(),
            Vector3::<f32>::unit_y() + Vector3::<f32>::unit_x(),
        ];
        let vertex_indices = vec![0, 4, 3, 0, 1, 4, 1, 5, 4, 1, 2, 5];
        let strip =
            super::SpringyMesh::new(vertex_positions, vertex_indices, 1.0, 2.0, 3.0, None, &None);
        for i in 0..9 {
            assert!(strip.struts[i].torsional_spring.is_none());
        }
    }

    // TODO Torsional forces unit test with on obtuse angle between the faces

    // TODO possibly a unit test for torsional forces where the faces are co-planar?

    // TODO these torsional force unit tests aren't accounting for when the velocities are non-zero, however. Could that be wrong?
}
