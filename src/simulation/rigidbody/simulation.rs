use std::time::Duration;

use crate::simulation::{
    collidable_mesh::CollidableMesh,
    state::{Integration, State},
};

use super::{config::Config, rigidbody::RigidBody};

pub struct Simulation {
    config: Config,
    rigidbodies: Vec<RigidBody>,
    obstacles: Vec<CollidableMesh>,
}

impl Simulation {
    pub fn new(rigidbodies: Vec<RigidBody>, obstacles: Vec<CollidableMesh>) -> Simulation {
        let config = Config::default();
        Simulation {
            config,
            rigidbodies,
            obstacles,
        }
    }

    pub fn step(&mut self) -> Duration {
        self.rigidbodies.iter_mut().for_each(|rigidbody| {
            rigidbody.accumulate_forces(&self.config);
            rigidbody.accumulate_torques(&self.config);

            let state = State::new(vec![*rigidbody.get_state()]);
            let new_state = match self.config.integration {
                Integration::Rk4 => state.rk4_step(self.config.dt),
                Integration::Euler => state.euler_step(self.config.dt),
            };
            let mut new_rigidbody_state = new_state.get_elements()[0];
            new_rigidbody_state.normalize_rotation();

            rigidbody.update_state(new_rigidbody_state, &self.obstacles, &self.config);

            // TODO The collision response should also handle other rigidbodies, which would require examining and updating all rigidbodies at once,
            //        rather than sequentially as here. Really, we should have all rigidbodies in a single State vector, and handle derivative calculation etc from
            //        that, rather than statefully determining accumulated forces and torques.
            //        Beware that the CollidableMesh in the rigidbodies is stored as local coordinates, so we'd need to transform into world coordinates
            //        for comparison/collisions.

            rigidbody.clear_forces();
            rigidbody.clear_torques();
        });

        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn get_rigidbodies(&self) -> &Vec<RigidBody> {
        &self.rigidbodies
    }

    pub fn get_obstacles(&self) -> &Vec<CollidableMesh> {
        &self.obstacles
    }

    pub fn sync_sim_from_ui(&mut self, ui: &mut crate::gui::rigidbody::RigidBodyUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.integration = ui_config_state.integration;
        self.config.dt = ui_config_state.dt;
        self.config.coefficient_of_restitution = ui_config_state.coefficient_of_restitution;
        self.config.gravity = ui_config_state.gravity;
        self.config.torque = ui_config_state.torque;

        if let Some((impulse, impulse_position)) = ui.get_free_impulse() {
            self.rigidbodies[0].apply_impulse(impulse, impulse_position);
        }
    }
}
