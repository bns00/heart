use zerocopy::IntoBytes;

use super::{
    renderer::{self, Color, Renderer},
    transform::Transform,
};

pub(crate) fn create_pipeline(
    uniform_layout: &wgpu::BindGroupLayout,
    device: &wgpu::Device,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("rectangle.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[uniform_layout],
        ..Default::default()
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: None,
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 24,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: 8,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: None,
            targets: &[Some(wgpu::ColorTargetState {
                format: renderer::TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

pub(crate) struct RectangleDrawInfo {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) color: Color,
    pub(crate) transform: Transform,
}

#[derive(Clone, Copy, zerocopy::Immutable, zerocopy::IntoBytes)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub(crate) struct RectangleBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl RectangleBatch {
    pub(crate) fn new(draw_info: &RectangleDrawInfo) -> Self {
        let mut batch = Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        batch.add(draw_info);
        batch
    }

    pub(crate) fn add(&mut self, draw_info: &RectangleDrawInfo) {
        self.indices.extend_from_slice(&[
            self.vertices.len() as u32,
            self.vertices.len() as u32 + 2,
            self.vertices.len() as u32 + 1,
            self.vertices.len() as u32 + 3,
            self.vertices.len() as u32 + 1,
            self.vertices.len() as u32 + 2,
        ]);

        self.vertices.extend_from_slice(&[
            Vertex {
                position: draw_info.transform.apply(draw_info.x, draw_info.y),
                color: [
                    draw_info.color.r,
                    draw_info.color.g,
                    draw_info.color.b,
                    draw_info.color.a,
                ],
            },
            Vertex {
                position: draw_info
                    .transform
                    .apply(draw_info.x + draw_info.width, draw_info.y),
                color: [
                    draw_info.color.r,
                    draw_info.color.g,
                    draw_info.color.b,
                    draw_info.color.a,
                ],
            },
            Vertex {
                position: draw_info
                    .transform
                    .apply(draw_info.x, draw_info.y + draw_info.height),
                color: [
                    draw_info.color.r,
                    draw_info.color.g,
                    draw_info.color.b,
                    draw_info.color.a,
                ],
            },
            Vertex {
                position: draw_info.transform.apply(
                    draw_info.x + draw_info.width,
                    draw_info.y + draw_info.height,
                ),
                color: [
                    draw_info.color.r,
                    draw_info.color.g,
                    draw_info.color.b,
                    draw_info.color.a,
                ],
            },
        ]);
    }

    pub(crate) fn render(&self, renderer: &mut Renderer, render_pass: &mut wgpu::RenderPass) {
        let vertices = self.vertices.as_bytes();
        let indices = self.indices.as_bytes();
        renderer.queue.write_buffer(
            &renderer.buffers.vertex,
            renderer.buffers.vertex_offset,
            vertices,
        );
        renderer.queue.write_buffer(
            &renderer.buffers.index,
            renderer.buffers.index_offset,
            indices,
        );
        render_pass.set_bind_group(0, &renderer.uniforms.bind_group, &[]);
        render_pass.set_pipeline(&renderer.rectangle_pipeline);
        render_pass.set_vertex_buffer(
            0,
            renderer.buffers.vertex.slice(
                renderer.buffers.vertex_offset
                    ..renderer.buffers.vertex_offset + vertices.len() as wgpu::BufferAddress,
            ),
        );
        render_pass.set_index_buffer(
            renderer.buffers.index.slice(
                renderer.buffers.index_offset
                    ..renderer.buffers.index_offset + indices.len() as wgpu::BufferAddress,
            ),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        renderer.buffers.vertex_offset += vertices.len() as wgpu::BufferAddress;
        renderer.buffers.index_offset += indices.len() as wgpu::BufferAddress;
    }
}
