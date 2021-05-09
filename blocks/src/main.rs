use engine::run;
use r0t0blocks::blocks::State;
use r0t0blocks::tetromino::create_frames;

fn main() -> Result<(), String> {
    let frames = create_frames();
    let mut state = State::new(&frames);

    run(&mut state, "assets/tileset_24_24.bmp")
}