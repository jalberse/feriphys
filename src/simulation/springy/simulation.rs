use std::time::Duration;

use super::super::state::State;
use super::obstacle::Obstacle;
use super::{config::Config, springy_mesh::SpringyMesh};

pub struct Simulation {
    config: Config,
    // Deformable springy meshes
    meshes: Vec<SpringyMesh>,
    obstacles: Vec<Obstacle>,
}

impl Simulation {
    pub fn new(meshes: Vec<SpringyMesh>, obstacles: Vec<Obstacle>) -> Simulation {
        let config = Config::default();
        Simulation {
            config,
            meshes,
            obstacles,
        }
    }

    pub fn step(&mut self) -> Duration {
        self.meshes.iter_mut().for_each(|mesh| {
            mesh.accumulate_forces(&self.config);

            let points = mesh.get_points();
            let state_vector = State::new(points.to_vec());
            // TODO Allow us to use Euler, not RK4, to show simulation blowing up.
            let new_state_vector = state_vector.rk4_step(self.config.dt.as_secs_f32());
            let new_points = new_state_vector.get_elements();

            mesh.update_points(new_points, &self.obstacles, &self.config);

            mesh.clear_forces();
        });

        self.config.dt
    }

    pub fn get_timestep(&self) -> Duration {
        self.config.dt
    }

    pub fn get_meshes(&self) -> &Vec<SpringyMesh> {
        &self.meshes
    }

    pub fn get_obstacles(&self) -> &Vec<Obstacle> {
        &self.obstacles
    }
}
