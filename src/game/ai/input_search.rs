use std::collections::{HashMap, HashSet, VecDeque};
use crate::game::ai::apply_inputs::ApplyInputs;
use crate::game::ai::input_sequence::InputSequence;
use crate::game::board::Board;
use crate::game::tetromino::{Minos, TetrominoShape};

pub trait InputSearch {
    fn search_all_inputs(&self, shape: TetrominoShape) -> Vec<InputSequenceResult>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputSequenceResult {
    inputs: InputSequence,
    board: Board,
    minos: Minos,
}

impl InputSequenceResult {
    pub fn inputs(&self) -> InputSequence {
        self.inputs
    }

    pub fn board(&self) -> Board {
        self.board
    }

    pub fn minos(&self) -> Minos {
        self.minos
    }
}

impl InputSearch for Board {
    fn search_all_inputs(&self, shape: TetrominoShape) -> Vec<InputSequenceResult> {
        let mut spawned_board = *self;
        spawned_board.clear_tetromino();
        if spawned_board.try_spawn_tetromino(shape).is_none() {
            // cannot even place the tetromino, no possible inputs
            return Vec::default();
        }

        let mut visited_inputs: HashSet<InputSequence> = HashSet::new();
        let mut results: HashMap<Minos, InputSequenceResult> = HashMap::new();

        let mut visit_queue: VecDeque<InputSequence> = VecDeque::new();
        visit_queue.push_back(InputSequence::default());

        while let Some(inputs) = visit_queue.pop_front() {
            let mut current_board = spawned_board;
            if !current_board.apply_inputs(inputs) {
                continue;
            }

            current_board.hard_drop();
            if let Some(mut minos) = current_board.lock() {
                minos.sort();
                results.insert(minos, InputSequenceResult { board: current_board, inputs, minos });
            }

            for input in [inputs.into_left(), inputs.into_right(), inputs.into_rotation()] {
                if !visited_inputs.insert(input) {
                    continue;
                }
                visit_queue.push_back(input);
            }
        }

        results.into_values()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches() {
        let board = Board::new();
        let inputs = board.search_all_inputs(TetrominoShape::I);
        assert_eq!(inputs.len(), 17); // 7 flat positions + 10 upright positions
    }
}