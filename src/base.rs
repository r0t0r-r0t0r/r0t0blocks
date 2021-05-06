use crate::input::Input;
use crate::video::ScreenBuffer;

pub type Number = i32;

pub trait App {
    fn handle_input(&mut self, input: &Input);
    fn tick(&mut self);
    fn draw(&self, buf: &mut ScreenBuffer);
}
