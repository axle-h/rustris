use crate::game::ai::apply_inputs::ApplyInputs;
use crate::game::ai::board_cost::BoardCost;
use crate::game::ai::input_search::{InputSearch, InputSequenceResult};
use crate::game::ai::input_sequence::InputSequence;
use crate::game::{Game, GameState};
use crate::game::board::Board;
use crate::game::tetromino::TetrominoShape;

pub struct AiAgent {
    cost: BoardCost,
    wait_for_alt: Option<InputSequence>,
    wait_for_spawn: bool,
    look_ahead: usize
}

impl AiAgent {

    pub fn new(cost: BoardCost, look_ahead: usize) -> Self {
        Self { cost, wait_for_alt: None, wait_for_spawn: true, look_ahead }
}

    pub fn act(&mut self, game: &mut Game) {
        if let Some(next_move) = match game.state {
            GameState::Spawn(_, _) => {
                self.wait_for_spawn = false;
                None
            }
            GameState::Fall(_) if self.wait_for_spawn => {
                let _ = game.hard_drop() || game.set_soft_drop(true);
                None
            }
            GameState::Fall(_) if self.wait_for_alt.is_some() => {
                let alt = self.wait_for_alt;
                self.wait_for_alt = None;
                alt
            }
            GameState::Fall(_) => {
                if let Some(shape) = game.board.tetromino().map(|t| t.shape()) {
                    let best_result = self.best_move(game, shape, &game.random.peek_buffer());

                    let (alt_next_shape, alt_next_peek) = game.hold
                        .map(|state| (state.shape, 0..))
                        .unwrap_or_else(|| (game.random.peek(), 1..));

                    let alt_best_move = self.best_move(game, alt_next_shape, &game.random.peek_buffer()[alt_next_peek]);
                    let (best_inputs, is_alt) = match (best_result, alt_best_move) {
                        (None, None) => return,
                        (Some((m, _)), None) => (m, false),
                        (None, Some((m, _))) => (m, true),
                        (Some((m1, c1)), Some((m2, c2))) =>
                            if c1 < c2 { (m1, false) } else { (m2, true) }
                    };
                    if is_alt {
                        // return and wait for a tetromino
                        self.wait_for_alt = Some(best_inputs);
                        game.hold();
                        None
                    } else {
                        Some(best_inputs)
                    }
                } else {
                    None
                }
            }
            _ => None
        } {
            game.apply_inputs(next_move);
            game.hard_drop();
            self.wait_for_spawn = true;
        }
    }
    
    fn best_move(&self, game: &Game, shape: TetrominoShape, peek: &[TetrominoShape]) -> Option<(InputSequence, f64)> {
        if peek.is_empty() {
            return self.best_single_move(&game.board, shape)
                .map(|(result, cost)| (result.inputs(), cost));
        }

        let current_moves = game.board.search_all_inputs(shape);
        if current_moves.is_empty() {
            return None;
        }

        // For each current move, evaluate all possible future positions
        let moves_with_costs: Vec<_> = current_moves
            .into_iter()
            .map(|initial_move| {
                let mut current_board = initial_move.board();
                let mut sum_cost = self.cost.cost(current_board, initial_move.minos());

                // Try each piece in the peek sequence
                for &next_shape in peek.iter().take(self.look_ahead) {
                    // TODO could use the held tetromino here
                    if let Some((next_result, next_move_cost)) = self.best_single_move(&current_board, next_shape) {
                        sum_cost += next_move_cost;
                        current_board = next_result.board();
                    } else {
                        // punish a bad sequence
                        sum_cost -= 1_000_000.0;
                        break;
                    }
                }

                (initial_move.inputs(), sum_cost / peek.len() as f64)
            })
            .collect();

        moves_with_costs.into_iter().max_by(|(result1, cost1), (result2, cost2)| {
            cost1.total_cmp(cost2).then_with(|| result1.cmp(&result2))
        })
    }


    fn best_single_move(&self, board: &Board, shape: TetrominoShape) -> Option<(InputSequenceResult, f64)> {
        let moves: Vec<_> = board.search_all_inputs(shape)
            .into_iter()
            .map(|r| (r, self.cost.cost(r.board(), r.minos())))
            .collect();

        if moves.is_empty() {
            return None;
        }

        moves.into_iter().max_by(|(result1, cost1), (result2, cost2)| {
            cost1.total_cmp(cost2).then_with(|| result1.inputs().cmp(&result2.inputs()))
        })
    }

}
