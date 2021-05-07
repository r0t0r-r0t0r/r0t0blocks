use std::convert::TryFrom;

use fastrand::Rng;
use sdl2::keyboard::Scancode;

use engine::base::{Number, App};
use engine::geometry::Point;
use engine::input::Input;
use engine::time::{BlinkAnimation, DelayedRepeat, TimeAware, Timer};
use engine::video::{ScreenBuffer, draw_rect, draw_str};
use enum_dispatch::enum_dispatch;

const FRAME_SIDE: usize = 4;

pub struct Frame {
    squares: [bool; FRAME_SIDE * FRAME_SIDE],
}

pub struct Tetromino<'frame> {
    pub frames: [&'frame Frame; 4],
}

impl<'frame> Tetromino<'frame> {
    pub fn new(frames: &[Frame]) -> Tetromino {
        Tetromino {
            frames: [
                &frames[0],
                &frames[1 % frames.len()],
                &frames[2 % frames.len()],
                &frames[3 % frames.len()],
            ],
        }
    }
}

pub fn create_frames() -> [Vec<Frame>; 7] {
    [
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 1, 1],
                [0, 0, 0, 0],
            ]),
            Frame::new([
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0],
            ]),
        ],
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 1, 1, 0],
                [0, 1, 1, 0],
            ]),
        ],
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 1, 0],
                [0, 1, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [0, 1, 0, 0],
                [1, 1, 0, 0],
                [0, 1, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [0, 1, 0, 0],
                [1, 1, 1, 0],
                [0, 0, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 1, 0],
                [0, 1, 0, 0],
            ]),
        ],
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [1, 1, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [1, 0, 0, 0],
                [1, 1, 1, 0],
                [0, 0, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0,],
                [1, 1, 0, 0,],
                [1, 0, 0, 0,],
                [1, 0, 0, 0,],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [1, 1, 1, 0],
                [0, 0, 1, 0],
                [0, 0, 0, 0],
            ]),
        ],
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 0, 0],
                [0, 1, 1, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 1, 0],
                [1, 0, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0,],
                [1, 1, 0, 0,],
                [0, 1, 0, 0,],
                [0, 1, 0, 0,],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 1, 0],
                [1, 1, 1, 0],
                [0, 0, 0, 0],
            ]),
        ],
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [0, 1, 1, 0],
                [1, 1, 0, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [1, 0, 0, 0],
                [1, 1, 0, 0],
                [0, 1, 0, 0],
            ]),
        ],
        vec![
            Frame::new([
                [0, 0, 0, 0],
                [0, 0, 0, 0],
                [1, 1, 0, 0],
                [0, 1, 1, 0],
            ]),
            Frame::new([
                [0, 0, 0, 0],
                [0, 1, 0, 0],
                [1, 1, 0, 0],
                [1, 0, 0, 0],
            ]),
        ],
    ]
}

pub const fn index(x: usize, y: usize, row_width: usize) -> usize {
    x + y * row_width
}

impl Frame {
    pub fn new(squares: [[u8; FRAME_SIDE]; FRAME_SIDE]) -> Frame {
        let mut inner_squares = [false; FRAME_SIDE * FRAME_SIDE];
        let mut i = 0;

        for row in squares.iter() {
            for &square in row.iter() {
                inner_squares[i] = square != 0;
                i += 1;
            }
        }

        Frame {
            squares: inner_squares,
        }
    }

    pub const fn width() -> Number {
        FRAME_SIDE as Number
    }

    pub const fn height() -> Number {
        FRAME_SIDE as Number
    }

    pub fn is_filled(&self, p: Point) -> bool {
        if let (Ok(x), Ok(y)) = (usize::try_from(p.x), usize::try_from(p.y)) {
            if x < FRAME_SIDE || y < FRAME_SIDE {
                return self.squares[index(x, y, FRAME_SIDE)];
            }
        }
        return false;
    }
}

const FIELD_WIDTH: usize = 10;
const FIELD_HEIGHT: usize = 18;
// const FIELD_WIDTH: usize = 4;
// const FIELD_HEIGHT: usize = 3;

pub struct Field {
    squares: [bool; FIELD_WIDTH * FIELD_HEIGHT],
}

impl Field {
    pub fn new() -> Field {
        Field {
            squares: [false; FIELD_WIDTH * FIELD_HEIGHT],
        }
    }

    pub const fn width() -> Number {
        FIELD_WIDTH as Number
    }

    pub const fn height() -> Number {
        FIELD_HEIGHT as Number
    }

    pub fn is_filled(&self, p: Point) -> bool {
        if let (Ok(x), Ok(y)) = (usize::try_from(p.x), usize::try_from(p.y)) {
            if x < FIELD_WIDTH || y < FIELD_HEIGHT {
                return self.squares[index(x, y, FIELD_WIDTH)];
            }
        }
        return false;
    }

    pub fn is_line_filled(&self, y: Number) -> bool {
        if let Ok(y) = usize::try_from(y) {
            if y < FIELD_HEIGHT {
                for x in (y * FIELD_WIDTH)..((y + 1) * FIELD_WIDTH) {
                    if !self.squares[x] {
                        return false;
                    }
                }
                return true;
            }
        }
        return false;
    }

    pub fn is_any_line_filled(&self) -> bool {
        for y in 0..Field::height() {
            if self.is_line_filled(y) {
                return true;
            }
        }
        return false;
    }

    pub fn clean_filled_lines(&mut self) {
        let mut read_line = FIELD_HEIGHT;
        let mut first_write_line = FIELD_HEIGHT - 1;
        let mut last_write_line = first_write_line + 1;
        loop {
            read_line -= 1;

            if self.is_line_filled(read_line as Number) {
                last_write_line -= 1;
            } else {
                if first_write_line >= last_write_line {
                    for i in 0..FIELD_WIDTH {
                        self.squares[index(i, first_write_line, FIELD_WIDTH)] = self.squares[index(i, read_line, FIELD_WIDTH)];
                    }

                    first_write_line -= 1;
                    last_write_line -= 1;
                } else {
                    if read_line > 0 {
                        first_write_line = read_line - 1;
                        last_write_line = first_write_line + 1;
                    }
                }
            }
            if read_line == 0 {
                break;
            }
        }
        if first_write_line < last_write_line {
            return;
        }
        for j in last_write_line..=first_write_line {
            for i in 0..FIELD_WIDTH {
                self.squares[index(i, j, FIELD_WIDTH)] = false;
            }
        }
    }

    pub fn copy_frame(&mut self, frame: &Frame, p: Point) {
        for j in 0..(FRAME_SIDE as Number) {
            for i in 0..(FRAME_SIDE as Number) {
                if frame.is_filled(Point::new(i, j)) {
                    //TODO: do not panic
                    self.squares[index((i + p.x) as usize, (j + p.y) as usize, FIELD_WIDTH)] = true;
                }
            }
        }
    }

    pub fn is_collide(&self, frame: &Frame, p: Point) -> bool {
        let Point {x, y} = p;
        if x + (FRAME_SIDE as Number) <= 0 {
            return true;
        }
        if x >= FIELD_WIDTH as Number {
            return true;
        }
        if y >= FIELD_HEIGHT as Number {
            return true;
        }

        for j in 0..(FRAME_SIDE as Number) {
            for i in 0..(FRAME_SIDE as Number) {
                if frame.is_filled(Point::new(i, j)) {
                    if x + (i) < 0 {
                        return  true;
                    }
                    if x + i >= FIELD_WIDTH as Number {
                        return true;
                    }
                    if y + j >= FIELD_HEIGHT as Number {
                        return true;
                    }

                    if self.is_filled(Point::new(x + i, y + j)) {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    pub fn clear(&mut self) {
        self.squares.fill(false);
    }
}

pub struct State<'frame> {
    // external
    tetrominos: [Tetromino<'frame>; 7],

    // logic
    curr_frame: usize,
    curr_tet_index: usize,
    next_tet_index: usize,
    field: Field,
    tet_pos: Point,
    fall_timer: Timer,
    filled_lines_animation: BlinkAnimation,
    rng: Rng,
    screen: Screen,
    popup_screen: Option<Screen>,
    left_repeater: DelayedRepeat,
    right_repeater: DelayedRepeat,
    down_repeater: DelayedRepeat,

    // visualisation
    field_pos: Point,
}

impl<'frame> State<'frame> {
    fn spawn_pos() -> Point {
        Point::new((Field::width() - Frame::width()) / 2, -1)
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

        let fall_timer = Timer::new(120);

        let rng = Rng::new();

        let initial_screen = GameScreen.into();

        let mut state = State {
            tetrominos,
            curr_frame: 0,
            curr_tet_index: 0,
            next_tet_index: 0,
            field: Field::new(),
            field_pos,
            tet_pos: Point::new(0, 0),
            fall_timer,
            filled_lines_animation: BlinkAnimation::new(),
            rng,
            screen: initial_screen,
            popup_screen: None,
            left_repeater: DelayedRepeat::new(30, 5),
            right_repeater: DelayedRepeat::new(30, 5),
            down_repeater: DelayedRepeat::new(30, 3),
        };

        initial_screen.enter(&mut state);

        state
    }

    fn next_tetromino(&mut self) {
        self.curr_tet_index = (self.curr_tet_index + 1) % self.tetrominos.len();
        self.curr_frame = 0;
    }

    fn prev_tetromino(&mut self) {
        self.curr_tet_index = (self.tetrominos.len() + self.curr_tet_index - 1) % self.tetrominos.len();
        self.curr_frame = 0;
    }

    fn current_frame(&self) -> &'frame Frame {
        self.tetrominos[self.curr_tet_index].frames[self.curr_frame]
    }

    fn copy_frame(&mut self) {
        let pos = self.tet_pos;
        let curr_frame = self.current_frame();
        self.field.copy_frame(curr_frame, pos);
    }

    fn is_collide(&self, frame: &'frame Frame, p: Point) -> bool {
        self.field.is_collide(frame, p)
    }

    fn clean_filled_lines(&mut self) {
        self.field.clean_filled_lines();
    }

    fn move_colliding_tetromino(&mut self, new_pos: Point) {
        if self.filled_lines_animation.is_started() {
            return;
        }
        if self.is_collide(self.current_frame(), self.tet_pos) ||
            !self.is_collide(self.current_frame(), new_pos) {
            self.tet_pos = new_pos;
        }
    }

    fn next_frame(&self) -> usize {
        (self.curr_frame + 1) % 4
    }

    fn rotate_colliding_tetromino(&mut self) {
        let new_frame_index = self.next_frame();
        let new_frame = self.tetrominos[self.curr_tet_index].frames[new_frame_index];
        if self.is_collide(self.current_frame(), self.tet_pos) ||
            !self.is_collide(new_frame, self.tet_pos) {
            self.curr_frame = new_frame_index;
        }
    }

    fn finish_turn(&mut self) {
        self.left_repeater.stop();
        self.right_repeater.stop();
        self.down_repeater.stop();
        self.curr_tet_index = self.next_tet_index;
        self.next_tet_index = self.rng.usize(0..7);
        self.curr_frame = 0;
        self.tet_pos = Self::spawn_pos();

        if self.is_collide(self.current_frame(), self.tet_pos) {
            self.change_screen(RetryScreen.into());
        }
    }

    fn move_down(&mut self) {
        if self.is_collide(self.current_frame(), self.tet_pos) {
            return;
        }

        let new_pos = self.tet_pos.add_y(1);

        if !self.is_collide(self.current_frame(), new_pos) {
            self.tet_pos = new_pos;
        } else {
            self.copy_frame();

            if self.field.is_any_line_filled() {
                self.filled_lines_animation.start();
                self.fall_timer.stop();
            } else {
                self.finish_turn();
            }
        }
    }

    fn change_screen(&mut self, new_screen: Screen) {
        if self.screen == new_screen {
            return;
        }

        self.screen = new_screen;

        new_screen.enter(self);
    }

    fn open_popup_screen(&mut self, popup_screen: Screen) {
        if self.popup_screen.is_none() {
            self.popup_screen = Some(popup_screen);
            popup_screen.enter(self);
        }
    }

    pub fn close_popup_screen(&mut self) {
        self.popup_screen = None;
    }
}

impl<'frame> App for State<'frame> {
    fn handle_input(&mut self, input: &Input) {
        let current_screen = self.popup_screen.unwrap_or(self.screen);
        current_screen.handle_input(self, input);
    }

    fn tick(&mut self) {
        let current_screen = self.popup_screen.unwrap_or(self.screen);
        current_screen.tick(self);
    }

    fn draw(&self, buf: &mut ScreenBuffer) {
        let current_screen = self.popup_screen.unwrap_or(self.screen);
        current_screen.draw(self, buf);
    }
}

#[enum_dispatch]
#[derive(Eq, PartialEq, Copy, Clone)]
enum Screen {
    GameScreen,
    RetryScreen,
    PauseScreen,
}

#[enum_dispatch(Screen)]
trait ScreenBehavior {
    fn enter(&self, state: &mut State);
    fn handle_input(&self, state: &mut State, input: &Input);
    fn tick(&self, state: &mut State);
    fn draw(&self, state: &State, buf: &mut ScreenBuffer);
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct GameScreen;

impl ScreenBehavior for GameScreen {
    fn enter(&self, state: &mut State) {
        state.fall_timer.start();
        state.curr_tet_index = state.rng.usize(0..7);
        state.next_tet_index = state.rng.usize(0..7);
        state.curr_frame = 0;
        state.field.clear();
        state.tet_pos = State::spawn_pos();
    }

    fn handle_input(&self, state: &mut State, input: &Input) {
        if input.is_back_edge(Scancode::Left) {
            state.left_repeater.stop();
        }
        if input.is_back_edge(Scancode::Right) {
            state.right_repeater.stop();
        }
        if input.is_back_edge(Scancode::Down) {
            state.down_repeater.stop();
        }

        if input.is_front_edge(Scancode::Equals) {
            state.next_tetromino();
        } else if input.is_front_edge(Scancode::Minus) {
            state.prev_tetromino();
        } else if input.is_front_edge(Scancode::Space) {
            state.rotate_colliding_tetromino();
        } else if input.is_front_edge(Scancode::Up) {
            state.rotate_colliding_tetromino();
        } else if input.is_front_edge(Scancode::Down) {
            let new_pos = state.tet_pos.add_y(1);
            state.move_colliding_tetromino(new_pos);
            state.down_repeater.start();
        } else if input.is_front_edge(Scancode::Left) {
            let new_pos = state.tet_pos.sub_x(1);
            state.move_colliding_tetromino(new_pos);
            state.left_repeater.start();
            state.right_repeater.stop();
        } else if input.is_front_edge(Scancode::Right) {
            let new_pos = state.tet_pos.add_x(1);
            state.move_colliding_tetromino(new_pos);
            state.right_repeater.start();
            state.left_repeater.stop();
        } else if input.is_front_edge(Scancode::V) {
            state.copy_frame();
            for y in 0..Field::height() {
                if state.field.is_line_filled(y) {
                    state.filled_lines_animation.start();
                    break;
                }
            }
        } else if input.is_front_edge(Scancode::R) {
            state.clean_filled_lines();
        } else if input.is_front_edge(Scancode::Escape) {
            state.open_popup_screen(PauseScreen.into());
        }
    }

    fn tick(&self, state: &mut State) {
        state.left_repeater.tick();
        state.right_repeater.tick();
        state.down_repeater.tick();
        state.fall_timer.tick();
        state.filled_lines_animation.tick();

        if state.left_repeater.is_triggered() {
            let new_pos = state.tet_pos.sub_x(1);
            state.move_colliding_tetromino(new_pos);
        }
        if state.right_repeater.is_triggered() {
            let new_pos = state.tet_pos.add_x(1);
            state.move_colliding_tetromino(new_pos);
        }
        if state.down_repeater.is_triggered() {
            let new_pos = state.tet_pos.add_y(1);
            state.move_colliding_tetromino(new_pos);
        }
        if state.filled_lines_animation.is_triggered() {
            state.clean_filled_lines();
            state.fall_timer.start();
            state.finish_turn();
        }
        if state.fall_timer.is_triggered() {
            state.fall_timer.start();
            state.move_down();
        }
    }

    fn draw(&self, state: &State, buf: &mut ScreenBuffer) {
        draw_rect(buf, state.field_pos, Field::width() + 2, Field::height() + 2, '+');

        for y in 0..Field::height() {
            let pos_y = state.field_pos.y + y + 1;
            if !state.field.is_line_filled(y) || state.filled_lines_animation.is_show() {
                for x in 0..Field::width() {
                    let pos_x = state.field_pos.x + x + 1;
                    if state.field.is_filled(Point::new(x, y)) {
                        buf.set_byte(Point::new(pos_x, pos_y), 0xb1u8);
                    }
                }
            }
        }

        if !state.filled_lines_animation.is_started() {
            for y in 0..Frame::height() {
                for x in 0..Frame::width() {
                    let pos = state.tet_pos + state.field_pos + Point::new(1, 1) + Point::new(x, y);
                    if state.current_frame().is_filled(Point::new(x, y)) {
                        buf.set_byte(pos, 0xb1u8);
                    }
                }
            }
        }

        for y in 0..Frame::height() {
            for x in 0..Frame::width() {
                let pos = state.field_pos.add_x(Field::width() + 4).add_y(Field::height() / 2) + Point::new(x, y);
                if state.tetrominos[state.next_tet_index].frames[0].is_filled(Point::new(x, y)) {
                    buf.set_byte(pos, 0xb1u8);
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct RetryScreen;

impl ScreenBehavior for RetryScreen {
    fn enter(&self, _state: &mut State) {

    }

    fn handle_input(&self, state: &mut State, input: &Input) {
        if input.is_front_edge(Scancode::Space) {
            state.change_screen(GameScreen.into());
        }
    }

    fn tick(&self, _state: &mut State) {

    }

    fn draw(&self, _state: &State, buf: &mut ScreenBuffer) {
        draw_str(buf, Point::new(0, 0), "Game over.");
        draw_str(buf, Point::new(0, 1), "Press space to try again.");
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct PauseScreen;

impl ScreenBehavior for PauseScreen {
    fn enter(&self, state: &mut State) {
        state.left_repeater.stop();
        state.right_repeater.stop();
        state.down_repeater.stop();
    }

    fn handle_input(&self, state: &mut State, input: &Input) {
        if input.is_front_edge(Scancode::Escape) {
            state.close_popup_screen();
        }
    }

    fn tick(&self, _state: &mut State) {

    }

    fn draw(&self, _state: &State, buf: &mut ScreenBuffer) {
        draw_str(buf, Point::new(0, 0), "Pause.");
    }
}
