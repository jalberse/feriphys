pub mod config;
mod kernals;

use self::config::Config;
use super::consts;

use cgmath::{InnerSpace, Vector3, Zero};
use itertools::Itertools;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use rustc_hash::FxHashMap;

use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Plane {
    point: Vector3<f32>,
    normal: Vector3<f32>,
}

impl Plane {
    pub fn normal(&self) -> &Vector3<f32> {
        &self.normal
    }

    pub fn distance_from_plane(&self, position: Vector3<f32>) -> f32 {
        (position - self.point).dot(*self.normal())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Particle {
    id: u32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
}

impl Particle {
    pub fn new(id: u32, position: Vector3<f32>, velocity: Vector3<f32>) -> Particle {
        Particle {
            id,
            position,
            velocity,
        }
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }
}

pub struct Simulation {
    config: Config,
    particles: Vec<Particle>,
    min_bounds: Vector3<f32>,
    max_bounds: Vector3<f32>,
}

impl Simulation {
    pub fn new(min_bounds: Vector3<f32>, max_bounds: Vector3<f32>) -> Self {
        let mut particles = Vec::<Particle>::new();

        let mut id = 0;
        for x in -3..3 {
            for z in -3..3 {
                for y in -3..3 {
                    let x_pos = x as f32 * 0.1;
                    let z_pos = z as f32 * 0.1;
                    let y_pos = y as f32 * 0.1;
                    particles.push(Particle::new(
                        id,
                        Vector3::<f32>::new(x_pos, y_pos, z_pos),
                        Vector3::<f32>::zero(),
                    ));
                    id += 1;
                }
            }
        }

        Simulation {
            config: Config::default(),
            particles,
            min_bounds,
            max_bounds,
        }
    }

    pub fn step(&mut self) -> Duration {
        // Build the kdtree
        let mut kdtree = KdTree::new();
        self.particles
            .iter()
            .for_each(|particle| kdtree.add(particle.position.as_ref(), particle).unwrap());

        // Find the neighbors for each particle
        let mut neighbor_map: FxHashMap<u32, Vec<Particle>> =
            FxHashMap::with_capacity_and_hasher(self.particles.len(), Default::default());
        let mut density_map: FxHashMap<u32, f32> =
            FxHashMap::with_capacity_and_hasher(self.particles.len(), Default::default());
        self.particles.iter().for_each(|particle| {
            let neighbors = kdtree
                .nearest(particle.position.as_ref(), 8, &squared_euclidean)
                .unwrap();
            let neighbors = neighbors
                .iter()
                .filter(|neighbor| neighbor.0 < self.config.kernal_max_distance)
                .collect_vec();
            let neighbors = neighbors
                .iter()
                .map(|(_, &&particle)| particle)
                .collect_vec();

            let density: f32 = neighbors
                .iter()
                .map(|neighbor| {
                    let r_ij = particle.position - neighbor.position;
                    let r = if r_ij.is_zero() {
                        0.0
                    } else {
                        r_ij.magnitude()
                    };
                    self.config.particle_mass
                        * kernals::monaghan(r, self.config.kernal_max_distance)
                })
                .sum();

            density_map.insert(particle.id, density);
            neighbor_map.insert(particle.id, neighbors);
        });

        // Do navier-stokes to find new particle positions, velocities.
        let mut new_particles = Vec::with_capacity(self.particles.len());
        self.particles.iter().for_each(|particle| {
            let neighbors = neighbor_map.get(&particle.id).unwrap();

            let density = *density_map.get(&particle.id).unwrap();
            let pressure = self.pressure(density);

            let pressure_gradient: Vector3<f32> = neighbors
                .iter()
                .map(|neighbor| {
                    if neighbor.id == particle.id {
                        return Vector3::<f32>::zero();
                    }
                    let neighbor_density = *density_map.get(&neighbor.id).unwrap();
                    let neighbor_pressure = self.pressure(neighbor_density);
                    self.config.particle_mass
                        * ((pressure / density.powi(2))
                            + (neighbor_pressure / neighbor_density.powi(2)))
                        * kernals::monaghan_gradient(
                            neighbor.position - particle.position,
                            self.config.kernal_max_distance,
                        )
                })
                .sum();

            let diffusion: Vector3<f32> = neighbors
                .iter()
                .map(|neighbor| {
                    let r_ij = neighbor.position - particle.position;
                    let r = if r_ij.is_zero() {
                        0.0
                    } else {
                        r_ij.magnitude()
                    };
                    self.config.particle_mass * (neighbor.velocity - particle.velocity) / density
                        * kernals::monaghan_laplacian(r, self.config.kernal_max_distance)
                })
                .sum::<Vector3<f32>>()
                * self.config.kinematic_viscosity;

            let surface_value: Vector3<f32> = neighbors
                .iter()
                .map(|neighbor| {
                    let neighbor_density = *density_map.get(&neighbor.id).unwrap();
                    self.config.particle_mass / neighbor_density
                        * kernals::monaghan_gradient(
                            particle.position - neighbor.position,
                            self.config.kernal_max_distance,
                        )
                })
                .sum();
            let surface_normal = if surface_value.is_zero() {
                Vector3::<f32>::zero()
            } else {
                surface_value.normalize()
            };
            let surface_divergence: f32 = neighbors
                .iter()
                .map(|neighbor| {
                    let r_ij = neighbor.position - particle.position;
                    let r = if r_ij.is_zero() {
                        0.0
                    } else {
                        r_ij.magnitude()
                    };
                    let neighbor_density = *density_map.get(&neighbor.id).unwrap();
                    self.config.particle_mass / neighbor_density
                        * kernals::monaghan_laplacian(r, self.config.kernal_max_distance)
                })
                .sum();

            let surface_tension_force =
                -self.config.surface_tension_proportionality * surface_divergence * surface_normal;

            let external_acceleration =
                self.config.gravity + surface_tension_force / self.config.particle_mass;

            let du_dt = -pressure_gradient + diffusion + external_acceleration;

            if particle.id == 0 {
                println!(
                    "Pressure gradient: {}, {}, {}",
                    pressure_gradient.x, pressure_gradient.y, pressure_gradient.z
                );
                println!(
                    "Diffusion: {}, {}, {}",
                    diffusion.x, diffusion.y, diffusion.z
                );
                println!(
                    "Surface tension force: {}, {}, {}",
                    surface_tension_force.x, surface_tension_force.y, surface_tension_force.z
                );
            }

            let new_position = particle.position + self.config.dt * particle.velocity;
            let new_velocity = particle.velocity + self.config.dt * du_dt;
            let new_particle = Particle::new(particle.id, new_position, new_velocity);
            new_particles.push(new_particle);
        });

        self.update_particles(new_particles);

        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    pub fn sync_sim_from_ui(&mut self, ui: &mut crate::gui::sph::SphUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.integration = ui_config_state.integration;
        self.config.dt = ui_config_state.dt;
        self.config.particle_mass = ui_config_state.particle_mass;
        self.config.kernal_max_distance = ui_config_state.kernal_max_distance;
        self.config.gravity = ui_config_state.gravity;
        self.config.coefficient_of_restitution = ui_config_state.coefficient_of_restitution;
        self.config.coefficient_of_friction = ui_config_state.coefficient_of_friction;
    }

    /// Updates the particles with the new particles, handling collisions with bounding box
    /// and zeroing accumulated forces, readying the simulation for the next step.
    fn update_particles(&mut self, mut new_particles: Vec<Particle>) {
        for (new_particle, old_particle) in new_particles.iter_mut().zip(&self.particles) {
            if let Some(plane) =
                self.get_collided_plane(old_particle.position, new_particle.position)
            {
                let old_distance_to_plane = plane.distance_from_plane(old_particle.position);
                let new_distance_to_plane = plane.distance_from_plane(new_particle.position);

                let fraction_timestep =
                    old_distance_to_plane / (old_distance_to_plane - new_distance_to_plane);

                let collision_point = old_particle.position
                    + self.config.dt * fraction_timestep * old_particle.velocity;
                let collision_point = collision_point + plane.normal() * consts::EPSILON;
                let new_position = Vector3::new(
                    collision_point.x.clamp(
                        self.min_bounds.x + consts::EPSILON,
                        self.max_bounds.x - consts::EPSILON,
                    ),
                    collision_point.y.clamp(
                        self.min_bounds.y + consts::EPSILON,
                        self.max_bounds.y - consts::EPSILON,
                    ),
                    collision_point.z.clamp(
                        self.min_bounds.z + consts::EPSILON,
                        self.max_bounds.z - consts::EPSILON,
                    ),
                );

                let velocity_collision = old_particle.velocity;

                let velocity_collision_normal =
                    velocity_collision.dot(*plane.normal()) * plane.normal();
                let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                let velocity_response_normal =
                    -1.0 * velocity_collision_normal * self.config.coefficient_of_restitution;
                let velocity_response_tangent = if velocity_collision_tangent.is_zero()
                    || velocity_collision_tangent.magnitude().is_nan()
                    || velocity_collision_normal.is_zero()
                {
                    Vector3::<f32>::zero()
                } else {
                    velocity_collision_tangent
                        - velocity_collision_tangent.normalize()
                            * f32::min(
                                self.config.coefficient_of_friction
                                    * velocity_collision_normal.magnitude(),
                                velocity_collision_tangent.magnitude(),
                            )
                };

                let velocity_response = velocity_response_normal + velocity_response_tangent;

                new_particle.position = new_position;
                new_particle.velocity = velocity_response;
            }
        }

        self.particles = new_particles;
    }

    fn get_bounding_planes(&self) -> Vec<Plane> {
        let bottom = Plane {
            point: self.min_bounds,
            normal: Vector3::unit_y(),
        };
        let top = Plane {
            point: self.max_bounds,
            normal: -Vector3::unit_y(),
        };
        let left = Plane {
            point: self.min_bounds,
            normal: Vector3::unit_x(),
        };
        let right = Plane {
            point: self.max_bounds,
            normal: -Vector3::unit_x(),
        };
        let back = Plane {
            point: self.min_bounds,
            normal: Vector3::unit_z(),
        };
        let front = Plane {
            point: self.max_bounds,
            normal: -Vector3::unit_z(),
        };
        vec![bottom, top, left, right, back, front]
    }

    fn get_collided_plane(
        &self,
        old_position: Vector3<f32>,
        new_position: Vector3<f32>,
    ) -> Option<Plane> {
        let planes = self.get_bounding_planes();

        planes
            .iter()
            .find(|plane| {
                let old_distance_to_plane = plane.distance_from_plane(old_position);
                let new_distance_to_plane = plane.distance_from_plane(new_position);

                // If the signs don't match, it crossed the plane
                old_distance_to_plane.is_sign_positive() != new_distance_to_plane.is_sign_positive()
            })
            .cloned()
    }

    fn pressure(&self, density: f32) -> f32 {
        self.config.pressure_siffness * (density - self.config.reference_density)
    }
}
