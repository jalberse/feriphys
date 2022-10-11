use wgpu::{BindGroupLayout, RenderPipeline};

use crate::{
    graphics::camera::CameraBundle,
    graphics::gpu_interface::GPUInterface,
    graphics::instance,
    graphics::model::{ColoredVertex, Vertex},
    graphics::texture,
};

use super::{camera::Projection, model::ModelVertex};

pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn create_colored_mesh_render_pipeline(
    gpu: &GPUInterface,
    camera_bundle: &CameraBundle,
    light_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Colored Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bundle.camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Colored Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/color_shader.wgsl").into()),
    };
    create_render_pipeline(
        &gpu.device,
        &layout,
        gpu.config.format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[ColoredVertex::desc(), instance::InstanceRaw::desc::<5>()],
        shader,
    )
}

pub fn create_model_render_pipeline(
    gpu: &GPUInterface,
    camera_bundle: &CameraBundle,
    light_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let texture_bind_group_layout = create_texture_bind_group_layout(gpu);
    let layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Textured Model Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bundle.camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
    let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Default Model Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
    };
    create_render_pipeline(
        &gpu.device,
        &layout,
        gpu.config.format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[ModelVertex::desc(), instance::InstanceRaw::desc::<5>()],
        shader,
    )
}

pub fn create_texture_bind_group_layout(gpu: &GPUInterface) -> BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
}

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
