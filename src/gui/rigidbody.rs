use crate::gui::Ui;
use crate::simulation::rigidbody::config::Config;
use crate::simulation::state::Integration;

use egui::Slider;

pub struct RigidBodyUi {
    sim_config: Config,
}

impl Ui for RigidBodyUi {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Config").show(&ctx, |ui| {
            egui::ComboBox::from_label("Integration")
                .selected_text(format!("{:?}", self.sim_config.integration))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sim_config.integration, Integration::Rk4, "RK4");
                    ui.selectable_value(
                        &mut self.sim_config.integration,
                        Integration::Euler,
                        "Euler",
                    );
                });
            ui.add(
                Slider::new(
                    &mut self.sim_config.dt,
                    RigidBodyUi::SIMULATION_DT_MIN.as_secs_f32()
                        ..=RigidBodyUi::SIMULATION_DT_MAX.as_secs_f32(),
                )
                .text("Simualtion dt (secs)"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.x,
                    RigidBodyUi::GRAVITY_MIN..=RigidBodyUi::GRAVITY_MAX,
                )
                .text("Gravity X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.y,
                    RigidBodyUi::GRAVITY_MIN..=RigidBodyUi::GRAVITY_MAX,
                )
                .text("Gravity Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.z,
                    RigidBodyUi::GRAVITY_MIN..=RigidBodyUi::GRAVITY_MAX,
                )
                .text("Gravity Z"),
            );
        });
    }
}

impl RigidBodyUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const GRAVITY_MIN: f32 = -20.0;
    const GRAVITY_MAX: f32 = 20.0;

    pub fn new() -> RigidBodyUi {
        RigidBodyUi {
            sim_config: Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &Config {
        &self.sim_config
    }
}
