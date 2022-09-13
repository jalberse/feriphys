/// The bounce module contains the logic for a bouncing ball simulation.
use cgmath::InnerSpace;

/// TODO
/// While I'm working on collision stuff, be wary of floating point accuracy. We will need some epsilon and use it.
/// * Add collision with a ground plane.
///     * First just detection
///     * then the response and stuff, ig
/// * Add friction to that collision
/// * Resting contacts
/// * Add horizontal motion to the ball's initial state so we can check out friciton and air resistance easier.
/// * Finally, add other planes for collision.
/// * Bonus: Add wind (trivial, do it)
/// * Bonus: Make the constants in this file configurable.
/// * Add custom initial state stuff, so we can test collisions with other planes.
///   I think this will be just "hit numbers 1-6" and we reset the state with some initial velocity to hit each wall.

// Constant values for the simulation.
// TODO: Make these configurable!
const SPHERE_MASS: f32 = 1.0;
const DRAG: f32 = 0.5;
const ACCELERATION_GRAVITY: f32 = -10.0;

pub struct State {
    position: cgmath::Vector3<f32>,
    velocity: cgmath::Vector3<f32>,
}

impl State {
    pub fn new() -> State {
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
        State { position, velocity }
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

        let acceleration = acceleration_air_resistance + acceleration_gravity;

        // Numerically integrate to get thew new state, updating the state.
        self.position = self.position + dt.as_secs_f32() * self.velocity;
        self.velocity = self.velocity + dt.as_secs_f32() * acceleration;

        dt
    }
}
