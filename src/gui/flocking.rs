use crate::gui::Ui;
use crate::simulation::flocking::flocking;
use egui::{Checkbox, Slider};

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
                    FlockingUi::AVOIDANCE_FACTOR_MIN..=FlockingUi::AVOIDANCE_FACTOR_MAX,
                )
                .text("Avoidance Factor"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.centering_factor,
                    FlockingUi::CENTERING_FACTOR_MIN..=FlockingUi::CENTERING_FACTOR_MAX,
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
            ui.add(
                Slider::new(
                    &mut self.sim_config.distance_weight_threshold,
                    FlockingUi::DISTANCE_WEIGHT_THRESHOLD_MIN
                        ..=FlockingUi::DISTANCE_WEIGHT_THRESHOLD_MAX,
                )
                .text("Distance Weight Threshold"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.distance_weight_threshold_falloff,
                    FlockingUi::DISTANCE_WEIGHT_THRESHOLD_FALLOFF_MIN
                        ..=FlockingUi::DISTANCE_WEIGHT_THRESHOLD_FALLOFF_MAX,
                )
                .text("Distance Weight Threshold Falloff"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.max_sight_angle,
                    FlockingUi::MAX_SIGHT_ANGLE_MIN..=FlockingUi::MAX_SIGHT_ANGLE_MAX,
                )
                .text("Max Sight Angle"),
            );
            ui.add(Checkbox::new(
                &mut self.sim_config.steering_overrides,
                "Steering Overrides",
            ));
        });
    }
}

impl FlockingUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const AVOIDANCE_FACTOR_MIN: f32 = 0.0;
    const AVOIDANCE_FACTOR_MAX: f32 = 10.0;

    const CENTERING_FACTOR_MIN: f32 = 0.0;
    const CENTERING_FACTOR_MAX: f32 = 2.0;

    const VELOCITY_MATCHING_FACTOR_MIN: f32 = 0.0;
    const VELOCITY_MATHCING_FACTOR_MAX: f32 = 10.0;

    const DISTANCE_WEIGHT_THRESHOLD_MIN: f32 = 0.0;
    const DISTANCE_WEIGHT_THRESHOLD_MAX: f32 = 10.0;

    const DISTANCE_WEIGHT_THRESHOLD_FALLOFF_MIN: f32 = 0.0;
    const DISTANCE_WEIGHT_THRESHOLD_FALLOFF_MAX: f32 = 10.0;

    const MAX_SIGHT_ANGLE_MIN: f32 = 0.0;
    const MAX_SIGHT_ANGLE_MAX: f32 = std::f32::consts::PI;

    pub fn new() -> FlockingUi {
        FlockingUi {
            sim_config: flocking::Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &flocking::Config {
        &self.sim_config
    }
}
