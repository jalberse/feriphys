use crate::gui::Ui;
use crate::simulation::sph::config::Config;
use crate::simulation::state::Integration;

use egui::Slider;

pub struct SphUi {
    sim_config: Config,
}

impl Ui for SphUi {
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
                    SphUi::SIMULATION_DT_MIN.as_secs_f32()..=SphUi::SIMULATION_DT_MAX.as_secs_f32(),
                )
                .text("Simualtion dt (secs)"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.x,
                    SphUi::GRAVITY_MIN..=SphUi::GRAVITY_MAX,
                )
                .text("Gravity X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.y,
                    SphUi::GRAVITY_MIN..=SphUi::GRAVITY_MAX,
                )
                .text("Gravity Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.gravity.z,
                    SphUi::GRAVITY_MIN..=SphUi::GRAVITY_MAX,
                )
                .text("Gravity Z"),
            );
        });
    }
}

impl SphUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const GRAVITY_MIN: f32 = -2.0;
    const GRAVITY_MAX: f32 = 2.0;

    pub fn new() -> SphUi {
        SphUi {
            sim_config: Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &Config {
        &self.sim_config
    }
}
