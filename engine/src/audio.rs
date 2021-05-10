use sdl2::audio::{AudioCallback, AudioFormatNum};

pub struct Silence;

impl AudioCallback for Silence {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        out.fill(Self::Channel::SILENCE);
    }
}
