use std::time::Duration;

use cgmath::{InnerSpace, Vector3, Zero};

use self::config::Config;

use super::consts;

pub mod config;

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

#[derive(Clone, Copy)]
pub struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
}

impl Particle {
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>) -> Particle {
        Particle { position, velocity }
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
        for x in -16..16 {
            for z in -10..10 {
                for y in -10..10 {
                    let x_pos = x as f32 * 0.1;
                    let z_pos = z as f32 * 0.1;
                    let y_pos = y as f32 * 0.1;
                    particles.push(Particle::new(
                        Vector3::<f32>::new(x_pos, y_pos, z_pos),
                        Vector3::<f32>::zero(),
                    ));
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
        // TODO construct kdtree

        let mut new_particles = Vec::with_capacity(self.particles.len());
        // Calculate the derivative of the velocity for each particle according to
        // Navier Stokes (momentum)
        self.particles.iter().for_each(|particle| {
            let external_acceleration = self.config.gravity / self.config.particle_mass;

            // TODO Presure gradient

            // TODO Diffusion

            let du_dt = external_acceleration;

            let new_position = particle.position + self.config.dt * particle.velocity;
            let new_velocity = particle.velocity + self.config.dt * du_dt;
            let new_particle = Particle::new(new_position, new_velocity);
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
                let velocity_collision = old_particle.velocity;

                let new_position = collision_point + plane.normal() * consts::EPSILON;

                let velocity_collision_normal =
                    velocity_collision.dot(*plane.normal()) * plane.normal();
                let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                let velocity_response_normal =
                    -1.0 * velocity_collision_normal * self.config.coefficient_of_restitution;
                let velocity_response_tangent = if velocity_collision_tangent.is_zero()
                    || velocity_collision_tangent.magnitude().is_nan()
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
}
