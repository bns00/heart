use std::{
    ops::DerefMut,
    sync::{Arc, Mutex, OnceLock},
};

use super::{
    renderer::{Color, RenderList, Renderer},
    transform::Transform,
};

pub(crate) struct Context {
    pub(crate) renderer: Renderer,
    pub(crate) render_list: RenderList,
    pub(crate) draw_state: DrawState,
}

impl Context {
    pub(crate) fn reset(&mut self) {
        self.renderer.reset();
        self.render_list.clear_color = Color::default();
        self.render_list.commands.clear();
        self.draw_state = DrawState::default();
    }

    pub(crate) fn render(&mut self, view: wgpu::TextureView) {
        self.renderer.render(&self.render_list, view)
    }
}

pub(crate) struct DrawState {
    pub(crate) color: Color,
    pub(crate) transform: Transform,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            color: Color::default(),
            transform: Transform::identity(),
        }
    }
}

static CONTEXT: OnceLock<Arc<Mutex<Context>>> = OnceLock::new();

pub(crate) fn init(renderer: Renderer) {
    let _ = CONTEXT.set(Arc::new(Mutex::new(Context {
        renderer,
        render_list: RenderList::default(),
        draw_state: DrawState::default(),
    })));
}

pub(crate) fn get() -> impl DerefMut<Target = Context> {
    CONTEXT.get().unwrap().lock().unwrap()
}

pub(crate) fn reset() {
    get().reset();
}

pub(crate) fn render(view: wgpu::TextureView) {
    get().render(view);
}
