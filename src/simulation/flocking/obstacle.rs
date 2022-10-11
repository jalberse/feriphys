use std::time::Duration;

use cgmath::{num_traits::Signed, InnerSpace, Vector3, Zero};

use crate::graphics::entity::ColoredMeshEntity;

use super::boid::{Boid, FlockingBoid};

/// An obstacle which FlockingBoids may avoid by steering, handled as a bounding sphere for some mesh.
pub struct Obstacle {
    pub position: Vector3<f32>,
    pub radius: f32,
}

impl Obstacle {
    /// The time it would take for the boid to collide with the plane perpendicular to the
    /// difference in positions, which includes the obstacle.
    /// If it will never collide with that plane, returns None.
    pub fn get_time_to_plane_collision(&self, boid: &FlockingBoid) -> Option<Duration> {
        if !self.will_collide_with_plane(boid) {
            return None;
        }
        let (velocity_i, _) = self.get_velocity_components(boid);
        Some(Duration::from_secs_f32(
            ((self.position - boid.position()).magnitude() - self.radius) / velocity_i.magnitude(),
        ))
    }

    /// Gets the acceleration required to apply to the boid, in order to avoid this obstacle.
    pub fn get_acceleration_to_avoid(&self, boid: &FlockingBoid) -> Vector3<f32> {
        let (_, velocity_t) = self.get_velocity_components(boid);
        let time_to_collision_maybe = self.get_time_to_plane_collision(boid);
        match time_to_collision_maybe {
            Some(time_to_collision) => {
                let time_to_collision = time_to_collision.as_secs_f32();
                if time_to_collision * velocity_t.magnitude() > self.radius {
                    return Vector3::<f32>::zero();
                }
                2.0 * (self.radius - time_to_collision * velocity_t.magnitude())
                    / time_to_collision.powi(2)
                    * velocity_t.normalize()
            }
            None => Vector3::<f32>::zero(),
        }
    }

    pub fn from_entity(entity: &ColoredMeshEntity, radius: f32) -> Vec<Obstacle> {
        entity
            .instances()
            .iter()
            .map(|instance| -> Obstacle {
                Obstacle {
                    position: instance.position,
                    radius: instance.scale * radius,
                }
            })
            .collect()
    }

    /// If the boid continues at its current velocity, will it collide with the plane perpendicular
    /// to the vector that is the difference between this obstacle's center and the boid's position?
    fn will_collide_with_plane(&self, boid: &FlockingBoid) -> bool {
        let normal = (boid.position() - self.position).normalize();
        let denom = normal.dot(boid.velocity());
        if f32::abs(denom) > f32::EPSILON {
            let t = (self.position - boid.position()).dot(normal) / denom;
            if t.is_positive() {
                return true;
            }
        }
        false
    }

    /// Decompose boid velocity into component towards obstacle and the tangent velocity
    fn get_velocity_components(&self, boid: &FlockingBoid) -> (Vector3<f32>, Vector3<f32>) {
        let direction_to_obstacle = (self.position - boid.position()).normalize();
        let velocity_i = direction_to_obstacle.dot(boid.velocity()) * direction_to_obstacle;
        let velocity_t = boid.velocity() - velocity_i;
        (velocity_i, velocity_t)
    }
}
