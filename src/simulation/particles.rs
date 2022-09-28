use arrayvec::ArrayVec;
use cgmath::{Rotation3, Vector3};
use rand;

use crate::{
    entity::{Entity, MAX_PARTICLE_INSTANCES},
    forms,
    gpu_interface::GPUInterface,
    instance::Instance,
};

/// TODO:
///
/// A simple step function that just changes the position of particles.
/// Start calling it.
///
/// A function to get instances from the state. We'll call that to get
/// the updated positions, and then pass those instances
/// to the scene to update the rendered positions.
///
/// Now that we can visualzie particles moving, we can add a simple force.
/// Add gravity and wind, and let them fall.
///
/// Next, we can add a lifetime. After some time, all the particles should die.
/// This will involve setting up our pool!
///
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

struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
}

pub struct Config {
    pub dt: f32, // secs as f32
    pub acceleration_gravity: Vector3<f32>,
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
        }
    }
}

pub struct State {
    config: Config,
    particles: Vec<Particle>,
}

impl State {
    pub fn new() -> State {
        let config = Config::default();

        let mut particles = vec![];
        for _ in 0..100 {
            particles.push(Particle {
                position: Vector3::<f32> {
                    x: rand::random::<f32>() * 3.0,
                    y: rand::random::<f32>() * 3.0,
                    z: rand::random::<f32>() * 3.0,
                },
                velocity: Vector3::<f32> {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            })
        }
        State { config, particles }
    }

    pub fn step(&mut self) -> std::time::Duration {
        for particle in self.particles.iter_mut() {
            // Calculate acceleration of particle from forces
            let acceleration = self.config.acceleration_gravity;

            let original_position = particle.position;
            let original_velocity = particle.velocity;

            // Euler integration to get the new location
            let new_position = original_position + self.config.dt * original_velocity;
            let new_velocity = original_velocity + self.config.dt * acceleration;

            particle.position = new_position;
            particle.velocity = new_velocity;
        }

        std::time::Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_particles_entity(&self, gpu: &GPUInterface) -> Entity {
        let mesh = forms::get_quad(&gpu.device, [1.0, 1.0, 1.0]);

        let mut instances = ArrayVec::<Instance, MAX_PARTICLE_INSTANCES>::new();
        for particle in self.particles.iter() {
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

        for particle in self.particles.iter() {
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
