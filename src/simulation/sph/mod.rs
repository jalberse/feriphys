use std::time::Duration;

use cgmath::{InnerSpace, Vector3, Zero};
use itertools::Itertools;

use self::config::Config;

use super::{
    collidable_mesh::CollidableMesh,
    consts,
    state::{Integration, State, Stateful},
};

pub mod config;

#[derive(Clone, Copy)]
pub struct Particle {
    mass: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,

    du_dt: Vector3<f32>,
}

impl Particle {
    pub fn new(mass: f32, position: Vector3<f32>, velocity: Vector3<f32>) -> Particle {
        Particle {
            mass,
            position,
            velocity,
            du_dt: Vector3::<f32>::zero(),
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
        3 // du_dt
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
            self.du_dt.x,
            self.du_dt.y,
            self.du_dt.z,
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
            self.du_dt.x,
            self.du_dt.y,
            self.du_dt.z,
            // du_dt
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
            du_dt: accumulated_force,
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

    pub fn step(&mut self, obstacles: &Vec<CollidableMesh>) -> Duration {
        // TODO construct kdtree

        // Calculate the derivative of the velocity for each particle according to
        // Navier Stokes (momentum)
        self.particles.iter_mut().for_each(|particle| {
            particle.du_dt += self.config.gravity / particle.mass;

            // TODO Presure gradient

            // TODO Diffusion
        });

        let state = State::new(self.particles.clone());
        let new_state = match self.config.integration {
            Integration::Rk4 => state.rk4_step(self.config.dt),
            Integration::Euler => state.euler_step(self.config.dt),
        };
        let new_particles = new_state.get_elements();

        self.update_particles(new_particles, obstacles);

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
        self.config.coefficient_of_restitution = ui_config_state.coefficient_of_restitution;
        self.config.coefficient_of_friction = ui_config_state.coefficient_of_friction;
    }

    /// Updates the particles with the new particles, handling collisions
    /// and zeroing accumulated forces, readying the simulation for the next step.
    fn update_particles(
        &mut self,
        mut new_particles: Vec<Particle>,
        obstacles: &Vec<CollidableMesh>,
    ) {
        // TODO Since fluid sims have so many particles, adding bounding box
        //        checks and other collision performance improvements would be
        //        an improvement. Since we're inside an inverted obstacle for
        //        our testing, though, we might not see benefits except in narrow phase
        //        for that test case.

        // TODO much or all of this could be shared with springy_mesh easily.
        let obstacle_faces = obstacles
            .iter()
            .map(|o| o.get_faces())
            .flatten()
            .collect_vec();
        for (new_particle, old_particle) in new_particles.iter_mut().zip(&self.particles) {
            if let Some(face) = CollidableMesh::get_collided_face_from_list(
                &obstacle_faces,
                old_particle.position,
                new_particle.position,
                Duration::from_secs_f32(self.config.dt),
            ) {
                let old_distance_to_plane = face.distance_from_plane(&old_particle.position);
                let new_distance_to_plane = face.distance_from_plane(&new_particle.position);

                let fraction_timestep =
                    old_distance_to_plane / (old_distance_to_plane - new_distance_to_plane);

                let collision_point = old_particle.position
                    + self.config.dt * fraction_timestep * old_particle.velocity;
                let velocity_collision = old_particle.velocity
                    + self.config.dt * fraction_timestep * old_particle.du_dt / old_particle.mass;

                let new_position = collision_point + face.normal() * consts::EPSILON;

                let velocity_collision_normal =
                    velocity_collision.dot(face.normal()) * face.normal();
                let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                let velocity_response_normal =
                    -1.0 * velocity_collision_normal * self.config.coefficient_of_restitution;
                let velocity_response_tangent = if velocity_collision_tangent.is_zero()
                    || velocity_collision_tangent.magnitude().is_nan()
                {
                    Vector3::<f32>::zero()
                } else {
                    velocity_collision_tangent
                        - velocity_collision_tangent.normalize()
                            * f32::min(
                                self.config.coefficient_of_friction
                                    * velocity_collision_normal.magnitude(),
                                velocity_collision_tangent.magnitude(),
                            )
                };

                let velocity_response = velocity_response_normal + velocity_response_tangent;

                new_particle.position = new_position;
                new_particle.velocity = velocity_response;
            }
        }

        self.particles = new_particles;

        self.particles
            .iter_mut()
            .for_each(|particle| particle.du_dt = Vector3::<f32>::zero());
    }
}
