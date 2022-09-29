use crate::camera::Projection;
use crate::gpu_interface::GPUInterface;
use crate::texture;

pub fn resize(
    new_size: winit::dpi::PhysicalSize<u32>,
    gpu: &mut GPUInterface,
    depth_texture: &mut texture::Texture,
    projection: &mut Projection,
) {
    if new_size.width > 0 && new_size.height > 0 {
        gpu.size = new_size;
        gpu.config.width = new_size.width;
        gpu.config.height = new_size.height;
        gpu.surface.configure(&gpu.device, &gpu.config);
        // depth_texture must be udpated *after* the config, to get new width and height.
        *depth_texture =
            texture::Texture::create_depth_texture(&gpu.device, &gpu.config, "depth_texture");
        projection.resize(new_size.width, new_size.height)
    }
}
