use crossbeam::channel::IntoIter;
use heart::{self, *};

fn main() {
    heart::build()
        .with_load(State::new)
        .with_update(State::update)
        .with_draw(State::draw)
        .run();
}

struct State {
    sprite: graphics::Sprite,
    instances: Vec<Instance>,
}

impl State {
    fn new() -> Self {
        Self {
            sprite: graphics::create_sprite(include_png!("creature.png")),
            instances: vec![
                Instance {
                    pos_x: 336.0,
                    pos_y: 150.0,
                    vel_x: -7.0,
                    vel_y: -6.0,
                },
                Instance {
                    pos_x: 464.0,
                    pos_y: 150.0,
                    vel_x: 8.0,
                    vel_y: -5.0,
                },
                Instance {
                    pos_x: 336.0,
                    pos_y: 314.0,
                    vel_x: -4.0,
                    vel_y: 9.0,
                },
                Instance {
                    pos_x: 464.0,
                    pos_y: 314.0,
                    vel_x: 6.0,
                    vel_y: 7.0,
                },
            ],
        }
    }

    fn update(&mut self) {
        for i in 0..self.instances.len() {
            for j in i + 1..self.instances.len() {
                let a = &self.instances[i];
                let b = &self.instances[j];
                if a.pos_x > b.pos_x - 32.0
                    && a.pos_x < b.pos_x + 32.0
                    && a.pos_y > b.pos_y - 32.0
                    && a.pos_y < b.pos_y + 32.0
                {
                    let a_x = a.vel_x;
                    let a_y = a.vel_y;
                    let b_x = b.vel_x;
                    let b_y = b.vel_y;
                    self.instances[i].vel_x = b_x;
                    self.instances[i].vel_y = b_y;
                    self.instances[j].vel_x = a_x;
                    self.instances[j].vel_y = a_y;
                }
            }
            let instance = &mut self.instances[i];
            if instance.pos_x < 0.0 || instance.pos_x > 736.0 {
                instance.vel_x = -instance.vel_x;
            }
            if instance.pos_y < 0.0 || instance.pos_y > 536.0 {
                instance.vel_y = -instance.vel_y;
            }
            instance.pos_x += instance.vel_x;
            instance.pos_y += instance.vel_y;
        }
    }

    fn draw(&mut self) {
        for instance in &self.instances {
            graphics::drawable(&self.sprite, instance.pos_x, instance.pos_y);
        }
    }
}

struct Instance {
    pos_x: f32,
    pos_y: f32,
    vel_x: f32,
    vel_y: f32,
}
