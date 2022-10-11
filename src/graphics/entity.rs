use super::gpu_interface::GPUInterface;
use super::instance::Instance;
use super::instance::InstanceRaw;
use super::model::ColoredMesh;
use super::model::DrawColoredMesh;
use crate::simulation::particles_cpu::particles::MAX_INSTANCES;

use cgmath::{EuclideanSpace, InnerSpace, Vector3};
use wgpu::{BindGroup, Buffer};

// TODO If we were to make our instancing system more robust, we would have a strategy for letting
//    the instance buffer grow and shrink, creating new larger/smaller instance buffers as needed.
//    But for now, we'll just have one buffer large enough for our purposes without a reallocation strategy.

// TODO Make this class work with a Model instead of a colored mesh. I think for now the easiest way would be to have an Enum
//      and have Model and ColoredMesh be the two variants? There are probably better solutions but this is like, one that might work for now.
//      Or the Dumbest way to do this is to rename this one to some ColoredMeshEntity, and then remake this Entity for textured meshes.
//      Honestly.. do that second one, I think. Do the dead simple thing to get this demo working. We can refactor and improve when we have a working base.

pub struct ColoredMeshEntity {
    mesh: ColoredMesh,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl ColoredMeshEntity {
    pub fn new(
        gpu: &GPUInterface,
        mesh: ColoredMesh,
        instances: Vec<Instance>,
    ) -> ColoredMeshEntity {
        let instance_buffer = InstanceRaw::create_buffer_from_vec(&gpu, &instances, MAX_INSTANCES);

        ColoredMeshEntity {
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

    pub fn update_instances(&mut self, gpu: &GPUInterface, instances: Vec<Instance>) {
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

    #[allow(dead_code)]
    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}
