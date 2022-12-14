use super::gpu_interface::GPUInterface;

use cgmath::{Rotation3, Vector3, Zero};
use wgpu::{Buffer, BufferDescriptor};

/// Stores an instance's transformations.
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: f32,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let model = cgmath::Matrix4::from_translation(self.position)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_scale(self.scale);
        InstanceRaw {
            model: model.into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Instance {
            position: Vector3::<f32>::zero(),
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            ),
            scale: 1.0,
        }
    }
}

/// Reduced matrix from an Instance to be placed in the buffer for shaders.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl InstanceRaw {
    // LOCATION is the first shader_location for the VertexAttributes.
    // It may be non-zero if there are other vertex layouts preceding
    // this one to be passed into the shader.
    // Beware that this buffer layout takes up multiple shader locations, as the InstanceRaw
    // matrices are passed to the shader as vectors and reconstructed later.
    // So, ensure the shader locations for vertex buffer layouts which follow this (if any) are correct.
    // See https://github.com/gfx-rs/wgpu/discussions/2050 on why this isn't a regular param.
    pub const fn desc<const LOCATION: u32>() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: LOCATION,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: LOCATION + 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: LOCATION + 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: LOCATION + 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: LOCATION + 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: LOCATION + 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: LOCATION + 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    /// Creates a buffer of InstanceRaw data from the instances,
    /// and schedules a write to that buffer with the instance data.
    pub fn create_buffer_from_vec(
        gpu: &GPUInterface,
        instances: &Vec<Instance>,
        capacity: Option<usize>,
    ) -> Buffer {
        let capacity = if let Some(capacity) = capacity {
            capacity
        } else {
            instances.len()
        };

        let buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: capacity as u64 * std::mem::size_of::<InstanceRaw>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Note we don't need to declare buffer as mut because this only *schedules*
        // an update to the buffer via Queue::write_buffer().
        InstanceRaw::update_buffer_from_vec(&gpu, &buffer, &instances);
        buffer
    }

    /// Updates the the instance buffer with the vector of instances,
    /// started from the beginning of the buffer. Panics if the instances
    /// vector is larger than the buffer.
    /// Really, this only schedules a write to the buffer via gpu.queue.write_buffer().
    /// The buffer is updated from 0..N where N is the number of instances. The remaining length of the buffer
    /// remains untouched.
    /// Useful for if all instances are likely to be updated each frame, such as in particle systems.
    pub fn update_buffer_from_vec(gpu: &GPUInterface, buffer: &Buffer, instances: &Vec<Instance>) {
        let instances_raw_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        for (index, instance_data) in instances_raw_data.iter().enumerate() {
            gpu.queue.write_buffer(
                &buffer,
                index as u64 * std::mem::size_of::<InstanceRaw>() as u64,
                bytemuck::cast_slice(&[*instance_data]),
            );
        }
    }
}

impl Default for InstanceRaw {
    fn default() -> InstanceRaw {
        InstanceRaw {
            model: [[0.0; 4]; 4],
            normal: [[0.0; 3]; 3],
        }
    }
}
