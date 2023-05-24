use super::tetromino::TetrominoShape;
use crate::game::geometry::Rotation;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockState {
    Empty,
    Tetromino(TetrominoShape, Rotation, u32),
    Ghost(TetrominoShape, Rotation, u32),
    Stack(TetrominoShape, Rotation, u32),
    Garbage,
}

impl BlockState {
    pub fn collides(&self) -> bool {
        matches!(self, BlockState::Garbage | BlockState::Stack(_, _, _))
    }
}
