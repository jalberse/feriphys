use cgmath::{Vector3, Zero};

use super::super::state::Integration;

use std::time::Duration;

pub struct Config {
    pub integration: Integration,
    pub dt: f32, // Seconds as f32
    pub particle_mass: f32,
    pub kernal_max_distance: f32,
    pub pressure_siffness: f32,
    pub reference_density: f32,
    pub kinematic_viscosity: f32,
    pub gravity: Vector3<f32>,
    pub coefficient_of_restitution: f32,
    pub coefficient_of_friction: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            integration: Integration::Euler,
            particle_mass: 0.001,
            kernal_max_distance: 0.15,
            pressure_siffness: 1.0,
            reference_density: 1.0,
            kinematic_viscosity: 0.973,
            dt: Duration::from_millis(1).as_secs_f32(),
            gravity: Vector3::<f32>::zero(),
            coefficient_of_restitution: 0.7,
            coefficient_of_friction: 0.3,
        }
    }
}
