use std::time::Duration;

use crate::gui;
use crate::simulation::state::Integration;

use super::super::collidable_mesh::CollidableMesh;
use super::super::state::State;
use super::{config::Config, springy_mesh::SpringyMesh};

pub struct Simulation {
    config: Config,
    // Deformable springy meshes
    meshes: Vec<SpringyMesh>,
    obstacles: Vec<CollidableMesh>,
}

impl Simulation {
    pub fn new(meshes: Vec<SpringyMesh>, obstacles: Vec<CollidableMesh>) -> Simulation {
        let config = Config::default();
        Simulation {
            config,
            meshes,
            obstacles,
        }
    }

    pub fn step(&mut self) -> Duration {
        self.meshes.iter_mut().for_each(|mesh| {
            mesh.accumulate_forces(&self.config);

            let points = mesh.get_points();
            let state_vector = State::new(points.to_vec());
            let new_state_vector = match self.config.integration {
                Integration::Rk4 => state_vector.rk4_step(self.config.dt),
                Integration::Euler => state_vector.euler_step(self.config.dt),
            };
            let new_points = new_state_vector.get_elements();

            mesh.update_points(new_points, &self.obstacles, &self.config);

            mesh.clear_forces();
        });

        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_meshes(&self) -> &Vec<SpringyMesh> {
        &self.meshes
    }

    pub fn get_obstacles(&self) -> &Vec<CollidableMesh> {
        &self.obstacles
    }

    // TODO consider extending this to allow for updating the springy mesh properties, i.e. changing nominal spring constant and damping, and the total mass of
    //      the springy mesh.
    //      Would need to be careful with updating strut values, since
    //      they aren't uniform (add_strut allows us to have varying values,
    //      which is important for cloth simulation)
    pub fn sync_sim_config_from_ui(
        &mut self,
        ui: &mut gui::spring_mass_damper::SpringMassDamperUi,
    ) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.integration = ui_config_state.integration;
        self.config.dt = ui_config_state.dt;
        self.config.gravity = ui_config_state.gravity;
        self.config.wind = ui_config_state.wind;
        self.config.lift_coefficient = ui_config_state.lift_coefficient;
        self.config.drag_coefficient = ui_config_state.drag_coefficient;
        self.config.coefficient_of_restitution = ui_config_state.coefficient_of_restitution;
        self.config.coefficient_of_friction = ui_config_state.coefficient_of_friction;
    }
}
