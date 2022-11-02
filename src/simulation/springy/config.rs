use super::super::state::Integration;
use std::time::Duration;

use cgmath::{Vector3, Zero};

const LIFT_COEFFICIENT_DEFAULT: f32 = 1.0;
const DRAG_COEFFICIENT_DEFAULT: f32 = 1.0;

pub struct Config {
    pub integration: Integration,
    pub dt: f32, // Seconds as f32
    pub gravity: Vector3<f32>,
    pub wind: Vector3<f32>,
    pub lift_coefficient: f32,
    pub drag_coefficient: f32,
    pub coefficient_of_restitution: f32,
    pub coefficient_of_friction: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            integration: Integration::Rk4,
            dt: Duration::from_millis(1).as_secs_f32(),
            gravity: Vector3::<f32>::unit_y() * -10.0,
            wind: Vector3::<f32>::zero(),
            lift_coefficient: LIFT_COEFFICIENT_DEFAULT,
            drag_coefficient: DRAG_COEFFICIENT_DEFAULT,
            coefficient_of_restitution: 0.95,
            coefficient_of_friction: 0.3,
        }
    }
}
