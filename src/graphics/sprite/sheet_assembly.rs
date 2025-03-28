use std::{cmp, mem, sync::Arc, thread};

use super::{Handle, Texture, TextureRegion};

pub(crate) struct SpriteData {
    pub(crate) handle: Handle,
    pub(crate) data: Box<[u8]>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) struct SheetAssemblyFeedback {
    pub(crate) updated_sprites: Vec<(Handle, TextureRegion<u32>)>,
    pub(crate) new_sheet: Arc<Texture>,
}

pub(crate) type AssemblySender = crossbeam::channel::Sender<SpriteData>;

pub(crate) type AssemblyReceiver = crossbeam::channel::Receiver<SheetAssemblyFeedback>;

pub(crate) struct SheetAssembler {
    sampler: wgpu::Sampler,
    bind_layout: wgpu::BindGroupLayout,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sender: crossbeam::channel::Sender<SheetAssemblyFeedback>,
    receiver: crossbeam::channel::Receiver<SpriteData>,
}

impl SheetAssembler {
    pub(crate) fn new(
        sampler: &wgpu::Sampler,
        bind_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (Self, AssemblySender, AssemblyReceiver) {
        let (sender, other_receiver) = crossbeam::channel::bounded(0);
        let (other_sender, receiver) = crossbeam::channel::unbounded();
        (
            Self {
                sampler: sampler.clone(),
                bind_layout: bind_layout.clone(),
                device: device.clone(),
                queue: queue.clone(),
                sender,
                receiver,
            },
            other_sender,
            other_receiver,
        )
    }

    pub(crate) fn work(self) -> ! {
        let mut layout = Node::empty();
        let mut sheet = Box::from([]);
        let mut sheet_size = 0;
        loop {
            thread::park();

            let mut allocations = Vec::with_capacity(self.receiver.len());
            let mut datas = Vec::with_capacity(self.receiver.len());
            for sprite in self.receiver.try_iter() {
                allocations.push((sprite.handle, layout.alloc(sprite.width, sprite.height)));
                datas.push(sprite.data);
            }

            if layout.size() > sheet_size {
                let mut new_sheet = vec![0; layout.size() as usize * layout.size() as usize * 4];
                copy(&mut new_sheet, layout.size(), 0, 0, &sheet, sheet_size);
                sheet = new_sheet.into_boxed_slice();
                sheet_size = layout.size();
            }

            for ((_, allocation), data) in allocations.iter_mut().zip(datas) {
                copy(
                    &mut sheet,
                    sheet_size,
                    allocation.left,
                    allocation.top,
                    &data,
                    allocation.right - allocation.left,
                );
            }

            let sheet_texture = Arc::new(Texture::from_data(
                &sheet,
                sheet_size,
                sheet_size,
                &self.sampler,
                &self.bind_layout,
                &self.device,
                &self.queue,
            ));

            let _ = self.sender.send(SheetAssemblyFeedback {
                updated_sprites: allocations,
                new_sheet: sheet_texture,
            });
        }
    }
}

fn copy(dest: &mut [u8], dest_size: u32, dest_x: u32, dest_y: u32, src: &[u8], src_width: u32) {
    if src_width == 0 {
        return;
    }
    let dest_size = dest_size as usize;
    let dest_x = dest_x as usize;
    let dest_y = dest_y as usize;
    let src_width = src_width as usize;
    for (src_y, row) in src.chunks_exact(src_width * 4).enumerate() {
        let dest_offset = (dest_y + src_y) * dest_size * 4 + dest_x * 4;
        dest[dest_offset..dest_offset + src_width * 4].copy_from_slice(row);
    }
}

struct Subdivision {
    top_left: Node,
    top_right: Node,
    bottom_left: Node,
    bottom_right: Node,
}

enum AllocError {
    Occupied,
    Undersized,
    EmptyUndersized,
}

enum Node {
    Leaf {
        size: u32,
    },
    Subdivided {
        size: u32,
        children: Box<Subdivision>,
    },
    Empty {
        size: u32,
        x: u32,
        y: u32,
    },
}

impl Node {
    fn empty() -> Self {
        Self::Empty {
            size: 0,
            x: 0,
            y: 0,
        }
    }

    fn subdivided(outer_size: u32) -> Self {
        Self::Subdivided {
            size: outer_size,
            children: Box::new(Subdivision {
                top_left: Self::Empty {
                    size: outer_size / 2,
                    x: 0,
                    y: 0,
                },
                top_right: Self::Empty {
                    size: outer_size / 2,
                    x: outer_size / 2,
                    y: 0,
                },
                bottom_left: Self::Empty {
                    size: outer_size / 2,
                    x: 0,
                    y: outer_size / 2,
                },
                bottom_right: Self::Empty {
                    size: outer_size / 2,
                    x: outer_size / 2,
                    y: outer_size / 2,
                },
            }),
        }
    }

    fn leaf(size: u32) -> Self {
        Self::Leaf { size }
    }

    fn size(&self) -> u32 {
        match self {
            Self::Leaf { size } => *size,
            Self::Subdivided { size, .. } => *size,
            Self::Empty { size, .. } => *size,
        }
    }

    fn alloc(&mut self, width: u32, height: u32) -> TextureRegion<u32> {
        let normalized_size = u32::max(width, height).next_power_of_two();
        let (x, y) = match self.try_alloc(normalized_size) {
            Ok(allocation) => allocation,

            Err(AllocError::Occupied) => {
                let new = Self::subdivided(self.size() * 2);
                let old = mem::replace(self, new);
                match self {
                    Self::Subdivided { children, .. } => {
                        children.top_left = old;
                        children.top_right.empty_alloc(normalized_size)
                    }
                    _ => unreachable!(),
                }
            }

            Err(AllocError::Undersized) => {
                let new = Self::subdivided(normalized_size * 2);
                let old = mem::replace(self, new);
                match self {
                    Self::Subdivided { children, .. } => {
                        children.top_left.realloc(old);
                        children.top_right = Self::leaf(normalized_size);
                        (normalized_size, 0)
                    }
                    _ => unreachable!(),
                }
            }

            Err(AllocError::EmptyUndersized) => {
                *self = Self::leaf(normalized_size);
                (0, 0)
            }
        };
        TextureRegion {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    fn try_alloc(&mut self, sprite_size: u32) -> Result<(u32, u32), AllocError> {
        match self {
            Self::Leaf { size } => match sprite_size.cmp(size) {
                cmp::Ordering::Less | cmp::Ordering::Equal => Err(AllocError::Occupied),

                cmp::Ordering::Greater => Err(AllocError::Undersized),
            },

            Self::Subdivided { size, children } => match sprite_size.cmp(size) {
                cmp::Ordering::Less => {
                    for child in [
                        &mut children.top_left,
                        &mut children.top_right,
                        &mut children.bottom_left,
                        &mut children.bottom_right,
                    ] {
                        match child.try_alloc(sprite_size) {
                            Ok(allocation) => return Ok(allocation),
                            Err(AllocError::Occupied) => continue,
                            _ => unreachable!(),
                        }
                    }
                    Err(AllocError::Occupied)
                }

                cmp::Ordering::Equal => Err(AllocError::Occupied),

                cmp::Ordering::Greater => Err(AllocError::Undersized),
            },

            Self::Empty { size, .. } => match sprite_size.cmp(size) {
                cmp::Ordering::Less | cmp::Ordering::Equal => Ok(self.empty_alloc(sprite_size)),

                cmp::Ordering::Greater => Err(AllocError::EmptyUndersized),
            },
        }
    }

    fn empty_alloc(&mut self, sprite_size: u32) -> (u32, u32) {
        match self {
            Self::Empty { size, x, y } => match sprite_size.cmp(size) {
                cmp::Ordering::Less => {
                    let mut top_left = Self::Empty {
                        size: *size / 2,
                        x: *x,
                        y: *y,
                    };
                    let allocation = top_left.empty_alloc(sprite_size);
                    *self = Self::Subdivided {
                        size: *size,
                        children: Box::new(Subdivision {
                            top_left,
                            top_right: Self::Empty {
                                size: *size / 2,
                                x: *x + *size / 2,
                                y: *y,
                            },
                            bottom_left: Self::Empty {
                                size: *size / 2,
                                x: *x,
                                y: *y + *size / 2,
                            },
                            bottom_right: Self::Empty {
                                size: *size / 2,
                                x: *x + *size / 2,
                                y: *y + *size / 2,
                            },
                        }),
                    };
                    allocation
                }

                cmp::Ordering::Equal => {
                    let allocation = (*x, *y);
                    *self = Self::Leaf { size: *size };
                    allocation
                }

                cmp::Ordering::Greater => panic!(),
            },
            _ => panic!(),
        }
    }

    fn realloc(&mut self, old: Self) {
        match self {
            Self::Empty { size, x, y } => match old.size().cmp(size) {
                cmp::Ordering::Less => {
                    let mut top_left = Self::Empty {
                        size: *size / 2,
                        x: *x,
                        y: *x,
                    };
                    top_left.realloc(old);
                    *self = Self::Subdivided {
                        size: *size,
                        children: Box::new(Subdivision {
                            top_left,
                            top_right: Self::Empty {
                                size: *size / 2,
                                x: *x + *size / 2,
                                y: *y,
                            },
                            bottom_left: Self::Empty {
                                size: *size / 2,
                                x: *x,
                                y: *y + *size / 2,
                            },
                            bottom_right: Self::Empty {
                                size: *size / 2,
                                x: *x + *size / 2,
                                y: *y + *size / 2,
                            },
                        }),
                    };
                }

                cmp::Ordering::Equal => *self = old,

                cmp::Ordering::Greater => panic!(),
            },
            _ => panic!(),
        }
    }
}
