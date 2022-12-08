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
                    &mut self.sim_config.particle_mass,
                    SphUi::PARTICLE_MASS_MIN..=SphUi::PARTICLE_MASS_MAX,
                )
                .text("Particle Mass"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.kernal_max_distance,
                    SphUi::KERNAL_MAX_DIST_MIN..=SphUi::KERNAL_MAX_DIST_MAX,
                )
                .text("Kernal Max Dist"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.pressure_siffness,
                    SphUi::PRESSURE_STIFFNESS_MIN..=SphUi::PRESSURE_STIFFNESS_MAX,
                )
                .text("Pressure Stiffness"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.reference_density,
                    SphUi::REFERENCE_DENSITY_MIN..=SphUi::REFERENCE_DENSITY_MAX,
                )
                .text("Reference Density"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.kinematic_viscosity,
                    SphUi::KINEMATIC_VISCOSITY_MIN..=SphUi::KINEMATIC_VISCOSITY_MAX,
                )
                .text("Kinematic Viscosity"),
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
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_restitution,
                    SphUi::MIN_COEFFICIENT_OF_RESTITUTION..=SphUi::MAX_COEFFICIENT_OF_RESTITUTION,
                )
                .text("Restitution"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_friction,
                    SphUi::MIN_COEFFICIENT_OF_FRICTION..=SphUi::MAX_COEFFICIENT_OF_FRICTION,
                )
                .text("Friction"),
            );
        });
    }
}

impl SphUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const PARTICLE_MASS_MIN: f32 = 0.001;
    const PARTICLE_MASS_MAX: f32 = 0.1;

    const KERNAL_MAX_DIST_MIN: f32 = 0.001;
    const KERNAL_MAX_DIST_MAX: f32 = 0.15;

    const PRESSURE_STIFFNESS_MIN: f32 = 0.0;
    const PRESSURE_STIFFNESS_MAX: f32 = 1.0;

    const REFERENCE_DENSITY_MIN: f32 = 0.1;
    const REFERENCE_DENSITY_MAX: f32 = 2.0;

    const KINEMATIC_VISCOSITY_MIN: f32 = 0.1;
    const KINEMATIC_VISCOSITY_MAX: f32 = 3.0;

    const GRAVITY_MIN: f32 = -2.0;
    const GRAVITY_MAX: f32 = 2.0;

    const MIN_COEFFICIENT_OF_RESTITUTION: f32 = 0.0;
    const MAX_COEFFICIENT_OF_RESTITUTION: f32 = 1.0;

    const MIN_COEFFICIENT_OF_FRICTION: f32 = 0.0;
    const MAX_COEFFICIENT_OF_FRICTION: f32 = 1.0;

    pub fn new() -> SphUi {
        SphUi {
            sim_config: Config::default(),
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &Config {
        &self.sim_config
    }
}
