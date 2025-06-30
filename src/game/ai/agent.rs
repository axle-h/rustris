use std::cmp::Ordering;
use itertools::Itertools;
use crate::game::ai::apply_inputs::ApplyInputs;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::input_search::{InputSearch, InputSequenceResult};
use crate::game::ai::input_sequence::InputSequence;
use crate::game::{Game, GameState};
use crate::game::ai::board_features::{BoardFeatures, StackStats};
use crate::game::ai::headless_game::DEFAULT_LOOKAHEAD;
use crate::game::ai::linear::LinearCoefficients;
use crate::game::ai::neural::{NeuralGenome, TetrisNeuralNetwork};
use crate::game::board::Board;
use crate::game::tetromino::TetrominoShape;

pub struct AiAgent {
    action_evaluate: ActionEvaluator,
    wait_sate: Option<AgentWaitState>,
    look_ahead: usize
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum AgentWaitState {
    Spawn,
    SoftDrop(InputSequence),
    Alt(TetrominoShape, InputSequence),
}

impl AiAgent {
    pub fn new(action_evaluate: ActionEvaluator, look_ahead: usize) -> Self {
        Self { action_evaluate, wait_sate: None, look_ahead }
    }
    
    pub fn default_linear() -> Self {
        Self::new(ActionEvaluator::Linear(LinearCoefficients::default()), DEFAULT_LOOKAHEAD)
    }
    
    pub fn default_neural() -> Self {
        Self::new(ActionEvaluator::NeuralNetwork(TetrisNeuralNetwork::default()), DEFAULT_LOOKAHEAD)
    }

    fn apply_inputs(&mut self, game: &mut Game, inputs: &InputSequence) {
        let (before_soft_drop, after_soft_drop) = inputs.split_at_soft_drop();
        game.apply_inputs(&before_soft_drop);

        if let Some(after_soft_drop) = after_soft_drop {
            self.wait_sate = Some(AgentWaitState::SoftDrop(after_soft_drop));
        } else {
            self.wait_sate = Some(AgentWaitState::Spawn);
        }
    }

    pub fn reset(&mut self) {
        self.wait_sate = None;
    }

    pub fn act(&mut self, game: &mut Game) {
        if let Some(wait_state) = self.wait_sate.clone() {
            match wait_state {
                AgentWaitState::Spawn => {
                    if matches!(game.state, GameState::Spawn(_, _)) {
                        self.wait_sate = None;
                    }
                }
                AgentWaitState::SoftDrop(post_soft_drop_inputs)  => {
                    match game.state {
                        GameState::Fall(_) => {
                            // continue soft dropping until a lock
                            game.set_soft_drop(true);
                            return;
                        }
                        GameState::Lock(_) => {
                            // if we are in a lock state, we can apply the soft drop inputs
                            self.wait_sate = None;
                            self.apply_inputs(game, &post_soft_drop_inputs);
                        }
                        _ => (),
                    }
                }
                AgentWaitState::Alt(alt_shape, alt_inputs) => {
                    if matches!(game.state, GameState::Fall(_)) {
                        if let Some(shape) = game.board.tetromino().map(|t| t.shape()) {
                            if shape == alt_shape {
                                // we are in the alt state, apply the inputs
                                self.wait_sate = None;
                                self.apply_inputs(game, &alt_inputs);
                            }
                        }
                    }
                }
            }
            return; // wait for wait state to be resolved
        }

        if !matches!(game.state, GameState::Fall(_)) {
            return; // only act when in a fall state
        }
        
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
                // hold the current and wait for the alt shape to fall
                self.wait_sate = Some(AgentWaitState::Alt(alt_next_shape, best_inputs));
                game.hold();
            } else {
                self.apply_inputs(game, &best_inputs);
            }
        }
    }
    
    fn best_move(&self, game: &Game, shape: TetrominoShape, peek: &[TetrominoShape]) -> Option<(InputSequence, f64)> {
        self.best_single_move(game.board, game.board.stack_stats(), shape)
            .map(|(result, cost)| (result.inputs().clone(), cost))

        // let stack_stats_before = game.board.stack_stats();
        // let inputs = game.board.search_all_inputs(shape);
        // let input_len = inputs.len();
        //
        // inputs
        //     .into_iter()
        //     // first order by the cost of the initial move
        //     .map(|r| (r, self.action_evaluate.evaluate_action(&game.board, stack_stats_before, r.board())))
        //     .sorted_by(|m1, m2| self.compare_moves(m2, m1))
        //
        //     // next look ahead and take the best result over the entire peek sequence
        //     .take(input_len / 2) // for performance, prune the search space to the top 50th percentile, TODO configurable
        //     .filter_map(|(initial_move, _)| {
        //         let mut current_board = initial_move.board();
        //         let mut bad_sequence = false;
        //
        //         // Try each piece in the peek sequence
        //         for &next_shape in peek.iter().take(self.look_ahead) {
        //             // TODO could use the held tetromino here
        //             if let Some((next_result, _)) = self.best_single_move(current_board, current_board.stack_stats(), next_shape) {
        //                 current_board = next_result.board();
        //             } else {
        //                 bad_sequence = true;
        //                 break;
        //             }
        //         }
        //
        //         if bad_sequence {
        //             None
        //         } else {
        //             // cost from the start to the end
        //             let score = self.action_evaluate.evaluate_action(&game.board, stack_stats_before, current_board);
        //             Some((initial_move, score))
        //         }
        //     })
        //     .max_by(|m1, m2| self.compare_moves(m1, m2))
        //     .map(|(result, score)| (result.inputs(), score))
    }


    fn best_single_move(&self, board_from: Board, stack_stats_before: StackStats, shape: TetrominoShape) -> Option<(InputSequenceResult, f64)> {
        board_from.search_all_inputs(shape)
            .into_iter()
            .map(|r| {
                let score = self.action_evaluate.evaluate_action(&board_from, stack_stats_before, r.board());
                (r, score)
            })
            .max_by(|m1, m2| self.compare_moves(m1, m2))
    }

    fn compare_moves(&self, (result1, cost1): &(InputSequenceResult, f64), (result2, cost2): &(InputSequenceResult, f64)) -> Ordering {
        // if multiple moves have teh same score then we must order them to deterministically choose
        cost1.total_cmp(cost2).then_with(|| result1.inputs().cmp(&result2.inputs()))
    }

}

impl Default for AiAgent {
    fn default() -> Self {
        Self::default_neural()
    }
}