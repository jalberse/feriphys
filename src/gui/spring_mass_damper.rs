use crate::gui::Ui;
use crate::simulation::springy::config::Config;
use crate::simulation::state::Integration;

use egui::Slider;

pub struct SpringMassDamperUi {
    sim_config: Config,
}

impl Ui for SpringMassDamperUi {
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
                    SpringMassDamperUi::SIMULATION_DT_MIN.as_secs_f32()
                        ..=SpringMassDamperUi::SIMULATION_DT_MAX.as_secs_f32(),
                )
                .text("Simualtion dt (secs)"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.x,
                    SpringMassDamperUi::GRAVITY_MIN..=SpringMassDamperUi::GRAVITY_MAX,
                )
                .text("Gravity X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.y,
                    SpringMassDamperUi::GRAVITY_MIN..=SpringMassDamperUi::GRAVITY_MAX,
                )
                .text("Gravity Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.z,
                    SpringMassDamperUi::GRAVITY_MIN..=SpringMassDamperUi::GRAVITY_MAX,
                )
                .text("Gravity Z"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_restitution,
                    SpringMassDamperUi::MIN_COEFFICIENT_OF_RESTITUTION
                        ..=SpringMassDamperUi::MAX_COEFFICIENT_OF_RESTITUTION,
                )
                .text("Restitution"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_friction,
                    SpringMassDamperUi::MIN_COEFFICIENT_OF_FRICTION
                        ..=SpringMassDamperUi::MAX_COEFFICIENT_OF_FRICTION,
                )
                .text("Friction"),
            );
        });
    }
}

impl SpringMassDamperUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const GRAVITY_MIN: f32 = -20.0;
    const GRAVITY_MAX: f32 = 20.0;

    const MIN_COEFFICIENT_OF_RESTITUTION: f32 = 0.0;
    const MAX_COEFFICIENT_OF_RESTITUTION: f32 = 1.0;

    const MIN_COEFFICIENT_OF_FRICTION: f32 = 0.0;
    const MAX_COEFFICIENT_OF_FRICTION: f32 = 1.0;

    pub fn new() -> SpringMassDamperUi {
        SpringMassDamperUi {
            sim_config: Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &Config {
        &self.sim_config
    }
}
