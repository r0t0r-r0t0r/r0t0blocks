use engine::{run, RunParams};
use engine::base::App;
use engine::input::{Input, Key};
use engine::video::ScreenBuffer;
use std::sync::mpsc;
use sdl2::audio::AudioCallback;
use std::f32::consts::PI;
use std::time::Instant;
use std::cmp::min;

pub trait Sound {
    fn render(&self, tick: i64) -> f32;
}

fn angular(frequency: f32) -> f32 {
    2.0 * PI * frequency
}

pub struct Sine {
    sample_rate: f32,
    start: Option<(i64, f32)>,
    stop_tick: Option<i64>,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Sine {
        Sine {
            sample_rate,
            start: None,
            stop_tick: None,
        }
    }

    pub fn start_at(&mut self, start_tick: i64, frequency: f32) {
        self.start = Some((start_tick, frequency));
        self.stop_tick = None;
    }

    pub fn stop_at(&mut self, stop_tick: i64) {
        self.stop_tick = Some(stop_tick);
    }
}

impl Sound for Sine {
    fn render(&self, tick: i64) -> f32 {
        if let Some((start_tick, frequency)) = self.start {
            if tick >= start_tick {
                if self.stop_tick.map_or(true, |x| tick < x) {
                    let time = (tick - start_tick) as f32 / self.sample_rate;
                    (angular(frequency) * time).sin() * 0.25
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

pub struct Audio {
    sample_rate: i64,
    major_tick: i64,
    rx: mpsc::Receiver<SoundMessage>,

    sine: Sine,
}

impl Audio {
    pub fn new(sample_rate: i64, rx: mpsc::Receiver<SoundMessage>) -> Audio {
        Audio {
            sample_rate,
            major_tick: 0,
            rx,

            sine: Sine::new(sample_rate as f32),
        }
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut previous_tick = None;
        let next_major_tick = self.major_tick + out.len() as i64;

        for msg in self.rx.try_iter() {
            match msg {
                SoundMessage::Key{is_pressed, elapsed_milliseconds, note} => {
                    let elapsed_ticks = elapsed_milliseconds * self.sample_rate / 1000;
                    let audio_tick = min(next_major_tick - 1, previous_tick.map_or(self.major_tick, |x| x + elapsed_ticks));

                    if is_pressed {
                        self.sine.start_at(audio_tick, frequency(note));
                    } else {
                        self.sine.stop_at(audio_tick);
                    }

                    previous_tick = Some(audio_tick);
                },
            }
        }

        for (i, y) in out.iter_mut().enumerate() {
            let tick = self.major_tick + i as i64;

            *y = self.sine.render(tick);
        }

        self.major_tick = next_major_tick;
    }
}

pub enum Note {
    A,
    Asharp,
    B,
    C,
    Csharp,
    D,
    Dsharp,
    E,
    F,
    Fsharp,
    G,
    Gsharp,
}

fn frequency(note: Note) -> f32 {
    match note {
        Note::C => 261.63,
        Note::Csharp => 277.18,
        Note::D => 293.66,
        Note::Dsharp => 311.13,
        Note::E => 329.63,
        Note::F => 349.23,
        Note::Fsharp => 369.99,
        Note::G => 392.00,
        Note::Gsharp => 415.30,
        Note::A => 440.00,
        Note::Asharp => 466.16,
        Note::B => 493.88,
    }
}

pub enum SoundMessage {
    Key {
        is_pressed: bool,
        elapsed_milliseconds: i64,
        note: Note,
    },
}

struct State {
    tx: mpsc::Sender<SoundMessage>,
    last_sound_instant: Option<Instant>,
}

impl State {
    fn new(tx: mpsc::Sender<SoundMessage>) -> State {
        State {
            tx,
            last_sound_instant: None,
        }
    }
}

impl State {
    fn hold_key(&mut self, note: Note) {
        let now = Instant::now();
        let elapsed_milliseconds = self.last_sound_instant.map_or(0, |x| (now - x).as_millis() as i64);
        let _ = self.tx.send(SoundMessage::Key {
            is_pressed: true,
            elapsed_milliseconds,
            note,
        });
        self.last_sound_instant = Some(now);
    }

    fn release_key(&mut self, note: Note) {
        let now = Instant::now();
        let elapsed_milliseconds = self.last_sound_instant.map_or(0, |x| (now - x).as_millis() as i64);
        let _ = self.tx.send(SoundMessage::Key {
            is_pressed: false,
            elapsed_milliseconds,
            note,
        });
        self.last_sound_instant = Some(now);
    }
}

impl App for State {
    fn handle_input(&mut self, input: &Input) {
        if input.is_front_edge(Key::Space) {
            self.hold_key(Note::C);
        }

        if input.is_back_edge(Key::Space) {
            self.release_key(Note::C);
        }
    }

    fn tick(&mut self) {
    }

    fn draw(&self, _buf: &mut ScreenBuffer) {
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

    run(&mut state, params, move |s| Audio::new(s.freq as i64, rx))
}