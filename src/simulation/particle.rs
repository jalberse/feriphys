use std::time::Duration;

use cgmath::{Vector3, Zero};

use super::particles::MAX_INSTANCES;

pub struct ParticlePool {
    pub particles: Vec<Particle>,
}

impl ParticlePool {
    pub fn new() -> ParticlePool {
        let particles = vec![Particle::default(); MAX_INSTANCES];
        ParticlePool { particles }
    }

    /// Activates a particle in the pool and initializes to values.
    /// If there are no free particles in the pool, does nothing.
    /// TODO: Use a free list instead of searching for first unused particle.
    pub fn create(
        &mut self,
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        lifetime: std::time::Duration,
        mass: f32,
        drag: f32,
    ) {
        for particle in self.particles.iter_mut() {
            if !particle.in_use() {
                particle.init(position, velocity, lifetime, mass, drag);
                return;
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Particle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub lifetime: std::time::Duration,
    pub mass: f32,
    pub drag: f32,
}

impl Particle {
    pub fn init(
        &mut self,
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        lifetime: std::time::Duration,
        mass: f32,
        drag: f32,
    ) {
        self.position = position;
        self.velocity = velocity;
        self.lifetime = lifetime;
        self.mass = mass;
        self.drag = drag;
    }

    pub fn in_use(&self) -> bool {
        !self.lifetime.is_zero()
    }
}

impl Default for Particle {
    fn default() -> Self {
        Particle {
            position: Vector3::<f32>::zero(),
            velocity: Vector3::<f32>::zero(),
            lifetime: Duration::ZERO,
            mass: 0.0,
            drag: 0.0,
        }
    }
}
