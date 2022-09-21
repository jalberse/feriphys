use crate::gui::Ui;
use crate::simulation::bounce;

pub struct BouncingBallUi {
    sim_config: bounce::Config,
}

impl Ui for BouncingBallUi {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Window").show(&ctx, |ui| {
            ui.label("Hello world!");
            ui.label("See https://github.com/emilk/egui for how to make other UI elements");
        });
    }
}

impl BouncingBallUi {
    pub fn new() -> BouncingBallUi {
        BouncingBallUi {
            sim_config: bounce::Config::default(),
        }
    }
}
