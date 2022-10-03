use cgmath::{InnerSpace, Vector3, Zero};

#[derive(PartialEq)]
pub struct Boid {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
}

impl Boid {
    pub fn distance(&self, other: &Boid) -> f32 {
        (other.position - self.position).magnitude()
    }

    /// Gets the acceleration of this boid due to the other boid due to the avoidance force.
    pub fn get_avoidance_acceleration(&self, other: &Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.position, self.position) {
            return Vector3::<f32>::zero();
        }
        -1.0 * factor / self.distance(other).powf(2.0)
            * (other.position - self.position).normalize()
    }

    /// Gets the acceleration of this boid due to the other boid due to the centering force.
    pub fn get_centering_acceleration(&self, other: &Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.position, self.position) {
            return Vector3::<f32>::zero();
        }
        factor * self.distance(other) * (other.position - self.position).normalize()
    }

    /// Gets the acceleration of this boid due to the other boid due to the velocity matching force.
    pub fn get_velocity_matching(&self, other: &Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.velocity, self.velocity) {
            return Vector3::<f32>::zero();
        }
        factor * (other.velocity - self.velocity)
    }

    /// Gets the acceleration of this boid due to the other boid.
    pub fn get_acceleration(
        &self,
        other: &Boid,
        avoidance_factor: f32,
        centering_factor: f32,
        velocity_matching_factor: f32,
        distance_weight_threshold: f32,
        distance_weight_threshold_falloff: f32,
    ) -> Vector3<f32> {
        let distance_weight = if self.distance(other) <= distance_weight_threshold {
            1.0
        } else if self.distance(other)
            >= distance_weight_threshold + distance_weight_threshold_falloff
        {
            0.0
        } else {
            // Linear interpolation from 1.0 at distance_weight_threshold to 0.0 at distance_weight_threshold_falloff.
            (self.distance(other) - distance_weight_threshold) / distance_weight_threshold_falloff
        };
        return distance_weight
            * (self.get_avoidance_acceleration(other, avoidance_factor)
                + self.get_centering_acceleration(other, centering_factor)
                + self.get_velocity_matching(other, velocity_matching_factor));
    }
}
