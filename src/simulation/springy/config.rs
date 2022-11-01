use std::time::Duration;

use cgmath::Vector3;

pub struct Config {
    pub dt: Duration,
    pub gravity: Vector3<f32>,
    pub coefficient_of_restitution: f32,
    pub coefficient_of_friction: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1),
            gravity: Vector3::<f32>::unit_y() * -1.0,
            coefficient_of_restitution: 0.95,
            coefficient_of_friction: 0.3,
        }
    }
}
