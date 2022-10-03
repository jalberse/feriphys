use super::boid::Boid;
use crate::{graphics::instance::Instance, gui};

use cgmath::{Rotation3, Vector3, Zero};

use std::time::Duration;

pub struct Config {
    pub dt: f32, // secs as f32
    pub avoidance_factor: f32,
    pub centering_factor: f32,
    pub velocity_matching_factor: f32,
    /// The maximum distance for which the weight for two boid's interaction is 1.0
    pub distance_weight_threshold: f32,
    /// The distance past the distance_threshold over which the weight interpolates to 0.0.
    pub distance_weight_threshold_falloff: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
            avoidance_factor: 1.0,
            centering_factor: 1.0,
            velocity_matching_factor: 1.0,
            distance_weight_threshold: 1.0,
            distance_weight_threshold_falloff: 1.0,
        }
    }
}

pub struct Simulation {
    config: Config,
    boids: Vec<Boid>,
}

impl Simulation {
    pub fn new() -> Simulation {
        let config = Config::default();

        let mut boids = Vec::with_capacity(25);
        for _ in 0..25 {
            let position = Vector3::<f32> {
                x: rand::random(),
                y: rand::random(),
                z: rand::random(),
            };
            let velocity = Vector3::<f32> {
                x: rand::random(),
                y: rand::random(),
                z: rand::random(),
            };
            boids.push(Boid { position, velocity });
        }

        Simulation { config, boids }
    }

    pub fn step(&mut self) -> Duration {
        // TODO we could use a double buffer here instead of allocating a new vector here every step.
        let mut new_state = Vec::with_capacity(self.boids.len());

        for boid in self.boids.iter() {
            let mut boid_acceleration = Vector3::<f32>::zero();
            for other_boid in self.boids.iter() {
                if other_boid == boid
                    || boid.distance(other_boid)
                        > self.config.distance_weight_threshold
                            + self.config.distance_weight_threshold_falloff
                {
                    continue;
                }

                boid_acceleration = boid_acceleration
                    + boid.get_acceleration(
                        other_boid,
                        self.config.avoidance_factor,
                        self.config.centering_factor,
                        self.config.velocity_matching_factor,
                        self.config.distance_weight_threshold,
                        self.config.distance_weight_threshold_falloff,
                    );
            }

            // TODO add handling for external forces on this boid

            let new_boid_position = boid.position + self.config.dt * boid.velocity;
            let new_boid_velocity = boid.velocity + self.config.dt * boid_acceleration;

            new_state.push(Boid {
                position: new_boid_position,
                velocity: new_boid_velocity,
            });
        }

        self.boids = new_state;

        self.get_timestep()
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn sync_sim_config_from_ui(&mut self, ui: &mut gui::flocking::FlockingUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.dt = ui_config_state.dt;
        self.config.avoidance_factor = ui_config_state.avoidance_factor;
        self.config.centering_factor = ui_config_state.centering_factor;
        self.config.velocity_matching_factor = ui_config_state.velocity_matching_factor;
        self.config.distance_weight_threshold = ui_config_state.distance_weight_threshold;
        self.config.distance_weight_threshold_falloff =
            ui_config_state.distance_weight_threshold_falloff;
    }

    pub fn get_boid_instances(&self) -> Vec<Instance> {
        let mut instances = Vec::<Instance>::with_capacity(self.boids.len());

        for boid in self.boids.iter() {
            instances.push(Instance {
                position: boid.position,
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 0.1,
            });
        }
        instances
    }
}
