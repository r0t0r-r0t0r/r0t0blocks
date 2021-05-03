mod blocks;
mod geometry;
mod input;
mod time;
mod video;

use std::path::Path;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;

use blocks::State;
use input::Input;
use video::ScreenBuffer;

use crate::blocks::create_frames;

struct TimerEvent;

fn main() -> Result<(), String> {
    let scale = 1;
    let tile_count = (30, 30);
    let tile_size = (24, 24);

    sdl2::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "r0t0blocks",
            scale * tile_count.0 * tile_size.0,
            scale * tile_count.1 * tile_size.1,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 255));

    let mut event_pump = sdl_context.event_pump()?;

    let event = sdl_context.event()?;
    event.register_custom_event::<TimerEvent>()?;

    let timer = sdl_context.timer()?;

    let _timer = timer.add_timer(8, Box::from(|| {
        let e = TimerEvent;
        if event.push_custom_event(e).is_ok() {
            8
        } else {
            // todo: notify about error somehow
            0
        }
    }));

    let tileset_surface = sdl2::surface::Surface::load_bmp(Path::new("assets/tileset_24_24.bmp"))?;
    let tileset_texture = texture_creator
        .create_texture_from_surface(&tileset_surface)
        .map_err(|e| e.to_string())?;

    let mut tileset_src_rect = Rect::new(16, 0, tile_size.0, tile_size.1);
    let mut tileset_dst_rect = Rect::new(0, 0, tile_size.0 * scale, tile_size.1 * scale);

    let mut screen_buffer: ScreenBuffer = ScreenBuffer::new(tile_count.0 as usize, tile_count.1 as usize);

    let mut input = Input::new(&[
        Scancode::Escape,
        Scancode::Return,
        Scancode::Space,
        Scancode::Minus,
        Scancode::Equals,
        Scancode::Up,
        Scancode::Left,
        Scancode::Down,
        Scancode::Right,
        Scancode::V,
        Scancode::R,
    ]);

    let update_period = 1.0 / 120.0;
    let mut update_now = Instant::now();

    let mut is_drawing_tick = false;

    let mut is_quit = false;

    // game specific definitions
    let frames = create_frames();
    let mut state = State::new(&frames);

    while !is_quit {
        let event = event_pump.wait_event();
        match event {
            Event::Quit { .. } => {
                is_quit = true;
            }
            e if e.is_user_event() => {
                let _ = e.as_user_event_type::<TimerEvent>()
                    .ok_or("Failed to receive user event")?;

                let new_update_now = Instant::now();
                if (new_update_now - update_now).as_secs_f64() >= update_period {
                    update_now = new_update_now;

                    // update world
                    state.handle_input(&input);

                    input.tick();
                    state.tick();
                }

                if is_drawing_tick {
                    // render chars
                    screen_buffer.clear();

                    state.draw(&mut screen_buffer);

                    canvas.clear();
                    for y in 0..tile_count.1 {
                        for x in 0..tile_count.0 {
                            let chr = screen_buffer.char_at(x as usize, y as usize);

                            tileset_src_rect.set_x(((chr as usize % 16) * tile_size.0 as usize) as i32);
                            tileset_src_rect.set_y(((chr as usize / 16) * tile_size.1 as usize) as i32);

                            tileset_dst_rect.set_x((x * tile_size.0 * scale) as i32);
                            tileset_dst_rect.set_y((y * tile_size.1 * scale) as i32);

                            canvas.copy_ex(
                                &tileset_texture,
                                Some(tileset_src_rect),
                                Some(tileset_dst_rect),
                                0.0,
                                None,
                                false,
                                false,
                            )?;
                        }
                    }
                    canvas.present();
                }

                is_drawing_tick = !is_drawing_tick;
            }
            e => input.on_event(e),
        }
    }

    Ok(())
}
