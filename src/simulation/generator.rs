use std::{ops::Range, time::Duration};

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use super::particle::ParticlePool;

// TODO we don't really need the Struct now - consider removing and making this a pure fn.

/// Generates particles in the plane defined by position, normal.
pub struct Generator {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
}

impl Generator {
    // Generates particles in a uniform distribution with
    // zero initial velocity.
    pub fn generate_particles(
        &mut self,
        position: Vector3<f32>,
        normal: Vector3<f32>,
        radius: f32,
        pool: &mut ParticlePool,
        num_particles: u32,
        // Speed in direction of normal vector to spawn with.
        speed: Range<f32>,
        lifetime: Range<Duration>,
        mass: Range<f32>,
        drag: Range<f32>,
    ) {
        self.position = position;
        self.normal = normal;

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
            let radius: f32 = (1.0 - rng.gen::<f32>() * radius.powi(2)) * radius;

            let rotated_vec = vec_in_plane * f32::cos(angle)
                + self.normal.cross(vec_in_plane) * f32::sin(angle)
                + self.normal * self.normal.dot(vec_in_plane) * (1.0 - f32::cos(angle));
            let gen_position = self.position + rotated_vec.normalize() * radius;

            pool.create(
                gen_position,
                self.normal * rng.gen_range(speed.start..=speed.end),
                rng.gen_range(lifetime.start..=lifetime.end),
                rng.gen_range(mass.start..=mass.end),
                rng.gen_range(drag.start..=drag.end),
            );
        }
    }
}
