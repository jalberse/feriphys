use super::gpu_interface::GPUInterface;
use super::instance::Instance;
use super::instance::InstanceRaw;
use super::model::ColoredMesh;
use super::model::DrawColoredMesh;
use super::model::DrawModel;
use super::model::Model;

use cgmath::{EuclideanSpace, InnerSpace, Vector3};
use wgpu::{BindGroup, Buffer};

// TODO If we were to make our instancing system more robust, we would have a strategy for letting
//    the instance buffer grow and shrink, creating new larger/smaller instance buffers as needed.
//    But for now, we'll just have one buffer large enough for our purposes without a reallocation strategy.

pub struct Entity {
    model: Model,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl Entity {
    /// Capacity is the maximum number of instances the Entity will allow. If None, the size of instances is used.
    ///     Specifying capacity allows the instances to grow or shrink for e.g. particle systems.
    pub fn new(
        gpu: &GPUInterface,
        model: Model,
        instances: Vec<Instance>,
        capacity: Option<usize>,
    ) -> Entity {
        let instance_buffer = InstanceRaw::create_buffer_from_vec(&gpu, &instances, capacity);

        Entity {
            model,
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
        render_pass.draw_model_instanced(
            &self.model,
            0..self.instances.len() as u32,
            &camera_bind_group,
            &light_bind_group,
        );
    }

    pub fn update_instances(&mut self, gpu: &GPUInterface, instances: Vec<Instance>) {
        self.instances = instances;
        InstanceRaw::update_buffer_from_vec(gpu, &self.instance_buffer, &self.instances);
    }

    pub fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }
}

pub struct ColoredMeshEntity {
    mesh: ColoredMesh,
    instances: Vec<Instance>,
    instance_buffer: Buffer,
}

impl ColoredMeshEntity {
    /// Capacity is the maximum number of instances the Entity will allow. If None, the size of instances is used.
    ///     Specifying capacity allows the instances to grow or shrink for e.g. particle systems.
    pub fn new(
        gpu: &GPUInterface,
        mesh: ColoredMesh,
        instances: Vec<Instance>,
        capacity: Option<usize>,
    ) -> ColoredMeshEntity {
        let instance_buffer = InstanceRaw::create_buffer_from_vec(&gpu, &instances, capacity);

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
