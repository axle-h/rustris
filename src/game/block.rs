use super::tetromino::TetrominoShape;
use crate::game::geometry::Rotation;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockState {
    Empty,
    Tetromino(TetrominoShape, Rotation, u32),
    Ghost(TetrominoShape, Rotation, u32),
    Stack(TetrominoShape, Rotation, u32),
    Garbage,
}

impl BlockState {
    pub fn is_stack(&self) -> bool {
        matches!(self, BlockState::Garbage | BlockState::Stack(_, _, _))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, BlockState::Empty | BlockState::Ghost(_, _, _))
    }

    pub fn is_tetromino(&self) -> bool {
        matches!(self, BlockState::Tetromino(_, _, _))
    }
}
