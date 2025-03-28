mod sheet_assembly;

use std::{
    sync::Arc,
    thread::{self, Thread},
};

use sheet_assembly::{AssemblyReceiver, AssemblySender, SheetAssembler, SpriteData};
use wgpu::util::DeviceExt;
use zerocopy::IntoBytes;

use super::{
    renderer::{self, Renderer},
    transform::Transform,
};

struct Texture {
    inner: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    fn create_bind_group(
        texture: &wgpu::Texture,
        sampler: &wgpu::Sampler,
        bind_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    fn empty(
        width: u32,
        height: u32,
        sampler: &wgpu::Sampler,
        bind_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let bind_group = Self::create_bind_group(&texture, sampler, bind_layout, device);

        Self {
            inner: texture,
            bind_group,
        }
    }

    fn from_data(
        data: &[u8],
        width: u32,
        height: u32,
        sampler: &wgpu::Sampler,
        bind_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &data,
        );

        let bind_group = Self::create_bind_group(&texture, sampler, bind_layout, device);

        Self {
            inner: texture,
            bind_group,
        }
    }
}

enum Sprite {
    Texture(Arc<Texture>),
    Sheet(TextureRegion<u32>),
}

#[derive(Clone, Copy)]
struct TextureRegion<T> {
    left: T,
    top: T,
    right: T,
    bottom: T,
}

pub(crate) type Handle = usize;

pub(crate) struct SpriteRenderer {
    sprites: Vec<Sprite>,
    spritesheet: Arc<Texture>,
    assembly_thread: Thread,
    assembly_sender: AssemblySender,
    assembly_receiver: AssemblyReceiver,
    bind_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl SpriteRenderer {
    pub(crate) fn new(
        uniform_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[uniform_layout, &bind_layout],
            ..Default::default()
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
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
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: renderer::TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let (sheet_assembler, assembly_sender, assembly_receiver) =
            SheetAssembler::new(&sampler, &bind_layout, device, queue);
        let assembly_thread = thread::spawn(move || sheet_assembler.work())
            .thread()
            .clone();

        Self {
            sprites: Vec::new(),
            spritesheet: Arc::new(Texture::empty(1, 1, &sampler, &bind_layout, device)),
            assembly_thread,
            assembly_sender,
            assembly_receiver,
            bind_layout,
            pipeline,
            sampler,
        }
    }

    pub(crate) fn poll(&mut self) {
        if !self.assembly_sender.is_empty() {
            self.assembly_thread.unpark();
        }

        if let Ok(feedback) = self.assembly_receiver.try_recv() {
            for (handle, region) in feedback.updated_sprites {
                self.sprites[handle] = Sprite::Sheet(region);
            }
            self.spritesheet = feedback.new_sheet;
        }
    }

    pub(crate) fn create_sprite(
        &mut self,
        data: Box<[u8]>,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Handle {
        let handle = self.sprites.len();
        self.sprites
            .push(Sprite::Texture(Arc::new(Texture::from_data(
                &data,
                width,
                height,
                &self.sampler,
                &self.bind_layout,
                device,
                queue,
            ))));
        let _ = self.assembly_sender.send(SpriteData {
            handle,
            data,
            width,
            height,
        });
        handle
    }
}

pub(crate) struct SpriteDrawInfo {
    pub(crate) handle: Handle,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) transform: Transform,
}

#[derive(Clone, Copy, zerocopy::Immutable, zerocopy::IntoBytes)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

pub(crate) struct SpriteBatch {
    texture: Arc<Texture>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl SpriteBatch {
    pub(crate) fn new(draw_info: &SpriteDrawInfo, renderer: &mut Renderer) -> Self {
        let sprite = &renderer.sprite_renderer.sprites[draw_info.handle];
        let texture = match sprite {
            Sprite::Texture(texture) => texture.clone(),
            Sprite::Sheet(_) => renderer.sprite_renderer.spritesheet.clone(),
        };
        let mut batch = Self {
            texture,
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        batch.add(draw_info, renderer);
        batch
    }

    pub(crate) fn try_add(
        &mut self,
        draw_info: &SpriteDrawInfo,
        renderer: &mut Renderer,
    ) -> Result<(), Self> {
        let sprite = &renderer.sprite_renderer.sprites[draw_info.handle];
        let texture = match sprite {
            Sprite::Texture(texture) => texture.clone(),
            Sprite::Sheet(_) => renderer.sprite_renderer.spritesheet.clone(),
        };
        if !Arc::ptr_eq(&self.texture, &texture) {
            let mut new_batch = Self {
                texture,
                vertices: Vec::new(),
                indices: Vec::new(),
            };
            new_batch.add(draw_info, renderer);
            return Err(new_batch);
        }
        self.add(draw_info, renderer);
        Ok(())
    }

    fn add(&mut self, draw_info: &SpriteDrawInfo, renderer: &mut Renderer) {
        let sprite = &renderer.sprite_renderer.sprites[draw_info.handle];
        let (width, height, region) = match sprite {
            Sprite::Texture(texture) => (
                texture.inner.width() as f32,
                texture.inner.height() as f32,
                TextureRegion {
                    left: 0.0,
                    top: 0.0,
                    right: 1.0,
                    bottom: 1.0,
                },
            ),
            Sprite::Sheet(region) => {
                let sheet = &renderer.sprite_renderer.spritesheet;
                (
                    (region.right - region.left) as f32,
                    (region.bottom - region.top) as f32,
                    TextureRegion {
                        left: region.left as f32 / sheet.inner.width() as f32,
                        top: region.top as f32 / sheet.inner.height() as f32,
                        right: region.right as f32 / sheet.inner.width() as f32,
                        bottom: region.bottom as f32 / sheet.inner.height() as f32,
                    },
                )
            }
        };

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
                tex_coords: [region.left, region.top],
            },
            Vertex {
                position: draw_info.transform.apply(draw_info.x + width, draw_info.y),
                tex_coords: [region.right, region.top],
            },
            Vertex {
                position: draw_info.transform.apply(draw_info.x, draw_info.y + height),
                tex_coords: [region.left, region.bottom],
            },
            Vertex {
                position: draw_info
                    .transform
                    .apply(draw_info.x + width, draw_info.y + height),
                tex_coords: [region.right, region.bottom],
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
        render_pass.set_bind_group(1, &self.texture.bind_group, &[]);
        render_pass.set_pipeline(&renderer.sprite_renderer.pipeline);
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
