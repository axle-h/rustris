use std::array;
use std::collections::BTreeMap;
use std::ops::{Add, Div};
use rand::{Rng, SeedableRng};
use rand::prelude::SliceRandom;
use rand_chacha::ChaChaRng;
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::board_cost::CostCoefficients;
use crate::game::ai::game_result::GameResult;
use crate::game::ai::generation_stats::{GenerationStatistics};
use crate::game::ai::headless::{HeadlessGameFixture, HeadlessGameOptions};
use crate::game::ai::mutation::{MutationRate, MutationRateLimits};
use crate::game::random::new_seed;


pub struct GeneticAlgorithm {
    population: [CostCoefficients; POPULATION_SIZE],
    generations: Vec<GenerationStatistics>,
    fixture: HeadlessGameFixture,
    rng: ChaChaRng,
    mutation: MutationRate,
    elite_count: usize,
    mutate_count: usize,
    breed_count: usize,
    cached_scores: BTreeMap<CostCoefficients, GameResult>
}

const POPULATION_SIZE: usize = 100;

impl GeneticAlgorithm {
    // TODO increase game length as games start to get longer
    pub fn new(fixture: HeadlessGameFixture, elite_ratio: f64, mutate_ratio: f64, breed_ratio: f64, mutation: MutationRate) -> Result<Self, String> {
        let size_f64 = POPULATION_SIZE as f64;
        let elite_count = (size_f64 * elite_ratio).ceil().clamp(0.0, size_f64) as usize;
        let mutate_count = (size_f64 * mutate_ratio).ceil().clamp(0.0, size_f64) as usize;
        let breed_count = (size_f64 * breed_ratio).ceil().clamp(0.0, size_f64) as usize;

        let total_count = elite_count.max(mutate_count + breed_count);

        if total_count > POPULATION_SIZE {
            return Err(format!(
                "Total population counts ({}) exceeds population size ({})",
                total_count, POPULATION_SIZE
            ));
        }

        let mut rng = ChaChaRng::from_os_rng();
        Ok(Self {
            population: array::from_fn(|_| rng.random()),
            generations: vec![],
            fixture,
            rng,
            mutation,
            elite_count,
            mutate_count,
            breed_count,
            cached_scores: BTreeMap::new()
        })
    }

    pub fn evolve(&mut self) -> GenerationStatistics {
        // Calculate fitness in parallel
        let mut labelled_population: Vec<_> = self.population
            .into_par_iter()
            .map(|coefficients| {
                let result = if let Some(cached) = self.cached_scores.get(&coefficients) {
                    *cached
                } else {
                    self.fixture.play(coefficients)
                };
                (coefficients, result)
            })
            .collect();

        // update cache outside of the parallel iterator
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
            self.population[0],
            self.mutation.current_rate()
        );        
        self.generations.push(stats);
        self.mutation.add_sample(stats);
        
        self.population = self.next_generation(&labelled_population).try_into().unwrap();

        stats
    }

    fn next_generation(&mut self, labelled_population: &[(CostCoefficients, GameResult)]) -> Vec<CostCoefficients> {
        let mut elite_population: Vec<CostCoefficients> = vec![];
        let mut mutated_population: Vec<CostCoefficients> = vec![];
        let mut breeding_population: Vec<CostCoefficients> = vec![];
        for (coefficient, _) in labelled_population.iter() {
            let mut added = false;
            if elite_population.len() < self.elite_count {
                elite_population.push(*coefficient);
                added = true;
            }
            if breeding_population.len() < self.breed_count {
                breeding_population.push(*coefficient);
                added = true;
            } else if mutated_population.len() < self.mutate_count {
                let mutated = self.mutation.mutate(1.0, *coefficient, &mut self.rng);
                mutated_population.push(mutated);
                added = true;
            }

            if !added {
                break;
            }
        }

        breeding_population.shuffle(&mut self.rng);
        let mut offspring = breeding_population
            .chunks(2)
            .filter_map(|chunk| {
                if let [x, y] = chunk {
                    Some(self.mutation.mutate(0.2, x.merge_with(y), &mut self.rng))
                } else { None }
            })
            .collect();

        let mut next_generation = vec![];
        next_generation.append(&mut elite_population);
        next_generation.append(&mut mutated_population);
        next_generation.append(&mut offspring);
        while next_generation.len() < POPULATION_SIZE {
            next_generation.push(self.rng.random());
        }

        next_generation
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
        vec![new_seed(), new_seed()],
        HeadlessGameOptions::default(),
        2
    );
    let mutation = MutationRate::of_max(MutationRateLimits::default(), 5);
    let mut ga = GeneticAlgorithm::new(fixture, 0.05, 0.5, 0.20, mutation)?;

    println!("starting");
    for _ in 0..10_000 {
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
    use crate::game::ai::headless::{HeadlessGameFixture, HeadlessGameOptions};
    use crate::game::ai::mutation::{MutationRate, MutationRateLimits};

    #[test]
    fn genetic_algorithm() {
        let fixture = HeadlessGameFixture::new(
            Config::default(),
            vec![100],
            HeadlessGameOptions::new(
                Duration::from_millis(1_000),
                Duration::from_millis(100),
                Duration::from_millis(16),
            ),
            2
        );
        let mutation = MutationRate::of_max(MutationRateLimits::default(), 5);
        let mut ga = GeneticAlgorithm::new(fixture, 0.01, 0.4, 0.2, mutation).unwrap();
        let stats = ga.evolve();
        println!("{:?}", stats);

        assert!(true);
    }
}