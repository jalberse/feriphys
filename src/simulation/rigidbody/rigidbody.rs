use std::time::Duration;

use cgmath::{InnerSpace, Matrix, Matrix3, One, Quaternion, SquareMatrix, Vector3, Zero};
use itertools::Itertools;

use crate::simulation::{
    consts,
    springy::collidable_mesh::{self, CollidableMesh},
    state::Stateful,
};

use super::config::Config;

#[derive(Clone, Copy)]
pub struct State {
    // The position of the center of mass of the RididBody, in worldspace
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    linear_momentum: Vector3<f32>,
    angular_momentum: Vector3<f32>,

    // These elements won't vary with time, but are necessary for calculating the rest of the state.
    mass: f32,
    initial_moment_of_intertia_inverted: Matrix3<f32>,
    accumulated_force: Vector3<f32>,
    accumulated_torque: Vector3<f32>,
}

impl State {
    pub fn normalize_rotation(&mut self) {
        self.rotation = self.rotation.normalize();
    }

    pub fn get_moment_of_inertia_inverted(&self) -> Matrix3<f32> {
        let rotation_matrix = Matrix3::<f32>::from(self.rotation);
        rotation_matrix * self.initial_moment_of_intertia_inverted * rotation_matrix.transpose()
    }

    pub fn apply_impulse(&mut self, impulse: Vector3<f32>, position: Vector3<f32>) {
        self.linear_momentum += impulse;
        self.angular_momentum += position.cross(impulse);
    }

    pub fn velocity(&self) -> Vector3<f32> {
        self.linear_momentum / self.mass
    }

    pub fn angular_velocity(&self) -> Vector3<f32> {
        self.get_moment_of_inertia_inverted() * self.angular_momentum
    }
}

impl Stateful for State {
    fn num_state_elements() -> usize {
        3 + // Position
        4 + // Rotation (vector scalar representation of a Quaternion)
        3 + // Linear Momentum
        3 + // Angular Momentum
        1 + // mass
        9 + // Moment of inertia
        3 + // accumulated force
        3 // accumulated torque
    }

    fn as_state(&self) -> Vec<f32> {
        let state_vec = vec![
            self.position.x,
            self.position.y,
            self.position.z,
            self.rotation.v.x,
            self.rotation.v.y,
            self.rotation.v.z,
            self.rotation.s,
            self.linear_momentum.x,
            self.linear_momentum.y,
            self.linear_momentum.z,
            self.angular_momentum.x,
            self.angular_momentum.y,
            self.angular_momentum.z,
            self.mass,
            self.initial_moment_of_intertia_inverted.x.x,
            self.initial_moment_of_intertia_inverted.x.y,
            self.initial_moment_of_intertia_inverted.x.z,
            self.initial_moment_of_intertia_inverted.y.x,
            self.initial_moment_of_intertia_inverted.y.y,
            self.initial_moment_of_intertia_inverted.y.z,
            self.initial_moment_of_intertia_inverted.z.x,
            self.initial_moment_of_intertia_inverted.z.y,
            self.initial_moment_of_intertia_inverted.z.z,
            self.accumulated_force.x,
            self.accumulated_force.y,
            self.accumulated_force.z,
            self.accumulated_torque.x,
            self.accumulated_torque.y,
            self.accumulated_torque.z,
        ];
        if state_vec.len() != Self::num_state_elements() {
            panic!("Incorrect size of state vector!");
        }
        state_vec
    }

    fn derivative(&self) -> Vec<f32> {
        let position_derivative = self.velocity();
        let rotation_derivative =
            0.5 * Quaternion::from_sv(0.0, self.angular_velocity()) * self.rotation;

        let derivative_state = vec![
            position_derivative.x,
            position_derivative.y,
            position_derivative.z,
            rotation_derivative.v.x,
            rotation_derivative.v.y,
            rotation_derivative.v.z,
            rotation_derivative.s,
            // Linear momentum derivative is force
            self.accumulated_force.x,
            self.accumulated_force.y,
            self.accumulated_force.z,
            // Angular momentum derivative is torque
            self.accumulated_torque.x,
            self.accumulated_torque.y,
            self.accumulated_torque.z,
            // The remaining elements of the state are constant
            // TODO I'd really like to remove the need for this, but the need to convert back to this Struct representation from
            //   the state vector (state::State::from_state_vector()), means we need to associate this constant data stored in the state alongside
            //   with the varying data, even though it's wasteful copies/ops.
            //   Less wasteful solutions might include either:
            //    1. In state::State, keep these constants associated with the state vectors as we reconstruct via some additional bookkeeping.
            //    2. Stop precomputing the forces/torques, and instead derive them from the state vector itself inside the derivation function.
            //       This is closer to how "Foundations of Physically Based Modeling and Animation" elects to represent State.
            //       However, this is a big rework, since it requires that the full simulation state is represented in a single State vector
            //       to allow for interaction between objects in the simulation. If that's not enough, state::State would need to allow for multiple
            //       kinds of Stateful objects, requiring dyn Traits, but that's not possible with Stateful::from_state_vector() because it would return
            //       Self, which isn't allowed since we can't know the size of Self at compile time. There are some potential workarounds, but for now we
            //       just accept this additional memory usage.
            //       TL;DR This approach needs some re-architecting.
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ];
        if derivative_state.len() != Self::num_state_elements() {
            panic!("Incorrect size of derivative state!");
        }
        derivative_state
    }

    fn from_state_vector(state_data: Vec<f32>) -> Self {
        if state_data.len() != Self::num_state_elements() {
            panic!("Incorrect size of state vector!");
        }
        let position = Vector3::<f32>::new(state_data[0], state_data[1], state_data[2]);
        let rotation_vec = Vector3::<f32>::new(state_data[3], state_data[4], state_data[5]);
        let rotation_scalar = state_data[6];
        let rotation = Quaternion::from_sv(rotation_scalar, rotation_vec);
        let linear_momentum = Vector3::<f32>::new(state_data[7], state_data[8], state_data[9]);
        let angular_momentum = Vector3::<f32>::new(state_data[10], state_data[11], state_data[12]);
        let mass = state_data[13];
        let moi_x = Vector3::<f32>::new(state_data[14], state_data[15], state_data[16]);
        let moi_y = Vector3::<f32>::new(state_data[17], state_data[18], state_data[19]);
        let moi_z = Vector3::<f32>::new(state_data[20], state_data[21], state_data[22]);
        let moi = Matrix3::from_cols(moi_x, moi_y, moi_z);
        let accumulated_force = Vector3::<f32>::new(state_data[23], state_data[24], state_data[25]);
        let accumulated_torque =
            Vector3::<f32>::new(state_data[26], state_data[27], state_data[28]);
        State {
            position,
            rotation,
            linear_momentum,
            angular_momentum,
            mass,
            initial_moment_of_intertia_inverted: moi,
            accumulated_force,
            accumulated_torque,
        }
    }
}

pub struct RigidBody {
    state: State,

    // The collidable mesh in local coordinates, where the center of mass (State.position) is at the origin.
    mesh: CollidableMesh,
}

impl RigidBody {
    // TODO we will add vector positions/indices in as params for this, and calculate the moment of intertia, center of mass etc from that.
    //      For now, we are working with only a 1x1x1 cube.
    pub fn new(position: Vector3<f32>, mass: f32) -> Result<RigidBody, &'static str> {
        let (cube_vertices, cube_indices) = crate::graphics::forms::get_cube_vertices();
        let mesh = CollidableMesh::new(cube_vertices, cube_indices);

        let moment_of_inertia = Matrix3::<f32>::new(
            mass / 6.0,
            0.0,
            0.0,
            0.0,
            mass / 6.0,
            0.0,
            0.0,
            0.0,
            mass / 6.0,
        );

        let initial_moment_of_intertia_inverted = moment_of_inertia
            .invert()
            .ok_or("Uninvertable moment of inertia!")?;

        let rotation = Quaternion::one();

        let linear_momentum = mass * Vector3::<f32>::zero();
        let angular_momentum = moment_of_inertia * Vector3::<f32>::zero();

        let state = State {
            position,
            rotation,
            linear_momentum,
            angular_momentum,
            mass,
            initial_moment_of_intertia_inverted,
            accumulated_force: Vector3::<f32>::zero(),
            accumulated_torque: Vector3::<f32>::zero(),
        };

        Ok(RigidBody { state, mesh })
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }

    pub fn update_state(
        &mut self,
        mut new_state: State,
        obstacles: &Vec<collidable_mesh::CollidableMesh>,
        config: &Config,
    ) {
        // The new state might need to be modified if there is a collision.
        //   For now, we are just going to pass in static obstacles, so we don't need to get obstacles from a rigidbody or whatever, that's good.
        //   We will need to use the new state's pos and rot to get new positions for verts to test etc.
        let obstacle_faces = obstacles
            .iter()
            .map(|o| o.get_faces())
            .flatten()
            .collect_vec();

        // Handle collisions between this rigidbody's vertices, and the world's faces.

        let vertices_old_world_positions = self
            .mesh
            .get_vertices()
            .to_owned()
            .iter()
            .map(|v| self.get_rotation_matrix() * v.position() + self.get_position())
            .collect_vec();
        let vertices_new_world_positions = self
            .mesh
            .get_vertices()
            .to_owned()
            .iter()
            .map(|v| Matrix3::<f32>::from(new_state.rotation) * v.position() + new_state.position)
            .collect_vec();
        for (new_point, old_point) in vertices_new_world_positions
            .iter()
            .zip(vertices_old_world_positions.iter())
        {
            if let Some(face) = CollidableMesh::get_collided_face_from_list(
                &obstacle_faces,
                *old_point,
                *new_point,
                Duration::from_secs_f32(config.dt),
            ) {
                let old_distance_to_plane = face.distance_from_plane(&old_point);
                let new_distance_to_plane = face.distance_from_plane(&new_point);
                let r = old_point - self.state.position;

                let fraction_timestep =
                    old_distance_to_plane / (old_distance_to_plane - new_distance_to_plane);
                let collision_velocity =
                    self.state.velocity() + self.state.angular_velocity().cross(r);
                let collision_point =
                    old_point + config.dt * fraction_timestep * collision_velocity;

                // The normal component of the velocity before the collision
                let normal_velocity = collision_velocity.dot(face.normal());

                let impulse_magnitude = (-(1.0 + config.coefficient_of_restitution)
                    * normal_velocity)
                    / (1.0 / self.state.mass
                        + face.normal().dot(
                            self.state.get_moment_of_inertia_inverted()
                                * r.cross(face.normal()).cross(r),
                        ));
                let impulse = impulse_magnitude * face.normal();

                new_state.position = collision_point - r + consts::EPSILON * face.normal();
                new_state.apply_impulse(impulse, r);
            }
        }

        // TODO this can further be improved by handling edge-edge collision, and
        //  by handling collisions between the world's vertices and this rigidbody's faces.

        self.state = new_state;
    }

    /// Accumulates the body forces on the rigidbody
    pub fn accumulate_forces(&mut self, config: &Config) {
        self.state.accumulated_force += config.gravity;
    }

    pub fn accumulate_torques(&mut self, config: &Config) {
        self.state.accumulated_torque += config.torque;
    }

    pub fn clear_forces(&mut self) {
        self.state.accumulated_force = Vector3::<f32>::zero();
    }

    pub fn clear_torques(&mut self) {
        self.state.accumulated_torque = Vector3::<f32>::zero();
    }

    pub fn get_rotation_matrix(&self) -> Matrix3<f32> {
        Matrix3::<f32>::from(self.state.rotation)
    }

    pub fn get_position(&self) -> &Vector3<f32> {
        &self.state.position
    }

    pub fn get_mesh(&self) -> &CollidableMesh {
        &self.mesh
    }

    /// Applies the impulse, updating the linear and angular momentum.
    /// The position describes the vector from the center of mass to the point that the impulse is applied.
    pub fn apply_impulse(&mut self, impulse: Vector3<f32>, position: Vector3<f32>) {
        self.state.apply_impulse(impulse, position);
    }
}
