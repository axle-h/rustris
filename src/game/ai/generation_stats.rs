use crate::game::ai::board_cost::AiCoefficients;
use crate::game::ai::game_result::GameResult;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerationStatistics {
    id: usize,
    max: GameResult,
    median: GameResult,
    best: AiCoefficients,
    mutation_rate: f64,
    crossover_rate: f64,
}

impl GenerationStatistics {
    pub fn new(id: usize, max: GameResult, median: GameResult, best: AiCoefficients, mutation_rate: f64, crossover_rate: f64) -> Self {
        Self { id, max, median, best, mutation_rate, crossover_rate }
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

    pub fn best(&self) -> AiCoefficients {
        self.best
    }

    pub fn mutation_rate(&self) -> f64 {
        self.mutation_rate
    }

    pub fn crossover_rate(&self) -> f64 {
        self.crossover_rate
    }
}