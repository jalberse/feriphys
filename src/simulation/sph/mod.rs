use std::time::Duration;

use cgmath::{Vector3, Zero};

use self::config::Config;

use super::state::{Integration, State, Stateful};

pub mod config;

#[derive(Clone, Copy)]
pub struct Particle {
    mass: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,

    accumulated_force: Vector3<f32>,
}

impl Particle {
    pub fn new(mass: f32, position: Vector3<f32>, velocity: Vector3<f32>) -> Particle {
        Particle {
            mass,
            position,
            velocity,
            accumulated_force: Vector3::<f32>::zero(),
        }
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }
}

impl Stateful for Particle {
    fn num_state_elements() -> usize {
        1 + // mass
        3 + // position
        3 + // velocity
        3 // accumulated force
    }

    fn as_state(&self) -> Vec<f32> {
        let state_vec = vec![
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
        ];
        if state_vec.len() != Self::num_state_elements() {
            panic!("Incorrect state vector size!");
        }
        state_vec
    }

    fn derivative(&self) -> Vec<f32> {
        let derivative_state = vec![
            // Conservation of mass!
            0.0,
            // Position derivative
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
            // Velocity derivative
            self.accumulated_force.x / self.mass,
            self.accumulated_force.y / self.mass,
            self.accumulated_force.z / self.mass,
            // Accumulated force
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
        let mass = state_data[0];
        let position = Vector3::<f32>::new(state_data[1], state_data[2], state_data[3]);
        let velocity = Vector3::<f32>::new(state_data[4], state_data[5], state_data[6]);
        let accumulated_force = Vector3::<f32>::new(state_data[7], state_data[8], state_data[9]);
        Particle {
            mass,
            position,
            velocity,
            accumulated_force,
        }
    }
}

pub struct Simulation {
    config: Config,
    particles: Vec<Particle>,
}

impl Simulation {
    pub fn new() -> Self {
        let particles = vec![Particle::new(
            1.0, // TODO very well might need tweaking
            Vector3::<f32>::zero(),
            Vector3::<f32>::zero(),
        )];
        Simulation {
            config: Config::default(),
            particles,
        }
    }

    pub fn step(&mut self) -> Duration {
        // TODO construct kdtree

        // Accumulate forces on the particles
        self.particles.iter_mut().for_each(|particle| {
            particle.accumulated_force += self.config.gravity;
        });

        let state = State::new(self.particles.clone());
        let new_state = match self.config.integration {
            Integration::Rk4 => state.rk4_step(self.config.dt),
            Integration::Euler => state.euler_step(self.config.dt),
        };
        let new_particles = new_state.get_elements();

        self.update_particles(new_particles);

        // TODO Since fluid sims have so many particles, adding bounding box
        //        checks and other collision performance improvements would be
        //        an improvement. Since we're inside an inverted obstacle for
        //        our testing, though, we might not see benefits except in narrow phase
        //        for that test case.

        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    pub fn sync_sim_from_ui(&mut self, ui: &mut crate::gui::sph::SphUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.integration = ui_config_state.integration;
        self.config.dt = ui_config_state.dt;
        self.config.gravity = ui_config_state.gravity;
    }

    /// Updates the particles with the new particles, handling collisions
    /// and zeroing accumulated forces, readying the simulation for the next step.
    fn update_particles(&mut self, new_particles: Vec<Particle>) {
        // TODO handle collisions
        self.particles = new_particles;

        self.particles
            .iter_mut()
            .for_each(|particle| particle.accumulated_force = Vector3::<f32>::zero());
    }
}
