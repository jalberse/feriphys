use arrayvec::ArrayVec;
use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use itertools::Itertools;
use rand::{self, Rng};
use std::time::Duration;

use crate::{
    entity::Entity, forms, gpu_interface::GPUInterface, instance::Instance, model::ColoredMesh,
};

// TODO Let's use 2500 as the max for when we add the GUI. We'll
//    Keep that constant since it's used for some static instance buffer sizing.
//    But our range will be 0..MAX_INSTANCES for particles.
//   For now, I'm lowering while we develop the simulation further.
pub const MAX_INSTANCES: usize = 2000;

const EPSILON: f32 = 0.001;

/// TODO:
/// Add GUI for moving generator, changing config, etc...
///
/// We should add colors to our particles. We can do that by adding color information to IntanceRaw,
/// and handling that in the shader instead of using our colored mesh's color. The colored mesh color
/// will only be used to inform the default instance color.
///
/// a vortex would be pretty easy to add. We can probably enable/disable as a bool.
/// We just apply a circular force around the y axis, proportional to the distance
/// from the center (stronger when closer up to some cap).

struct Tri {
    v1: Vector3<f32>,
    v2: Vector3<f32>,
    v3: Vector3<f32>,
}

impl Tri {
    pub fn normal(&self) -> Vector3<f32> {
        (self.v2 - self.v1).cross(self.v3 - self.v1).normalize()
    }

    pub fn distance_from_plane(&self, point: cgmath::Vector3<f32>) -> f32 {
        (point - self.v1).dot(self.normal())
    }
}

struct Obstacle {
    tris: Vec<Tri>,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl Obstacle {
    pub fn new(mesh: &ColoredMesh) -> Obstacle {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        for vertex_index in mesh.vertex_indices.iter() {
            let v = mesh.vertex_positions[*vertex_index as usize];
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
            min_z = min_z.min(v.z);
            max_z = max_z.max(v.z);
        }

        let mut tris = vec![];
        for (i1, i2, i3) in mesh.vertex_indices.iter().tuple_windows() {
            let v1 = mesh.vertex_positions[*i1 as usize];
            let v2 = mesh.vertex_positions[*i2 as usize];
            let v3 = mesh.vertex_positions[*i3 as usize];
            tris.push(Tri { v1, v2, v3 });
        }
        Obstacle {
            tris,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    /// True if the position is in the bounds of the box.
    /// Useful for quick preliminary checks.
    /// Should call with the NEW position, not the old position.
    pub fn in_bounds(&self, position: &Vector3<f32>) -> bool {
        position.x >= self.min_x
            && position.x <= self.max_x
            && position.y >= self.min_y
            && position.y <= self.max_y
            && position.z >= self.min_z
            && position.z <= self.max_z
    }

    /// Returns None if the particle did not collide with the tri.
    /// Otherwise, returns the first polygon it finds that it did collide with.
    pub fn get_collided_tri(
        &self,
        old_position: Vector3<f32>,
        old_velocity: Vector3<f32>,
        new_position: Vector3<f32>,
        dt: f32,
    ) -> Option<&Tri> {
        self.tris.iter().find(|tri| -> bool {
            // TODO add preliminary check for bounding box.
            let old_distance_to_plane = tri.distance_from_plane(old_position);
            let new_distance_to_plane = tri.distance_from_plane(new_position);
            // If the signs are different, the point has crossed the plane
            let crossed_plane = old_distance_to_plane.is_sign_positive()
                != new_distance_to_plane.is_sign_positive();
            if !crossed_plane {
                false
            } else {
                // Get the point in the plane of the tri
                let fraction_timestep =
                    old_distance_to_plane / old_distance_to_plane - new_distance_to_plane;

                let collision_point = old_position + dt * fraction_timestep * old_velocity;

                // Flatten the tri and the point into 2D to check containment.
                let (v1_flat, v2_flat, v3_flat, point_flat) =
                    if tri.normal().x >= tri.normal().y && tri.normal().x >= tri.normal().z {
                        // Eliminate the x component of all the elements
                        let v1_flat = Vector3::<f32> {
                            x: 0.0,
                            y: tri.v1.y,
                            z: tri.v1.z,
                        };
                        let v2_flat = Vector3::<f32> {
                            x: 0.0,
                            y: tri.v2.y,
                            z: tri.v2.z,
                        };
                        let v3_flat = Vector3::<f32> {
                            x: 0.0,
                            y: tri.v3.y,
                            z: tri.v3.z,
                        };
                        let point_flat = Vector3::<f32> {
                            x: 0.0,
                            y: collision_point.y,
                            z: collision_point.z,
                        };
                        (v1_flat, v2_flat, v3_flat, point_flat)
                    } else if tri.normal().y >= tri.normal().x && tri.normal().y >= tri.normal().z {
                        // Eliminate the y component of all the elements
                        let v1_flat = Vector3::<f32> {
                            x: tri.v1.x,
                            y: 0.0,
                            z: tri.v1.z,
                        };
                        let v2_flat = Vector3::<f32> {
                            x: tri.v2.x,
                            y: 0.0,
                            z: tri.v2.z,
                        };
                        let v3_flat = Vector3::<f32> {
                            x: tri.v3.x,
                            y: 0.0,
                            z: tri.v3.z,
                        };
                        let point_flat = Vector3::<f32> {
                            x: collision_point.x,
                            y: 0.0,
                            z: collision_point.z,
                        };
                        (v1_flat, v2_flat, v3_flat, point_flat)
                    } else {
                        // Eliminate the z component of all the elements
                        let v1_flat = Vector3::<f32> {
                            x: tri.v1.x,
                            y: tri.v1.y,
                            z: 0.0,
                        };
                        let v2_flat = Vector3::<f32> {
                            x: tri.v2.x,
                            y: tri.v2.y,
                            z: 0.0,
                        };
                        let v3_flat = Vector3::<f32> {
                            x: tri.v3.x,
                            y: tri.v3.y,
                            z: 0.0,
                        };
                        let point_flat = Vector3::<f32> {
                            x: collision_point.x,
                            y: collision_point.y,
                            z: 0.0,
                        };
                        (v1_flat, v2_flat, v3_flat, point_flat)
                    };

                // Then check the point by comparing the orientation of the cross products
                let cross1 = (v2_flat - v1_flat).cross(point_flat - v1_flat);
                let cross2 = (v3_flat - v2_flat).cross(point_flat - v2_flat);
                let cross3 = (v1_flat - v3_flat).cross(point_flat - v3_flat);

                let cross1_orientation = cross1.dot(tri.normal()).is_sign_positive();
                let cross2_orientation = cross2.dot(tri.normal()).is_sign_positive();
                let cross3_orientation = cross3.dot(tri.normal()).is_sign_positive();

                // The point is in the polygon iff the orientation for all three cross products are equal.
                cross1_orientation == cross2_orientation && cross2_orientation == cross3_orientation
            }
        })
    }
}

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
        // Speed in direction of normal vector to spawn with.
        speed: f32,
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
                self.normal * speed,
                lifetime,
                rng.gen_range(0.9..1.1),
                rng.gen_range(0.4..0.6),
            );
        }
    }
}

struct ParticlePool {
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
    pub particles_initial_speed: f32,
    pub acceleration_gravity: Vector3<f32>,
    pub wind: cgmath::Vector3<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
            particles_generated_per_step: 1,
            particles_lifetime: Duration::from_secs(5),
            particles_initial_speed: 1.0,
            acceleration_gravity: Vector3::<f32> {
                x: 0.0,
                y: -10.0,
                z: 0.0,
            },
            wind: Vector3::<f32> {
                x: 0.5,
                y: 0.0,
                z: 0.0,
            },
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
        self.generator.generate_particles(
            &mut self.particles,
            self.config.particles_generated_per_step,
            self.config.particles_initial_speed,
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

            // TODO here, check for collisions with the tris. Set new_position and new_velocity accordingly.
            //  Similar to ball collision code, but no partial timestep.

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

            particle.lifetime = std::time::Duration::ZERO
                .max(particle.lifetime - std::time::Duration::from_secs_f32(self.config.dt));
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
}
