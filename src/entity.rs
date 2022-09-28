use crate::forms;
use crate::gpu_interface::GPUInterface;
use crate::instance::Instance;
use crate::instance::InstanceRaw;
use crate::model::ColoredMesh;
use crate::model::DrawColoredMesh;

use arrayvec::ArrayVec;
use cgmath::Rotation3;
use wgpu::BindGroup;
use wgpu::Buffer;

// TODO If we were to make our instancing system more robust, we would have a strategy for letting
//    the instance buffer grow and shrink, creating new larger/smaller instance buffers as needed.
//    But for now, we'll just have one buffer large enough for our purposes without a reallocation strategy.
// TODO raise this to a much higher value, just a small number for dev right now.
const MAX_PARTICLE_INSTANCES: usize = 1000;

pub struct Entity {
    mesh: ColoredMesh,
    instances: ArrayVec<Instance, MAX_PARTICLE_INSTANCES>,
    instance_buffer: Buffer,
}

impl Entity {
    // TODO this fn should take a mesh, and instance data. For now, hard coded is fine.
    pub fn new(gpu: &GPUInterface) -> Entity {
        let mesh = forms::get_quad(&gpu.device, [1.0, 1.0, 1.0]);

        let mut instances = ArrayVec::<Instance, MAX_PARTICLE_INSTANCES>::new();

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
        instances.push(tmp_instance);

        let instance_buffer = InstanceRaw::create_buffer_from_vec(&gpu, &instances);

        Entity {
            mesh,
            instances,
            instance_buffer,
        }
    }

    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    ) where
        'a: 'b,
    {
        // TODO don't like the literal int here. Create const *_SLOT values in rendering.rs, for each render pipeline.
        //    Speaking of, move the creation of each type of render pipeline to that file as well (we share the colored render pipeline, e.g.)
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw_colored_mesh_instanced(
            &self.mesh,
            0..self.instances.len() as u32,
            &camera_bind_group,
            &light_bind_group,
        );
    }
}
