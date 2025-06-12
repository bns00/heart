fn main() {
    heart::build()
        .with_load(State::new)
        .with_update(State::update)
        .with_draw(State::draw)
        .run();
}

struct State {
    clock: u32,
}

impl State {
    fn new() -> Self {
        Self { clock: 0 }
    }

    fn update(&mut self) {
        self.clock += 1;
    }

    fn draw(&mut self) {
        let clock = self.clock as f32 / 30.0;
        draw_square(clock);
        draw_square(clock + std::f32::consts::PI * 0.5);
        draw_square(clock + std::f32::consts::PI);
        draw_square(clock + std::f32::consts::PI * 1.5);
    }
}

fn draw_square(t: f32) {
    heart::graphics::set_color(t.sin() * 0.5 + 0.5, t.cos() * 0.5 + 0.5, 0.5, 1.0);
    heart::graphics::rectangle(
        350.0 + t.sin() * 100.0,
        250.0 + t.cos() * 100.0,
        100.0,
        100.0,
    );
}
