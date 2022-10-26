use itertools::Itertools;

pub trait Stateful {
    /// Number of f32 elements that are used to represent this object in the State vector.
    fn num_state_elements() -> usize;
    fn from_state_vector(state_data: Vec<f32>) -> Self;
    fn derivative(&self) -> Vec<f32>;
    fn as_state(&self) -> Vec<f32>;
}

// TODO We'd like for State to be able to contain some dyn Stateful type, instead of being over just one
//      Stateful type. However, from_state_vector() makes that very difficult. The current solution would be
//      to just have multiple State objects, one for each Stateful type.

/// For numerical integration, it's useful to easily get a State Vector S which represents a system's
/// state as a vector of floats, as well as to get S', its derivative. Then, the integration can be
/// expressed simply, e.g. in Euler integration as S_new = S + S' * h where h is the timestep.
/// It's then useful to be able to convert S_new back into a useful struct representation.
/// The State struct allows for these operations for any physical system composed of a Vec of a Stateful type.
///
/// A State should live one simulation step.
/// First, create Vec<T> and perform preliminary calculations.
/// Then create the State from the Vec<T> (giving State ownership of the Vec<T>)
/// Then integrate to find S_new using derivative(), as_vector(), and from_state_vector().
/// Finally, get the updated Vec<T> from S_new, destroying it.
/// Repeat the next timestep.
///
/// See "Foundations of Physically Based Modeling and Animation" by John C. Keyser
/// and Donald H. House, 6.2 "Expanding the Concept of State" (page 87).
struct State<T: Stateful> {
    elements: Vec<T>,
}

impl<T: Stateful> State<T> {
    pub fn new(elements: Vec<T>) -> State<T> {
        State { elements }
    }

    pub fn from_state_vector(mut state_vector: Vec<f32>) -> State<T> {
        let num_elements_in_state = state_vector.len() / T::num_state_elements();
        let mut data: Vec<T> = Vec::with_capacity(num_elements_in_state);
        for _ in 0..num_elements_in_state {
            let element_state_vector = state_vector.drain(0..T::num_state_elements()).collect_vec();
            let element = T::from_state_vector(element_state_vector);
            data.push(element);
        }
        State { elements: data }
    }

    pub fn derivative(&self) -> Vec<f32> {
        self.elements
            .iter()
            .map(|e| -> Vec<f32> { e.derivative() })
            .flatten()
            .collect_vec()
    }

    pub fn as_vector(&self) -> Vec<f32> {
        self.elements
            .iter()
            .map(|e| e.as_state())
            .flatten()
            .collect_vec()
    }

    /// Performs first-order Euler integration on the State, returning the
    /// next state.
    /// S_new = S + h * S'
    pub fn euler_step(&self, timestep: f32) -> State<T> {
        let state_delta = self
            .derivative()
            .into_iter()
            .map(|x| x * timestep)
            .collect_vec();
        let new_state_vector = crate::utils::vec_add(&self.as_vector(), &state_delta);
        State::from_state_vector(new_state_vector)
    }

    /// Drops self, returning the State as a Vec<T>.
    /// Intended to be called at the end of a simulation step, where a new State will be made the next simulation step.
    pub fn get_elements(self) -> Vec<T> {
        self.elements
    }
}

mod tests {
    use cgmath::Vector3;

    use super::Stateful;

    struct Point {
        position: Vector3<f32>,
        velocity: Vector3<f32>,
    }

    impl Stateful for Point {
        fn num_state_elements() -> usize {
            6
        }

        fn from_state_vector(state_data: Vec<f32>) -> Self {
            if state_data.len() != Self::num_state_elements() {
                panic!("State Vector incorrect size!")
            }
            let position = Vector3::<f32> {
                x: state_data[0],
                y: state_data[1],
                z: state_data[2],
            };
            let velocity = Vector3::<f32> {
                x: state_data[3],
                y: state_data[4],
                z: state_data[5],
            };
            Point { position, velocity }
        }

        fn derivative(&self) -> Vec<f32> {
            vec![
                // The derivative of the positoin is just the velocity
                self.velocity.x,
                self.velocity.y,
                self.velocity.z,
                // The accelerations.
                // In a not-test implementation, Point would have some accumulated forces, torques, etc
                // and calculate the acceleration from that data.
                1.0,
                -1.0,
                0.0,
            ]
        }

        fn as_state(&self) -> Vec<f32> {
            vec![
                self.position.x,
                self.position.y,
                self.position.z,
                self.velocity.x,
                self.velocity.y,
                self.velocity.z,
            ]
        }
    }

    #[test]
    fn euler_step() {
        let h = 0.5; // Timestep
        let points = vec![Point {
            position: Vector3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            velocity: Vector3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }];
        let state = super::State::new(points);
        let expected_initial_state = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(expected_initial_state, state.as_vector());

        let next_state = state.euler_step(h);
        let new_points = next_state.get_elements();
        let new_point = &new_points[0];

        let expected_position = Vector3::<f32> {
            x: 0.0,
            y: 0.0,
            z: 0.5,
        };
        let expected_velocity = Vector3::<f32> {
            x: 0.5,
            y: -0.5,
            z: 1.0,
        };
        assert_eq!(expected_position, new_point.position);
        assert_eq!(expected_velocity, new_point.velocity);
    }
}
