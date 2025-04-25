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
    column_heights: [f32; BOARD_WIDTH as usize],
    cleared_blocks: [f32; 5], // 0 through 4
}

impl CostCoefficients {
    pub const SENSIBLE_DEFAULTS: CostCoefficients = CostCoefficients {
        open_holes: -0.95,
        closed_holes: -1.0,
        max_stack_height: -0.2,
        delta_stack_height: -0.1,
        column_heights: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        cleared_blocks: [0.0, 0.0, 0.0, 0.0, 100.0]
    };
}

trait RngCoefficients {
    fn coefficient(&mut self) -> f32;
    fn nudge(&mut self, magnitude: f32, value: f32) -> f32;
}

impl<R: Rng + ?Sized> RngCoefficients for R {
    fn coefficient(&mut self) -> f32 {
        self.random_range(-1.0 .. 1.0)
    }

    fn nudge(&mut self, magnitude: f32, value: f32) -> f32 {
        let from = value - magnitude;
        let to = value + magnitude;
        value + self.random_range(from ..= to)
    }
}

impl Distribution<CostCoefficients> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CostCoefficients {
        CostCoefficients {
            open_holes: rng.coefficient(),
            closed_holes: rng.coefficient(),
            max_stack_height: rng.coefficient(),
            delta_stack_height: rng.coefficient(),
            // column_heights: array::from_fn(|_| rng.coefficient()),
            column_heights: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            cleared_blocks: array::from_fn(|_| rng.coefficient()),
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
            column_heights: array::from_fn(|i| avg(self.column_heights[i], other.column_heights[i])),
            cleared_blocks: array::from_fn(|i| avg(self.cleared_blocks[i], other.cleared_blocks[i])),
        }
    }
}

impl Mutate for CostCoefficients {
    fn mutate(&self, magnitude: f32, rng: &mut ChaChaRng) -> Self {
        CostCoefficients {
            open_holes: rng.nudge(magnitude, self.open_holes),
            closed_holes: rng.nudge(magnitude, self.closed_holes),
            max_stack_height: rng.nudge(magnitude, self.max_stack_height),
            delta_stack_height: rng.nudge(magnitude, self.delta_stack_height),
            // column_heights: array::from_fn(|i| rng.nudge(magnitude, self.column_heights[i])),
            column_heights: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            cleared_blocks: array::from_fn(|i| rng.nudge(magnitude, self.cleared_blocks[i])),
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
        
        let column_heights = stack_stats
            .max_heights().into_iter()
            .zip(self.coefficients.column_heights).map(|(height, coef)| height as f32 * coef)
            .sum::<f32>();

        let cleared_blocks = self.coefficients.cleared_blocks[stack_stats.cleared_lines() as usize];

        open_holes + closed_holes + max_stack_height + delta_stack_height + column_heights + cleared_blocks
    }
}