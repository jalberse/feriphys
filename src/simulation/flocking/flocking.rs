use super::{
    boid::{Boid, FlockingBoid, LeadBoid},
    obstacle::Obstacle,
};
use crate::{
    graphics::instance::Instance,
    gui,
    simulation::{bounding_box::BoundingBox, point_attractor::PointAttractor},
};

use cgmath::{InnerSpace, Vector3, Zero};

use std::time::Duration;

pub struct Config {
    pub dt: f32, // secs as f32
    pub avoidance_factor: f32,
    pub centering_factor: f32,
    pub velocity_matching_factor: f32,
    /// The maximum distance for which the weight for two boid's interaction is 1.0
    pub distance_weight_threshold: f32,
    /// The distance past the distance_threshold over which the weight interpolates to 0.0.
    pub distance_weight_threshold_falloff: f32,
    /// The maximal angle in radians (0 to pi) for which boids can "see" other boids,
    /// where pi means boids can see all other boids, including those directly behind
    /// them. The forward direction is in the direction of the boid's velocity.
    pub max_sight_angle: f32,
    pub max_sight_angle_to_lead_boid: f32,
    pub time_to_start_steering: Duration,
    /// If true, then when a boid is steering to avoid an obstacle, it will ignore
    /// other sources of acceleration. This can help prevent cases where
    /// a boid will clip through obstacles, but can cause unnatural motion.
    pub steering_overrides: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
            avoidance_factor: 1.0,
            centering_factor: 0.1,
            velocity_matching_factor: 0.5,
            distance_weight_threshold: 15.0,
            distance_weight_threshold_falloff: 1.0,
            max_sight_angle: std::f32::consts::PI / 2.0,
            max_sight_angle_to_lead_boid: std::f32::consts::PI,
            time_to_start_steering: Duration::from_secs(4),
            steering_overrides: false,
        }
    }
}

pub struct Simulation {
    config: Config,
    boids: Vec<FlockingBoid>,
    lead_boids: Option<Vec<LeadBoid>>,
    bounding_box: Option<BoundingBox>,
    obstacles: Option<Vec<Obstacle>>,
    attractors: Option<Vec<PointAttractor>>,
}

impl Simulation {
    pub fn new(
        initial_positions: Vec<Vector3<f32>>,
        num_boids: u32,
        bounding_box: Option<BoundingBox>,
        lead_boids: Option<Vec<LeadBoid>>,
        obstacles: Option<Vec<Obstacle>>,
        attractors: Option<Vec<PointAttractor>>,
    ) -> Simulation {
        let config = Config::default();

        let mut boids = Vec::with_capacity(num_boids as usize);
        // TODO if initial_positions is empty, this crashes. Fix that.
        for position in &initial_positions {
            for _ in 0..num_boids / initial_positions.len() as u32 {
                let position = Vector3::<f32> {
                    x: position.x + rand::random::<f32>(),
                    y: position.y + rand::random::<f32>(),
                    z: position.z + rand::random::<f32>(),
                };
                let velocity = Vector3::<f32> {
                    x: rand::random(),
                    y: rand::random(),
                    z: rand::random(),
                };
                boids.push(FlockingBoid::new(position, velocity));
            }
        }

        Simulation {
            config,
            boids,
            lead_boids,
            bounding_box,
            obstacles,
            attractors,
        }
    }

    pub fn step(&mut self) -> Duration {
        // TODO we could use a double buffer here instead of allocating a new vector here every step.
        let mut new_state = Vec::with_capacity(self.boids.len());

        for boid in self.boids.iter() {
            let boid_acceleration = if self.config.steering_overrides {
                self.get_acceleration_from_steering(boid)
            } else {
                self.get_acceleration_from_boids(boid)
                    + self.get_acceleration_from_lead_boids(boid)
                    + self.get_acceleration_from_attractors(boid)
                    + if let Some(bounding_box) = &self.bounding_box {
                        bounding_box.get_repelling_acceleration(boid.position())
                    } else {
                        Vector3::<f32>::zero()
                    }
                    + self.get_acceleration_from_steering(boid)
            };

            let new_boid_position = boid.position() + self.config.dt * boid.velocity();
            let new_boid_velocity = boid.velocity() + self.config.dt * boid_acceleration;

            new_state.push(FlockingBoid::new(new_boid_position, new_boid_velocity));
        }

        self.boids = new_state;

        if let Some(lead_boids) = &mut self.lead_boids {
            for lead_boid in lead_boids.iter_mut() {
                lead_boid.step(Duration::from_secs_f32(self.config.dt));
            }
        }

        self.get_timestep()
    }

    fn get_acceleration_from_boids(&self, boid: &FlockingBoid) -> Vector3<f32> {
        // TODO use a functional approach
        let mut total_acceleration = Vector3::<f32>::zero();
        for other_boid in self.boids.iter() {
            if other_boid == boid {
                continue;
            }
            total_acceleration += boid.get_acceleration(
                other_boid,
                self.config.avoidance_factor,
                self.config.centering_factor,
                self.config.velocity_matching_factor,
                self.config.distance_weight_threshold,
                self.config.distance_weight_threshold_falloff,
                self.config.max_sight_angle,
            );
        }
        total_acceleration
    }

    fn get_acceleration_from_lead_boids(&self, boid: &FlockingBoid) -> Vector3<f32> {
        // TODO use functional approach
        let mut total_accel = Vector3::<f32>::zero();
        if let Some(lead_boids) = &self.lead_boids {
            for lead_boid in lead_boids.iter() {
                total_accel += boid.get_acceleration(
                    lead_boid,
                    self.config.avoidance_factor,
                    self.config.centering_factor,
                    self.config.velocity_matching_factor,
                    self.config.distance_weight_threshold,
                    self.config.distance_weight_threshold_falloff,
                    self.config.max_sight_angle_to_lead_boid,
                )
            }
        }
        total_accel
    }

    fn get_acceleration_from_attractors(&self, boid: &FlockingBoid) -> Vector3<f32> {
        let mut total_accel = Vector3::<f32>::zero();
        if let Some(point_attractors) = &self.attractors {
            for attractor in point_attractors.iter() {
                total_accel += attractor.get_acceleration(boid.position(), boid.mass());
            }
        }
        total_accel
    }

    fn get_acceleration_from_steering(&self, boid: &FlockingBoid) -> Vector3<f32> {
        if let Some(obstacles) = &self.obstacles {
            // Find the first obstacle we might hit, which is the one we'll steer to avoid.
            let closest_obstacle_maybe = obstacles.iter().min_by(|x, y| {
                let x_time = match x.get_time_to_plane_collision(boid) {
                    Some(duration) => duration,
                    None => Duration::MAX,
                };
                let y_time = match y.get_time_to_plane_collision(boid) {
                    Some(duration) => duration,
                    None => Duration::MAX,
                };
                x_time.cmp(&y_time)
            });
            if let Some(closest_obstacle) = closest_obstacle_maybe {
                // The list of obstacles wasn't empty
                if let Some(time_to_plane_collision) =
                    closest_obstacle.get_time_to_plane_collision(boid)
                {
                    // There's at least one obstacle the boid may eventually hit
                    if time_to_plane_collision < self.config.time_to_start_steering {
                        return closest_obstacle.get_acceleration_to_avoid(boid);
                    }
                }
            }
        }
        Vector3::<f32>::zero()
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn sync_sim_config_from_ui(&mut self, ui: &mut gui::flocking::FlockingUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.dt = ui_config_state.dt;
        self.config.avoidance_factor = ui_config_state.avoidance_factor;
        self.config.centering_factor = ui_config_state.centering_factor;
        self.config.velocity_matching_factor = ui_config_state.velocity_matching_factor;
        self.config.distance_weight_threshold = ui_config_state.distance_weight_threshold;
        self.config.distance_weight_threshold_falloff =
            ui_config_state.distance_weight_threshold_falloff;
        self.config.max_sight_angle = ui_config_state.max_sight_angle;
        self.config.max_sight_angle_to_lead_boid = ui_config_state.max_sight_angle_to_lead_boid;
        self.config.time_to_start_steering = ui_config_state.time_to_start_steering;
        self.config.steering_overrides = ui_config_state.steering_overrides;
    }

    pub fn get_boid_instances(&self) -> Vec<Instance> {
        let mut instances = Vec::<Instance>::with_capacity(self.boids.len());

        for boid in self.boids.iter() {
            instances.push(Instance {
                position: boid.position(),
                rotation: cgmath::Quaternion::from_arc(
                    cgmath::Vector3::unit_z(),
                    boid.velocity().normalize(),
                    None,
                ),
                scale: 0.1,
            });
        }
        instances
    }
}
