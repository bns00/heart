// Hi! Thanks for reading the code!
// The implementation details of the callbacks are hidden
// away in this file because they are pretty complex. For
// type system reasons, the callbacks cannot be overloaded
// in a simple way. Every trait except the `Load` one is
// generic over an `A` parameter, which represents the
// arguments of the callback function as a tuple. For
// example, the `UpdateCallback` trait is implemented for
// two kinds of functions:
// `fn()`
// `fn(&mut S)`
// In the first case, the `A` parameter is the empty tuple
// `()` and in the second case it's `(&mut S,)`. It doesn't
// actually matter what types are chosen for `A` as long as
// they are different. However, using the function argument
// tuple is a sure way to guarantee that each signature gets
// a unique `UpdateCallback`. The reason for needing them to
// be unique is called "coherence". If we tried implementing
// both types of signatures for the same exact trait, we
// would get an overlapping implementation error.

use crate::{keyboard, mouse};

use super::State;

pub(crate) trait LoadCallback<R> {
    fn call(&mut self, state: &mut State);
}

impl<F, R> LoadCallback<R> for F
where
    F: FnMut() -> R,
    R: 'static,
{
    fn call(&mut self, state: &mut State) {
        state.store(self());
    }
}

pub(crate) trait UpdateCallback<A> {
    fn call(&mut self, state: &mut State);
}

impl<F> UpdateCallback<()> for F
where
    F: FnMut(),
{
    fn call(&mut self, _: &mut State) {
        self();
    }
}

impl<F, S> UpdateCallback<(&mut S,)> for F
where
    F: FnMut(&mut S),
    S: 'static,
{
    fn call(&mut self, state: &mut State) {
        if let Some(s) = state.retrieve() {
            self(s);
        }
    }
}

pub(crate) trait DrawCallback<A> {
    fn call(&mut self, state: &mut State);
}

impl<F> DrawCallback<()> for F
where
    F: FnMut(),
{
    fn call(&mut self, _: &mut State) {
        self();
    }
}

impl<F, S> DrawCallback<(&mut S,)> for F
where
    F: FnMut(&mut S),
    S: 'static,
{
    fn call(&mut self, state: &mut State) {
        if let Some(s) = state.retrieve() {
            self(s);
        }
    }
}

pub(crate) trait KeyCallback<A> {
    fn call(&mut self, state: &mut State, scancode: keyboard::Scancode);
}

impl<F> KeyCallback<(keyboard::Scancode,)> for F
where
    F: FnMut(keyboard::Scancode),
{
    fn call(&mut self, _: &mut State, scancode: keyboard::Scancode) {
        self(scancode);
    }
}

impl<F, S> KeyCallback<(&mut S, keyboard::Scancode)> for F
where
    F: FnMut(&mut S, keyboard::Scancode),
    S: 'static,
{
    fn call(&mut self, state: &mut State, scancode: keyboard::Scancode) {
        if let Some(s) = state.retrieve() {
            self(s, scancode);
        }
    }
}

pub(crate) trait MouseCallback<A> {
    fn call(&mut self, state: &mut State, x: f32, y: f32, button: mouse::Button);
}

impl<F> MouseCallback<(f32, f32, mouse::Button)> for F
where
    F: FnMut(f32, f32, mouse::Button),
{
    fn call(&mut self, _: &mut State, x: f32, y: f32, button: mouse::Button) {
        self(x, y, button);
    }
}

impl<F, S> MouseCallback<(&mut S, f32, f32, mouse::Button)> for F
where
    F: FnMut(&mut S, f32, f32, mouse::Button),
    S: 'static,
{
    fn call(&mut self, state: &mut State, x: f32, y: f32, button: mouse::Button) {
        if let Some(s) = state.retrieve() {
            self(s, x, y, button);
        }
    }
}

pub(crate) trait MouseMovedCallback<A> {
    fn call(&mut self, state: &mut State, x: f32, y: f32, dx: f32, dy: f32);
}

impl<F> MouseMovedCallback<(f32, f32, f32, f32)> for F
where
    F: FnMut(f32, f32, f32, f32),
{
    fn call(&mut self, _: &mut State, x: f32, y: f32, dx: f32, dy: f32) {
        self(x, y, dx, dy);
    }
}

impl<F, S> MouseMovedCallback<(&mut S, f32, f32, f32, f32)> for F
where
    F: FnMut(&mut S, f32, f32, f32, f32),
    S: 'static,
{
    fn call(&mut self, state: &mut State, x: f32, y: f32, dx: f32, dy: f32) {
        if let Some(s) = state.retrieve() {
            self(s, x, y, dx, dy);
        }
    }
}
