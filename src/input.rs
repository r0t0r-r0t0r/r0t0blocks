use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

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

pub struct Input {
    keys: HashMap<Scancode, Latch>,
}

impl Input {
    pub fn new(keys: &[Scancode]) -> Input {
        Input {
            keys: keys.iter().map(|&x| (x, Latch::new())).collect(),
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

    pub fn is_front_edge(&self, scancode: Scancode) -> bool {
        if let Some(latch) = self.keys.get(&scancode) {
            latch.is_front_edge()
        } else {
            false
        }
    }
}
