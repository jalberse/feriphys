use cgmath::{Vector3, Zero};

use super::super::state::Integration;

use std::time::Duration;

pub struct Config {
    pub integration: Integration,
    pub dt: f32, // Seconds as f32
    pub gravity: Vector3<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            integration: Integration::Rk4,
            dt: Duration::from_millis(1).as_secs_f32(),
            gravity: Vector3::<f32>::zero(),
        }
    }
}
