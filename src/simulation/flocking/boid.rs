use std::time::Duration;

use cgmath::{InnerSpace, Vector3, Zero};

use crate::simulation::parametric::Parametric;

pub trait Boid {
    fn position(&self) -> Vector3<f32>;
    fn velocity(&self) -> Vector3<f32>;
    fn weight(&self) -> f32;
}

pub struct LeadBoid {
    parametric: Parametric,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    weight: f32,
}

impl Boid for LeadBoid {
    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn velocity(&self) -> Vector3<f32> {
        self.velocity
    }

    fn weight(&self) -> f32 {
        self.weight
    }
}

impl LeadBoid {
    pub fn new(path: fn(t: f32) -> Vector3<f32>) -> LeadBoid {
        let parametric = Parametric::new(path);
        LeadBoid {
            parametric,
            position: path(0.0),
            velocity: Vector3::<f32>::zero(),
            weight: 3.0,
        }
    }

    pub fn step(&mut self, dt: Duration) {
        if dt.is_zero() {
            return;
        }
        let new_position = self.parametric.step(dt.as_secs_f32());
        self.velocity = (new_position - self.position) / dt.as_secs_f32();
        self.position = new_position;
    }
}

#[derive(PartialEq)]
pub struct FlockingBoid {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    /// Weight for interactions with other boids
    weight: f32,
    /// Mass for gravitational attraction to e.g. a PointAttractor
    mass: f32,
}

impl Boid for FlockingBoid {
    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn velocity(&self) -> Vector3<f32> {
        self.velocity
    }

    fn weight(&self) -> f32 {
        self.weight
    }
}

impl FlockingBoid {
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>) -> FlockingBoid {
        FlockingBoid {
            position,
            velocity,
            weight: 1.0,
            mass: 1.0,
        }
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn distance(&self, other: &impl Boid) -> f32 {
        (other.position() - self.position).magnitude()
    }

    /// Returns the "sight angle" from this boid to the other,
    /// i.e. the angle away from the forward (velocity) vector of this
    /// boid to the other. Radians.
    pub fn sight_angle(&self, other: &impl Boid) -> f32 {
        f32::acos(
            self.velocity
                .normalize()
                .dot((other.position() - self.position).normalize()),
        )
    }

    /// Gets the acceleration of this boid due to the other boid due to the avoidance force.
    pub fn get_avoidance_acceleration(&self, other: &impl Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.position(), self.position) {
            return Vector3::<f32>::zero();
        }
        -1.0 * factor / self.distance(other).powf(2.0)
            * (other.position() - self.position).normalize()
            * other.weight()
    }

    /// Gets the acceleration of this boid due to the other boid due to the centering force.
    pub fn get_centering_acceleration(&self, other: &impl Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.position(), self.position) {
            return Vector3::<f32>::zero();
        }
        factor
            * self.distance(other)
            * (other.position() - self.position).normalize()
            * other.weight()
    }

    /// Gets the acceleration of this boid due to the other boid due to the velocity matching force.
    pub fn get_velocity_matching(&self, other: &impl Boid, factor: f32) -> Vector3<f32> {
        if cgmath::abs_diff_eq!(other.velocity(), self.velocity) {
            return Vector3::<f32>::zero();
        }
        factor * (other.velocity() - self.velocity) * other.weight()
    }

    /// Gets the acceleration of this boid due to the other boid.
    pub fn get_acceleration(
        &self,
        other: &impl Boid,
        avoidance_factor: f32,
        centering_factor: f32,
        velocity_matching_factor: f32,
        distance_weight_threshold: f32,
        distance_weight_threshold_falloff: f32,
        max_sight_angle: f32,
    ) -> Vector3<f32> {
        if self.sight_angle(other) > max_sight_angle {
            return Vector3::<f32>::zero();
        }
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
