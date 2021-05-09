use engine::{run, RunParams};
use r0t0blocks::blocks::State;
use r0t0blocks::tetromino::create_frames;
use engine::audio::Silence;

fn main() -> Result<(), String> {
    let frames = create_frames();
    let mut state = State::new(&frames);

    let params = RunParams {
        tileset_path: "assets/tileset_24_24.bmp",
        app_name: "r0t0blocks",
        scale: 1,
        width_in_tiles: 22,
        height_in_tiles: 24,
    };

    run(&mut state, params, |_| Silence)
}