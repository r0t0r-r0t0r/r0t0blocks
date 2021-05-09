use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

struct Latch {
    prev: bool,
    curr: bool,
}

impl Latch {
    fn new() -> Latch {
        Latch {
            prev: false,
            curr: false,
        }
    }

    fn set(&mut self, value: bool) {
        self.curr = value;
    }

    fn is_front_edge(&self) -> bool {
        self.curr && !self.prev
    }

    fn is_back_edge(&self) -> bool {
        self.prev && !self.curr
    }

    fn tick(&mut self) {
        self.prev = self.curr;
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, EnumIter)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Escape,
    Return,
    Space,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equals,
}

impl From<Key> for Scancode {
    fn from(key: Key) -> Self {
        match key {
            Key::Up => Scancode::Up,
            Key::Down => Scancode::Down,
            Key::Left => Scancode::Left,
            Key::Right => Scancode::Right,
            Key::Escape => Scancode::Escape,
            Key::Return => Scancode::Return,
            Key::Space => Scancode::Space,
            Key::A => Scancode::A,
            Key::B => Scancode::B,
            Key::C => Scancode::C,
            Key::D => Scancode::D,
            Key::E => Scancode::E,
            Key::F => Scancode::F,
            Key::G => Scancode::G,
            Key::H => Scancode::H,
            Key::I => Scancode::I,
            Key::J => Scancode::J,
            Key::K => Scancode::K,
            Key::L => Scancode::L,
            Key::M => Scancode::M,
            Key::N => Scancode::N,
            Key::O => Scancode::O,
            Key::P => Scancode::P,
            Key::Q => Scancode::Q,
            Key::R => Scancode::R,
            Key::S => Scancode::S,
            Key::T => Scancode::T,
            Key::U => Scancode::U,
            Key::V => Scancode::V,
            Key::W => Scancode::W,
            Key::X => Scancode::X,
            Key::Y => Scancode::Y,
            Key::Z => Scancode::Z,
            Key::Num1 => Scancode::Num1,
            Key::Num2 => Scancode::Num2,
            Key::Num3 => Scancode::Num3,
            Key::Num4 => Scancode::Num4,
            Key::Num5 => Scancode::Num5,
            Key::Num6 => Scancode::Num6,
            Key::Num7 => Scancode::Num7,
            Key::Num8 => Scancode::Num8,
            Key::Num9 => Scancode::Num9,
            Key::Num0 => Scancode::Num0,
            Key::Minus => Scancode::Minus,
            Key::Equals => Scancode::Equals,
        }
    }
}

pub struct Input {
    keys: HashMap<Scancode, Latch>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            keys: Key::iter().map(|x| (x.into(), Latch::new())).collect(),
        }
    }

    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(latch) = self.keys.get_mut(&scancode) {
                    latch.set(true);
                }
            }
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(latch) = self.keys.get_mut(&scancode) {
                    latch.set(false);
                }
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        for latch in self.keys.values_mut() {
            latch.tick();
        }
    }

    pub fn is_front_edge(&self, key: Key) -> bool {
        let scancode = key.into();
        if let Some(latch) = self.keys.get(&scancode) {
            latch.is_front_edge()
        } else {
            false
        }
    }

    pub fn is_back_edge(&self, key: Key) -> bool {
        let scancode = key.into();
        if let Some(latch) = self.keys.get(&scancode) {
            latch.is_back_edge()
        } else {
            false
        }
    }
}
