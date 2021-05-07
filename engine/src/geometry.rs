use std::ops::{Add, Sub};

use crate::base::Number;

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
