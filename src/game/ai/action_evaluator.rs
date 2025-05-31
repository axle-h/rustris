use std::fmt::Debug;
use crate::game::ai::board_features::{BoardFeatures, BoardStats, StackStats};
use crate::game::ai::linear::LinearCoefficients;
use crate::game::ai::neural::{Tensor, TetrisNeuralNetwork};
use crate::game::board::Board;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionEvaluator {
    Linear(LinearCoefficients),
    NeuralNetwork(TetrisNeuralNetwork)
}

impl ActionEvaluator {
    pub fn evaluate_action(&self, board_before_action: &Board, stats_before_action: StackStats, board_after_action: Board) -> f64 {
        let stats = board_after_action.features_after_action(board_before_action, stats_before_action);
        match self {
            ActionEvaluator::Linear(coefficients) => Self::linear_score(coefficients, stats),
            ActionEvaluator::NeuralNetwork(network) => Self::neural_score(network, stats)
        }
    }

    fn linear_score(coefficients: &LinearCoefficients, stats: BoardStats) -> f64 {
        let delta = stats.delta();

        delta.open_holes() as f64 * coefficients.open_holes() +
            delta.closed_holes() as f64 * coefficients.closed_holes() +
            delta.max_height() as f64 * coefficients.max_stack_height() +
            delta.sum_roughness() as f64 * coefficients.sum_stack_roughness() +
            delta.max_roughness() as f64 * coefficients.max_stack_roughness() +
            stats.max_tetromino_y() as f64 * coefficients.max_tetromino_y() +
            delta.pillars() as f64 * coefficients.pillars() +
            match stats.cleared_lines() {
                1..=3 => stats.cleared_lines() as f64 * coefficients.line_clear(),
                4 => stats.cleared_lines() as f64 * coefficients.tetris_clear(),
                _ => 0.0
            }
    }
    
    fn neural_score(network: &TetrisNeuralNetwork, stats: BoardStats) -> f64 {
        let delta = stats.delta();
        let global = stats.global();
        let input = Tensor::vector([
            delta.holes() as f64,
            delta.max_height() as f64,
            delta.sum_roughness() as f64,
            delta.max_roughness() as f64,
            delta.pillars() as f64,
            global.holes() as f64,
            global.max_height() as f64,
            global.sum_roughness() as f64,
            global.max_roughness() as f64,
            global.pillars() as f64,
            stats.max_tetromino_y() as f64,
            stats.cleared_lines() as f64,
        ]);
        network.forward(&input).value()
    }
}

