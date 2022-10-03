
use std::time::Duration;

use crate::gui;

pub struct Config {
    pub dt: f32, // secs as f32
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dt: Duration::from_millis(1).as_secs_f32(),
        }
    }
}

pub struct Simulation {
    config: Config,
}

impl Simulation {
    pub fn new() -> Simulation {
        let config = Config::default();

        Simulation {
            config,
        }
    }

    pub fn step(&mut self) -> Duration {

        self.get_timestep()
    }

    pub fn get_timestep(&self) -> Duration {
        Duration::from_secs_f32(self.config.dt)
    }

    pub fn sync_sim_config_from_ui(&mut self, ui: &mut gui::flocking::FlockingUi) {
        let ui_config_state = ui.get_gui_state_mut();
        self.config.dt = ui_config_state.dt;
    }
}
