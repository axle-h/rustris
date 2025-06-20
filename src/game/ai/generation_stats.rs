use std::fmt::{Display, Formatter};
use crate::game::ai::organism::Organism;
use crate::game::random::Seed;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GenerationStatistics<const GENOME: usize> {
    id: usize,
    seed: Seed,
    max: Organism<GENOME>,
    p95: Organism<GENOME>,
    median: Organism<GENOME>,
    mutation_rate: f64,
    crossover_rate: f64,
}

impl<const GENOME: usize> Display for GenerationStatistics<GENOME> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] p100: {{{}}}, p95: {{{}}}, p50: {{{}}}, mutation_rate: {:.2}, crossover_rate: {:.2}",
               self.id, self.max, self.p95.result(), self.median.result(), self.mutation_rate, self.crossover_rate)
    }
}

impl<const GENOME: usize> GenerationStatistics<GENOME> {
    pub fn new(id: usize, seed: Seed, max: Organism<GENOME>, p95: Organism<GENOME>, median: Organism<GENOME>, mutation_rate: f64, crossover_rate: f64) -> Self {
        Self { id, seed, max, p95, median, mutation_rate, crossover_rate }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn max(&self) -> Organism<GENOME> {
        self.max
    }

    pub fn p95(&self) -> Organism<GENOME> {
        self.p95
    }

    pub fn median(&self) -> Organism<GENOME> {
        self.median
    }

    pub fn mutation_rate(&self) -> f64 {
        self.mutation_rate
    }

    pub fn crossover_rate(&self) -> f64 {
        self.crossover_rate
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }
}

