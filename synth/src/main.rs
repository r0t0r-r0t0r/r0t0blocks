use engine::{run, RunParams};
use engine::base::App;
use engine::input::{Input, Key};
use engine::video::ScreenBuffer;
use std::sync::mpsc;
use sdl2::audio::{AudioCallback, AudioFormatNum};
use std::f32::consts::PI;
use std::time::Instant;
use std::cmp::max;
use std::collections::VecDeque;

pub trait Sound {
    fn render(&self, tick: u64) -> f32;
}

fn angular(frequency: f32) -> f32 {
    2.0 * PI * frequency
}

pub struct Sine {
    sample_rate: f32,
    start: Option<(u64, f32)>,
    stop_tick: Option<u64>,
}

impl Sine {
    pub fn new(sample_rate: f32) -> Sine {
        Sine {
            sample_rate,
            start: None,
            stop_tick: None,
        }
    }

    pub fn start_at(&mut self, start_tick: u64, frequency: f32) {
        self.start = Some((start_tick, frequency));
        self.stop_tick = None;
    }

    pub fn stop_at(&mut self, stop_tick: u64) {
        self.stop_tick = Some(stop_tick);
    }
}

impl Sound for Sine {
    fn render(&self, tick: u64) -> f32 {
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
    main_frequency: u64,
    audio_frequency: i64,
    start_main_tick: Option<u64>,
    audio_tick: u64,
    rx: Option<mpsc::Receiver<SoundMessage>>,

    sine: Sine,

    offset: i64,
    future_messages: Option<VecDeque<SoundMessage>>,
}

impl Audio {
    pub fn new(main_frequency: u64, audio_frequency: i64, rx: mpsc::Receiver<SoundMessage>) -> Audio {
        Audio {
            main_frequency,
            audio_frequency,
            audio_tick: 0,
            start_main_tick: None,
            rx: Some(rx),

            sine: Sine::new(audio_frequency as f32),

            offset: 0,
            future_messages: Some(VecDeque::with_capacity(5)),
        }
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut last_processed_tick = None;
        let mut previous_tick = None;
        let mut future_messages = VecDeque::with_capacity(5);

        let postponed_messages = self.future_messages.take();
        let rx = self.rx.take();

        let mut process_message = |msg| {
            match msg {
                SoundMessage::Key{tick: main_tick, is_pressed, elapsed_milliseconds, note} => {
                    if elapsed_milliseconds as f32 / 1000.0 <= 1.0 / self.audio_frequency as f32 * out.len() as f32 {
                        println!("{}", elapsed_milliseconds as f32 / 1000.0);
                    }
                    let elapsed_milliseconds = 0;
                    let audio_tick_delta = max(0, elapsed_milliseconds) * self.audio_frequency / 1000;
                    let audio_tick = max(self.audio_tick as i64, previous_tick.unwrap_or((self.audio_tick as i64) - self.offset) + audio_tick_delta);

                    if (audio_tick as usize) >= (self.audio_tick as usize + out.len()) {
                        future_messages.push_back(SoundMessage::Key { tick: main_tick, is_pressed, elapsed_milliseconds, note });
                    } else {
                        if is_pressed {
                             self.sine.start_at(audio_tick as u64, frequency(note));
                        } else {
                            self.sine.stop_at(audio_tick as u64);
                        }

                        last_processed_tick = Some(audio_tick);
                    }

                    previous_tick = Some(audio_tick);
                },
                SoundMessage::Stat => {}
            }
        };

        if let Some(postponed_messages) = postponed_messages {
            for message in postponed_messages.into_iter() {
                process_message(message);
            }
        }
        let mut counter = 0;
        if let Some(rx) = rx {
            for message in rx.try_iter() {
                process_message(message);
                counter += 1;
            }
            self.rx = Some(rx);
        }
        if counter > 1 {
            println!("Events: {}", counter);
        }

        self.future_messages = Some(future_messages);

        for (i, y) in out.iter_mut().enumerate() {
            let tick = self.audio_tick + i as u64;

            *y = self.sine.render(tick);
        }

        self.offset = last_processed_tick.map_or(0, |x| (out.len() as i64) - (x - self.audio_tick as i64));
        self.audio_tick += out.len() as u64;
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
        tick: u64,
        is_pressed: bool,
        elapsed_milliseconds: i64,
        note: Note,
    },
    Stat,
}

struct State {
    tx: mpsc::Sender<SoundMessage>,
    tick: u64,
    last_sound_instant: Option<Instant>,
}

impl State {
    fn new(tx: mpsc::Sender<SoundMessage>) -> State {
        State {
            tx,
            tick: 0,
            last_sound_instant: None,
        }
    }
}

impl State {
    fn hold_key(&mut self, note: Note) {
        let now = Instant::now();
        let elapsed_milliseconds = self.last_sound_instant.map_or(0, |x| (now - x).as_millis() as i64);
        self.tx.send(SoundMessage::Key {
            tick: self.tick,
            is_pressed: true,
            elapsed_milliseconds,
            note,
        });
        self.last_sound_instant = Some(now);
    }

    fn release_key(&mut self, note: Note) {
        let now = Instant::now();
        let elapsed_milliseconds = self.last_sound_instant.map_or(0, |x| (now - x).as_millis() as i64);
        self.tx.send(SoundMessage::Key {
            tick: self.tick,
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

        if input.is_front_edge(Key::S) {
            self.tx.send(SoundMessage::Stat);
        }
    }

    fn tick(&mut self) {
        self.tick += 1;
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

    // todo: eliminate main frequency hardcode
    run(&mut state, params, move |s| Audio::new(123, s.freq as i64, rx))
}