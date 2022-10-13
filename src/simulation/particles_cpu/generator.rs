use super::particle::ParticlePool;

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use std::{ops::Range, time::Duration};

/// Generates particles in the plane defined by position, normal in a circular disk,
/// with a uniform distribution.
pub fn generate_particles(
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
    let mut rng = rand::thread_rng();
    let non_parallel_vec = if cgmath::relative_eq!(normal.normalize(), Vector3::<f32>::unit_z()) {
        Vector3::<f32>::unit_x()
    } else {
        Vector3::<f32>::unit_z()
    };
    let vec_in_plane = normal.cross(non_parallel_vec).normalize();
    for _ in 0..num_particles {
        let angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
        let radius: f32 = (1.0 - rng.gen::<f32>() * radius.powi(2)) * radius;
        let rotated_vec = vec_in_plane * f32::cos(angle)
            + normal.cross(vec_in_plane) * f32::sin(angle)
            + normal * normal.dot(vec_in_plane) * (1.0 - f32::cos(angle));
        let gen_position = position + rotated_vec.normalize() * radius;
        pool.create(
            gen_position,
            normal * rng.gen_range(speed.start..=speed.end),
            rng.gen_range(lifetime.start..=lifetime.end),
            rng.gen_range(mass.start..=mass.end),
            rng.gen_range(drag.start..=drag.end),
        );
    }
}
