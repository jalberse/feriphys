use crate::forms;
/// So, I guess we want https://gameprogrammingpatterns.com/flyweight.html
/// According to that, We would create some Entity that stores the instance data (i.e. position, rotation, scale)
/// and a reference to some single Mesh for the Scene. For now, that Mesh can just be explicitly a particle mesh field
/// stored for the scene. We'd eventually want to abstract that in some way for arbitrary meshes.
///
///
/// We can make a Entity that stores a mesh (really, a model, but we'll start with just ColoredMesh)
/// and an array of Intances.
///     We'll actually use an arrayvec of instances so we have a vec of all active instances, while capping at what our buffer can manage.
///     We'll repopulate the instances vector every frame based on all the active particles from the simulation/pool. Worry about performance later.
/// The Scene will actually have a draw() call, which iterates over Entities and draws them instanced as needed.
/// Each Entity has its own InstanceBuffer. The instance buffer will always be a static length.
use crate::instance::InstanceRaw;
use crate::model::DrawColoredMesh;
use crate::{gpu_interface::GPUInterface, instance::Instance, model::ColoredMesh};

use arrayvec::ArrayVec;
use cgmath::Rotation3;
use wgpu::util::DeviceExt;
use wgpu::BindGroup;
use wgpu::Buffer;

// TODO If we were to make our instancing system more robust, we would have a strategy for letting
//    the instance buffer grow and shrink, creating new larger/smaller instance buffers as needed.
//    But for now, we'll just have one buffer large enough for our purposes without a reallocation strategy.
// TODO raise this to a much higher value, just a small number for dev right now.
const MAX_PARTICLE_INSTANCES: usize = 1000;

// TODO Really, this should be a more general struct for storing any instanced mesh (or textured models!), since
//      we're not treating particles in any unique way. But we'll keep it like this for now, since
//      we're focussed on the particle system.
struct Particles {
    mesh: ColoredMesh,
    instances: ArrayVec<Instance, MAX_PARTICLE_INSTANCES>,
}

pub struct Scene {
    // TODO really, their instance buffer should probably be in the Particles struct.
    //    So any given mesh has its model, its instances, and its buffer. The update_particle_instance_buffer() fn
    //    we have can then be generatlized to just call that on all of the Particles (which is a struct we should really rename to like, Entity or something?)
    particles: Particles,
    particle_instance_buffer: Buffer,
}

impl Scene {
    pub fn new(gpu: &GPUInterface) -> Scene {
        let mut particle_instances = ArrayVec::<Instance, MAX_PARTICLE_INSTANCES>::new();

        // TODO For now we're just using a single instance, but we'll add more. Eventully the instances will be determined
        //  by the initial state o the simulation.
        let tmp_instance = Instance {
            position: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        };
        particle_instances.push(tmp_instance);

        let particle_mesh = forms::get_quad(&gpu.device, [1.0, 1.0, 1.0]);

        let particles = Particles {
            mesh: particle_mesh,
            instances: particle_instances,
        };
        let particle_instance_buffer = Scene::create_particles_instance_buffer(&gpu);

        let mut scene = Scene {
            particles,
            particle_instance_buffer,
        };

        scene.update_particle_instance_buffer(&gpu);

        scene
    }

    pub fn create_particles_instance_buffer(gpu: &GPUInterface) -> Buffer {
        let zeroed_raw_instance_array = [InstanceRaw::default(); MAX_PARTICLE_INSTANCES];
        gpu.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Dynamic Instance Buffer"),
                contents: bytemuck::cast_slice(&zeroed_raw_instance_array),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
    }

    /// Updates the particles_instance_buffer from the Scene's particle data.
    /// The instance data for all active particles is updated, since they are all likely to change each frame.
    /// The buffer is updated from 0..N where N is the number of instances. The remaining length of the buffer
    /// remains untouched.
    fn update_particle_instance_buffer(&mut self, gpu: &GPUInterface) {
        let particle_instances_raw_data = self
            .particles
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();

        for instance_data in particle_instances_raw_data {
            gpu.queue.write_buffer(
                &self.particle_instance_buffer,
                0,
                bytemuck::cast_slice(&[instance_data]),
            );
        }
    }

    /// TODO for now, we're just assuming the render_pass has a render pipeline set up that is compatible with
    /// what we're drawing here. We should develop a system for ensuring it's correct.
    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    ) where
        'a: 'b,
    {
        self.draw_particles(render_pass, camera_bind_group, light_bind_group);
    }

    pub fn draw_particles<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    ) where
        'a: 'b,
    {
        // TODO don't like the literal int here. Create const *_SLOT values in rendering.rs, for each render pipeline.
        //    Speaking of, move the creation of each type of render pipeline to that file as well (we share the colored render pipeline, e.g.)
        render_pass.set_vertex_buffer(1, self.particle_instance_buffer.slice(..));
        render_pass.draw_colored_mesh_instanced(
            &self.particles.mesh,
            0..self.particles.instances.len() as u32,
            &camera_bind_group,
            &light_bind_group,
        );
    }

    // TODO some function to orient particles instances to face the camera (give it a position and we'll point the normals to that position).
    //     Do this after I just like, get a particle rendered in whatever orientation so I know all the drawing stuff works.

    // TODO add a function(s) to update the scene data, from the simulation data. After that we'd call the function to write to the buffer.
    // TODO Add a function to write to the buffer, like we do in bouncing_ball_demo::State::update().
    //   Remember that our particle instances here are State.dynamic_instances there. This is just better organizaiton.

    // TODO Following bouncing_ball_demo.rs, create functions for drawing the particles as appropriate, considering we're storing all the meshes and instances and stuff
    //      in this scene rather than in the State for the demo. I think that means we have a Scene::draw() method, which takes in a render_pass.
    //      Then we do all the setting of buffers and draw calls etc here.
}
