/// The bounce module contains the logic for a bouncing ball simulation.
use cgmath::{InnerSpace, Zero};

const EPSILON: f32 = 0.001;

struct Config {
    sphere_mass: f32,
    drag: f32,
    wind: cgmath::Vector3<f32>,
    acceleration_gravity: f32,
    coefficient_of_restitution: f32,
    coefficient_of_friction: f32,
    static_coefficient_of_friction: f32,
}

impl Config {
    const MIN_SPHERE_MASS: f32 = 0.05;
    const MAX_SPHERE_MASS: f32 = 10.0;
    const SPHERE_MASS_STEP: f32 = 0.5;

    const MIN_DRAG: f32 = 0.05;
    const MAX_DRAG: f32 = 2.0;
    const DRAG_STEP: f32 = 0.05;

    const MIN_WIND: f32 = -5.0;
    const MAX_WIND: f32 = 5.0;
    const WIND_STEP: f32 = 0.1;

    const ACCELERATION_GRAVITY_MIN: f32 = -20.0;
    const ACCELERATION_GRAVITY_MAX: f32 = 20.0;
    const ACCELERATION_GRAVITY_STEP: f32 = 0.1;

    const COEFFICIENT_OF_RESTITUTION_MIN: f32 = 0.0;
    const COEFFICIENT_OF_RESTITUTION_MAX: f32 = 1.0;
    const COEFFICIENT_OF_RESTITUTION_STEP: f32 = 0.05;

    const COEFFICIENT_OF_FRICTION_MIN: f32 = 0.05;
    const COEFFICIENT_OF_FRICTION_MAX: f32 = 1.0;
    const COEFFICIENT_OF_FRICTION_STEP: f32 = 0.05;

    const STATIC_COEFFICIENT_OF_FRICTION_MIN: f32 = 0.05;
    const STATIC_COEFFICIENT_OF_FRICTION_MAX: f32 = 1.0;
    const STATIC_COEFFICIENT_OF_FRICTION_STEP: f32 = 0.05;
}

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
    config: Config,
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

        let config = Config {
            sphere_mass: 1.0,
            drag: 0.5,
            wind: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            acceleration_gravity: -10.0,
            coefficient_of_restitution: 0.95,
            coefficient_of_friction: 0.25,
            static_coefficient_of_friction: 0.5,
        };

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
            config,
            position,
            velocity,
        }
    }

    pub fn increase_sphere_mass(&mut self) {
        self.config.sphere_mass = f32::min(
            self.config.sphere_mass + Config::SPHERE_MASS_STEP,
            Config::MAX_SPHERE_MASS,
        );
        println!("Sphere mass: {}", self.config.sphere_mass);
    }

    pub fn decrease_sphere_mass(&mut self) {
        self.config.sphere_mass = f32::max(
            self.config.sphere_mass - Config::SPHERE_MASS_STEP,
            Config::MIN_SPHERE_MASS,
        );
        println!("Sphere mass: {}", self.config.sphere_mass);
    }

    pub fn increase_drag(&mut self) {
        self.config.drag = f32::min(self.config.drag + Config::DRAG_STEP, Config::MAX_DRAG);
        println!("Drag: {}", self.config.drag);
    }

    pub fn decrease_drag(&mut self) {
        self.config.drag = f32::max(self.config.drag - Config::DRAG_STEP, Config::MIN_DRAG);
        println!("Drag: {}", self.config.drag);
    }

    pub fn increase_wind_x(&mut self) {
        self.config.wind.x = f32::min(self.config.wind.x + Config::WIND_STEP, Config::MAX_WIND);
        println!("Wind: {:?}", self.config.wind);
    }

    pub fn decrease_wind_x(&mut self) {
        self.config.wind.x = f32::max(self.config.wind.x - Config::WIND_STEP, Config::MIN_WIND);
        println!("Wind: {:?}", self.config.wind);
    }

    pub fn increase_wind_y(&mut self) {
        self.config.wind.y = f32::min(self.config.wind.y + Config::WIND_STEP, Config::MAX_WIND);
        println!("Wind: {:?}", self.config.wind);
    }

    pub fn decrease_wind_y(&mut self) {
        self.config.wind.y = f32::max(self.config.wind.y - Config::WIND_STEP, Config::MIN_WIND);
        println!("Wind: {:?}", self.config.wind);
    }

    pub fn increase_wind_z(&mut self) {
        self.config.wind.z = f32::min(self.config.wind.z + Config::WIND_STEP, Config::MAX_WIND);
        println!("Wind: {:?}", self.config.wind);
    }

    pub fn decrease_wind_z(&mut self) {
        self.config.wind.z = f32::max(self.config.wind.z - Config::WIND_STEP, Config::MIN_WIND);
        println!("Wind: {:?}", self.config.wind);
    }

    pub fn increase_gravity(&mut self) {
        self.config.acceleration_gravity = f32::min(
            self.config.acceleration_gravity + Config::ACCELERATION_GRAVITY_STEP,
            Config::ACCELERATION_GRAVITY_MAX,
        );
        println!("Gravity: {}", self.config.acceleration_gravity);
    }

    pub fn decrease_gravity(&mut self) {
        self.config.acceleration_gravity = f32::max(
            self.config.acceleration_gravity - Config::ACCELERATION_GRAVITY_STEP,
            Config::ACCELERATION_GRAVITY_MIN,
        );
        println!("Gravity: {}", self.config.acceleration_gravity);
    }

    pub fn increase_coefficient_of_restitution(&mut self) {
        self.config.coefficient_of_restitution = f32::min(
            self.config.coefficient_of_restitution + Config::COEFFICIENT_OF_RESTITUTION_STEP,
            Config::COEFFICIENT_OF_RESTITUTION_MAX,
        );
        println!(
            "Coefficient of restitution: {}",
            self.config.coefficient_of_restitution
        );
    }

    pub fn decrease_coefficient_of_restitution(&mut self) {
        self.config.coefficient_of_restitution = f32::max(
            self.config.coefficient_of_restitution - Config::COEFFICIENT_OF_RESTITUTION_STEP,
            Config::COEFFICIENT_OF_RESTITUTION_MIN,
        );
        println!(
            "Coefficient of restitution: {}",
            self.config.coefficient_of_restitution
        );
    }

    pub fn increase_coefficient_of_friction(&mut self) {
        self.config.coefficient_of_friction = f32::min(
            self.config.coefficient_of_friction + Config::COEFFICIENT_OF_FRICTION_STEP,
            Config::COEFFICIENT_OF_FRICTION_MAX,
        );
        println!(
            "Coefficient of friciton: {}",
            self.config.coefficient_of_friction
        );
    }

    pub fn decrease_coefficient_of_friciton(&mut self) {
        self.config.coefficient_of_friction = f32::max(
            self.config.coefficient_of_friction - Config::COEFFICIENT_OF_FRICTION_STEP,
            Config::COEFFICIENT_OF_FRICTION_MIN,
        );
        println!(
            "Coefficient of friciton: {}",
            self.config.coefficient_of_friction
        );
    }

    pub fn increase_static_coefficient_of_friction(&mut self) {
        self.config.static_coefficient_of_friction = f32::min(
            self.config.static_coefficient_of_friction
                + Config::STATIC_COEFFICIENT_OF_FRICTION_STEP,
            Config::STATIC_COEFFICIENT_OF_FRICTION_MAX,
        );
        println!(
            "Coefficient of static friction: {}",
            self.config.static_coefficient_of_friction
        );
    }

    pub fn decrease_static_coefficient_of_friciton(&mut self) {
        self.config.static_coefficient_of_friction = f32::max(
            self.config.static_coefficient_of_friction
                - Config::STATIC_COEFFICIENT_OF_FRICTION_STEP,
            Config::STATIC_COEFFICIENT_OF_FRICTION_MIN,
        );
        println!(
            "Coefficient of static friction: {}",
            self.config.static_coefficient_of_friction
        );
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
            y: self.config.acceleration_gravity,
            z: 0.0,
        };

        // Force due to air resistance is equal to the drag times the square of the velocity,
        // in the direction opposite the velocity.
        // By F = ma, the acceleration due to air resistance is thus that value, divided by the mass of the sphere.
        let acceleration_air_resistance =
            -1.0 * self.config.drag * self.velocity * self.velocity.magnitude()
                / self.config.sphere_mass;

        let acceleration_wind = self.config.drag * self.config.wind * self.config.wind.magnitude()
            / self.config.sphere_mass;

        let acceleration = acceleration_air_resistance + acceleration_gravity + acceleration_wind;

        if self.is_resting(acceleration) {
            return dt;
        }

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
            old_distance_to_plane.is_sign_positive() != new_distance_to_plane.is_sign_positive()
        });

        let time_elapsed;
        (self.position, self.velocity, time_elapsed) = match collided_plane_maybe {
            Some(plane) => {
                // Take the min to avoid a negative fraction
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
                // precision errors that would occur if we were directly on the plane - such as clipping through it.
                let new_position = collision_point + plane.normal * EPSILON;

                let velocity_collision_normal = velocity_collision.dot(plane.normal) * plane.normal;
                let velocity_collision_tangent = velocity_collision - velocity_collision_normal;

                let velocity_response_normal =
                    -1.0 * velocity_collision_normal * self.config.coefficient_of_restitution;
                let velocity_response_tangent = if velocity_collision_tangent.is_zero() {
                    velocity_collision_tangent
                } else {
                    velocity_collision_tangent
                        - velocity_collision_tangent.normalize()
                            * f32::min(
                                self.config.coefficient_of_friction
                                    * velocity_collision_normal.magnitude(),
                                velocity_collision_tangent.magnitude(),
                            )
                };

                let velocity_response = velocity_response_normal + velocity_response_tangent;

                (
                    new_position,
                    velocity_response,
                    std::time::Duration::from_secs_f32(dt.as_secs_f32() * fraction_timestep),
                )
            }
            None => (new_position, new_velocity, dt),
        };

        // Cheat a little bit to ensure we stay in the bounds of the box.
        // Floating point precision could otherwise cause us to clip through the bounds
        // in some edge cases - fixing that would be a great improvement.
        self.position.x = self.position.x.clamp(-0.9999, 0.9999);
        self.position.y = self.position.y.clamp(-0.9999, 0.9999);
        self.position.z = self.position.z.clamp(-0.9999, 0.9999);

        time_elapsed
    }

    fn is_resting(&self, acceleration: cgmath::Vector3<f32>) -> bool {
        let epsilon_velocity = 0.01;
        // If the velocity is non-zero (above an allowable tolerance), we're not at rest
        if self.velocity.magnitude() > epsilon_velocity {
            return false;
        }

        let distance_epsilon = 0.02;
        let contact_walls = self
            .planes
            .iter()
            .filter(|&plane| -> bool { plane.distance_to(self.position) < distance_epsilon })
            .collect::<Vec<_>>();

        // If we're not touching a wall, we aren't at rest (we assume we're not in a zero-G environment)
        if contact_walls.is_empty() {
            return false;
        }

        // See if we're accelerating towards any of our surfaces.
        let acceleration_epsilon = 0.00001;
        let walls_being_accelerated_into = contact_walls
            .iter()
            .filter(|&&plane| -> bool { acceleration.dot(plane.normal) < acceleration_epsilon })
            .collect::<Vec<_>>();

        // If the acceleration isn't towards any of our surfaces, then we're not at rest.
        // We may be in contact with a wall, for example, but accelerating straight down, or we may be touching a ceiling.
        if walls_being_accelerated_into.is_empty() {
            return false;
        }

        // To be at rest, the friction of some surface must be enough to stop
        // the potential motion for cases where the component of the acceleration tangent
        // to the surface is non-zero.
        let any_wall_friction_overcomes_acceleration =
            walls_being_accelerated_into.iter().any(|&&plane| -> bool {
                let acceleration_normal_magnitude = plane.normal.dot(acceleration);
                let acceleration_tangent_magnitude =
                    (acceleration - plane.normal * acceleration_normal_magnitude).magnitude();
                // If the acceleration is too small to overcome static friction, this wall
                // is "grippy" enough to prevent the object from sliding.
                acceleration_tangent_magnitude.is_nan()
                    || acceleration_tangent_magnitude.is_zero()
                    || acceleration_tangent_magnitude
                        < self.config.static_coefficient_of_friction * acceleration_normal_magnitude
            });

        // If any wall's static friction overcomes the other forces' acceleration, we're at rest!
        any_wall_friction_overcomes_acceleration
    }
}
