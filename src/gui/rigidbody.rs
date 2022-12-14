use crate::gui::Ui;
use crate::simulation::rigidbody::config::Config;
use crate::simulation::state::Integration;

use cgmath::{Vector3, Zero};
use egui::Slider;

pub struct RigidBodyUi {
    sim_config: Config,
    impulse: Vector3<f32>,
    impulse_position: Vector3<f32>,
    free_impulse: bool,
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
            ui.add(
                Slider::new(
                    &mut self.sim_config.coefficient_of_restitution,
                    RigidBodyUi::COEFFICIENT_OF_RESTITUTION_MIN
                        ..=RigidBodyUi::COEFFICIENT_OF_RESTITUTION_MAX,
                )
                .text("Coefficient of Restitution"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.torque.x,
                    RigidBodyUi::TORQUE_MIN..=RigidBodyUi::TORQUE_MAX,
                )
                .text("Torque X"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.torque.y,
                    RigidBodyUi::TORQUE_MIN..=RigidBodyUi::TORQUE_MAX,
                )
                .text("Torque Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.sim_config.torque.z,
                    RigidBodyUi::TORQUE_MIN..=RigidBodyUi::TORQUE_MAX,
                )
                .text("Torque Z"),
            );
            ui.separator();
            ui.add(
                Slider::new(
                    &mut self.impulse.x,
                    RigidBodyUi::IMPULSE_MIN..=RigidBodyUi::IMPULSE_MAX,
                )
                .text("Impulse X"),
            );
            ui.add(
                Slider::new(
                    &mut self.impulse.y,
                    RigidBodyUi::IMPULSE_MIN..=RigidBodyUi::IMPULSE_MAX,
                )
                .text("Impulse Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.impulse.z,
                    RigidBodyUi::IMPULSE_MIN..=RigidBodyUi::IMPULSE_MAX,
                )
                .text("Impulse Z"),
            );
            ui.add(
                Slider::new(
                    &mut self.impulse_position.x,
                    RigidBodyUi::IMPULSE_POSITION_MIN..=RigidBodyUi::IMPULSE_POSITION_MAX,
                )
                .text("Impulse Position X"),
            );
            ui.add(
                Slider::new(
                    &mut self.impulse_position.y,
                    RigidBodyUi::IMPULSE_POSITION_MIN..=RigidBodyUi::IMPULSE_POSITION_MAX,
                )
                .text("Impulse Position Y"),
            );
            ui.add(
                Slider::new(
                    &mut self.impulse_position.z,
                    RigidBodyUi::IMPULSE_POSITION_MIN..=RigidBodyUi::IMPULSE_POSITION_MAX,
                )
                .text("Impulse Position Z"),
            );
            self.free_impulse = ui.button("Free Impulse").clicked();
            ui.separator();
        });
    }
}

impl RigidBodyUi {
    const SIMULATION_DT_MAX: std::time::Duration = std::time::Duration::from_millis(10);
    const SIMULATION_DT_MIN: std::time::Duration = std::time::Duration::from_micros(100);

    const GRAVITY_MIN: f32 = -2.0;
    const GRAVITY_MAX: f32 = 2.0;

    const COEFFICIENT_OF_RESTITUTION_MIN: f32 = 0.0;
    const COEFFICIENT_OF_RESTITUTION_MAX: f32 = 1.0;

    const TORQUE_MIN: f32 = -1.0;
    const TORQUE_MAX: f32 = 1.0;

    const IMPULSE_MIN: f32 = -1.0;
    const IMPULSE_MAX: f32 = 1.0;

    const IMPULSE_POSITION_MIN: f32 = -0.5;
    const IMPULSE_POSITION_MAX: f32 = 0.5;

    pub fn new() -> RigidBodyUi {
        RigidBodyUi {
            sim_config: Config::default(),
            impulse: Vector3::zero(),
            impulse_position: Vector3::zero(),
            free_impulse: false,
        }
    }

    pub fn get_gui_state_mut(&mut self) -> &Config {
        &self.sim_config
    }

    /// Returns None if we should not impart a free impulse this frame.
    /// Returns Some pair of vectors for the impulse and impulse position if the user
    /// has clicked to impart a free impulse
    pub fn get_free_impulse(&self) -> Option<(Vector3<f32>, Vector3<f32>)> {
        if self.free_impulse {
            Some((self.impulse, self.impulse_position))
        } else {
            None
        }
    }
}
