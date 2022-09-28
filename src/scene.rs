use crate::entity::Entity;
use crate::gpu_interface::GPUInterface;
use crate::instance::Instance;
use crate::simulation::particles::MAX_PARTICLES;

use arrayvec::ArrayVec;
use wgpu::BindGroup;

pub struct Scene {
    particles: Entity,
}

impl Scene {
    pub fn new(particles: Entity) -> Scene {
        // TODO we could have particles be a Vec<Entity>, so that we could have multiple particle systems getting rendered.
        Scene { particles }
    }

    /// TODO for now, we're just assuming the render_pass has a render pipeline set up that is compatible with
    /// what we're drawing here. We should develop a system for ensuring it's correct.
    /// If I try to draw a not-ColoredMesh thing, we'll need to do that. Maybe we'd have multiple render passes,
    /// each associated with a render pipeline. We can bundle those two, and then iterate over them. We can match
    /// on the type of render pipeline (or maybe the type of entity?) and use the correct one. Something like that.
    pub fn draw<'a, 'b>(
        &'a mut self,
        gpu: &GPUInterface,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        camera_position: cgmath::Point3<f32>,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    ) where
        'a: 'b,
    {
        self.particles.orient_instances(&gpu, camera_position);
        self.particles
            .draw(render_pass, camera_bind_group, light_bind_group);
    }

    pub fn update_particle_locations(
        &mut self,
        gpu: &GPUInterface,
        instances: ArrayVec<Instance, MAX_PARTICLES>,
    ) {
        self.particles.update_instances(gpu, instances);
    }
}
