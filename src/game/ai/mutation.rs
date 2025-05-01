use std::array;
use std::ops::RangeInclusive;
use rand::{Rng, SeedableRng};
use std::collections::{HashSet, VecDeque};
use rand_chacha::ChaChaRng;
use crate::game::ai::board_cost::{Genome, COEFFICIENTS_COUNT};
use crate::game::ai::coefficient::Coefficient;
use crate::game::ai::generation_stats::GenerationStatistics;
use crate::game::random::Seed;

#[derive(Debug, Clone)]
pub struct RateLimits {
    range: RangeInclusive<f64>,
    step: f64,
    current: f64,
}

impl RateLimits {
    pub const fn from_static(chance: f64) -> Self {
        Self { range: chance ..= chance, step: 0.0, current: chance }
    }

    pub const NEVER: Self = Self::from_static(0.0);
    pub const ALWAYS: Self = Self::from_static(1.0);
    
    pub fn new(range: RangeInclusive<f64>, step: f64) -> Self {
        assert!(*range.start() >= 0.0 && *range.end() <= 1.0);
        let current = (range.start() + range.end()) / 2.0;
        Self { range, step, current }
    }
    
    fn decrement(&mut self) {
        self.current = (self.current - self.step).max(*self.range.start())
    }
    
    fn increment(&mut self) {
        self.current = (self.current + self.step).min(*self.range.end())
    }
    
    fn test(&self, rng: &mut impl Rng) -> bool {
        let chance: f64 = rng.random();
        chance <= self.current
    }
}

impl Default for RateLimits {
    fn default() -> Self {
        Self::new(0.05 ..= 0.2, 0.01)
    }
}

pub struct GenomeMutation {
    mutation_rate: RateLimits,
    crossover_rate: RateLimits,
    mutation_magnitude: f64,
    samples: VecDeque<f64>,
    stat_fn: fn(GenerationStatistics) -> f64,
    rng: ChaChaRng,
}

impl GenomeMutation {
    fn new(
        mutation_rate: RateLimits,
        crossover_rate: RateLimits,
        mutation_magnitude: f64,
        max_samples: usize,
        seed: Seed,
        stat_fn: fn(GenerationStatistics) -> f64
    ) -> Self {
        Self { mutation_rate, crossover_rate, mutation_magnitude, samples: VecDeque::with_capacity(max_samples), stat_fn, rng: seed.into() }
    }
    
    pub fn current_mutation_rate(&self) -> f64 {
        self.mutation_rate.current
    }
    
    pub fn current_crossover_rate(&self) -> f64 {
        self.crossover_rate.current
    }

    pub fn of_median(
        mutation_rate: RateLimits,
        crossover_rate: RateLimits,
        mutation_magnitude: f64,
        max_samples: usize,
        seed: Seed,
    ) -> Self {
        Self::new(
            mutation_rate,
            crossover_rate,
            mutation_magnitude,
            max_samples,
            seed,
            |stats: GenerationStatistics| stats.median().score() as f64
        )
    }

    pub fn of_max(
        mutation_rate: RateLimits,
        crossover_rate: RateLimits,
        mutation_magnitude: f64,
        max_samples: usize,
        seed: Seed,
    ) -> Self {
        Self::new(
            mutation_rate,
            crossover_rate,
            mutation_magnitude,
            max_samples,
            seed,
            |stats: GenerationStatistics| stats.max().score() as f64
        )
    }

    pub fn add_sample(&mut self, stats: GenerationStatistics) {
        let value = (self.stat_fn)(stats);
        if self.samples.len() >= self.samples.capacity() {
            self.samples.pop_front(); // Remove oldest sample if at capacity
        }
        self.samples.push_back(value);

        if self.samples.len() < self.samples.capacity() {
            return;
        }

        // Calculate trend by comparing latest half of samples to earlier half
        let mid = self.samples.len() / 2;
        let older: f64 = self.samples.iter().take(mid).sum::<f64>() / mid as f64;
        let newer: f64 = self.samples.iter().skip(mid).sum::<f64>() / (self.samples.len() - mid) as f64;
        
        // Adjust rates based on trend
        if newer > older {
            self.mutation_rate.decrement();
            self.crossover_rate.decrement();
        } else {
            self.mutation_rate.increment();
            self.crossover_rate.increment();
        }
    }
    
    pub fn random(&mut self) -> Genome {
        array::from_fn(|_| self.rng.random())
    }
    
    fn mutate(&mut self, genome: Genome) -> Genome {
        genome.map(|coefficient| {
            if self.mutation_rate.test(&mut self.rng) {
                self.rng.mutate(self.mutation_magnitude, coefficient)
            } else {
                coefficient
            }
        })
    }

    pub fn crossover(&mut self, genome1: Genome, genome2: Genome) -> [Genome; 2] {
        let mut child1 = [Coefficient::ZERO; COEFFICIENTS_COUNT];
        let mut child2 = [Coefficient::ZERO; COEFFICIENTS_COUNT];

        for i in 0..COEFFICIENTS_COUNT {
            if self.crossover_rate.test(&mut self.rng) {
                child1[i] = genome2[i];
                child2[i] = genome1[i];
            } else {
                child1[i] = genome1[i];
                child2[i] = genome2[i];
            }
        }

        [self.mutate(child1), self.mutate(child2)]
    }

    pub fn parents(&mut self, population: &[(Genome, f64)], count: usize) -> Vec<[Genome; 2]> {
        let population_softmax = scale(&population);
        let mut parents: HashSet<[Genome; 2]> = HashSet::new();
        for _ in 0..count {
            let mut next_parent = || {
                let p: f64 = self.rng.random();
                let mut cumsum = 0.0;
                for (index, (_, prob)) in population_softmax.iter().enumerate() {
                    cumsum += prob;
                    if p <= cumsum {
                        return index;
                    }
                }
                population_softmax.len() - 1
            };

            let parent1_index = next_parent();
            let (parent1, _) = population_softmax[parent1_index];

            let mut parent2_index = next_parent();
            loop {
                if parent2_index != parent1_index {
                    let (parent2, _) = population_softmax[parent2_index];
                    if parent1 != parent2
                        && !parents.contains(&[parent1, parent2])
                        && !parents.contains(&[parent2, parent1]){
                        break;
                    }
                }
                parent2_index = (parent2_index + 1) % population_softmax.len();
            }

            let (parent2, _) = population_softmax[parent2_index];
            parents.insert([parent1, parent2]);
        }
        parents.into_iter().collect()
    }
}

fn scale(population: &[(Genome, f64)]) -> Vec<(Genome, f64)> {
    // TODO maybe softmax might be better?
    let sum_fitness: f64 = population.iter()
        .map(|(_, fitness)| *fitness)
        .sum();

    population.into_iter().map(|(genome, fitness)| (*genome, fitness / sum_fitness)).collect()
}

trait RngMutation {
    fn mutate(&mut self, magnitude: f64, value: Coefficient) -> Coefficient;
}

impl<R: Rng + ?Sized> RngMutation for R {
    fn mutate(&mut self, magnitude: f64, value: Coefficient) -> Coefficient {
        let value_f64: f64 = value.into();
        let delta = (value_f64 * magnitude).abs().clamp(0.001, 10.0);
        Coefficient::from_f64(value_f64 + self.random_range(-delta ..= delta))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use super::*;

    fn mutation<M : Into<Option<f64>>, MR: Into<Option<RateLimits>>, CR : Into<Option<RateLimits>>>(
        mutation_magnitude: M,
        mutation_rate: MR,
        crossover_rate: CR
    ) -> GenomeMutation {
        GenomeMutation::of_max(
            mutation_rate.into().unwrap_or(RateLimits::default()),
            crossover_rate.into().unwrap_or(RateLimits::default()),
            mutation_magnitude.into().unwrap_or(0.01),
            10,
            Seed::default()
        )
    }
    
    fn genome(i: i32) -> Genome {
        [Coefficient(i as i64); COEFFICIENTS_COUNT]
    }
    
    #[test]
    fn fittest_genome_can_be_parent_multiple_times() {
        const N: usize = 100;
        let population: Vec<_> = (0 .. N as i32).map(|i| {
            let fitness = 0.5f64.powi(i); // 1.0, 0.5, 0.25, 0.125 etc
            (genome(i + 1), fitness)
        }).collect();
        let parents = mutation(None, None, None).parents(&population, N / 2);

        assert_eq!(parents.len(), N / 2);

        let counts: Vec<_> = parents.iter()
            .flatten()
            .map(|[c0, ..]| *c0)
            .sorted()
            .dedup_with_count()
            .collect();

        assert!(counts[0].0 > 1, "Coefficient(1) should appear more than once, got {}", counts[0].0);
        assert!(counts[0].0 > counts[1].0, "Coefficient(1) should appear more often than Coefficient(2), got {} & {}", counts[0].0, counts[1].0);
    }
    

    #[test]
    fn crossover_can_swap_all_genes_without_modification() {
        let parent1 = genome(1);
        let parent2 = genome(2);

        let [child1, child2] = mutation(0.0, RateLimits::NEVER, RateLimits::ALWAYS)
            .crossover(parent1, parent2);

        // With 100% crossover rate and 0% mutation rate, children should have swapped all genes without modification
        assert_eq!(child1, parent2);
        assert_eq!(child2, parent1);
    }

    #[test]
    fn crossover_swaps_no_genes_when_disabled() {
        let parent1 = genome(1);
        let parent2 = genome(2);

        let [child1, child2] = mutation(0.0, RateLimits::NEVER, RateLimits::NEVER)
            .crossover(parent1, parent2);

        // With 0% crossover rate and 0% mutation rate, children should be identical to parents
        assert_eq!(child1, parent1);
        assert_eq!(child2, parent2);
    }

    #[test]
    fn crossover_can_mutate_all_genes() {
        let parent1 = genome(1);
        let parent2 = genome(2);

        let [child1, child2] = mutation(0.01, RateLimits::ALWAYS, RateLimits::NEVER)
            .crossover(parent1, parent2);

        // With 100% mutation rate and non-zero magnitude, all genes should be modified
        assert!(child1 != parent1 && child1 != parent2);
        assert!(child2 != parent1 && child2 != parent2);
    }
    
    #[test]
    fn crossover_can_both_mutate_and_swap_some_genes() {
        let parent1 = genome(1);
        let parent2 = genome(2);

        let children = mutation(0.01, RateLimits::from_static(0.1), RateLimits::from_static(0.2))
            .crossover(parent1, parent2);

        for child in children {
            // With 50% rates, some genes should be swapped and some should be mutated
            let has_parent1_genes = child.iter().any(|&gene| gene == parent1[0]);
            let has_parent2_genes = child.iter().any(|&gene| gene == parent2[0]);
            let has_mutated_genes = child.iter().any(|&gene| gene != parent1[0] && gene != parent2[0]);

            assert!(has_parent1_genes, "Child should contain some genes from parent1");
            assert!(has_parent2_genes, "Child should contain some genes from parent2");
            assert!(has_mutated_genes, "Child should contain some mutated genes");
        }
    }

    #[test]
    fn mutates_coefficients() {
        let mut rng = ChaChaRng::seed_from_u64(100);
        let coefficient = Coefficient::from_f64(10.0);
        let result = rng.mutate(0.1, coefficient);
        assert_ne!(result, coefficient);
        assert!(result >= Coefficient::from_f64(9.0) && result <= Coefficient::from_f64(11.0));
    }
    
    #[test]
    fn mutates_negative_coefficients() {
        let mut rng = ChaChaRng::seed_from_u64(100);
        let coefficient = Coefficient::from_f64(-10.0);
        let result = rng.mutate(0.1, coefficient);
        assert_ne!(result, coefficient);
        assert!(result >= Coefficient::from_f64(-11.0) && result <= Coefficient::from_f64(-9.0));
    }
}
