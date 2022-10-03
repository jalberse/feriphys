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
        });
    }
}

impl FlockingUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    pub fn new() -> FlockingUi {
        FlockingUi {
            sim_config: flocking::Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &flocking::Config {
        &self.sim_config
    }
}
