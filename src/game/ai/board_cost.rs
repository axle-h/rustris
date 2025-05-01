use std::fmt::{Debug, Display, Formatter};
use crate::game::ai::board_features::BoardFeatures;
use crate::game::ai::coefficient::Coefficient;
use crate::game::board::{Board, BOARD_WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AiCoefficients {
    open_holes: Coefficient,
    closed_holes: Coefficient,
    max_stack_height: Coefficient,
    sum_delta_stack_height: Coefficient,
    max_delta_stack_height: Coefficient,
    line_clear: Coefficient,
    tetris_clear: Coefficient,
}

impl Default for AiCoefficients {
    fn default() -> Self {
        AiCoefficients::from_f64(
            -1672.4227,
            -1567.3356,
            -50.282489,
            -246.88223,
            -16.355071,
            185.28869,
            409.99751,
        )
    }
}

impl AiCoefficients {
    pub const ZERO: Self = Self {
        open_holes: Coefficient::ZERO,
        closed_holes: Coefficient::ZERO,
        max_stack_height: Coefficient::ZERO,
        sum_delta_stack_height: Coefficient::ZERO,
        max_delta_stack_height: Coefficient::ZERO,
        line_clear: Coefficient::ZERO,
        tetris_clear: Coefficient::ZERO,
    };

    pub fn new(open_holes: Coefficient, closed_holes: Coefficient, max_stack_height: Coefficient, sum_delta_stack_height: Coefficient, max_delta_stack_height: Coefficient, line_clear: Coefficient, tetris_clear: Coefficient) -> Self {
        Self { open_holes, closed_holes, max_stack_height, sum_delta_stack_height, max_delta_stack_height, line_clear, tetris_clear }
    }

    pub fn from_f64(open_holes: f64, closed_holes: f64, max_stack_height: f64, sum_delta_stack_height: f64, max_delta_stack_height: f64, line_clear: f64, tetris_clear: f64) -> Self {
        Self {
            open_holes: open_holes.into(),
            closed_holes: closed_holes.into(),
            max_stack_height: max_stack_height.into(),
            sum_delta_stack_height: sum_delta_stack_height.into(),
            max_delta_stack_height: max_delta_stack_height.into(),
            line_clear: line_clear.into(),
            tetris_clear: tetris_clear.into(),
        }
    }

    pub fn from_i64(open_holes: i64, closed_holes: i64, max_stack_height: i64, sum_delta_stack_height: i64, max_delta_stack_height: i64, line_clear: i64, tetris_clear: i64) -> Self {
        Self {
            open_holes: open_holes.into(),
            closed_holes: closed_holes.into(),
            max_stack_height: max_stack_height.into(),
            sum_delta_stack_height: sum_delta_stack_height.into(),
            max_delta_stack_height: max_delta_stack_height.into(),
            line_clear: line_clear.into(),
            tetris_clear: tetris_clear.into(),
        }
    }

    pub fn open_holes(&self) -> Coefficient {
        self.open_holes
    }

    pub fn closed_holes(&self) -> Coefficient {
        self.closed_holes
    }

    pub fn max_stack_height(&self) -> Coefficient {
        self.max_stack_height
    }

    pub fn sum_delta_stack_height(&self) -> Coefficient {
        self.sum_delta_stack_height
    }

    pub fn max_delta_stack_height(&self) -> Coefficient {
        self.max_delta_stack_height
    }

    pub fn line_clear(&self) -> Coefficient {
        self.line_clear
    }

    pub fn tetris_clear(&self) -> Coefficient {
        self.tetris_clear
    }
}

impl Display for AiCoefficients {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "open_holes: {}, closed_holes: {}, max_stack_height: {}, sum_delta_stack_height: {}, max_delta_stack_height: {}, line_clear: {}, tetris_clear: {}",
            self.open_holes, self.closed_holes, self.max_stack_height, self.sum_delta_stack_height, self.max_delta_stack_height, self.line_clear, self.tetris_clear)
    }
}

pub const COEFFICIENTS_COUNT: usize = 7;
pub type Genome = [Coefficient; COEFFICIENTS_COUNT];

impl Into<Genome> for AiCoefficients {
    fn into(self) -> Genome {
        [
            self.open_holes,
            self.closed_holes,
            self.max_stack_height,
            self.sum_delta_stack_height,
            self.max_delta_stack_height,
            self.line_clear,
            self.tetris_clear
        ]
    }
}

impl From<Genome> for AiCoefficients {
    fn from(flat: Genome) -> Self {
        assert_eq!(flat.len(), COEFFICIENTS_COUNT);
        AiCoefficients::new(flat[0], flat[1], flat[2], flat[3], flat[4], flat[5], flat[6])
    }
}

pub struct BoardCost {
    coefficients: AiCoefficients
}

impl BoardCost {
    pub fn new(coefficients: AiCoefficients) -> Self {
        Self { coefficients }
    }

    pub fn cost(&self, board: Board) -> f64 {
        let stack_stats = board.stack_stats();
        let cleared_lines = stack_stats.cleared_lines() as f64;
        stack_stats.open_holes() as f64 * self.coefficients.open_holes +
            stack_stats.closed_holes() as f64 * self.coefficients.closed_holes +
            stack_stats.max_heights().into_iter().max().unwrap() as f64 * self.coefficients.max_stack_height +
            stack_stats.sum_delta_height() as f64 * self.coefficients.sum_delta_stack_height +
            stack_stats.max_delta_height() as f64 * self.coefficients.max_delta_stack_height +
            match stack_stats.cleared_lines() {
                1..=3 => cleared_lines * self.coefficients.line_clear,
                4 => cleared_lines * self.coefficients.tetris_clear,
                _ => 0.0
            }
    }
}
