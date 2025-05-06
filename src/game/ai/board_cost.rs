use std::fmt::{Debug, Display, Formatter};
use crate::game::ai::board_features::{BoardFeatures, StackStats};
use crate::game::ai::coefficient::Coefficient;
use crate::game::board::{Board, BOARD_WIDTH};
use crate::game::tetromino::Minos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AiCoefficients {
    open_holes: Coefficient,
    closed_holes: Coefficient,
    max_stack_height: Coefficient,
    sum_stack_roughness: Coefficient,
    max_stack_roughness: Coefficient,
    line_clear: Coefficient,
    tetris_clear: Coefficient,
    max_tetromino_y: Coefficient,
    pillars: Coefficient
    // TODO rhs column height
}

impl Default for AiCoefficients {
    fn default() -> Self {
        AiCoefficients::from_f64(
            -89.32,
            -104.13,
            -10.12,
            -7.96,
            -7.19,
            -25.65,
            18.87,
            -4.49,
            -57.45
        )
    }
}

impl Display for AiCoefficients {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
               self.open_holes, self.closed_holes, self.max_stack_height, self.sum_stack_roughness, self.max_stack_roughness, self.line_clear, self.tetris_clear, self.max_tetromino_y, self.pillars)
    }
}

impl AiCoefficients {
    pub const ZERO: Self = Self {
        open_holes: Coefficient::ZERO,
        closed_holes: Coefficient::ZERO,
        max_stack_height: Coefficient::ZERO,
        sum_stack_roughness: Coefficient::ZERO,
        max_stack_roughness: Coefficient::ZERO,
        line_clear: Coefficient::ZERO,
        tetris_clear: Coefficient::ZERO,
        max_tetromino_y: Coefficient::ZERO,
        pillars: Coefficient::ZERO,
    };

    pub fn from_f64(
        open_holes: f64,
        closed_holes: f64,
        max_stack_height: f64,
        sum_delta_stack_height: f64,
        max_delta_stack_height: f64,
        line_clear: f64,
        tetris_clear: f64,
        max_tetromino_y: f64,
        pillars: f64
    ) -> Self {
        Self {
            open_holes: open_holes.into(),
            closed_holes: closed_holes.into(),
            max_stack_height: max_stack_height.into(),
            sum_stack_roughness: sum_delta_stack_height.into(),
            max_stack_roughness: max_delta_stack_height.into(),
            line_clear: line_clear.into(),
            tetris_clear: tetris_clear.into(),
            max_tetromino_y: max_tetromino_y.into(),
            pillars: pillars.into(),
        }
    }

    pub fn from_i64(
        open_holes: i64,
        closed_holes: i64,
        max_stack_height: i64,
        sum_delta_stack_height: i64,
        max_delta_stack_height: i64,
        line_clear: i64,
        tetris_clear: i64,
        max_tetromino_y: i64,
        pillars: i64
    ) -> Self {
        Self {
            open_holes: open_holes.into(),
            closed_holes: closed_holes.into(),
            max_stack_height: max_stack_height.into(),
            sum_stack_roughness: sum_delta_stack_height.into(),
            max_stack_roughness: max_delta_stack_height.into(),
            line_clear: line_clear.into(),
            tetris_clear: tetris_clear.into(),
            max_tetromino_y: max_tetromino_y.into(),
            pillars: pillars.into(),
        }
    }   
    
}

pub const COEFFICIENTS_COUNT: usize = 9;
pub type Genome = [Coefficient; COEFFICIENTS_COUNT];

impl Into<Genome> for AiCoefficients {
    fn into(self) -> Genome {
        [
            self.open_holes,
            self.closed_holes,
            self.max_stack_height,
            self.sum_stack_roughness,
            self.max_stack_roughness,
            self.line_clear,
            self.tetris_clear,
            self.max_tetromino_y,
            self.pillars,
        ]
    }
}

impl From<Genome> for AiCoefficients {
    fn from(flat: Genome) -> Self {
        let [
            open_holes,
            closed_holes,
            max_stack_height,
            sum_stack_roughness,
            max_stack_roughness,
            line_clear,
            tetris_clear,
            max_tetromino_y,
            pillars,
        ] = flat;
        Self {
            open_holes,
            closed_holes,
            max_stack_height,
            sum_stack_roughness,
            max_stack_roughness,
            line_clear,
            tetris_clear,
            max_tetromino_y,
            pillars,
        }
    }
}

pub struct BoardCost {
    coefficients: AiCoefficients
}

impl BoardCost {
    pub fn new(coefficients: AiCoefficients) -> Self {
        Self { coefficients }
    }

    pub fn cost(&self, board_before_action: &Board, stats_before_action: StackStats, board_after_action: Board) -> f64 {
        let features = board_after_action.features_after_action(board_before_action, stats_before_action);
        
        let delta = features.delta();

        delta.open_holes() as f64 * self.coefficients.open_holes +
            delta.closed_holes() as f64 * self.coefficients.closed_holes +
            delta.max_height() as f64 * self.coefficients.max_stack_height +
            delta.sum_roughness() as f64 * self.coefficients.sum_stack_roughness +
            delta.max_roughness() as f64 * self.coefficients.max_stack_roughness +
            features.max_tetromino_y() as f64 * self.coefficients.max_tetromino_y +
            delta.pillars() as f64 * self.coefficients.pillars +
            match features.cleared_lines() {
                1..=3 => features.cleared_lines() as f64 * self.coefficients.line_clear,
                4 => features.cleared_lines() as f64 * self.coefficients.tetris_clear,
                _ => 0.0
            }
    }
}
