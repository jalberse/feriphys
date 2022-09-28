use crate::entity::Entity;
use crate::gpu_interface::GPUInterface;

use wgpu::BindGroup;

pub struct Scene {
    particles: Entity,
}

impl Scene {
    pub fn new(gpu: &GPUInterface) -> Scene {
        let particles = Entity::new(&gpu);
        Scene { particles }
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
        self.particles
            .draw(render_pass, camera_bind_group, light_bind_group);
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
