use crate::gui::Ui;
use crate::simulation::flocking;
use egui::Slider;

pub struct FlockingUi {
    sim_config: flocking::Config,
}

impl Ui for FlockingUi {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Config").show(&ctx, |ui| {
            ui.add(
                Slider::new(
                    &mut self.sim_config.dt,
                    FlockingUi::SIMULATION_DT_MIN.as_secs_f32()
                        ..=FlockingUi::SIMULATION_DT_MAX.as_secs_f32(),
                )
                .text("Simualtion dt (secs)"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.avoidance_factor,
                    FlockingUi::AVOIDANCE_FACTOR_MIN
                        ..=FlockingUi::AVOIDANCE_FACTOR_MAX,
                )
                .text("Avoidance Factor"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.centering_factor,
                    FlockingUi::CENTERING_FACTOR_MIN
                        ..=FlockingUi::CENTERING_FACTOR_MAX,
                )
                .text("Centering Factor"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.velocity_matching_factor,
                    FlockingUi::VELOCITY_MATCHING_FACTOR_MIN
                        ..=FlockingUi::VELOCITY_MATHCING_FACTOR_MAX,
                )
                .text("Velocity Matching Factor"),
            );
        });
    }
}

impl FlockingUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const AVOIDANCE_FACTOR_MIN: f32 = 0.0;
    const AVOIDANCE_FACTOR_MAX: f32 = 10.0;

    const CENTERING_FACTOR_MIN: f32 = 0.0;
    const CENTERING_FACTOR_MAX: f32 = 10.0;

    const VELOCITY_MATCHING_FACTOR_MIN: f32 = 0.0;
    const VELOCITY_MATHCING_FACTOR_MAX: f32 = 10.0;

    pub fn new() -> FlockingUi {
        FlockingUi {
            sim_config: flocking::Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &flocking::Config {
        &self.sim_config
    }
}
