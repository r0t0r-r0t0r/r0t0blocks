pub mod blocks;
pub mod tetromino;
mod field;

pub const fn index(x: usize, y: usize, row_width: usize) -> usize {
    x + y * row_width
}

