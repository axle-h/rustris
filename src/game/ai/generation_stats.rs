use crate::game::ai::board_cost::CostCoefficients;
use crate::game::ai::game_result::GameResult;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerationStatistics {
    id: usize,
    max: GameResult,
    median: GameResult,
    best: CostCoefficients,
    mutation_rate: f64
}

impl GenerationStatistics {
    pub fn new(id: usize, max: GameResult, median: GameResult, best: CostCoefficients, mutation_rate: f64) -> Self {
        Self { id, max, median, best, mutation_rate }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn max(&self) -> GameResult {
        self.max
    }

    pub fn median(&self) -> GameResult {
        self.median
    }

    pub fn best(&self) -> CostCoefficients {
        self.best
    }

    pub fn mutation_rate(&self) -> f64 {
        self.mutation_rate
    }
}