mod args;
mod bouncing_ball_demo;
mod camera;
mod forms;
mod gpu_interface;
mod gui;
mod instance;
mod light;
mod model;
mod particles_demo;
mod rendering;
mod resources;
mod scene;
mod simulation;
mod texture;
mod utilities;

use args::{Demos, FeriphysArgs};
use clap::Parser;

fn main() {
    let args = FeriphysArgs::parse();
    match args.demo {
        Demos::BouncingBall => bouncing_ball_demo::run(),
        Demos::Particles => particles_demo::run(),
    }
}
