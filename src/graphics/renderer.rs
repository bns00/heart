use zerocopy::IntoBytes;

use super::{rectangle, sprite};

pub(crate) const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Color {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
    pub(crate) a: f32,
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r.into(),
            g: value.g.into(),
            b: value.b.into(),
            a: value.a.into(),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

pub(crate) struct Buffers {
    pub(crate) vertex: wgpu::Buffer,
    pub(crate) vertex_offset: wgpu::BufferAddress,
    pub(crate) index: wgpu::Buffer,
    pub(crate) index_offset: wgpu::BufferAddress,
}

impl Buffers {
    fn new(device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 0x1000000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 0x1000000,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            vertex: vertex_buffer,
            vertex_offset: 0,
            index: index_buffer,
            index_offset: 0,
        }
    }
}

pub(crate) struct Uniforms {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) viewport_buffer: wgpu::Buffer,
}

impl Uniforms {
    fn create_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn new(layout: &wgpu::BindGroupLayout, device: &wgpu::Device) -> Self {
        let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_buffer.as_entire_binding(),
            }],
        });

        Self {
            viewport_buffer,
            bind_group,
        }
    }
}

pub(crate) struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) buffers: Buffers,
    pub(crate) uniforms: Uniforms,
    pub(crate) rectangle_pipeline: wgpu::RenderPipeline,
    pub(crate) sprite_renderer: sprite::SpriteRenderer,
}

impl Renderer {
    pub(crate) fn new(adapter: wgpu::Adapter) -> Option<Self> {
        let (device, queue) = create_device(&adapter)?;

        let buffers = Buffers::new(&device);

        let uniform_layout = Uniforms::create_layout(&device);

        let uniforms = Uniforms::new(&uniform_layout, &device);

        let rectangle_pipeline = rectangle::create_pipeline(&uniform_layout, &device);

        let sprite_renderer = sprite::SpriteRenderer::new(&uniform_layout, &device, &queue);

        Some(Self {
            device,
            queue,
            buffers,
            uniforms,
            rectangle_pipeline,
            sprite_renderer,
        })
    }

    pub(crate) fn reset(&mut self) {
        self.sprite_renderer.poll();
        self.buffers.vertex_offset = 0;
        self.buffers.index_offset = 0;
    }

    pub(crate) fn render(&mut self, render_list: &RenderList, target: wgpu::TextureView) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(render_list.clear_color.into()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        for command in render_list.commands.iter() {
            match command {
                RenderCommand::RectangleBatch(batch) => batch.render(self, &mut render_pass),
                RenderCommand::SpriteBatch(batch) => batch.render(self, &mut render_pass),
            }
        }

        drop(render_pass);
        self.queue.submit(Some(encoder.finish()));
    }

    pub(crate) fn set_viewport_uniform(&self, width: f32, height: f32) {
        self.queue.write_buffer(
            &self.uniforms.viewport_buffer,
            0,
            [width, height].as_bytes(),
        );
    }
}

#[derive(Default)]
pub(crate) struct RenderList {
    pub(crate) clear_color: Color,
    pub(crate) commands: Vec<RenderCommand>,
}

pub(crate) enum RenderCommand {
    RectangleBatch(rectangle::RectangleBatch),
    SpriteBatch(sprite::SpriteBatch),
}

fn create_device(adapter: &wgpu::Adapter) -> Option<(wgpu::Device, wgpu::Queue)> {
    struct Waker(std::thread::Thread);

    impl std::task::Wake for Waker {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.unpark();
        }
    }

    let future = adapter.request_device(&wgpu::DeviceDescriptor::default(), None);
    let mut future = std::pin::pin!(future);
    let thread = std::thread::current();
    let waker = std::sync::Arc::new(Waker(thread)).into();
    let mut context = std::task::Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(result) => break result.ok(),
            std::task::Poll::Pending => std::thread::park(),
        }
    }
}
