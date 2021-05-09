use std::path::Path;
use std::time::Instant;

use sdl2::audio::{AudioSpecDesired, AudioCallback, AudioSpec};
use sdl2::event::Event;
use sdl2::rect::Rect;

use crate::base::App;
use crate::geometry::Point;
use crate::input::Input;
use crate::video::{draw_str, ScreenBuffer};

pub mod base;
pub mod input;
pub mod geometry;
pub mod time;
pub mod video;
pub mod audio;

struct TimerEvent;

pub struct RunParams<'str> {
    pub tileset_path: &'str str,
    pub app_name: &'str str,
    pub scale: u32,
    pub width_in_tiles: u32,
    pub height_in_tiles: u32,
}

pub fn run<A, F, C>(app: &mut A, params: RunParams, audio: F) -> Result<(), String>
    where
        A: App,
        C: AudioCallback,
        F: FnOnce(AudioSpec) -> C,
{
    let scale = params.scale;
    let tile_count = (params.width_in_tiles, params.height_in_tiles);

    sdl2::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");

    let sdl_context = sdl2::init()?;

    let tileset_surface = sdl2::surface::Surface::load_bmp(Path::new(params.tileset_path))?;

    if tileset_surface.width() % 16 != 0 {
        return Err("Tileset width must be multiple of 16".into());
    }
    if tileset_surface.height() % 16 != 0 {
        return Err("Tileset height must be multiple of 16".into());
    }

    let tile_size = (tileset_surface.width() / 16, tileset_surface.height() / 16);

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            params.app_name,
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

    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, move|spec| {
        audio(spec)
    })?;

    device.resume();

    let tileset_texture = texture_creator
        .create_texture_from_surface(&tileset_surface)
        .map_err(|e| e.to_string())?;

    let mut tileset_src_rect = Rect::new(0, 0, tile_size.0, tile_size.1);
    let mut tileset_dst_rect = Rect::new(0, 0, tile_size.0 * scale, tile_size.1 * scale);

    let mut screen_buffer: ScreenBuffer = ScreenBuffer::new(tile_count.0 as usize, tile_count.1 as usize);

    let mut input = Input::new();

    let mut is_drawing_tick = false;

    let mut is_quit = false;

    let mut fps = 0;
    let mut fps_counter = 0;
    let mut ticks_prev = Instant::now();

    while !is_quit {
        let event = event_pump.wait_event();
        match event {
            Event::Quit { .. } => {
                is_quit = true;
            }
            e if e.is_user_event() => {
                let _ = e.as_user_event_type::<TimerEvent>()
                    .ok_or("Failed to receive user event")?;

                // update world
                app.handle_input(&input);

                input.tick();
                app.tick();

                if is_drawing_tick {

                    fps_counter += 1;
                    let now = Instant::now();
                    let delta = (now - ticks_prev).as_secs_f64();
                    if delta >= 1.0 {
                        fps = ((fps_counter as f64) / delta) as i32;
                        fps_counter = 0;
                        ticks_prev = now;
                    }

                    // render chars
                    screen_buffer.clear();

                    app.draw(&mut screen_buffer);

                    draw_str(&mut screen_buffer, Point::new(0, 0), &fps.to_string());

                    canvas.clear();
                    for y in 0..tile_count.1 {
                        for x in 0..tile_count.0 {
                            let chr = screen_buffer.byte_at(x as usize, y as usize);

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