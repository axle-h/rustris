use sdl2::pixels::Color;
use crate::game::geometry::Rotation;
use super::tetromino::TetrominoShape;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockState {
    Empty,
    Tetromino(TetrominoShape, Rotation, u32),
    Ghost(TetrominoShape, Rotation, u32),
    Stack(TetrominoShape, Rotation, u32),
}

impl BlockState {
    pub fn collides(&self) -> bool {
        matches!(self, BlockState::Stack(_, _, _))
    }
}
