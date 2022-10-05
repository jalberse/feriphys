use cgmath::{InnerSpace, Vector3};

use crate::graphics::entity::Entity;

use super::consts::GRAVITY;

/// Attracts or repels (if the attractor mass is negative) objects in simulations.
/// Follows a gravitational model.
pub struct PointAttractor {
    pub position: Vector3<f32>,
    pub mass: f32,
}

impl PointAttractor {
    /// Gets the acceleration of some object with the specified mass at position due to this repeller.
    pub fn get_acceleration(&self, position: Vector3<f32>, mass: f32) -> Vector3<f32> {
        -GRAVITY * (self.mass + mass) / (position - self.position).magnitude().powi(2)
            * (position - self.position).normalize()
    }

    /// A utility to get a list of point attractors from an entity, where each instance
    /// of the entity corresponds to a point attractor. Useful for getting point attractors/
    /// repellers for objects in a scene which particles/boids should avoid, e.g.
    /// The mass of the objects is scaled by each Instance's scale.
    /// Note that a negative mass would result in repellers.
    #[allow(dead_code)]
    pub fn from_entity(entity: &Entity, mass: f32) -> Vec<PointAttractor> {
        entity
            .instances()
            .iter()
            .map(|instance| -> PointAttractor {
                PointAttractor {
                    position: instance.position,
                    mass: mass * instance.scale,
                }
            })
            .collect()
    }
}
