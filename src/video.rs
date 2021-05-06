use std::cmp::{max, min};
use std::iter;

use crate::base::Number;
use crate::geometry::Point;

pub struct ScreenBuffer {
    chars: Vec<u8>,
    width: usize,
    height: usize,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> ScreenBuffer {
        ScreenBuffer {
            chars: vec![0; width * height],
            width,
            height,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn char_at(&self, x: usize, y: usize) -> u8 {
        self.chars[self.index(x, y)]
    }

    pub fn clear(&mut self) {
        self.chars.fill(0);
    }

    pub fn draw_chars(&mut self, p: Point, s: &[u8]) {
        let Point { x, y } = p;
        if y >= 0 && y < self.height as Number {
            if x < self.width as Number && x + s.len() as Number >= 0 {
                let clipped_start_x = max(x, 0);
                let clipped_end_x = min(x + s.len() as Number, self.width as Number);
                let slice_start = clipped_start_x - x;
                let slice_end = clipped_end_x - x;
                let index = self.index(clipped_start_x as usize, y as usize);

                self.chars[index..(index + (clipped_end_x - clipped_start_x) as usize)].copy_from_slice(&s[slice_start as usize..slice_end as usize]);
            }
        }
    }
}

pub fn draw_rect(buf: &mut ScreenBuffer, p: Point, width: Number, height: Number, char: &[u8]) {
    if width >= 2 && height >= 2 {
        let horizontal_line = iter::repeat(char[0]).take(width as usize).collect::<Vec<_>>();
        buf.draw_chars(p, &horizontal_line);
        buf.draw_chars(p.add_y(height as Number - 1), &horizontal_line);
        for j in p.y + 1..p.y + height as Number - 1 {
            buf.draw_chars(p.with_y(j), char);
            buf.draw_chars(p.with_y(j).add_x(width as Number - 1), char);
        }
    }
}
