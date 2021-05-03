use std::convert::TryFrom;
use std::ops::{Add, Sub};

pub type Number = i32;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: Number,
    pub y: Number,
}

impl Point {
    pub fn new(x: Number, y: Number) -> Point {
        Point { x, y }
    }

    pub fn add_x(&self, x: Number) -> Point {
        Point::new(self.x + x, self.y)
    }

    pub fn add_y(&self, y: Number) -> Point {
        Point::new(self.x, self.y + y)
    }

    pub fn sub_x(&self, x: Number) -> Point {
        Point::new(self.x - x, self.y)
    }

    pub fn sub_y(&self, y: Number) -> Point {
        Point::new(self.x, self.y - y)
    }

    pub fn with_x(&self, x: Number) -> Point {
        Point::new(x, self.y)
    }

    pub fn with_y(&self, y: Number) -> Point {
        Point::new(self.x, y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

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
}
