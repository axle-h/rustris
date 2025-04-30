use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::RangeInclusive;
use rand::distr::{Distribution, StandardUniform};
use rand::Rng;
use crate::game::ai::board_features::BoardFeatures;
use crate::game::ai::floats::{is_near_f64, precision_f64};
use crate::game::board::Board;

#[derive(Clone, Copy, Debug, PartialOrd, Default)]
pub struct CostCoefficients {
    open_holes: f64,
    closed_holes: f64,
    max_stack_height: f64,
    sum_delta_stack_height: f64,
    max_delta_stack_height: f64,
    rhs_column_height: f64,
    line_clear: f64,
    tetris_clear: f64,
}

impl PartialEq for CostCoefficients {
    fn eq(&self, other: &Self) -> bool {
        // fuzzy equality is fine for these
        is_near_f64(self.open_holes, other.open_holes) &&
            is_near_f64(self.closed_holes, other.closed_holes) &&
            is_near_f64(self.max_stack_height, other.max_stack_height) &&
            is_near_f64(self.sum_delta_stack_height, other.sum_delta_stack_height) &&
            is_near_f64(self.max_delta_stack_height, other.max_delta_stack_height) &&
            is_near_f64(self.rhs_column_height, other.rhs_column_height) &&
            is_near_f64(self.line_clear, other.line_clear) &&
            is_near_f64(self.tetris_clear, other.tetris_clear)
    }
}

impl Eq for CostCoefficients {}

impl Ord for CostCoefficients {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl CostCoefficients {
    const COEFFICIENT_PRECISION: u32 = 8;

    fn precision_f64(value: f64) -> f64 {
        precision_f64(value, Self::COEFFICIENT_PRECISION)
    }

    pub const SENSIBLE_DEFAULTS: CostCoefficients = CostCoefficients {
        open_holes: -1672.4227,
        closed_holes: -1567.3356,
        max_stack_height: -50.282489,
        sum_delta_stack_height: -246.88223,
        max_delta_stack_height: -16.355071,
        rhs_column_height: 493.07072,
        line_clear: 185.28869,
        tetris_clear: 409.99751
    };

    pub fn new(open_holes: f64, closed_holes: f64, max_stack_height: f64, sum_delta_stack_height: f64, max_delta_stack_height: f64, rhs_column_height: f64, line_clear: f64, tetris_clear: f64) -> Self {
        Self {
            open_holes: Self::precision_f64(open_holes),
            closed_holes: Self::precision_f64(closed_holes),
            max_stack_height: Self::precision_f64(max_stack_height),
            sum_delta_stack_height: Self::precision_f64(sum_delta_stack_height),
            max_delta_stack_height: Self::precision_f64(max_delta_stack_height),
            rhs_column_height: Self::precision_f64(rhs_column_height),
            line_clear: Self::precision_f64(line_clear),
            tetris_clear: Self::precision_f64(tetris_clear),
        }
    }

    pub fn open_holes(&self) -> f64 {
        self.open_holes
    }

    pub fn closed_holes(&self) -> f64 {
        self.closed_holes
    }

    pub fn max_stack_height(&self) -> f64 {
        self.max_stack_height
    }

    pub fn sum_delta_stack_height(&self) -> f64 {
        self.sum_delta_stack_height
    }

    pub fn max_delta_stack_height(&self) -> f64 {
        self.max_delta_stack_height
    }

    pub fn rhs_column_height(&self) -> f64 {
        self.rhs_column_height
    }

    pub fn line_clear(&self) -> f64 {
        self.line_clear
    }

    pub fn tetris_clear(&self) -> f64 {
        self.tetris_clear
    }

    pub fn merge_with(&self, other: &Self) -> Self {
        CostCoefficients::new(
            avg(self.open_holes, other.open_holes),
            avg(self.closed_holes, other.closed_holes),
            avg(self.max_stack_height, other.max_stack_height),
            avg(self.sum_delta_stack_height, other.sum_delta_stack_height),
            avg(self.max_delta_stack_height, other.max_delta_stack_height),
            avg(self.rhs_column_height, other.rhs_column_height),
            avg(self.line_clear, other.line_clear),
            avg(self.tetris_clear, other.tetris_clear),
        )
    }

    pub fn mutate(&self, magnitude: f64, rng: &mut impl Rng) -> CostCoefficients {
        CostCoefficients::new(
            rng.nudge(magnitude, self.open_holes),
            rng.nudge(magnitude, self.closed_holes),
            rng.nudge(magnitude, self.max_stack_height),
            rng.nudge(magnitude, self.sum_delta_stack_height),
            rng.nudge(magnitude, self.max_delta_stack_height),
            rng.nudge(magnitude, self.rhs_column_height),
            rng.nudge(magnitude, self.line_clear),
            rng.nudge(magnitude, self.tetris_clear),
        )
    }
}

impl Display for CostCoefficients {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "open_holes: {}, closed_holes: {}, max_stack_height: {}, sum_delta_stack_height: {}, max_delta_stack_height: {}, rhs_column_height: {}, line_clear: {}, tetris_clear: {}",
            self.open_holes, self.closed_holes, self.max_stack_height, self.sum_delta_stack_height, self.max_delta_stack_height, self.rhs_column_height, self.line_clear, self.tetris_clear)
    }
}

pub const COEFFICIENTS_COUNT: usize = 8;
pub type FlatCostCoefficients = [f64; COEFFICIENTS_COUNT];

impl Into<FlatCostCoefficients> for CostCoefficients {
    fn into(self) -> FlatCostCoefficients {
        [
            self.open_holes,
            self.closed_holes,
            self.max_stack_height,
            self.sum_delta_stack_height,
            self.max_delta_stack_height,
            self.rhs_column_height,
            self.line_clear,
            self.tetris_clear
        ]
    }
}

fn avg(v1: f64, v2: f64) -> f64 {
    (v1 + v2) / 2.0
}

pub const COEFFICIENTS_RANGE: RangeInclusive<f64> = -10.0 ..= 10.0;

trait RngCoefficients {
    fn coefficient(&mut self) -> f64;
    fn nudge(&mut self, magnitude: f64, value: f64) -> f64;
}

impl<R: Rng + ?Sized> RngCoefficients for R {
    fn coefficient(&mut self) -> f64 {
        self.random_range(COEFFICIENTS_RANGE)
    }

    fn nudge(&mut self, magnitude: f64, value: f64) -> f64 {
        let delta = (value * magnitude).max(0.01).min(*COEFFICIENTS_RANGE.end());
        let range = value - delta ..= value + delta;
        value + self.random_range(range)
    }
}

impl Distribution<CostCoefficients> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CostCoefficients {
        CostCoefficients::new(
            rng.coefficient(),
            rng.coefficient(),
            rng.coefficient(),
            rng.coefficient(),
            rng.coefficient(),
            rng.coefficient(),
            rng.coefficient(),
            rng.coefficient(),
        )
    }
}

pub struct BoardCost {
    coefficients: CostCoefficients
}

impl BoardCost {
    pub fn new(coefficients: CostCoefficients) -> Self {
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
            // stack_stats.max_heights()[BOARD_WIDTH as usize - 1] as f64 * self.coefficients.rhs_column_height +
            match stack_stats.cleared_lines() {
                1..=3 => cleared_lines * self.coefficients.line_clear,
                4 => cleared_lines * self.coefficients.tetris_clear,
                _ => 0.0
            }
    }
}
