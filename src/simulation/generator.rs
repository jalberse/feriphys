use std::time::Duration;

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use super::particle::ParticlePool;

/// Generates particles in the plane defined by position, normal.
pub struct Generator {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
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