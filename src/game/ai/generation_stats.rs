use std::fmt::{Display, Formatter};
use crate::game::ai::game_result::GameResult;
use crate::game::ai::genome::Genome;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerationStatistics<const N: usize> {
    id: usize,
    max: GenerationResult<N>,
    p95: GenerationResult<N>,
    median: GenerationResult<N>,
    mutation_rate: f64,
    crossover_rate: f64,
}

impl<const N: usize> Display for GenerationStatistics<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] p100: {{{}}}, p95: {{{}}}, p50: {{{}}}, mutation_rate: {:.2}, crossover_rate: {:.2}",
               self.id, self.max, self.p95.result, self.median.result, self.mutation_rate, self.crossover_rate)
    }
}

impl<const N: usize> GenerationStatistics<N> {
    pub fn new(id: usize, max: GenerationResult<N>, p95: GenerationResult<N>, median: GenerationResult<N>, mutation_rate: f64, crossover_rate: f64) -> Self {
        Self { id, max, p95, median, mutation_rate, crossover_rate }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn max(&self) -> GenerationResult<N> {
        self.max
    }

    pub fn p95(&self) -> GenerationResult<N> {
        self.p95
    }

    pub fn median(&self) -> GenerationResult<N> {
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
pub struct GenerationResult<const N: usize> {
    genome: Genome<N>,
    result: GameResult,
}

impl<const N: usize> Display for GenerationResult<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.result, self.genome)
    }
}

impl<const N: usize> GenerationResult<N> {
    pub fn new(genome: Genome<N>, result: GameResult) -> Self {
        Self { genome, result }
    }

    pub fn result(&self) -> GameResult {
        self.result
    }

    pub fn genome(&self) -> Genome<N> {
        self.genome
    }
}

impl<const N: usize>  From<(Genome<N>, GameResult)> for GenerationResult<N> {
    fn from((genome, result): (Genome<N>, GameResult)) -> Self {
        Self::new(genome.into(), result)
    }
}