use crate::game::ai::input_sequence::InputSequence;
use crate::game::board::Board;
use crate::game::Game;

pub trait ApplyInputs {
    fn apply_inputs(&mut self, seq: InputSequence) -> bool;
}

impl ApplyInputs for Board {
    fn apply_inputs(&mut self, seq: InputSequence) -> bool {

        for _ in 0..seq.rotations() {
            if !self.rotate(true) {
                return false;
            }
        }
        for _ in 0..seq.lefts() {
            if !self.left() {
                return false;
            }
        }
        for _ in 0..seq.rights() {
            if !self.right() {
                return false;
            }
        }

        true
    }
}

impl ApplyInputs for Game {
    fn apply_inputs(&mut self, seq: InputSequence) -> bool {

        for _ in 0..seq.rotations() {
            if !self.rotate(true) {
                return false;
            }
        }
        for _ in 0..seq.lefts() {
            if !self.left() {
                return false;
            }
        }
        for _ in 0..seq.rights() {
            if !self.right() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::game::geometry::Rotation;
    use crate::game::tetromino::TetrominoShape;
    use super::*;

    fn having_inputs(shape: TetrominoShape, inputs: InputSequence) -> (Board, bool) {
        let mut board = Board::new();
        board.try_spawn_tetromino(shape).unwrap();
        let result = board.apply_inputs(inputs);
        (board, result)
    }

    mod applies_empty_inputs {
        use super::*;

        macro_rules! tests {
            ($($name:ident),*) => {
                $(
                    #[test]
                    fn $name() {
                        let shape = TetrominoShape::$name;
                        let (board, success) = having_inputs(shape, InputSequence::default());
                        assert!(success);
                        assert_eq!(board.tetromino().map(|t| t.position()), Some(shape.meta().spawn_point()));
                    }
                )*
            };
        }

        tests! { I, O, T, S, Z, J, L }
    }

    mod moves_left {
        use super::*;

        macro_rules! tests {
            ($($name:ident),*) => {
                $(
                    #[test]
                    fn $name() {
                        let shape = TetrominoShape::$name;
                        let (board, success) = having_inputs(shape, InputSequence::default().into_left());
                        assert!(success);
                        assert_eq!(board.tetromino().map(|t| t.position()), Some(shape.meta().spawn_point().translate(-1, 0)));
                    }
                )*
            };
        }

        tests! { I, O, T, S, Z, J, L }
    }

    mod moves_right {
        use super::*;

        macro_rules! tests {
            ($($name:ident),*) => {
                $(
                    #[test]
                    fn $name() {
                        let shape = TetrominoShape::$name;
                        let (board, success) = having_inputs(shape, InputSequence::default().into_right());
                        assert!(success);
                        assert_eq!(board.tetromino().map(|t| t.position()), Some(shape.meta().spawn_point().translate(1, 0)));
                    }
                )*
            };
        }

        tests! { I, O, T, S, Z, J, L }
    }

    mod rotates {
        use super::*;

        macro_rules! tests {
            ($($name:ident),*) => {
                $(
                    #[test]
                    fn $name() {
                        let shape = TetrominoShape::$name;
                        let (board, success) = having_inputs(shape, InputSequence::default().into_rotation());
                        assert!(success);
                        assert_eq!(board.tetromino().map(|t| t.rotation()), Some(Rotation::East));
                    }
                )*
            };
        }

        tests! { I, O, T, S, Z, J, L }
    }

    #[test]
    fn follows_sequence() {
        let shape = TetrominoShape::J;
        let sequence = InputSequence::default()
            .into_left().into_left().into_left()
            .into_rotation().into_rotation();
        let (board, success) = having_inputs(shape, sequence);
        assert!(success);
        assert_eq!(board.tetromino().map(|t| t.position()), Some(shape.meta().spawn_point().translate(-3, 0)));
        assert_eq!(board.tetromino().map(|t| t.rotation()), Some(Rotation::South));
    }

    #[test]
    fn blocks_invalid_moves() {
        let shape = TetrominoShape::J;
        let sequence = InputSequence::new(4, 0, Rotation::North);
        let (_, success) = having_inputs(shape, sequence);
        assert!(!success);
    }
}