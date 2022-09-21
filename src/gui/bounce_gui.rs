use crate::gui::Ui;
use crate::simulation::bounce;
use egui::Slider;

pub struct BouncingBallUi {
    sim_config: bounce::Config,
}

impl Ui for BouncingBallUi {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Config").show(&ctx, |ui| {
            ui.add(
                Slider::new(
                    &mut self.sim_config.dt,
                    BouncingBallUi::SIMULATION_DT_MIN.as_secs_f32()
                        ..=BouncingBallUi::SIMULATION_DT_MAX.as_secs_f32(),
                )
                .text("Simualtion dt (secs)"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.acceleration_gravity,
                    BouncingBallUi::ACCELERATION_GRAVITY_MIN
                        ..=BouncingBallUi::ACCELERATION_GRAVITY_MAX,
                )
                .text("Gravity"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.sphere_mass,
                    BouncingBallUi::MIN_SPHERE_MASS..=BouncingBallUi::MAX_SPHERE_MASS,
                )
                .text("Sphere Mass"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.drag,
                    BouncingBallUi::MIN_DRAG..=BouncingBallUi::MAX_DRAG,
                )
                .text("Drag"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.wind.x,
                    BouncingBallUi::MIN_WIND..=BouncingBallUi::MAX_WIND,
                )
                .text("Wind X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.wind.y,
                    BouncingBallUi::MIN_WIND..=BouncingBallUi::MAX_WIND,
                )
                .text("Wind Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.wind.z,
                    BouncingBallUi::MIN_WIND..=BouncingBallUi::MAX_WIND,
                )
                .text("Wind Z"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_restitution,
                    BouncingBallUi::COEFFICIENT_OF_RESTITUTION_MIN
                        ..=BouncingBallUi::COEFFICIENT_OF_RESTITUTION_MAX,
                )
                .text("Coefficient of Restitution"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_friction,
                    BouncingBallUi::COEFFICIENT_OF_FRICTION_MIN
                        ..=BouncingBallUi::COEFFICIENT_OF_FRICTION_MAX,
                )
                .text("Coefficient of Friction"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.static_coefficient_of_friction,
                    BouncingBallUi::STATIC_COEFFICIENT_OF_FRICTION_MIN
                        ..=BouncingBallUi::STATIC_COEFFICIENT_OF_FRICTION_MAX,
                )
                .text("Static Coefficient of Friction"),
            );
        });
    }
}

impl BouncingBallUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const ACCELERATION_GRAVITY_MIN: f32 = -20.0;
    const ACCELERATION_GRAVITY_MAX: f32 = 20.0;

    const MIN_SPHERE_MASS: f32 = 0.05;
    const MAX_SPHERE_MASS: f32 = 10.0;

    const MIN_DRAG: f32 = 0.05;
    const MAX_DRAG: f32 = 2.0;

    const MIN_WIND: f32 = -5.0;
    const MAX_WIND: f32 = 5.0;

    const COEFFICIENT_OF_RESTITUTION_MIN: f32 = 0.0;
    const COEFFICIENT_OF_RESTITUTION_MAX: f32 = 1.0;

    const COEFFICIENT_OF_FRICTION_MIN: f32 = 0.05;
    const COEFFICIENT_OF_FRICTION_MAX: f32 = 1.0;

    const STATIC_COEFFICIENT_OF_FRICTION_MIN: f32 = 0.05;
    const STATIC_COEFFICIENT_OF_FRICTION_MAX: f32 = 1.0;

    pub fn new() -> BouncingBallUi {
        BouncingBallUi {
            sim_config: bounce::Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &bounce::Config {
        &self.sim_config
    }
}
