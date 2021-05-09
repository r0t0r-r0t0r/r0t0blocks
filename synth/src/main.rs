use engine::{run, RunParams};
use engine::base::App;
use engine::input::{Input, Key};
use engine::video::ScreenBuffer;
use std::sync::mpsc;
use sdl2::audio::AudioCallback;

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
            volume: 0.025,
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

struct State {
    tx: mpsc::Sender<Sound>,
}

impl State {
    fn new(tx: mpsc::Sender<Sound>) -> State {
        State {
            tx,
        }
    }
}

impl App for State {
    fn handle_input(&mut self, input: &Input) {
        if input.is_front_edge(Key::Space) {
            self.tx.send(Sound::Beep);
        }
    }

    fn tick(&mut self) {
    }

    fn draw(&self, buf: &mut ScreenBuffer) {
    }
}

fn main() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();

    let mut state = State::new(tx);

    let params = RunParams {
        tileset_path: "assets/tileset_24_24.bmp",
        app_name: "r0t0synth",
        scale: 1,
        width_in_tiles: 30,
        height_in_tiles: 30,
    };

    run(&mut state, params, move |s| Audio::new(s.freq, rx))
}