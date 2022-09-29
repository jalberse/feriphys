use arrayvec::ArrayVec;
use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use std::time::Duration;

use crate::{
    entity::Entity, forms, gpu_interface::GPUInterface, gui, instance::Instance,
    model::ColoredMesh, simulation::generator::Generator, simulation::obstacle::Obstacle,
};

use super::particle::ParticlePool;

// TODO Let's use 2500 as the max for when we add the GUI. We'll
//    Keep that constant since it's used for some static instance buffer sizing.
//    But our range will be 0..MAX_INSTANCES for particles.
//   For now, I'm lowering while we develop the simulation further.
pub const MAX_INSTANCES: usize = 2000;

const EPSILON: f32 = 0.001;

/// TODO:
/// We should add colors to our particles. We can do that by adding color information to IntanceRaw,
/// and handling that in the shader instead of using our colored mesh's color. The colored mesh color
/// will only be used to inform the default instance color.
///
/// a vortex would be pretty easy to add. We can probably enable/disable as a bool.
/// We just apply a circular force around the y axis, proportional to the distance
/// from the center (stronger when closer up to some cap).

pub struct Config {
    pub dt: f32, // secs as f32
    pub particles_generated_per_step: u32,
    pub particles_lifetime: f32, // secs as f32
    pub particles_initial_speed: f32,
    pub acceleration_gravity: Vector3<f32>,
    pub wind: cgmath::Vector3<f32>,
    pub generator_radius: f32,
    pub generator_position: Vector3<f32>,
    pub generator_normal: Vector3<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
            particles_generated_per_step: 1,
            particles_lifetime: Duration::from_secs(5).as_secs_f32(),
            particles_initial_speed: 1.0,
            acceleration_gravity: Vector3::<f32> {
                x: 0.0,
                y: -10.0,
                z: 0.0,
            },
            wind: Vector3::<f32> {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            generator_radius: 1.0,
            generator_position: Vector3::<f32>::unit_y() * 2.0,
            generator_normal: Vector3::<f32>::unit_y(),
        }
    }
}

pub struct Simulation {
    config: Config,
    generator: Generator,
    particles: ParticlePool,
    obstacle: Obstacle,
}

impl Simulation {
    pub fn new(obstacle: &ColoredMesh) -> Simulation {
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

        let obstacle = Obstacle::new(&obstacle);

        Simulation {
            config,
            generator,
            particles,
            obstacle,
        }
    }

    pub fn step(&mut self) -> std::time::Duration {
        // TODO we want a way to generate fewer particles, maybe tying it "number generated per second".
        //   Right now we just get to max very quickly, so it generates in waves.

        // For all of these, we can do this in UI by a center of range, and range from that. We'll call it *_mean and *_range.
        // TODO For speed and lifetime, change the UI to the "mean" of those values.
        //   Then add a slider for range around that mean, and pass that in here instead of having same value for every particle.
        //   Ensure we clamp the ranges as appropriate.
        // TODO then do the same for the mass and drag of the particles, which are just hardcoded ranges right now.
        self.generator.generate_particles(
            self.config.generator_position,
            self.config.generator_normal,
            &mut self.particles,
            self.config.particles_generated_per_step,
            self.config.particles_initial_speed,
            Duration::from_secs_f32(self.config.particles_lifetime),
            self.config.generator_radius,
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

            let collided_tri_maybe = if self.obstacle.in_bounds(&new_position) {
                self.obstacle.get_collided_tri(
                    original_position,
                    original_velocity,
                    new_position,
                    self.config.dt,
                )
            } else {
                None
            };

            (particle.position, particle.velocity) = match collided_tri_maybe {
                None => (new_position, new_velocity),
                Some(tri) => {
                    let old_distance_to_plane = tri.distance_from_plane(original_position);
                    let new_distance_to_plane = tri.distance_from_plane(new_position);

                    // Get the point in the plane of the tri
                    let fraction_timestep =
                        old_distance_to_plane / old_distance_to_plane - new_distance_to_plane;

                    let collision_point =
                        original_position + self.config.dt * fraction_timestep * original_velocity;
                    let velocity_collision =
                        original_velocity + self.config.dt * fraction_timestep * acceleration;

                    let new_position = collision_point + tri.normal() * EPSILON;

                    let velocity_collision_normal =
                        velocity_collision.dot(tri.normal()) * tri.normal();
                    let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                    // TODO make the coefficient of restitution (0.9 here) configurable.
                    let velocity_response_normal = -1.0 * velocity_collision_normal * 0.95;
                    let velocity_response_tangent = if velocity_collision_tangent.is_zero() {
                        velocity_collision_tangent
                    } else {
                        // TODO make the coefficient of friction (0.3 here) configurable.
                        velocity_collision_tangent
                            - velocity_collision_tangent.normalize()
                                * f32::min(
                                    0.9 * velocity_collision_normal.magnitude(),
                                    velocity_collision_tangent.magnitude(),
                                )
                    };

                    let velocity_response = velocity_response_normal + velocity_response_tangent;

                    (new_position, velocity_response)
                }
            };

            particle.lifetime = match particle
                .lifetime
                .checked_sub(Duration::from_secs_f32(self.config.dt))
            {
                None => Duration::ZERO,
                Some(duration) => duration,
            };
        }

        std::time::Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_particles_entity(&self, gpu: &GPUInterface) -> Entity {
        let mesh = forms::get_quad(&gpu.device, [1.0, 1.0, 1.0]);

        let mut instances = ArrayVec::<Instance, MAX_INSTANCES>::new();
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

    pub fn get_particles_instances(&self) -> ArrayVec<Instance, MAX_INSTANCES> {
        let mut instances = ArrayVec::<Instance, MAX_INSTANCES>::new();

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

    pub fn sync_sim_config_from_ui(&mut self, ui: &mut gui::particles_gui::ParticlesUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.dt = ui_config_state.dt;
        self.config.acceleration_gravity = ui_config_state.acceleration_gravity;
        self.config.wind = ui_config_state.wind;
        self.config.particles_lifetime = ui_config_state.particles_lifetime;
        self.config.particles_initial_speed = ui_config_state.particles_initial_speed;
        self.config.generator_radius = ui_config_state.generator_radius;
        self.config.generator_position = ui_config_state.generator_position;
        self.config.generator_normal = ui_config_state.generator_normal;
    }
}
