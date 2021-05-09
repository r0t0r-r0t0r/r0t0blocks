use std::cmp::min;
use std::sync::mpsc::Sender;

use enum_dispatch::enum_dispatch;
use fastrand::Rng;
use sdl2::keyboard::Scancode;

use engine::audio::Sound;
use engine::base::{App, Number};
use engine::geometry::Point;
use engine::input::Input;
use engine::time::{BlinkAnimation, DelayedRepeat, TimeAware, Timer};
use engine::video::{draw_rect, draw_str, ScreenBuffer};

use crate::field::Field;
use crate::tetromino::{Frame, Tetromino};

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
    score: Number,

    // visualisation
    field_pos: Point,
    // audio
    audio: Option<Sender<Sound>>,
}

impl<'frame> State<'frame> {
    fn spawn_pos() -> Point {
        Point::new((Field::width() - Frame::width()) / 2, -2)
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

        let score = 0;
        let level = Self::level(score);
        let fall_timer = Timer::new(Self::fall_period(level));

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
            score,
            audio: None,
        };

        initial_screen.enter(&mut state);

        state
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

    fn clean_filled_lines(&mut self) -> Number {
        self.field.clean_filled_lines()
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

    pub fn update_score(&mut self, lines: Number) {
        let score = if lines <= 0 {
            0
        } else if lines == 1 {
            100
        } else if lines == 2 {
            250
        } else if lines == 3 {
            500
        } else {
            1000
        };

        self.score = min(self.score + score, 9999999);
    }

    fn fall_period(level: Number) -> Number {
        match level {
            x if x <= 0 => 120,
            1 => 60,
            2 => 50,
            3 => 40,
            4 => 34,
            5 => 28,
            6 => 24,
            7 => 16,
            8 => 10,
            _ => 8,
        }
    }

    fn level(score: Number) -> Number {
        if score < 0 {
            0
        } else {
            min(9, score / 5000)
        }
    }

    pub fn actualize_level(&mut self) {
        let level = Self::level(self.score);
        self.fall_timer = Timer::new(Self::fall_period(level));
    }

    fn make_beep(&self) {
        if let Some(audio) = self.audio.as_ref() {
            audio.send(Sound::Beep);
        }
    }
}

impl<'frame> App for State<'frame> {
    fn init_audio(&mut self, tx: Sender<Sound>) {
        self.audio = Some(tx);
    }

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
        state.score = 0;
        state.fall_timer = Timer::new(State::fall_period(State::level(state.score)));
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

        if input.is_front_edge(Scancode::Up) {
            state.rotate_colliding_tetromino();
            state.make_beep();
        } else if input.is_front_edge(Scancode::Down) {
            let new_pos = state.tet_pos.add_y(1);
            state.move_colliding_tetromino(new_pos);
            state.down_repeater.start();
            state.make_beep();
        } else if input.is_front_edge(Scancode::Left) {
            let new_pos = state.tet_pos.sub_x(1);
            state.move_colliding_tetromino(new_pos);
            state.left_repeater.start();
            state.right_repeater.stop();
            state.make_beep();
        } else if input.is_front_edge(Scancode::Right) {
            let new_pos = state.tet_pos.add_x(1);
            state.move_colliding_tetromino(new_pos);
            state.right_repeater.start();
            state.left_repeater.stop();
            state.make_beep();
        } else if input.is_front_edge(Scancode::Escape) {
            state.open_popup_screen(PauseScreen.into());
        } else if input.is_front_edge(Scancode::Equals) {
            state.score += 5000;
            state.fall_timer = Timer::new(State::fall_period(State::level(state.score)));
            state.fall_timer.start();
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
            let filled_lines = state.clean_filled_lines();
            state.update_score(filled_lines);
            state.actualize_level();
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

        draw_str(buf, state.field_pos.add_x(3 + Field::width()).add_y(1), &state.score.to_string());
        draw_str(buf, state.field_pos.add_x(3 + Field::width()).add_y(2), &(State::level(state.score) + 1).to_string());
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

    fn draw(&self, state: &State, buf: &mut ScreenBuffer) {
        draw_str(buf, Point::new(0, 0), "Game over.");
        draw_str(buf, Point::new(0, 1), &format!("Score: {}.", state.score));
        draw_str(buf, Point::new(0, 2), "Press space to try again.");
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
