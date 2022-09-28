use arrayvec::ArrayVec;
use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use rand::{self, Rng};

use crate::{
    entity::{Entity, MAX_PARTICLE_INSTANCES},
    forms,
    gpu_interface::GPUInterface,
    instance::Instance,
};

/// TODO:
/// Next, we can add a generator. We'll now have something like snow falling.
///
/// Next, we need to add collisions with a polygon.
///
/// We should add colors to our particles. We can do that by adding color information to IntanceRaw,
/// and handling that in the shader instead of using our colored mesh's color. The colored mesh color
/// will only be used to inform the default instance color.
///
/// a vortex would be pretty easy to add. We can probably enable/disable as a bool.
/// We just apply a circular force around the y axis, proportional to the distance
/// from the center (stronger when closer up to some cap).

struct ParticlePool {
    pub particles: [Particle; MAX_PARTICLE_INSTANCES],
}

impl ParticlePool {
    pub fn new() -> ParticlePool {
        let particles: [Particle; MAX_PARTICLE_INSTANCES] =
            [Particle::default(); MAX_PARTICLE_INSTANCES];
        ParticlePool { particles }
    }

    /// Activates a particle in the pool and initializes to values.
    /// If there are no free particles in the pool, does nothing.
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
struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    lifetime: std::time::Duration,
    pub mass: f32,
    pub drag: f32,
}

impl Particle {
    fn init(
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

    fn in_use(&self) -> bool {
        !self.lifetime.is_zero()
    }
}

impl Default for Particle {
    fn default() -> Self {
        Particle {
            position: Vector3::<f32>::zero(),
            velocity: Vector3::<f32>::zero(),
            lifetime: std::time::Duration::ZERO,
            mass: 0.0,
            drag: 0.0,
        }
    }
}

pub struct Config {
    pub dt: f32, // secs as f32
    pub acceleration_gravity: Vector3<f32>,
    pub wind: cgmath::Vector3<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: std::time::Duration::from_millis(1).as_secs_f32(),
            acceleration_gravity: Vector3::<f32> {
                x: 0.0,
                y: -10.0,
                z: 0.0,
            },
            wind: Vector3::<f32>::zero(),
        }
    }
}

pub struct State {
    config: Config,
    particles: ParticlePool,
}

impl State {
    pub fn new() -> State {
        let config = Config::default();

        let mut particles = ParticlePool::new();

        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            particles.create(
                Vector3::<f32> {
                    x: rng.gen_range(-1.0..1.0),
                    y: rng.gen_range(-1.0..1.0),
                    z: rng.gen_range(-1.0..1.0),
                },
                Vector3::<f32>::zero(),
                std::time::Duration::from_secs(1),
                rng.gen_range(0.9..1.1),
                rng.gen_range(0.4..0.6),
            )
        }
        State { config, particles }
    }

    pub fn step(&mut self) -> std::time::Duration {
        for particle in self.particles.particles.iter_mut() {
            // TODO rather than manually checking this here, the pool
            //  should offer an iterator over the active particles.
            if !particle.in_use() {
                continue;
            }

            // Calculate acceleration of particle from forces
            let acceleration_air_resistance =
                -1.0 * particle.drag * particle.velocity * particle.velocity.magnitude()
                    / particle.mass;

            let acceleration_wind =
                particle.drag * self.config.wind * self.config.wind.magnitude() / particle.mass;

            let acceleration =
                self.config.acceleration_gravity + acceleration_air_resistance + acceleration_wind;

            let original_position = particle.position;
            let original_velocity = particle.velocity;

            // Euler integration to get the new location
            let new_position = original_position + self.config.dt * original_velocity;
            let new_velocity = original_velocity + self.config.dt * acceleration;

            particle.position = new_position;
            particle.velocity = new_velocity;
        }

        // Finally, decrement each particle's lifetime, possible killing them.
        for particle in self.particles.particles.iter_mut() {
            if !particle.in_use() {
                continue;
            }
            particle.lifetime = std::time::Duration::ZERO
                .max(particle.lifetime - std::time::Duration::from_secs_f32(self.config.dt));
        }

        std::time::Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_particles_entity(&self, gpu: &GPUInterface) -> Entity {
        let mesh = forms::get_quad(&gpu.device, [1.0, 1.0, 1.0]);

        let mut instances = ArrayVec::<Instance, MAX_PARTICLE_INSTANCES>::new();
        for particle in self.particles.particles.iter() {
            if !particle.in_use() {
                continue;
            }
            let instance = Instance {
                position: particle.position,
                // TODO this should be some Default.
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 0.05,
            };
            instances.push(instance);
        }

        Entity::new(&gpu, mesh, instances)
    }

    pub fn get_particles_instances(&self) -> ArrayVec<Instance, MAX_PARTICLE_INSTANCES> {
        let mut instances = ArrayVec::<Instance, MAX_PARTICLE_INSTANCES>::new();

        for particle in self.particles.particles.iter() {
            if !particle.in_use() {
                continue;
            }
            instances.push(Instance {
                position: particle.position,
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 0.05,
            });
        }
        instances
    }

    pub fn get_timestep(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f32(self.config.dt)
    }
}
