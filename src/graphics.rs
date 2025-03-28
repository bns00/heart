//! Drawing.
//!
//! Contains functions for drawing things, manipulating the coordinate
//! system and creating sprites. Most of these functions should only be
//! called from a [draw][crate::HeartBuilder::with_draw] call.
//!
//! Functions that can be called outside of [draw][crate::HeartBuilder::with_draw]:
//!
//! [create_sprite]

pub(crate) mod context;
pub(crate) mod rectangle;
pub(crate) mod renderer;
pub(crate) mod sprite;
pub(crate) mod transform;

/// Trait for drawable types.
pub trait Draw {
    /// Draw `self`.
    fn draw(&self, x: f32, y: f32);
}

/// A handle to a sprite.
#[derive(Clone)]
pub struct Sprite(sprite::Handle);

/// Create a [Sprite] from an [Image][crate::image::Image].
pub fn create_sprite(image: crate::image::Image) -> Sprite {
    let context = &mut *context::get();
    Sprite(context.renderer.sprite_renderer.create_sprite(
        image.data,
        image.width,
        image.height,
        &context.renderer.device,
        &context.renderer.queue,
    ))
}

/// Resets the drawing settings.
pub fn reset() {
    context::get().draw_state = context::DrawState::default();
}

/// Sets the color used for drawing.
pub fn set_color(r: f32, g: f32, b: f32, a: f32) {
    context::get().draw_state.color = renderer::Color { r, g, b, a };
}

/// Clears the screen with the set color.
pub fn clear() {
    let context = &mut *context::get();
    context.render_list.commands.clear();
    context.render_list.clear_color = context.draw_state.color;
}

/// Draws a rectangle.
pub fn rectangle(x: f32, y: f32, width: f32, height: f32) {
    let context = &mut *context::get();
    let draw_info = rectangle::RectangleDrawInfo {
        x,
        y,
        width,
        height,
        color: context.draw_state.color,
        transform: context.draw_state.transform,
    };
    match context.render_list.commands.last_mut() {
        Some(renderer::RenderCommand::RectangleBatch(batch)) => batch.add(&draw_info),
        _ => context
            .render_list
            .commands
            .push(renderer::RenderCommand::RectangleBatch(
                rectangle::RectangleBatch::new(&draw_info),
            )),
    }
}

/// Draws a [drawable][Draw].
pub fn drawable<T>(drawable: &T, x: f32, y: f32)
where
    T: Draw,
{
    drawable.draw(x, y);
}

/// Resets the coordinate system.
pub fn origin() {
    context::get().draw_state.transform = transform::Transform::identity();
}

/// Translates the coordinate system.
pub fn translate(x: f32, y: f32) {
    let context = &mut *context::get();
    context.draw_state.transform = context.draw_state.transform.translate(x, y)
}

/// Scales the coordinate system.
pub fn scale(x: f32, y: f32) {
    let context = &mut *context::get();
    context.draw_state.transform = context.draw_state.transform.scale(x, y)
}

/// Rotates the coordinate system.
pub fn rotate(angle: f32) {
    let context = &mut *context::get();
    context.draw_state.transform = context.draw_state.transform.rotate(angle)
}

/// Shears the coordinate system.
pub fn shear(x: f32, y: f32) {
    let context = &mut *context::get();
    context.draw_state.transform = context.draw_state.transform.shear(x, y)
}

impl<T> Draw for &T
where
    T: Draw,
{
    fn draw(&self, x: f32, y: f32) {
        (*self).draw(x, y);
    }
}

impl Draw for Sprite {
    fn draw(&self, x: f32, y: f32) {
        let context = &mut *context::get();
        let draw_info = sprite::SpriteDrawInfo {
            handle: self.0,
            x,
            y,
            transform: context.draw_state.transform,
        };
        if let Some(batch) = match context.render_list.commands.last_mut() {
            Some(renderer::RenderCommand::SpriteBatch(batch)) => {
                match batch.try_add(&draw_info, &mut context.renderer) {
                    Err(batch) => Some(batch),
                    _ => None,
                }
            }
            _ => Some(sprite::SpriteBatch::new(&draw_info, &mut context.renderer)),
        } {
            context
                .render_list
                .commands
                .push(renderer::RenderCommand::SpriteBatch(batch));
        }
    }
}
