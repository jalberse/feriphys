use arrayvec::ArrayVec;
use cgmath::{Rotation3, Vector3};
use rand;

use crate::{
    entity::{Entity, MAX_PARTICLE_INSTANCES},
    forms,
    gpu_interface::GPUInterface,
    instance::Instance,
};

/// TODO:
///
/// State. Initialize. We can initialize a bunch on a plane.
///
/// A fn to get entities from the state. We'll use that to
/// populate the Scene initially.
///
/// A simple step function that just changes the position of particles.
/// Start calling it.
///
/// A function to get instances from the state. We'll call that to get
/// the updated positions, and then pass those instances
/// to the scene to update the rendered positions.
///
/// Now that we can visualzie particles moving, we can add a simple force.
/// Add gravity and wind, and let them fall.
///
/// Next, we can add a lifetime. After some time, all the particles should die.
/// This will involve setting up our pool!
///
/// Next, we can add a generator. We'll now have something like snow falling.
///
/// Next, we need to add collisions with a polygon.
///
/// We should add colors to our particles. We can do that by adding color information to IntanceRaw,
/// and handling that in the shader instead of using our colored mesh's color. The colored mesh color
/// will only be used to inform the default instance color.

struct Particle {
    position: Vector3<f32>,
}

pub struct State {
    particles: Vec<Particle>,
}

impl State {
    pub fn new() -> State {
        let mut particles = vec![];
        for _ in 0..10 {
            particles.push(Particle {
                position: Vector3::<f32> {
                    x: rand::random::<f32>() * 3.0,
                    y: rand::random::<f32>() * 3.0,
                    z: rand::random::<f32>() * 3.0,
                },
            })
        }
        State { particles }
    }

    pub fn get_particles_entity(&self, gpu: &GPUInterface) -> Entity {
        let mesh = forms::get_quad(&gpu.device, [1.0, 1.0, 1.0]);

        let mut instances = ArrayVec::<Instance, MAX_PARTICLE_INSTANCES>::new();
        for particle in self.particles.iter() {
            let instance = Instance {
                position: particle.position,
                // TODO this should be some Default.
                rotation: cgmath::Quaternion::from_axis_angle(
                    cgmath::Vector3::unit_z(),
                    cgmath::Deg(0.0),
                ),
                scale: 0.05,
            };
            instances.push(instance);
        }

        Entity::new(&gpu, mesh, instances)
    }
}
