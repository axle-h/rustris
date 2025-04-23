use crate::game::ai::board_features::BoardFeatures;
use crate::game::board::{Board, BOARD_WIDTH};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CostCoefficients {
    open_holes: f32,
    closed_holes: f32,
    max_stack_height: f32,
    delta_stack_height: f32,
    column_heights: [f32; BOARD_WIDTH as usize],
    
    // TODO clearing less than 4
}

impl CostCoefficients {
    pub const SENSIBLE_DEFAULTS: CostCoefficients = CostCoefficients {
        open_holes: 0.95,
        closed_holes: 1.0,
        max_stack_height: 0.1,
        delta_stack_height: 0.1,
        column_heights: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    };
}

pub struct BoardCost {
    coefficients: CostCoefficients,
}

impl BoardCost {
    pub fn new(coefficients: CostCoefficients) -> Self {
        Self { coefficients }
    }

    pub fn cost(&self, board: Board) -> f32 {
        let holes = board.holes();
        let stack_stats = board.stack_stats();

        let open_holes = holes.open() as f32 * self.coefficients.open_holes;
        let closed_holes = holes.closed() as f32 * self.coefficients.closed_holes;
        let max_stack_height = stack_stats.max_heights().into_iter().max().unwrap() as f32
            * self.coefficients.max_stack_height;
        let delta_stack_height = stack_stats.delta_height() as f32 * self.coefficients.delta_stack_height;
        
        let column_heights = stack_stats
            .max_heights().into_iter()
            .zip(self.coefficients.column_heights).map(|(height, coef)| height as f32 * coef)
            .sum::<f32>();

        (open_holes + closed_holes + max_stack_height + delta_stack_height + column_heights) / 5.0
    }
}