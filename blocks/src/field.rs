use engine::base::Number;
use engine::geometry::Point;

use crate::tetromino::{Frame, FRAME_SIDE};
use std::convert::TryFrom;

const FIELD_WIDTH: usize = 10;
const FIELD_HEIGHT: usize = 18;

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
                return self.squares[crate::index(x, y, FIELD_WIDTH)];
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

    pub fn clean_filled_lines(&mut self) -> Number {
        let mut filled_lines: Number = 0;
        let mut read_line = FIELD_HEIGHT;
        let mut first_write_line = FIELD_HEIGHT - 1;
        let mut last_write_line = first_write_line + 1;
        loop {
            read_line -= 1;

            if self.is_line_filled(read_line as Number) {
                last_write_line -= 1;
                filled_lines += 1;
            } else {
                if first_write_line >= last_write_line {
                    for i in 0..FIELD_WIDTH {
                        self.squares[crate::index(i, first_write_line, FIELD_WIDTH)] = self.squares[crate::index(i, read_line, FIELD_WIDTH)];
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
        if first_write_line >= last_write_line {
            for j in last_write_line..=first_write_line {
                for i in 0..FIELD_WIDTH {
                    self.squares[crate::index(i, j, FIELD_WIDTH)] = false;
                }
            }
        }

        filled_lines
    }

    pub fn copy_frame(&mut self, frame: &Frame, p: Point) {
        for j in 0..(FRAME_SIDE as Number) {
            for i in 0..(FRAME_SIDE as Number) {
                if frame.is_filled(Point::new(i, j)) {
                    let x = i + p.x;
                    let y = j + p.y;

                    if x >= 0 && x < Self::width() && y >= 0 && y < Self::height() {
                        self.squares[crate::index(x as usize, y as usize, FIELD_WIDTH)] = true;
                    }
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
