mod args;
mod bouncing_ball_demo;
mod camera;
mod forms;
mod gpu_interface;
mod gui;
mod instance;
mod model;
mod resources;
mod simulation;
mod texture;

use crate::bouncing_ball_demo::run;
use args::{Demos, FeriphysArgs};
use clap::Parser;

fn main() {
    let args = FeriphysArgs::parse();
    match args.demo {
        Demos::BouncingBall => run(),
    }
}
