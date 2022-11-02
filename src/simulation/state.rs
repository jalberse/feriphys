use crate::utils;
use itertools::{izip, Itertools};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Integration {
    Euler,
    Rk4,
}

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
pub struct State<T: Stateful> {
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

    /// Performs one step of runge kutta fourth order integration, returning the next state.
    pub fn rk4_step(&self, timestep: f32) -> State<T> {
        let k1 = self.derivative();
        let half_k1_delta = utils::scale(&k1, timestep * 0.5);
        let k2 = State::<T>::from_state_vector(utils::vec_add(&self.as_vector(), &half_k1_delta))
            .derivative();
        let half_k2_delta = utils::scale(&k2, timestep * 0.5);
        let k3 = State::<T>::from_state_vector(utils::vec_add(&self.as_vector(), &half_k2_delta))
            .derivative();
        let k3_delta = utils::scale(&k3, timestep);
        let k4 = State::<T>::from_state_vector(utils::vec_add(&self.as_vector(), &k3_delta))
            .derivative();
        let delta = izip!(k1, k2, k3, k4)
            .map(|(k1i, k2i, k3i, k4i)| {
                timestep / 6.0 * k1i
                    + timestep / 3.0 * k2i
                    + timestep / 3.0 * k3i
                    + timestep / 6.0 * k4i
            })
            .collect_vec();
        State::from_state_vector(utils::vec_add(&self.as_vector(), &delta))
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
            let position = Vector3::<f32>::new(state_data[0], state_data[1], state_data[2]);
            let velocity = Vector3::<f32>::new(state_data[3], state_data[4], state_data[5]);
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
            position: Vector3::<f32>::new(0.0, 0.0, 0.0),
            velocity: Vector3::<f32>::new(0.0, 0.0, 1.0),
        }];
        let state = super::State::new(points);
        let expected_initial_state = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(expected_initial_state, state.as_vector());

        let next_state = state.euler_step(h);
        let new_points = next_state.get_elements();
        let new_point = &new_points[0];

        let expected_position = Vector3::<f32>::new(0.0, 0.0, 0.5);
        let expected_velocity = Vector3::<f32>::new(0.5, -0.5, 1.0);
        assert_eq!(expected_position, new_point.position);
        assert_eq!(expected_velocity, new_point.velocity);
    }

    struct ExampleFn {
        y: f32,
        t: f32,
        timestep: f32,
    }

    impl Stateful for ExampleFn {
        fn num_state_elements() -> usize {
            3
        }

        fn from_state_vector(state_data: Vec<f32>) -> Self {
            if state_data.len() != Self::num_state_elements() {
                panic!("State Vector incorrect size!")
            }
            ExampleFn {
                y: state_data[0],
                t: state_data[1],
                timestep: state_data[2],
            }
        }

        fn derivative(&self) -> Vec<f32> {
            vec![self.y - f32::powi(self.t, 2) + 1.0, 1.0, 0.0]
        }

        fn as_state(&self) -> Vec<f32> {
            vec![self.y, self.t, self.timestep]
        }
    }

    #[test]
    fn rk4_step() {
        let h = 0.5;
        let ex = vec![ExampleFn {
            y: 0.5,
            t: 0.0,
            timestep: h,
        }];
        let state = super::State::new(ex);
        let expected_initial_state = vec![0.5, 0.0, 0.5];
        assert_eq!(expected_initial_state, state.as_vector());

        let acceptable_error = 0.005;

        // The exact solution is y = t^2 + 2t + 1 - .5e^t

        // Take the first step, t = 0.5
        let state = state.rk4_step(h);
        let new_state_vec = state.get_elements();
        let new_state_ex = &new_state_vec[0];
        assert!(
            1.425130208333333 + acceptable_error > new_state_ex.y
                && 1.425130208333333 - acceptable_error < new_state_ex.y
        );
        assert_eq!(0.5, new_state_ex.t);
        assert_eq!(0.5, new_state_ex.timestep);

        // Take the second step
        let state = super::State::new(new_state_vec);
        let state = state.rk4_step(h);
        let new_state_vec = state.get_elements();
        let new_state_ex = &new_state_vec[0];
        assert!(
            2.640859085770477 + acceptable_error > new_state_ex.y
                && 2.640859085770477 - acceptable_error < new_state_ex.y
        );
        assert_eq!(1.0, new_state_ex.t);
        assert_eq!(0.5, new_state_ex.timestep);

        // Third step
        let state = super::State::new(new_state_vec);
        let state = state.rk4_step(h);
        let new_state_vec = state.get_elements();
        let new_state_ex = &new_state_vec[0];
        assert!(
            4.009155464830968 + acceptable_error > new_state_ex.y
                && 4.009155464830968 - acceptable_error < new_state_ex.y
        );
        assert_eq!(1.5, new_state_ex.t);
        assert_eq!(0.5, new_state_ex.timestep);

        // Fourth step
        let state = super::State::new(new_state_vec);
        let state = state.rk4_step(h);
        let new_state_vec = state.get_elements();
        let new_state_ex = &new_state_vec[0];
        assert!(
            5.305471950534675 + acceptable_error > new_state_ex.y
                && 5.305471950534675 - acceptable_error < new_state_ex.y
        );
        assert_eq!(2.0, new_state_ex.t);
        assert_eq!(0.5, new_state_ex.timestep);
    }
}
