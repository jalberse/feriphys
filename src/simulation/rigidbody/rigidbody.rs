use cgmath::{InnerSpace, Matrix, Matrix3, One, Quaternion, SquareMatrix, Vector3, Zero};

use crate::simulation::{springy::collidable_mesh::CollidableMesh, state::Stateful};

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
        // Position derivative is the velocity, which we derive from the linear momentum
        let position_derivative = 1.0 / self.mass * self.linear_momentum;

        let rotation_matrix = Matrix3::<f32>::from(self.rotation);
        let current_moment_inertia_inverted = rotation_matrix
            * self.initial_moment_of_intertia_inverted
            * rotation_matrix.transpose();
        let angular_velocity = current_moment_inertia_inverted * self.angular_momentum;

        let rotation_derivative = 0.5 * Quaternion::from_sv(0.0, angular_velocity) * self.rotation;

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

    pub fn update_state(&mut self, new_state: State) {
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

    // TODO I think to test this, we should have a GUI container to specify the impulse and position,
    //         and when a button is pressed, we call this function in something similar to the sync_from_gui function.
    //         (but a separate one, since it's not just syncing the configs, it's a separate field of the UI)
    //      Thanks to immediate mode, that should just be checking .clicked() on that button, and grabbing the current values
    //         of fields in that block and then calling this function. ez peezy. Try wrapping it in some impulse container.
    /// Applies the impulse, updating the linear and angular momentum.
    /// The position describes the vector from the center of mass to the point that the impulse is applied.
    pub fn apply_impulse(&mut self, impulse: Vector3<f32>, position: Vector3<f32>) {
        self.state.linear_momentum += impulse;
        self.state.angular_momentum += position.cross(impulse);
    }
}
