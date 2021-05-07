use r0t0blocks::blocks::{create_frames, State};
use engine::run;

fn main() -> Result<(), String> {
    let frames = create_frames();
    let mut state = State::new(&frames);

    run(&mut state, "assets/tileset_24_24.bmp")
}