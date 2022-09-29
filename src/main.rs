mod args;
mod camera;
mod entity;
mod forms;
mod gpu_interface;
mod gui;
mod instance;
mod light;
mod model;
mod rendering;
mod resources;
mod scene;
mod simulation;
mod texture;
mod utilities;
mod demos;

use args::{Demos, FeriphysArgs};
use clap::Parser;

fn main() {
    let args = FeriphysArgs::parse();
    match args.demo {
        Demos::BouncingBall => demos::bouncing_ball::run(),
        Demos::Particles => demos::particles_cpu::run(),
    }
}
