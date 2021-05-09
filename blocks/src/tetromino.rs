use std::convert::TryFrom;

use engine::base::Number;
use engine::geometry::Point;

pub const FRAME_SIDE: usize = 4;

pub struct Frame {
    squares: [bool; FRAME_SIDE * FRAME_SIDE],
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
                return self.squares[crate::index(x, y, FRAME_SIDE)];
            }
        }
        return false;
    }
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
