use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Demos {
    BouncingBall,
    ParticlesCpu,
    Flocking,
    SpringMassDamper,
    Cloth,
}

#[derive(Parser)]
pub struct FeriphysArgs {
    /// The first argument!
    #[clap(value_enum)]
    pub demo: Demos,
}
