use crate::gui::Ui;
use crate::simulation::particles;
use egui::Slider;

pub struct ParticlesUi {
    sim_config: particles::Config,
}

impl Ui for ParticlesUi {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Config").show(&ctx, |ui| {
            ui.add(
                Slider::new(
                    &mut self.sim_config.dt,
                    ParticlesUi::SIMULATION_DT_MIN.as_secs_f32()
                        ..=ParticlesUi::SIMULATION_DT_MAX.as_secs_f32(),
                )
                .text("Simualtion dt (secs)"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.acceleration_gravity.x,
                    ParticlesUi::ACCELERATION_GRAVITY_MIN..=ParticlesUi::ACCELERATION_GRAVITY_MAX,
                )
                .text("Gravity X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.acceleration_gravity.y,
                    ParticlesUi::ACCELERATION_GRAVITY_MIN..=ParticlesUi::ACCELERATION_GRAVITY_MAX,
                )
                .text("Gravity Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.acceleration_gravity.z,
                    ParticlesUi::ACCELERATION_GRAVITY_MIN..=ParticlesUi::ACCELERATION_GRAVITY_MAX,
                )
                .text("Gravity Z"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.wind.x,
                    ParticlesUi::MIN_WIND..=ParticlesUi::MAX_WIND,
                )
                .text("Wind X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.wind.y,
                    ParticlesUi::MIN_WIND..=ParticlesUi::MAX_WIND,
                )
                .text("Wind Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.wind.z,
                    ParticlesUi::MIN_WIND..=ParticlesUi::MAX_WIND,
                )
                .text("Wind Z"),
            );
        });
    }
}

impl ParticlesUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const ACCELERATION_GRAVITY_MIN: f32 = -20.0;
    const ACCELERATION_GRAVITY_MAX: f32 = 20.0;

    // TODO we'll apply these as a min/max range for particle generation
    // This range would actually be the bounds for a min/max we set ourselves... Confusing lol
    // const MIN_DRAG: f32 = 0.05;
    // const MAX_DRAG: f32 = 2.0;

    const MIN_WIND: f32 = -5.0;
    const MAX_WIND: f32 = 5.0;

    pub fn new() -> ParticlesUi {
        ParticlesUi {
            sim_config: particles::Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &particles::Config {
        &self.sim_config
    }
}