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

#[derive(Clone)]
pub struct Sine {
    sample_rate: f32,
    start: Option<(i64, f32)>,
    line: Line,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Sine {
        Sine {
            sample_rate,
            start: None,
            line: Line::new(0, 0, 0.0, 0.0),
        }
    }

    pub fn start_at(&mut self, start_tick: i64, frequency: f32) {
        self.start = Some((start_tick, frequency));
        self.line = Line::new(start_tick, start_tick + (self.sample_rate * 0.05) as i64, 0.0, 1.0);
    }

    pub fn stop_at(&mut self, stop_tick: i64) {
        self.line = Line::new(stop_tick, stop_tick + (self.sample_rate * 0.05) as i64, 1.0, 0.0);
    }
}

impl Sound for Sine {
    fn render(&self, tick: i64) -> f32 {
        if let Some((start_tick, frequency)) = self.start {
            if tick >= start_tick {
                let time = (tick - start_tick) as f32 / self.sample_rate;
                let mut value = 0.0;
                for i in 0..4 {
                    value += (angular(frequency * i as f32) * time).sin();
                }
                value * 0.1 * self.line.render(tick)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

#[derive(Clone)]
struct Line {
    start_tick: i64,
    stop_tick: i64,

    start_value: f32,
    stop_value: f32,
}

impl Line {
    fn new(
        start_tick: i64,
        stop_tick: i64,
        start_value: f32,
        stop_value: f32,
    ) -> Line {
        Line {
            start_tick,
            stop_tick,
            start_value,
            stop_value,
        }
    }
}

impl Sound for Line {
    fn render(&self, tick: i64) -> f32 {
        if tick < self.start_tick {
            self.start_value
        } else if tick >= self.stop_tick {
            self.stop_value
        } else {
            let width = self.stop_tick - self.start_tick;
            let height = self.stop_value - self.start_value;

            let progress = (tick - self.start_tick) as f32 / (width - 1) as f32;
            let value = self.start_value + progress * height;

            value
        }
    }
}

pub struct Audio {
    sample_rate: i64,
    major_tick: i64,
    rx: mpsc::Receiver<SoundMessage>,

    oscillators: Vec<Sine>,
}

impl Audio {
    pub fn new(sample_rate: i64, rx: mpsc::Receiver<SoundMessage>) -> Audio {
        Audio {
            sample_rate,
            major_tick: 0,
            rx,

            oscillators: vec![Sine::new(sample_rate as f32); 12],
        }
    }

    fn index(note: Note) -> usize {
        match note {
            Note::C => 0,
            Note::Csharp => 1,
            Note::D => 2,
            Note::Dsharp => 3,
            Note::E => 4,
            Note::F => 5,
            Note::Fsharp => 6,
            Note::G => 7,
            Note::Gsharp => 8,
            Note::A => 9,
            Note::Asharp => 10,
            Note::B => 11,
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
                        self.oscillators[Self::index(note)].start_at(audio_tick, frequency(note));
                    } else {
                        self.oscillators[Self::index(note)].stop_at(audio_tick);
                    }

                    previous_tick = Some(audio_tick);
                },
            }
        }

        for (i, y) in out.iter_mut().enumerate() {
            let tick = self.major_tick + i as i64;

            *y = 0.0;
            for osc in self.oscillators.iter() {
                *y += osc.render(tick);
            }
        }

        self.major_tick = next_major_tick;
    }
}

#[derive(Copy, Clone)]
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

    fn note_by_key(key: Key) -> Option<Note> {
        match key {
            Key::A => Some(Note::C),
            Key::W => Some(Note::Csharp),
            Key::S => Some(Note::D),
            Key::E => Some(Note::Dsharp),
            Key::D => Some(Note::E),
            Key::F => Some(Note::F),
            Key::T => Some(Note::Fsharp),
            Key::G => Some(Note::G),
            Key::Y => Some(Note::Gsharp),
            Key::H => Some(Note::A),
            Key::U => Some(Note::Asharp),
            Key::J => Some(Note::B),
            _ => None,
        }
    }
}

impl App for State {
    fn handle_input(&mut self, input: &Input) {
        static KEYS: [Key; 12] = [
            Key::A,
            Key::W,
            Key::S,
            Key::E,
            Key::D,
            Key::F,
            Key::T,
            Key::G,
            Key::Y,
            Key::H,
            Key::U,
            Key::J,
        ];

        for key in KEYS.iter().copied() {
            if let Some(note) = Self::note_by_key(key) {
                if input.is_front_edge(key) {
                    self.hold_key(note);
                }

                if input.is_back_edge(key) {
                    self.release_key(note);
                }
            }
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