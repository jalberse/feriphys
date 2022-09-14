/// The bounce module contains the logic for a bouncing ball simulation.
use cgmath::InnerSpace;

/// TODO
/// * Resting contacts
/// * Make the constants in this file configurable.
/// * Add custom initial state stuff, so we can test collisions with other planes.
///   I think this will be just "hit numbers 1-6" and we reset the state with some initial velocity to hit each wall.

const SPHERE_MASS: f32 = 1.0;
const DRAG: f32 = 0.5;
const WIND: cgmath::Vector3<f32> = cgmath::Vector3 {
    x: 2.0,
    y: 0.0,
    z: 0.0,
};
const ACCELERATION_GRAVITY: f32 = -10.0;
const COEFFICIENT_OF_RESTITUTION: f32 = 0.76;
const COEFFICIENT_OF_FRICTION: f32 = 0.25;
const EPSILON: f32 = 0.001;

#[derive(Debug)]
struct Plane {
    point: cgmath::Vector3<f32>,
    normal: cgmath::Vector3<f32>,
}

impl Plane {
    pub fn new(point: cgmath::Vector3<f32>, normal: cgmath::Vector3<f32>) -> Plane {
        let normal = normal.normalize();
        Plane {
            point: point,
            normal: normal,
        }
    }

    pub fn distance_to(&self, point: cgmath::Vector3<f32>) -> f32 {
        (point - self.point).dot(self.normal)
    }
}

pub struct State {
    planes: Vec<Plane>,
    position: cgmath::Vector3<f32>,
    velocity: cgmath::Vector3<f32>,
}

impl State {
    pub fn new() -> State {
        let planes = vec![
            // Top
            Plane::new(
                cgmath::Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                cgmath::Vector3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
            ),
            // Bottom
            Plane::new(
                cgmath::Vector3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                cgmath::Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            ),
            // Left
            Plane::new(
                cgmath::Vector3 {
                    x: -1.0,
                    y: 0.0,
                    z: 0.0,
                },
                cgmath::Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            // Right
            Plane::new(
                cgmath::Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                cgmath::Vector3 {
                    x: -1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            // Front
            Plane::new(
                cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            ),
            // Back
            Plane::new(
                cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
                cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
            ),
        ];

        let position = cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let velocity = cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        State {
            planes,
            position,
            velocity,
        }
    }

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    /// Advance the simulation by dt. Uses first order Euler integration.
    /// If the full timestep wouuld result in a collision before dt,
    /// advances only until the moment after the collision.
    /// Returns the time the simulation has advanced.
    /// That is, dt if no collision has occured, or some duration <= dt if a collision did occur.
    pub fn step(&mut self, dt: std::time::Duration) -> std::time::Duration {
        // Determine the acceleration due to the forces acting on the sphere.
        let acceleration_gravity = cgmath::Vector3 {
            x: 0.0,
            y: ACCELERATION_GRAVITY,
            z: 0.0,
        };

        // Force due to air resistance is equal to the drag times the square of the velocity,
        // in the direction opposite the velocity.
        // By F = ma, the acceleration due to air resistance is thus that value, divided by the mass of the sphere.
        let acceleration_air_resistance =
            -1.0 * DRAG * self.velocity * self.velocity.magnitude() / SPHERE_MASS;

        let acceleration_wind = DRAG * WIND * WIND.magnitude() / SPHERE_MASS;

        let acceleration = acceleration_air_resistance + acceleration_gravity + acceleration_wind;

        let old_position = self.position;
        let old_velocity = self.velocity;

        // Numerically integrate to get thew new state, updating the state.
        let new_position = old_position + dt.as_secs_f32() * old_velocity;
        let new_velocity = old_velocity + dt.as_secs_f32() * acceleration;

        // TODO note that technically, you can collide with two planes at the same time.
        //      That case really *should* be handled.
        let collided_plane_maybe = self.planes.iter().find(|plane| -> bool {
            let old_distance_to_plane = plane.distance_to(old_position);
            let new_distance_to_plane = plane.distance_to(new_position);
            // If the signs are different, the point has crossed the plane
            (old_distance_to_plane > 0.0) != (new_distance_to_plane > 0.0)
        });

        let time_elapsed;
        (self.position, self.velocity, time_elapsed) = match collided_plane_maybe {
            Some(plane) => {
                let fraction_timestep = plane.distance_to(old_position)
                    / plane.distance_to(old_position)
                    - plane.distance_to(new_position);

                // Since the collision occured at fraction_timestep into the timestep,
                // we need to integrate to find the position at that fraction of a timestep.
                // This assumes that the path is linear.
                let collision_point =
                    old_position + dt.as_secs_f32() * fraction_timestep * old_velocity;
                // The velocity the moment before the collision
                let velocity_collision =
                    old_velocity + dt.as_secs_f32() * fraction_timestep * acceleration;

                // We ensure the position is slightly away from the plane to avoid floating-point
                // precision errors that would occur if we were directly on the plane (which
                // would include e.g. incredibly small timesteps as we continuously "collide"
                // with the plane as the ball comes to a rest).
                // TODO We should add resting contacts, and when we do, this can be modified or removed.
                let new_position = collision_point + plane.normal * EPSILON;

                let velocity_collision_normal = velocity_collision.dot(plane.normal) * plane.normal;
                let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                let velocity_response_normal =
                    -1.0 * velocity_collision_normal * COEFFICIENT_OF_RESTITUTION;
                let velocity_response_tangent = velocity_collision_tangent
                    - velocity_collision_tangent.normalize()
                        * f32::min(
                            COEFFICIENT_OF_FRICTION * velocity_collision_normal.magnitude(),
                            velocity_collision_tangent.magnitude(),
                        );

                let velocity_response = velocity_response_normal + velocity_response_tangent;

                (
                    new_position,
                    velocity_response,
                    std::time::Duration::from_secs_f32(dt.as_secs_f32() * fraction_timestep),
                )
            }
            None => (new_position, new_velocity, dt),
        };

        time_elapsed
    }
}
