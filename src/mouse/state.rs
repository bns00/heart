use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use super::Button;

struct State {
    buttons: [AtomicBool; 3],
    x: AtomicU32,
    y: AtomicU32,
}

static STATE: OnceLock<Arc<State>> = OnceLock::new();

pub(crate) fn init() {
    let _ = STATE.set(Arc::new(State {
        buttons: [const { AtomicBool::new(false) }; 3],
        x: AtomicU32::new(f32::NAN.to_bits()),
        y: AtomicU32::new(f32::NAN.to_bits()),
    }));
}

pub(crate) fn get_button(button: Button) -> bool {
    STATE.get().unwrap().buttons[button as usize].load(Ordering::Relaxed)
}

pub(crate) fn set_button(button: Button, pressed: bool) {
    STATE.get().unwrap().buttons[button as usize].store(pressed, Ordering::Relaxed);
}

pub(crate) fn get_position() -> (f32, f32) {
    let state = STATE.get().unwrap();
    (
        f32::from_bits(state.x.load(Ordering::Relaxed)),
        f32::from_bits(state.y.load(Ordering::Relaxed)),
    )
}

pub(crate) fn set_position(x: f32, y: f32) {
    let state = STATE.get().unwrap();
    state.x.store(x.to_bits(), Ordering::Relaxed);
    state.y.store(y.to_bits(), Ordering::Relaxed);
}
