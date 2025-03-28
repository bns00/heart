//! Call [build] to configure and run [heart][crate].
//!
//! ```
//! fn main() {
//!     heart::build()
//!         .with_title("heart")
//!         .run();
//! }
//! ```
//!
//! ❤️

mod internal {
    pub(crate) mod app;
    pub(crate) mod executor;
}
pub(crate) use internal::*;

pub mod graphics;
pub mod image;
pub mod keyboard;
pub mod mouse;

/// Returns a builder for configuring and running [heart][crate].
///
/// See [HeartBuilder] for usage.
pub fn build() -> HeartBuilder {
    HeartBuilder::new()
}

/// Builder for [heart][crate].
///
/// Provides `with_*` methods to configure [heart][crate] before running it.
/// Call [run][HeartBuilder::run] to consume the builder and run [heart][crate].
pub struct HeartBuilder {
    app_config: app::Config,
    executor_config: executor::Config,
}

impl HeartBuilder {
    /// Same as calling [build].
    pub fn new() -> Self {
        Self {
            app_config: app::Config::default(),
            executor_config: executor::Config::default(),
        }
    }

    /// Sets the title for the window created by [heart][crate].
    pub fn with_title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.app_config.title = Some(title.into());
        self
    }

    /// Sets the amount of ticks to be generated each second. Default is 60.
    pub fn with_tick_rate(mut self, tick_rate: u64) -> Self {
        self.executor_config.tick_duration = executor::calculate_tick_duration(tick_rate);
        self
    }

    /// Adds a function to be called once before any [update][HeartBuilder::with_update] or [draw][HeartBuilder::with_draw] calls.
    ///
    /// This should be used for one-time initialization of the game.
    ///
    /// See [Load] for accepted functions.
    pub fn with_load<F, R>(mut self, mut load: F) -> Self
    where
        F: Load<R> + 'static,
    {
        self.executor_config
            .load
            .push(Box::new(move |state| load.call(state)));
        self
    }

    /// Adds a function to be called repeatedly at the frequency
    /// of the tick rate (60 times per second by default).
    ///
    /// This should be used for the game logic.
    ///
    /// See [Update] for accepted functions.
    pub fn with_update<F, A>(mut self, mut update: F) -> Self
    where
        F: Update<A> + 'static,
    {
        self.executor_config
            .update
            .push(Box::new(move |state| update.call(state)));
        self
    }

    /// Adds a function to be will be called once every frame.
    ///
    /// This should be used for drawing the game.
    ///
    /// See [Draw] for accepted functions.
    pub fn with_draw<F, A>(mut self, mut draw: F) -> Self
    where
        F: Draw<A> + 'static,
    {
        self.executor_config
            .draw
            .push(Box::new(move |state| draw.call(state)));
        self
    }

    /// Adds a function to be called on keyboard key press.
    ///
    /// See [Key] for accepted functions.
    pub fn with_key_pressed<F, A>(mut self, mut key_pressed: F) -> Self
    where
        F: Key<A> + 'static,
    {
        self.executor_config
            .key_pressed
            .push(Box::new(move |state, scancode| {
                key_pressed.call(state, scancode)
            }));
        self
    }

    /// Adds a function to be called on keyboard key release.
    ///
    /// See [Key] for accepted functions.
    pub fn with_key_released<F, A>(mut self, mut key_released: F) -> Self
    where
        F: Key<A> + 'static,
    {
        self.executor_config
            .key_released
            .push(Box::new(move |state, scancode| {
                key_released.call(state, scancode)
            }));
        self
    }

    /// Adds a function to be called on mouse button press.
    ///
    /// See [Mouse] for accepted functions.
    pub fn with_mouse_pressed<F, A>(mut self, mut mouse_pressed: F) -> Self
    where
        F: Mouse<A> + 'static,
    {
        self.executor_config
            .mouse_pressed
            .push(Box::new(move |state, x, y, button| {
                mouse_pressed.call(state, x, y, button)
            }));
        self
    }

    /// Adds a function to be called on mouse button release.
    ///
    /// See [Mouse] for accepted functions.
    pub fn with_mouse_released<F, A>(mut self, mut mouse_released: F) -> Self
    where
        F: Mouse<A> + 'static,
    {
        self.executor_config
            .mouse_released
            .push(Box::new(move |state, x, y, button| {
                mouse_released.call(state, x, y, button)
            }));
        self
    }

    /// Adds a function to be called whenever the mouse is moved.
    ///
    /// See [MouseMoved] for accepted functions.
    pub fn with_mouse_moved<F, A>(mut self, mut mouse_moved: F) -> Self
    where
        F: MouseMoved<A> + 'static,
    {
        self.executor_config
            .mouse_moved
            .push(Box::new(move |state, x, y, dx, dy| {
                mouse_moved.call(state, x, y, dx, dy)
            }));
        self
    }

    /// Consumes the builder and runs [heart][crate] with the configured parameters.
    pub fn run(self) {
        keyboard::state::init();
        mouse::state::init();
        app::run(
            self.app_config,
            executor::Executor::new(self.executor_config),
        );
    }
}

/// A [load][HeartBuilder::with_load] function.
///
/// It may optionally return a **state** value, which will later get passed back to any calls
/// that take the same type of **state** as an argument.
///
/// Accepted function signatures:
///
/// `fn()`
///
/// `fn() -> S`
#[allow(private_bounds)]
pub trait Load<R>: executor::callbacks::LoadCallback<R> {}

impl<F, R> Load<R> for F where F: executor::callbacks::LoadCallback<R> {}

/// An [update][HeartBuilder::with_update] function.
///
/// It may optionally take a **state** argument, which must have previously been returned by a
/// [load][HeartBuilder::with_load] call.
///
/// Accepted function signatures:
///
/// `fn()`
///
/// `fn(state: &mut S)`
#[allow(private_bounds)]
pub trait Update<A>: executor::callbacks::UpdateCallback<A> {}

impl<F, A> Update<A> for F where F: executor::callbacks::UpdateCallback<A> {}

/// A [draw][HeartBuilder::with_draw] function.
///
/// It may optionally take a **state** argument, which must have previously been returned by a
/// [load][HeartBuilder::with_load] call.
///
/// Accepted function signatures:
///
/// `fn()`
///
/// `fn(state: &mut S)`
#[allow(private_bounds)]
pub trait Draw<A>: executor::callbacks::DrawCallback<A> {}

impl<F, A> Draw<A> for F where F: executor::callbacks::DrawCallback<A> {}

/// A [key pressed][HeartBuilder::with_key_pressed] or [key released][HeartBuilder::with_key_released] function.
///
/// Must take a [Scancode][keyboard::Scancode] argument.
///
/// It may optionally take a **state** argument, which must have previously been returned by a
/// [load][HeartBuilder::with_load] call.
///
/// Accepted function signatures:
///
/// `fn(scancode: Scancode)`
///
/// `fn(state: &mut S, scancode: Scancode)`
#[allow(private_bounds)]
pub trait Key<A>: executor::callbacks::KeyCallback<A> {}

impl<F, A> Key<A> for F where F: executor::callbacks::KeyCallback<A> {}

/// A [mouse pressed][HeartBuilder::with_mouse_pressed] or [mouse released][HeartBuilder::with_mouse_released] function.
///
/// Must take 2 arguments for the mouse's x and y coordinates, respectively, and a 3rd [Button][mouse::Button] argument.
///
/// It may optionally take a **state** argument, which must have previously been returned by a
/// [load][HeartBuilder::with_load] call.
///
/// Accepted function signatures:
///
/// `fn(x: f32, y: f32, button: Button)`
///
/// `fn(state: &mut S, x: f32, y: f32, button: Button)`
#[allow(private_bounds)]
pub trait Mouse<A>: executor::callbacks::MouseCallback<A> {}

impl<F, A> Mouse<A> for F where F: executor::callbacks::MouseCallback<A> {}

/// A [mouse moved][HeartBuilder::with_mouse_moved] function.
///
/// Must take 2 arguments for the mouse's x and y coordinates, respectively, and 2 more arguments
/// for the change in x and y, respectively.
///
/// It may optionally take a **state** argument, which must have previously been returned by a
/// [load][HeartBuilder::with_load] call.
///
/// Accepted function signatures:
///
/// `fn(x: f32, y: f32, dx: f32, dy: f32)`
///
/// `fn(state: &mut S, x: f32, y: f32, dx: f32, dy: f32)`
#[allow(private_bounds)]
pub trait MouseMoved<A>: executor::callbacks::MouseMovedCallback<A> {}

impl<F, A> MouseMoved<A> for F where F: executor::callbacks::MouseMovedCallback<A> {}
