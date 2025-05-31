use std::cmp::Ordering;
use itertools::Itertools;
use crate::game::ai::apply_inputs::ApplyInputs;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::input_search::{InputSearch, InputSequenceResult};
use crate::game::ai::input_sequence::InputSequence;
use crate::game::{Game, GameState};
use crate::game::ai::board_features::{BoardFeatures, StackStats};
use crate::game::ai::linear::LinearCoefficients;
use crate::game::ai::neural::{NeuralGenome, TetrisNeuralNetwork};
use crate::game::board::Board;
use crate::game::tetromino::TetrominoShape;

pub struct AiAgent {
    action_evaluate: ActionEvaluator,
    wait_for_alt: Option<InputSequence>,
    wait_for_spawn: bool,
    look_ahead: usize
}

const DEFAULT_LOOKAHEAD: usize = 0;

impl AiAgent {
    pub fn new(action_evaluate: ActionEvaluator, look_ahead: usize) -> Self {
        Self { action_evaluate, wait_for_alt: None, wait_for_spawn: true, look_ahead }
    }
    
    pub fn default_linear() -> Self {
        Self::new(ActionEvaluator::Linear(LinearCoefficients::default()), DEFAULT_LOOKAHEAD)
    }
    
    pub fn default_neural() -> Self {
        Self::new(ActionEvaluator::NeuralNetwork(TetrisNeuralNetwork::default()), DEFAULT_LOOKAHEAD)
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
        let stack_stats_before = game.board.stack_stats();
        
        if peek.is_empty() || self.look_ahead == 0 {
            return self.best_single_move(game.board, game.board.stack_stats(), shape)
                .map(|(result, cost)| (result.inputs(), cost));
        }
        
        let inputs = game.board.search_all_inputs(shape);
        let input_len = inputs.len();

        inputs
            .into_iter()
            // first order by the cost of the initial move
            .map(|r| (r, self.action_evaluate.evaluate_action(&game.board, stack_stats_before, r.board())))
            .sorted_by(|m1, m2| self.compare_moves(m2, m1))
            
            // next look ahead and take the best result over the entire peek sequence
            .take(input_len / 2) // for performance, prune the search space to the top 50th percentile, TODO configurable
            .filter_map(|(initial_move, _)| {
                let mut current_board = initial_move.board();
                let mut bad_sequence = false;

                // Try each piece in the peek sequence
                for &next_shape in peek.iter().take(self.look_ahead) {
                    // TODO could use the held tetromino here
                    if let Some((next_result, _)) = self.best_single_move(current_board, current_board.stack_stats(), next_shape) {
                        current_board = next_result.board();
                    } else {
                        bad_sequence = true;
                        break;
                    }
                }
                
                if bad_sequence {
                    None
                } else {
                    // cost from the start to the end
                    let score = self.action_evaluate.evaluate_action(&game.board, stack_stats_before, current_board);
                    Some((initial_move, score))
                }
            })
            .max_by(|m1, m2| self.compare_moves(m1, m2))
            .map(|(result, score)| (result.inputs(), score))
    }


    fn best_single_move(&self, board_from: Board, stack_stats_before: StackStats, shape: TetrominoShape) -> Option<(InputSequenceResult, f64)> {
        board_from.search_all_inputs(shape)
            .into_iter()
            .map(|r| (r, self.action_evaluate.evaluate_action(&board_from, stack_stats_before, r.board())))
            .max_by(|m1, m2| self.compare_moves(m1, m2))
    }

    fn compare_moves(&self, (result1, cost1): &(InputSequenceResult, f64), (result2, cost2): &(InputSequenceResult, f64)) -> Ordering {
        cost1.total_cmp(cost2).then_with(|| result1.inputs().cmp(&result2.inputs()))
    }

}

impl Default for AiAgent {
    fn default() -> Self {
        Self::default_neural()
    }
}