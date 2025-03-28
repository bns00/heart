use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use super::Scancode;

const KEY_COUNT: usize = Scancode::Unidentified as usize;

struct State {
    keys: [AtomicBool; KEY_COUNT],
}

static STATE: OnceLock<Arc<State>> = OnceLock::new();

pub(crate) fn init() {
    let _ = STATE.set(Arc::new(State {
        keys: [const { AtomicBool::new(false) }; KEY_COUNT],
    }));
}

pub(crate) fn get_key(key: Scancode) -> bool {
    STATE.get().unwrap().keys[key as usize].load(Ordering::Relaxed)
}

pub(crate) fn set_key(key: Scancode, pressed: bool) {
    STATE.get().unwrap().keys[key as usize].store(pressed, Ordering::Relaxed);
}
