use std::array;
use rand::distr::{Distribution, StandardUniform};
use rand::Rng;
use rand_chacha::ChaChaRng;
use crate::game::ai::board_features::BoardFeatures;
use crate::game::ai::genetic::{Mutate, Merge};
use crate::game::board::{Board, BOARD_WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CostCoefficients {
    open_holes: f32,
    closed_holes: f32,
    max_stack_height: f32,
    delta_stack_height: f32,
    rhs_column_height: f32,
    line_clear: f32,
    tetris_clear: f32,
}

impl CostCoefficients {
    pub const SENSIBLE_DEFAULTS: CostCoefficients = CostCoefficients {
        open_holes: -0.95,
        closed_holes: -1.0,
        max_stack_height: -0.2,
        delta_stack_height: -0.1,
        rhs_column_height: 0.0,
        line_clear: 0.0,
        tetris_clear: 1.0
    };

    pub fn open_holes(&self) -> f32 {
        self.open_holes
    }

    pub fn closed_holes(&self) -> f32 {
        self.closed_holes
    }

    pub fn max_stack_height(&self) -> f32 {
        self.max_stack_height
    }

    pub fn delta_stack_height(&self) -> f32 {
        self.delta_stack_height
    }

    pub fn rhs_column_height(&self) -> f32 {
        self.rhs_column_height
    }

    pub fn line_clear(&self) -> f32 {
        self.line_clear
    }

    pub fn tetris_clear(&self) -> f32 {
        self.tetris_clear
    }
}

pub const COEFFICIENTS_COUNT: usize = 7;
pub type FlatCostCoefficients = [f32; COEFFICIENTS_COUNT];

impl Into<FlatCostCoefficients> for CostCoefficients {
    fn into(self) -> FlatCostCoefficients {
        [
            self.open_holes,
            self.closed_holes,
            self.max_stack_height,
            self.delta_stack_height,
            self.rhs_column_height,
            self.line_clear,
            self.tetris_clear
        ]
    }
}

trait RngCoefficients {
    fn coefficient(&mut self) -> f32;
    fn nudge(&mut self, magnitude: f32, value: f32) -> f32;
}

impl<R: Rng + ?Sized> RngCoefficients for R {
    fn coefficient(&mut self) -> f32 {
        self.random_range(-1.0 ..= 1.0)
    }

    fn nudge(&mut self, magnitude: f32, value: f32) -> f32 {
        let from = value - magnitude;
        let to = value + magnitude;
        (value + self.random_range(from ..= to)).clamp(-1.0, 1.0)
    }
}

impl Distribution<CostCoefficients> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CostCoefficients {
        // CostCoefficients::SENSIBLE_DEFAULTS.mutate(0.2, rng)
        CostCoefficients {
            open_holes: rng.coefficient(),
            closed_holes: rng.coefficient(),
            max_stack_height: rng.coefficient(),
            delta_stack_height: rng.coefficient(),
            rhs_column_height: rng.coefficient(),
            line_clear: rng.coefficient(),
            tetris_clear: rng.coefficient(),
        }
    }
}

fn avg(v1: f32, v2: f32) -> f32 {
    (v1 + v2) / 2.0
}

impl Merge for CostCoefficients {
    fn merge_with(&self, other: &Self) -> Self {
        CostCoefficients {
            open_holes: avg(self.open_holes, other.open_holes),
            closed_holes: avg(self.closed_holes, other.closed_holes),
            max_stack_height: avg(self.max_stack_height, other.max_stack_height),
            delta_stack_height: avg(self.delta_stack_height, other.delta_stack_height),
            rhs_column_height: avg(self.rhs_column_height, other.rhs_column_height),
            line_clear: avg(self.line_clear, other.line_clear),
            tetris_clear: avg(self.tetris_clear, other.tetris_clear),
        }
    }
}

impl Mutate for CostCoefficients {
    fn mutate<R: Rng + ?Sized>(&self, magnitude: f32, rng: &mut R) -> Self {
        CostCoefficients {
            open_holes: rng.nudge(magnitude, self.open_holes),
            closed_holes: rng.nudge(magnitude, self.closed_holes),
            max_stack_height: rng.nudge(magnitude, self.max_stack_height),
            delta_stack_height: rng.nudge(magnitude, self.delta_stack_height),
            rhs_column_height: rng.nudge(magnitude, self.rhs_column_height),
            line_clear: rng.nudge(magnitude, self.line_clear),
            tetris_clear: rng.nudge(magnitude, self.tetris_clear),
        }
    }
}

pub struct BoardCost {
    coefficients: CostCoefficients
}

impl BoardCost {
    pub fn new(coefficients: CostCoefficients) -> Self {
        Self { coefficients }
    }

    pub fn cost(&self, board: Board) -> f32 {
        let stack_stats = board.stack_stats();

        let open_holes = stack_stats.open_holes() as f32 * self.coefficients.open_holes;
        let closed_holes = stack_stats.closed_holes() as f32 * self.coefficients.closed_holes;
        let max_stack_height = stack_stats.max_heights().into_iter().max().unwrap() as f32
            * self.coefficients.max_stack_height;
        let delta_stack_height = stack_stats.delta_height() as f32 * self.coefficients.delta_stack_height;
        
        let rhs_column_height = stack_stats.max_heights()[BOARD_WIDTH as usize - 1] as f32 * self.coefficients.rhs_column_height;

        let line_clear = match stack_stats.cleared_lines() {
            1..=3 => self.coefficients.line_clear,
            4 => self.coefficients.tetris_clear,
            _ => 0.0
        };

        (open_holes + closed_holes + max_stack_height + delta_stack_height + rhs_column_height + line_clear) / 6.0
    }
}