mod blocks;

use std::path::Path;

use crate::blocks::{create_frames, Number, Tetromino, Field, Frame, Point};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use std::collections::{HashMap};
use std::time::Instant;
use std::cmp::{max, min};
use std::iter;
use fastrand::Rng;

struct Timer {
    period: Number,
    current: Number,
}

impl Timer {
    pub fn new(period: Number) -> Timer {
        Timer {
            period,
            current: period + 1,
        }
    }

    pub fn tick(&mut self) {
        if self.current <= self.period {
            self.current += 1;
        }
    }

    pub fn is_triggered(&self) -> bool {
        self.current == self.period
    }

    pub fn is_not_triggered_yet(&self) -> bool {
        self.current < self.period
    }

    pub fn is_started(&self) -> bool {
        self.is_not_triggered_yet() || self.is_triggered()
    }

    pub fn start(&mut self) {
        self.current = 0;
    }

    pub fn stop(&mut self) {
        self.current = self.period + 1;
    }
}

struct BlinkAnimation {
    timer: Timer,
    changes_remain: Number,
    show: bool,
}

impl BlinkAnimation {
    pub fn new() -> BlinkAnimation {
        BlinkAnimation {
            timer: Timer::new(15),
            changes_remain: 0,
            show: true,
        }
    }

    pub fn start(&mut self) {
        self.changes_remain = 6;
        self.show = false;
        self.timer.start();
    }

    pub fn tick(&mut self) {
        self.timer.tick();

        if self.timer.is_triggered() {
            self.show = !self.show;
            self.changes_remain -= 1;

            if self.changes_remain > 0 {
                self.timer.start();
            } else {
                self.show = true;
            }
        }
    }

    pub fn is_show(&self) -> bool {
        self.show
    }

    pub fn is_triggered(&self) -> bool {
        self.changes_remain == 0 && self.timer.is_triggered()
    }
    pub fn is_not_triggered_yet(&self) -> bool {
        self.changes_remain != 0 || self.timer.is_not_triggered_yet()
    }
    pub fn is_started(&self) -> bool {
        self.is_not_triggered_yet() || self.is_triggered()
    }
}

struct Latch {
    prev: bool,
    curr: bool,
}

impl Latch {
    fn new() -> Latch {
        Latch {
            prev: false,
            curr: false,
        }
    }

    fn set(&mut self, value: bool) {
        self.curr = value;
    }

    fn is_front_edge(&self) -> bool {
        self.curr && !self.prev
    }

    fn tick(&mut self) {
        self.prev = self.curr;
    }
}

struct ScreenBuffer {
    chars: Vec<u8>,
    width: usize,
    height: usize,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize) -> ScreenBuffer {
        ScreenBuffer {
            chars: vec![0; width * height],
            width,
            height,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);

        (y * self.width + x) as usize
    }

    fn clear(&mut self) {
        self.chars.fill(0);
    }
}

fn draw_chars(buf: &mut ScreenBuffer, p: Point, s: &[u8]) {
    let Point { x, y } = p;
    if y >= 0 && y < buf.height as Number {
        if x < buf.width as Number && x + s.len() as Number >= 0 {
            let clipped_start_x = max(x, 0);
            let clipped_end_x = min(x + s.len() as Number, buf.width as Number);
            let slice_start = clipped_start_x - x;
            let slice_end = clipped_end_x - x;
            let index = buf.index(clipped_start_x as usize, y as usize);

            buf.chars[index..(index + (clipped_end_x - clipped_start_x) as usize)].copy_from_slice(&s[slice_start as usize..slice_end as usize]);
        }
    }
}

fn draw_rect(buf: &mut ScreenBuffer, p: Point, width: Number, height: Number, char: &[u8]) {
    if width >= 2 && height >= 2 {
        let horizontal_line = iter::repeat(char[0]).take(width as usize).collect::<Vec<_>>();
        draw_chars(buf, p, &horizontal_line);
        draw_chars(buf, p.add_y(height as Number - 1), &horizontal_line);
        for j in p.y + 1..p.y + height as Number - 1 {
            draw_chars(buf, p.with_y(j), char);
            draw_chars(buf, p.with_y(j).add_x(width as Number - 1), char);
        }
    }
}

struct Input {
    keys: HashMap<Scancode, Latch>,
}

impl Input {
    pub fn new(keys: &[Scancode]) -> Input {
        Input {
            keys: keys.iter().map(|&x| (x, Latch::new())).collect(),
        }
    }

    pub fn on_event(&mut self, event: Event) {
        match event {
            Event::KeyDown {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(latch) = self.keys.get_mut(&scancode) {
                    latch.set(true);
                }
            }
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(latch) = self.keys.get_mut(&scancode) {
                    latch.set(false);
                }
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        for latch in self.keys.values_mut() {
            latch.tick();
        }
    }

    pub fn is_front_edge(&self, scancode: Scancode) -> bool {
        if let Some(latch) = self.keys.get(&scancode) {
            latch.is_front_edge()
        } else {
            false
        }
    }
}

struct TimerEvent;

struct State<'frame> {
    tetrominos: [Tetromino<'frame>; 7],
    curr_frame: usize,
    curr_tet_index: usize,
    next_tet_index: usize,
    field: Field,
    field_pos: Point,
    pub tet_pos: Point,
    fall_timer: Timer,
    flashing_animation: BlinkAnimation,
    rng: Rng,
}

impl<'frame> State<'frame> {
    pub fn spawn_pos(field_pos: Point) -> Point {
        let tet_x: Number = field_pos.x + 1 + (Field::width() - Frame::width()) / 2;
        let tet_y: Number = field_pos.y;

        Point::new(tet_x, tet_y)
    }

    pub fn new(frames: &'frame [Vec<Frame>; 7]) -> State {
        let tetrominos = [
            Tetromino::new(&frames[0]),
            Tetromino::new(&frames[1]),
            Tetromino::new(&frames[2]),
            Tetromino::new(&frames[3]),
            Tetromino::new(&frames[4]),
            Tetromino::new(&frames[5]),
            Tetromino::new(&frames[6]),
        ];

        let field_pos = Point::new(3, 3);

        let mut fall_timer = Timer::new(120);
        fall_timer.start();

        let mut field_line_timer = Timer::new(15);
        field_line_timer.start();

        //let rng = Rng::with_seed(42);
        let rng = Rng::new();
        let curr_tet_index = rng.usize(0..7);
        let next_tet_index = rng.usize(0..7);

        State {
            tetrominos,
            curr_frame: 0,
            curr_tet_index,
            next_tet_index,
            field: Field::new(),
            field_pos,
            tet_pos: Self::spawn_pos(field_pos),
            fall_timer,
            flashing_animation: BlinkAnimation::new(),
            rng,
        }
    }

    pub fn next_tetromino(&mut self) {
        self.curr_tet_index = (self.curr_tet_index + 1) % self.tetrominos.len();
        self.curr_frame = 0;
    }

    pub fn prev_tetromino(&mut self) {
        self.curr_tet_index = (self.tetrominos.len() + self.curr_tet_index - 1) % self.tetrominos.len();
        self.curr_frame = 0;
    }

    pub fn current_frame(&self) -> &'frame Frame {
        self.tetrominos[self.curr_tet_index].frames[self.curr_frame]
    }

    pub fn translate_tet_to_field_pos(&self, p: Point) -> Point {
        p - self.field_pos - Point::new(1, 1)
    }

    pub fn copy_frame(&mut self) {
        let pos = self.translate_tet_to_field_pos(self.tet_pos);
        let curr_frame = self.current_frame();
        self.field.copy_frame(curr_frame, pos);
    }

    pub fn is_collide(&self, frame: &'frame Frame, p: Point) -> bool {
        self.field.is_collide(frame, p)
    }

    pub fn clean_filled_lines(&mut self) {
        self.field.clean_filled_lines();
    }

    pub fn draw(&self, buf: &mut ScreenBuffer) {
        draw_rect(buf, self.field_pos, Field::width() + 2, Field::height() + 2, b"+");

        for y in 0..Field::height() {
            let pos_y = self.field_pos.y + y + 1;
            if !self.field.is_line_filled(y) || self.flashing_animation.is_show() {
                for x in 0..Field::width() {
                    let pos_x = self.field_pos.x + x + 1;
                    if self.field.is_filled(Point::new(x, y)) {
                        draw_chars(buf, Point::new(pos_x, pos_y), &[0xb1u8]);
                    }
                }
            }
            if self.field.is_line_filled(y) {
                draw_chars(buf, self.field_pos.with_y(pos_y).add_x(Field::width() + 2), b"&");
            }
        }

        if !self.flashing_animation.is_started() {
            for y in 0..Frame::height() {
                for x in 0..Frame::width() {
                    let pos = self.tet_pos + Point::new(x, y);
                    if self.current_frame().is_filled(Point::new(x, y)) {
                        draw_chars(buf, pos, &[0xb1u8]);
                    }
                }
            }
        }

        for y in 0..Frame::height() {
            for x in 0..Frame::width() {
                let pos = self.field_pos.add_x(Field::width() + 4).add_y(Field::height() / 2) + Point::new(x, y);
                if self.tetrominos[self.next_tet_index].frames[0].is_filled(Point::new(x, y)) {
                    draw_chars(buf, pos, &[0xb1u8]);
                }
            }
        }

        if self.field.is_collide(self.current_frame(), self.translate_tet_to_field_pos(self.tet_pos)) {
            draw_chars(buf, self.field_pos.add_x(Field::width() + 2 + 3), b"c");
        }
    }

    pub fn move_colliding_tetromino(&mut self, new_pos: Point) {
        if self.flashing_animation.is_started() {
            return;
        }
        if self.is_collide(self.current_frame(), self.translate_tet_to_field_pos(self.tet_pos)) ||
            !self.is_collide(self.current_frame(), self.translate_tet_to_field_pos(new_pos)) {
            self.tet_pos = new_pos;
        }
    }

    fn next_frame(&self) -> usize {
        (self.curr_frame + 1) % 4
    }

    pub fn rotate_colliding_tetromino(&mut self) {
        let new_frame_index = self.next_frame();
        let new_frame = self.tetrominos[self.curr_tet_index].frames[new_frame_index];
        if self.is_collide(self.current_frame(), self.translate_tet_to_field_pos(self.tet_pos)) ||
            !self.is_collide(new_frame, self.translate_tet_to_field_pos(self.tet_pos)) {
            self.curr_frame = new_frame_index;
        }
    }

    fn finish_turn(&mut self) {
        self.curr_tet_index = self.next_tet_index;
        self.next_tet_index = self.rng.usize(0..7);
        self.curr_frame = 0;
        self.tet_pos = Self::spawn_pos(self.field_pos);
    }

    pub fn move_down(&mut self) {
        if self.is_collide(self.current_frame(), self.translate_tet_to_field_pos(self.tet_pos)) {
            return;
        }

        let new_pos = self.tet_pos.add_y(1);

        if !self.is_collide(self.current_frame(), self.translate_tet_to_field_pos(new_pos)) {
            self.tet_pos = new_pos;
        } else {
            self.copy_frame();

            if self.field.is_any_line_filled() {
                self.flashing_animation.start();
                self.fall_timer.stop();
            } else {
                self.finish_turn();
            }
        }
    }

    pub fn handle_input(&mut self, input: &Input) {
        if input.is_front_edge(Scancode::Equals) {
            self.next_tetromino();
        } else if input.is_front_edge(Scancode::Minus) {
            self.prev_tetromino();
        } else if input.is_front_edge(Scancode::Space) {
            self.rotate_colliding_tetromino();
        } else if input.is_front_edge(Scancode::Up) {
            let new_pos = self.tet_pos.sub_y(1);
            self.move_colliding_tetromino(new_pos);
        } else if input.is_front_edge(Scancode::Down) {
            let new_pos = self.tet_pos.add_y(1);
            self.move_colliding_tetromino(new_pos);
        } else if input.is_front_edge(Scancode::Left) {
            let new_pos = self.tet_pos.sub_x(1);
            self.move_colliding_tetromino(new_pos);
        } else if input.is_front_edge(Scancode::Right) {
            let new_pos = self.tet_pos.add_x(1);
            self.move_colliding_tetromino(new_pos);
        } else if input.is_front_edge(Scancode::V) {
            self.copy_frame();
            for y in 0..Field::height() {
                if self.field.is_line_filled(y) {
                    self.flashing_animation.start();
                    break;
                }
            }
        } else if input.is_front_edge(Scancode::R) {
            self.clean_filled_lines();
        }
    }

    pub fn tick(&mut self) {
        self.fall_timer.tick();
        self.flashing_animation.tick();

        if self.flashing_animation.is_triggered() {
            self.clean_filled_lines();
            self.fall_timer.start();
            self.finish_turn();
        }
        if self.fall_timer.is_triggered() {
            self.fall_timer.start();
            self.move_down();
        }
    }
}

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
                            let chr = screen_buffer.chars[(y * tile_count.0 + x) as usize];

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
