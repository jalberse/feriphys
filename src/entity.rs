use crate::gpu_interface::GPUInterface;
use crate::instance::Instance;
use crate::instance::InstanceRaw;
use crate::model::ColoredMesh;
use crate::model::DrawColoredMesh;

use arrayvec::ArrayVec;
use cgmath::EuclideanSpace;
use cgmath::InnerSpace;
use cgmath::Vector3;
use wgpu::BindGroup;
use wgpu::Buffer;

// TODO If we were to make our instancing system more robust, we would have a strategy for letting
//    the instance buffer grow and shrink, creating new larger/smaller instance buffers as needed.
//    But for now, we'll just have one buffer large enough for our purposes without a reallocation strategy.
// TODO raise this to a much higher value, just a small number for dev right now.
pub const MAX_PARTICLE_INSTANCES: usize = 1000;

pub struct Entity {
    mesh: ColoredMesh,
    instances: ArrayVec<Instance, MAX_PARTICLE_INSTANCES>,
    instance_buffer: Buffer,
}

impl Entity {
    pub fn new(
        gpu: &GPUInterface,
        mesh: ColoredMesh,
        instances: ArrayVec<Instance, MAX_PARTICLE_INSTANCES>,
    ) -> Entity {
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

    pub fn update_instances(
        &mut self,
        gpu: &GPUInterface,
        instances: ArrayVec<Instance, MAX_PARTICLE_INSTANCES>,
    ) {
        self.instances = instances;
        InstanceRaw::update_buffer_from_vec(gpu, &self.instance_buffer, &self.instances);
    }

    /// Orients the normal of all the instances to face the position.
    /// This is useful when rendering particles, e.g., by making
    /// their quads face the camera postiion.
    pub fn orient_instances(&mut self, gpu: &GPUInterface, position: cgmath::Point3<f32>) {
        for instance in self.instances.iter_mut() {
            instance.rotation = cgmath::Quaternion::from_arc(
                Vector3::unit_y(),
                (position.to_vec() - instance.position).normalize(),
                None,
            );
        }
        InstanceRaw::update_buffer_from_vec(gpu, &self.instance_buffer, &self.instances);
    }
}
