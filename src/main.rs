mod args;
mod gui;
mod graphics;
mod simulation;
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
