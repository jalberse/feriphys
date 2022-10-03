use crate::{gui, graphics::instance::Instance};

use cgmath::{Vector3, InnerSpace, Zero, Rotation3};

use std::time::Duration;

#[derive(PartialEq)]
struct Boid {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
}

impl Boid {
    /// Gets the acceleration of this boid due to the other boid due to the avoidance force.
    pub fn get_avoidance_acceleration(&self, other: &Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.position, self.position) {
            return Vector3::<f32>::zero()
        }
        -1.0 * factor / (other.position - self.position).magnitude().powf(2.0) * (other.position - self.position).normalize() 
    }

    /// Gets the acceleration of this boid due to the other boid due to the centering force.
    pub fn get_centering_acceleration(&self, other: &Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.position, self.position) {
            return Vector3::<f32>::zero()
        }
        factor * (other.position - self.position).magnitude() * (other.position - self.position).normalize()
    }

    /// Gets the acceleration of this boid due to the other boid due to the velocity matching force.
    pub fn get_velocity_matching(&self, other: &Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.velocity, self.velocity) {
            return Vector3::<f32>::zero()
        }
        factor * (other.velocity - self.velocity)
    }
}

pub struct Config {
    pub dt: f32, // secs as f32
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
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
            let velocity = Vector3::<f32>::zero();
            boids.push(Boid { position, velocity });
        }

        Simulation {
            config,
            boids,
        }
    }

    pub fn step(&mut self) -> Duration {
        // TODO we could use a double buffer here instead of allocating a new vector here every step.
        let mut new_state = Vec::with_capacity(self.boids.len());
        
        for boid in self.boids.iter() {
            let mut boid_acceleration = Vector3::<f32>::zero();
            for other_boid in self.boids.iter() {
                if other_boid == boid {
                    continue;
                }

                // TODO make these factors configurable through the UI.
                boid_acceleration = boid_acceleration
                    + boid.get_avoidance_acceleration(other_boid, 0.1)
                    + boid.get_centering_acceleration(other_boid, 0.1)
                    + boid.get_velocity_matching(other_boid, 0.1);
            }

            // TODO add handling for external forces on this boid

            let new_boid_position = boid.position + self.config.dt * boid.velocity;
            let new_boid_velocity = boid.velocity + self.config.dt * boid_acceleration;
            
            new_state.push(Boid { position: new_boid_position, velocity: new_boid_velocity});
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
