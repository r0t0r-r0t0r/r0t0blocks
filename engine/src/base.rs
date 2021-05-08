use crate::input::Input;
use crate::video::ScreenBuffer;
use std::sync::mpsc::Sender;
use crate::audio::Sound;

pub type Number = i32;

pub trait App {
    fn init_audio(&mut self, tx: Sender<Sound>);
    fn handle_input(&mut self, input: &Input);
    fn tick(&mut self);
    fn draw(&self, buf: &mut ScreenBuffer);
}
