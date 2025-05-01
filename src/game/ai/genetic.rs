use std::array;
use std::collections::HashMap;
use std::ops::{Add, Div};
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::board_cost::Genome;
use crate::game::ai::game_result::GameResult;
use crate::game::ai::generation_stats::{GenerationStatistics};
use crate::game::ai::headless_game::{HeadlessGameFixture, HeadlessGameOptions};
use crate::game::ai::mutation::{GenomeMutation, RateLimits};

pub struct GeneticAlgorithm {
    population: [Genome; POPULATION_SIZE],
    generations: Vec<GenerationStatistics>,
    fixture: HeadlessGameFixture,
    mutation: GenomeMutation,
    elite_count: usize,
    cached_scores: HashMap<Genome, GameResult>
}

const POPULATION_SIZE: usize = 1000;

impl GeneticAlgorithm {
    // TODO increase game length as games start to get longer
    pub fn new(fixture: HeadlessGameFixture, elite_ratio: f64, mut mutation: GenomeMutation) -> Self {
        Self {
            population: array::from_fn(|_| mutation.random()),
            generations: vec![],
            fixture,
            mutation,
            elite_count: (POPULATION_SIZE as f64 * elite_ratio).ceil().clamp(0.0, POPULATION_SIZE as f64) as usize,
            cached_scores: HashMap::new()
        }
    }

    pub fn evolve(&mut self) -> GenerationStatistics {
        // Calculate fitness in parallel
        let mut labelled_population: Vec<_> = self.population
            .into_par_iter()
            .map(|genome| {
                let result = if let Some(cached) = self.cached_scores.get(&genome) {
                    *cached
                } else {
                    self.fixture.play(genome.into())
                };
                (genome, result)
            })
            .collect();

        // update cache in sync
        for (coefficients, result) in labelled_population.iter().copied() {
            self.cached_scores.insert(coefficients, result);
        }

        labelled_population.sort_by(|(_, s1), (_, s2)| s2.cmp(s1));

        let results: Vec<_> = labelled_population.iter().map(|(_, s1)| *s1).collect();
        // let coefficient_diversity_sample: Vec<FlatCostCoefficients> = self.population.iter()
        //     .take(self.diversity_sample_size)
        //     .map(|&p| p.into())
        //     .collect();
        // let mean_coefficient_std_dev = (0 .. COEFFICIENTS_COUNT).map(|index|
        //     coefficient_diversity_sample.iter()
        //         .map(|coefficients| coefficients[index])
        //         .collect::<Vec<f64>>()
        //         .std_dev()
        // ).collect::<Vec<f64>>().into_iter().sum::<f64>() / COEFFICIENTS_COUNT as f64;

        let stats = GenerationStatistics::new(
            self.generations.len(),
            results[0],
            pre_sorted_median_of(&results),
            self.population[0].into(),
            self.mutation.current_mutation_rate(),
            self.mutation.current_crossover_rate()
        );
        self.generations.push(stats);
        self.mutation.add_sample(stats);
        
        self.population = self.next_generation(labelled_population).try_into().unwrap();

        stats
    }

    fn next_generation(&mut self, labelled_population: Vec<(Genome, GameResult)>) -> Vec<Genome> {
        // TODO use this same score for classification above
        let mut genome_by_fitness: Vec<_> = labelled_population.into_iter()
            .map(|(genome, result)| {
                let game_over_penalty = if result.game_over() { 100_000.0 } else { 0.0 };
                let fitness = result.score() as f64 - game_over_penalty;
                (genome, fitness)
            }).collect();
        genome_by_fitness.sort_by(|(_, f1), (_, f2)| f2.partial_cmp(f1).unwrap());

        let elite_population: Vec<_> = genome_by_fitness.iter().take(self.elite_count).map(|(genome, _)| *genome).collect();
        let children_count = ((POPULATION_SIZE as f64 - elite_population.len() as f64) / 2.0).ceil() as usize;
        let children: Vec<_> = self.mutation.parents(&genome_by_fitness, children_count).into_iter()
            .flat_map(|[parent1, parent2]| self.mutation.crossover(parent1, parent2))
            .collect();
        
        elite_population.into_iter().chain(children).take(POPULATION_SIZE).collect()
    }
}

/// the input must be pre-sorted
fn pre_sorted_median_of<T: PartialOrd + Copy + Add<Output = T> + Div<usize, Output = T>>(arr: &[T]) -> T {
    let len = arr.len();
    let mid = len / 2;
    if len % 2 == 0 {
        // Even number of elements: median is the average of the two middle elements
        let left = arr[mid - 1];
        let right = arr[mid];
        (left + right) / 2usize
    } else {
        // Odd number of elements: median is the middle element
        arr[mid]
    }
}

#[derive(Debug, Copy, Clone)]
struct PopulationCounts {
    elite: usize,
    mutate: usize,
    breed: usize,
}

impl PopulationCounts {
    fn from_ratios(elite_ratio: f64, mutate_ratio: f64, breed_ratio: f64) -> Result<Self, String> {
        let size_f64 = POPULATION_SIZE as f64;
        let elite_count = (size_f64 * elite_ratio).ceil().clamp(0.0, size_f64) as usize;
        let mutation_count = (size_f64 * mutate_ratio).ceil().clamp(0.0, size_f64) as usize;
        let breed_count = (size_f64 * breed_ratio).ceil().clamp(0.0, size_f64) as usize;

        let total_count = elite_count + mutation_count + breed_count;

        if total_count > POPULATION_SIZE {
            return Err(format!(
                "Total population counts ({}) exceeds population size ({})",
                total_count, POPULATION_SIZE
            ));
        }

        Ok(Self {
            elite: elite_count,
            mutate: mutation_count,
            breed: breed_count,
        })
    }
}


pub fn ga_main() -> Result<(), String> {
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        vec![rand::random()],
        HeadlessGameOptions::default(),
        0
    );
    let mutation = GenomeMutation::of_max(
        RateLimits::default(),
        RateLimits::default(),
        0.05,
        5,
        rand::random()
    );
    let mut ga = GeneticAlgorithm::new(fixture, 0.02, mutation);

    println!("starting");
    for _ in 0..100_000 {
        let stats = ga.evolve();
        println!("{:?}", stats);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::config::Config;
    use crate::game::ai::genetic::GeneticAlgorithm;
    use crate::game::ai::headless_game::{HeadlessGameFixture, HeadlessGameOptions};
    use crate::game::ai::mutation::{GenomeMutation, RateLimits};

    #[test]
    fn genetic_algorithm() {
        let fixture = HeadlessGameFixture::new(
            Config::default(),
            vec![100.into()],
            HeadlessGameOptions::new(
                Duration::from_millis(100),
                Duration::from_millis(100),
                Duration::from_millis(16),
            ),
            0
        );
        let mutation = GenomeMutation::of_max(RateLimits::default(), RateLimits::default(), 0.05, 5, 100.into());
        let mut ga = GeneticAlgorithm::new(fixture, 0.01, mutation);
        let stats = ga.evolve();
        println!("{:?}", stats);

        assert!(true);
    }
}