use crate::graphics::entity::Entity;
use crate::graphics::gpu_interface::GPUInterface;
use crate::graphics::instance::Instance;
use wgpu::BindGroup;

pub struct Scene {
    // TODO we don't enforce at compile time whether we passed in the correct entities for particles vs
    //   for the entities field, so we may get bad behavior if order flips. WE should make a type to differentiate.
    //   Or reeally, Entity might be best as a typed Enum - ParticleEntity, ColoredMeshEntity (or something)
    //   so that the Scene just stores a list of entities. The behavior for drawing them etc is handled within each Entity variant.
    entities: Option<Vec<Entity>>,
    particles: Option<Vec<Entity>>,
}

impl Scene {
    pub fn new(entities: Option<Vec<Entity>>, particles: Option<Vec<Entity>>) -> Scene {
        Scene {
            entities,
            particles,
        }
    }

    /// TODO for now, we're just assuming the render_pass has a render pipeline set up that is compatible with
    /// what we're drawing here (i.e. colored meshes). We should develop a system for ensuring it's correct.
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
        if let Some(entities) = &self.entities {
            entities.iter().for_each(|entity|
                entity.draw(render_pass, camera_bind_group, light_bind_group));
        }

        if let Some(particles) = &mut self.particles {
            for particle_group in particles.iter_mut(){
                particle_group.orient_instances(&gpu, camera_position);
                particle_group.draw(render_pass, camera_bind_group, light_bind_group)
            }
        }
    }

    /// Updates the instances of the particle entity at the specific index.
    /// Panics if the index is out of range of the scene's particles
    /// TODO - Can we improve this API so we never panic?
    pub fn update_particle_instances(&mut self, gpu: &GPUInterface, particles_entity_index: usize, instances: Vec<Instance>) {
        if let Some(particles) = &mut self.particles {
            particles[particles_entity_index].update_instances(gpu, instances);
        }
    }

    /// Updates the instances of the entity at the specific index.
    /// Panics if the index is out of range of the scene's entities.
    /// TODO - Can we improve this API so we never panic? And related to comment about entities, can we
    /// combine with the particles one?
    pub fn update_entity_instances(&mut self, gpu: &GPUInterface, entity_index: usize, instances: Vec<Instance>) {
        if let Some(entities) = &mut self.entities {
            entities[entity_index].update_instances(gpu, instances);
        }
    }
}
