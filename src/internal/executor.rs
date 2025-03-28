pub(crate) mod callbacks;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{keyboard, mouse};

pub(crate) struct Config {
    pub(crate) tick_duration: Duration,
    pub(crate) load: Vec<Box<dyn FnMut(&mut State)>>,
    pub(crate) update: Vec<Box<dyn FnMut(&mut State)>>,
    pub(crate) draw: Vec<Box<dyn FnMut(&mut State)>>,
    pub(crate) key_pressed: Vec<Box<dyn FnMut(&mut State, keyboard::Scancode)>>,
    pub(crate) key_released: Vec<Box<dyn FnMut(&mut State, keyboard::Scancode)>>,
    pub(crate) mouse_pressed: Vec<Box<dyn FnMut(&mut State, f32, f32, mouse::Button)>>,
    pub(crate) mouse_released: Vec<Box<dyn FnMut(&mut State, f32, f32, mouse::Button)>>,
    pub(crate) mouse_moved: Vec<Box<dyn FnMut(&mut State, f32, f32, f32, f32)>>,
    // pub(crate) wheel_moved: Vec<Box<dyn FnMut(&mut State, f32)>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_duration: calculate_tick_duration(60),
            load: Vec::new(),
            update: Vec::new(),
            draw: Vec::new(),
            key_pressed: Vec::new(),
            key_released: Vec::new(),
            mouse_pressed: Vec::new(),
            mouse_released: Vec::new(),
            mouse_moved: Vec::new(),
            // wheel_moved: Vec::new(),
        }
    }
}

pub(crate) struct State {
    storage: HashMap<TypeId, Box<dyn Any>>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub(crate) fn store<T>(&mut self, value: T)
    where
        T: Any,
    {
        self.storage.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub(crate) fn retrieve<T>(&mut self) -> Option<&mut T>
    where
        T: Any,
    {
        self.storage.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }
}

pub(crate) fn calculate_tick_duration(tick_rate: u64) -> Duration {
    Duration::from_nanos(1_000_000_000 / tick_rate)
}

struct Clock {
    last: Instant,
    collected: Duration,
}

impl Clock {
    fn new() -> Self {
        Self {
            last: Instant::now(),
            collected: Duration::ZERO,
        }
    }

    fn tick(&mut self, duration: Duration) -> bool {
        let now = Instant::now();
        self.collected += now - self.last;
        self.last = now;
        if self.collected > duration {
            self.collected -= duration;
            if self.collected > duration {
                self.collected = Duration::ZERO;
            }
            true
        } else {
            false
        }
    }
}

pub(crate) struct Executor {
    config: Config,
    state: State,
    clock: Clock,
}

impl Executor {
    pub(crate) fn new(config: Config) -> Self {
        Self {
            config,
            state: State::new(),
            clock: Clock::new(),
        }
    }

    pub(crate) fn load(&mut self) {
        self.config
            .load
            .iter_mut()
            .for_each(|load| load(&mut self.state));
    }

    pub(crate) fn draw(&mut self) {
        self.config
            .draw
            .iter_mut()
            .for_each(|draw| draw(&mut self.state));
    }

    pub(crate) fn update(&mut self) {
        if self.clock.tick(self.config.tick_duration) {
            self.config
                .update
                .iter_mut()
                .for_each(|update| update(&mut self.state));
        }
    }

    pub(crate) fn key_pressed(&mut self, scancode: keyboard::Scancode) {
        self.config
            .key_pressed
            .iter_mut()
            .for_each(|key_pressed| key_pressed(&mut self.state, scancode));
    }

    pub(crate) fn key_released(&mut self, scancode: keyboard::Scancode) {
        self.config
            .key_released
            .iter_mut()
            .for_each(|key_released| key_released(&mut self.state, scancode));
    }

    pub(crate) fn mouse_pressed(&mut self, x: f32, y: f32, button: mouse::Button) {
        self.config
            .mouse_pressed
            .iter_mut()
            .for_each(|mouse_pressed| mouse_pressed(&mut self.state, x, y, button));
    }

    pub(crate) fn mouse_released(&mut self, x: f32, y: f32, button: mouse::Button) {
        self.config
            .mouse_released
            .iter_mut()
            .for_each(|mouse_released| mouse_released(&mut self.state, x, y, button));
    }

    pub(crate) fn mouse_moved(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        self.config
            .mouse_moved
            .iter_mut()
            .for_each(|mouse_moved| mouse_moved(&mut self.state, x, y, dx, dy));
    }

    // pub(crate) fn wheel_moved(&mut self, delta: f32) {
    //     self.config
    //         .wheel_moved
    //         .iter_mut()
    //         .for_each(|wheel_moved| wheel_moved(&mut self.state, delta));
    // }
}
