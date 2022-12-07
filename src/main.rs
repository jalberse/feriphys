mod args;
mod demos;
mod graphics;
mod gui;
mod simulation;
mod utils;

use args::{Demos, FeriphysArgs};
use clap::Parser;

fn main() {
    let args = FeriphysArgs::parse();
    match args.demo {
        Demos::BouncingBall => demos::bouncing_ball::run(),
        Demos::ParticlesCpu => demos::particles_cpu::run(),
        Demos::Flocking => demos::flocking::run(),
        Demos::SpringMassDamper => demos::spring_mass_damper::run(),
        Demos::Cloth => demos::cloth::run(),
        Demos::RigidBody => demos::rigidbody::run(),
        Demos::Sph => demos::sph::run(),
    }
}
