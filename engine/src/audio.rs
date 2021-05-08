use sdl2::audio::AudioCallback;
use std::sync::mpsc;

pub struct Audio {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    rx: mpsc::Receiver<Sound>,
}

impl Audio {
    pub fn new(freq: i32, rx: mpsc::Receiver<Sound>) -> Audio {
        Audio {
            phase_inc: 440.0 / freq as f32,
            phase: 0.0,
            volume: 0.25,
            rx,
        }
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if let Ok(sound) = self.rx.try_recv() {
            // Generate a square wave
            for x in out.iter_mut() {
                *x = if self.phase <= 0.5 {
                    self.volume
                } else {
                    -self.volume
                };
                self.phase = (self.phase + self.phase_inc) % 1.0;
            }
        } else {
            out.fill(0.0);
        }
    }
}

pub enum Sound {
    Beep,
}
