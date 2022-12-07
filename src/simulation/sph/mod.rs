use std::time::Duration;

use cgmath::Vector3;

use self::config::Config;

pub mod config;

pub struct Particle {
    position: Vector3<f32>,
}

impl Particle {
    pub fn new(position: Vector3<f32>) -> Particle {
        Particle { position }
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }
}

pub struct Simulation {
    config: Config,
    particles: Vec<Particle>,
}

impl Simulation {
    pub fn new() -> Self {
        let particles = vec![Particle::new(Vector3::new(0.0, 0.0, 0.0))];
        Simulation {
            config: Config::default(),
            particles,
        }
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_particles(&self) -> &Vec<Particle> {
        &self.particles
    }
}
