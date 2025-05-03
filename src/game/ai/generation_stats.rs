use std::fmt::{Display, Formatter};
use crate::game::ai::board_cost::{AiCoefficients, Genome};
use crate::game::ai::game_result::GameResult;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerationStatistics {
    id: usize,
    max: GenerationResult,
    p95: GenerationResult,
    median: GenerationResult,
    mutation_rate: f64,
    crossover_rate: f64,
}

impl Display for GenerationStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] p100: {{{}}}, p95: {{{}}}, p50: {{{}}}, mutation_rate: {:.2}, crossover_rate: {:.2}",
               self.id, self.max, self.p95, self.median, self.mutation_rate, self.crossover_rate)
    }
}

impl GenerationStatistics {
    pub fn new(id: usize, max: GenerationResult, p95: GenerationResult, median: GenerationResult, mutation_rate: f64, crossover_rate: f64) -> Self {
        Self { id, max, p95, median, mutation_rate, crossover_rate }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn max(&self) -> GenerationResult {
        self.max
    }

    pub fn p95(&self) -> GenerationResult {
        self.p95
    }

    pub fn median(&self) -> GenerationResult {
        self.median
    }

    pub fn mutation_rate(&self) -> f64 {
        self.mutation_rate
    }

    pub fn crossover_rate(&self) -> f64 {
        self.crossover_rate
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerationResult {
    coefficients: AiCoefficients,
    result: GameResult,
}

impl Display for GenerationResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.result, self.coefficients)
    }
}

impl GenerationResult {
    pub fn new(coefficients: AiCoefficients, result: GameResult) -> Self {
        Self { coefficients, result }
    }

    pub fn result(&self) -> GameResult {
        self.result
    }

    pub fn coefficients(&self) -> AiCoefficients {
        self.coefficients
    }
}

impl From<(Genome, GameResult)> for GenerationResult {
    fn from((genome, result): (Genome, GameResult)) -> Self {
        Self::new(genome.into(), result)
    }
}