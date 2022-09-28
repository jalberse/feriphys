use arrayvec::ArrayVec;
use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use rand::{self, Rng};
use std::time::Duration;

use crate::{entity::Entity, forms, gpu_interface::GPUInterface, instance::Instance};

// TODO When we add GUI, set the max to higher than this.
pub const MAX_PARTICLES: usize = 1000;

/// TODO:
/// We can't use many particles because the Pool is on the stack.
/// We should allocate it on the heap to avoid stack overflow.
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

/// Generates particles in the plane defined by position, normal.
struct Generator {
    position: Vector3<f32>,
    normal: Vector3<f32>,
}

impl Generator {
    // Generates particles in a uniform distribution with
    // zero initial velocity.
    pub fn generate_particles(
        &self,
        pool: &mut ParticlePool,
        num_particles: u32,
        lifetime: Duration,
    ) {
        let mut rng = rand::thread_rng();

        let non_parallel_vec =
            if cgmath::relative_eq!(self.normal.normalize(), Vector3::<f32>::unit_z()) {
                Vector3::<f32>::unit_x()
            } else {
                Vector3::<f32>::unit_z()
            };

        let vec_in_plane = self.normal.cross(non_parallel_vec).normalize();
        for _ in 0..num_particles {
            let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
            let radius: f32 = 1.0 - rng.gen::<f32>().powi(2);

            let rotated_vec = vec_in_plane * f32::cos(angle)
                + self.normal.cross(vec_in_plane) * f32::sin(angle)
                + self.normal * self.normal.dot(vec_in_plane) * (1.0 - f32::cos(angle));
            let gen_position = self.position + rotated_vec.normalize() * radius;

            // TODO make the mass, range configurable. I guess we might pass some
            //   particle config with min/max values.
            pool.create(
                gen_position,
                Vector3::<f32>::zero(),
                lifetime,
                rng.gen_range(0.9..1.1),
                rng.gen_range(0.4..0.6),
            );
        }
    }
}

struct ParticlePool {
    pub particles: [Particle; MAX_PARTICLES],
}

impl ParticlePool {
    pub fn new() -> ParticlePool {
        let particles: [Particle; MAX_PARTICLES] = [Particle::default(); MAX_PARTICLES];
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
            lifetime: Duration::ZERO,
            mass: 0.0,
            drag: 0.0,
        }
    }
}

pub struct Config {
    pub dt: f32, // secs as f32
    pub particles_generated_per_step: u32,
    pub particles_lifetime: Duration,
    pub acceleration_gravity: Vector3<f32>,
    pub wind: cgmath::Vector3<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
            particles_generated_per_step: 1,
            particles_lifetime: Duration::from_secs(1),
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
    generator: Generator,
    particles: ParticlePool,
}

impl State {
    pub fn new() -> State {
        let config = Config::default();

        let particles = ParticlePool::new();

        let generator = Generator {
            position: Vector3::<f32> {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
            normal: Vector3::<f32> {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
        };

        State {
            config,
            generator,
            particles,
        }
    }

    pub fn step(&mut self) -> std::time::Duration {
        self.generator.generate_particles(
            &mut self.particles,
            self.config.particles_generated_per_step,
            self.config.particles_lifetime,
        );

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

        let mut instances = ArrayVec::<Instance, MAX_PARTICLES>::new();
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

    pub fn get_particles_instances(&self) -> ArrayVec<Instance, MAX_PARTICLES> {
        let mut instances = ArrayVec::<Instance, MAX_PARTICLES>::new();

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
