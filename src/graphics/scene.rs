use crate::graphics::entity::ColoredMeshEntity;
use crate::graphics::gpu_interface::GPUInterface;
use crate::graphics::instance::Instance;
use wgpu::BindGroup;

use super::entity::Entity;

pub struct Scene {
    // TODO we don't enforce at compile time whether we passed in the correct entities for particles vs
    //   for the entities field, so we may get bad behavior if order flips. WE should make a type to differentiate.
    //   Or reeally, Entity might be best as a typed Enum - ParticleEntity, ColoredMeshEntity (or something)
    //   so that the Scene just stores a list of entities. The behavior for drawing them etc is handled within each Entity variant.
    entities: Option<Vec<Entity>>,
    colored_mesh_entities: Option<Vec<ColoredMeshEntity>>,
    particles: Option<Vec<ColoredMeshEntity>>,
}

impl Scene {
    pub fn new(
        entities: Option<Vec<Entity>>,
        colored_mesh_entities: Option<Vec<ColoredMeshEntity>>,
        particles: Option<Vec<ColoredMeshEntity>>,
    ) -> Scene {
        Scene {
            entities,
            colored_mesh_entities,
            particles,
        }
    }

    /// Draws the entities in the scene.
    /// Note, assumes the caller has set the correct render pass for drawing Entity objects.
    pub fn draw_entities<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    ) where
        'a: 'b,
    {
        if let Some(entities) = &self.entities {
            entities
                .iter()
                .for_each(|entity| entity.draw(render_pass, camera_bind_group, light_bind_group));
        }
    }

    /// Draws the colored mesh entities, including the particles, which are themselves instances colored
    /// meshes.
    /// Note, assumes the caller has set the correct render pass for drawing colored meshes.
    pub fn draw_colored_mesh_entities<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a BindGroup,
        light_bind_group: &'a BindGroup,
    ) where
        'a: 'b,
    {
        if let Some(entities) = &self.colored_mesh_entities {
            entities
                .iter()
                .for_each(|entity| entity.draw(render_pass, camera_bind_group, light_bind_group));
        }

        if let Some(particles) = &self.particles {
            for particle_group in particles.iter() {
                particle_group.draw(render_pass, camera_bind_group, light_bind_group)
            }
        }
    }

    /// Updates the instances of the particle entity at the specific index.
    /// Panics if the index is out of range of the scene's particles
    /// TODO - Can we improve this API so we never panic?
    pub fn update_particle_instances(
        &mut self,
        gpu: &GPUInterface,
        particles_entity_index: usize,
        instances: Vec<Instance>,
        camera_position: cgmath::Point3<f32>,
    ) {
        if let Some(particles) = &mut self.particles {
            particles[particles_entity_index].update_instances(gpu, instances);
            particles[particles_entity_index].orient_instances(&gpu, camera_position);
        }
    }

    /// Updates the instances of the entity at the specific index.
    /// Panics if the index is out of range of the scene's entities.
    /// TODO - Can we improve this API so we never panic? And related to comment about entities, can we
    /// combine with the particles one?
    /// TODO - and can we combine with the update_colored_mesh_entity_instances()
    pub fn update_entity_instances(
        &mut self,
        gpu: &GPUInterface,
        entity_index: usize,
        instances: Vec<Instance>,
    ) {
        if let Some(entities) = &mut self.entities {
            entities[entity_index].update_instances(gpu, instances);
        }
    }

    #[allow(dead_code)]
    pub fn update_colored_mesh_entity_instances(
        &mut self,
        gpu: &GPUInterface,
        entity_index: usize,
        instances: Vec<Instance>,
    ) {
        if let Some(entities) = &mut self.colored_mesh_entities {
            entities[entity_index].update_instances(gpu, instances);
        }
    }
}
