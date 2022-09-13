/// The bounce module contains the logic for a bouncing ball simulation.

/// TODO
/// While I'm working on collision stuff, be wary of floating point accuracy. We will need some epsilon and use it.
/// * Let's add gravity, to get a basic force going
/// * Add horizontal motion to the ball's initial state. We need this to test air resistance and friction, I guess
/// * Add air resistance
/// * Add collision with a ground plane.
///     * First just detection
///     * then the response and stuff, ig
/// * Add friction to that collision
/// * Resting contacts
/// * Finally, add other planes for collision.
/// * Add custom initial state stuff, so we can test collisions with other planes.
///   I think this will be just "hit numbers 1-6" and we reset the state with some initial velocity to hit each wall.

pub struct State {
    position: cgmath::Vector3<f32>,
    // veloctiy: cgmath::Vector3<f32>,
}

impl State {
    pub fn new() -> State {
        let position = cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        State { position }
    }

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    /// Advance the simulation by dt.
    /// If that wouuld result in a collision before dt, advance only until the moment
    /// after the collision.
    /// Returns the time the simulation has advanced.
    /// That is, dt if no collision has occured, or some duration <= dt if a collision did occur.
    pub fn step(&mut self, dt: std::time::Duration) -> std::time::Duration {
        self.position = self.position
            + cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            } * dt.as_secs_f32();

        dt
    }
}
