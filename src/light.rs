use crate::gpu_interface::GPUInterface;

use wgpu::util::DeviceExt;
use wgpu::BindGroup;
use wgpu::BindGroupLayout;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> LightUniform {
        LightUniform {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        }
    }
}

pub fn create_light_bind_group(
    gpu: &GPUInterface,
    light_uniform: LightUniform,
) -> (BindGroupLayout, BindGroup) {
    // We'll want to be able to update our lights position, so we use COPY_DST
    let light_buffer = gpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    let light_bind_group_layout =
        gpu.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });
    let light_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &light_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: light_buffer.as_entire_binding(),
        }],
        label: None,
    });
    (light_bind_group_layout, light_bind_group)
}
