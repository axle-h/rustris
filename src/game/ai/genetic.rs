use std::array;
use std::ops::{Add, Div};
use std::time::Duration;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;
use rand_chacha::ChaChaRng;
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::board_cost::{CostCoefficients, FlatCostCoefficients, COEFFICIENTS_COUNT};
use crate::game::ai::game_result::GameResult;
use crate::game::ai::headless::HeadlessGameFixture;
use crate::game::ai::stats::StdDev;
use crate::game::random::new_seed;


#[derive(Clone, Debug, PartialEq)]
pub struct GenerationStatistics {
    id: usize,
    max: GameResult,
    mean: GameResult,
    median: GameResult,
    best: CostCoefficients,
    mutation_rate: f32,
    fitness_diversity: f32,
    genetic_diversity: f32
}

pub struct GeneticAlgorithm {
    population: [CostCoefficients; POPULATION_SIZE],
    generations: Vec<GenerationStatistics>,
    fixture: HeadlessGameFixture,
    rng: ChaChaRng,
    mutation_rate: f32,
    elite_count: usize,
    mutate_count: usize,
    breed_count: usize,
    diversity_sample_size: usize,
}

const POPULATION_SIZE: usize = 100;
const DIVERSITY_SAMPLE_RATIO: f32 = 0.5; // percent of the top population that will be sampled for diversity calculations
const MIN_MUTATION_RATE: f32 = 0.01;
const MAX_MUTATION_RATE: f32 = 1.0;
const MUTATION_RATE_STEP: f32 = 0.05;
const TARGET_GENETIC_DIVERSITY: f32 = 0.5;


impl GeneticAlgorithm {
    pub fn new(fixture: HeadlessGameFixture, elite_ratio: f32, mutate_ratio: f32, breed_ratio: f32) -> Result<Self, String> {
        let size_f32 = POPULATION_SIZE as f32;
        let elite_count = (size_f32 * elite_ratio).ceil().clamp(0.0, size_f32) as usize;
        let mutate_count = (size_f32 * mutate_ratio).ceil().clamp(0.0, size_f32) as usize;
        let breed_count = (size_f32 * breed_ratio).ceil().clamp(0.0, size_f32) as usize;

        let total_count = elite_count + mutate_count + breed_count;

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
            mutation_rate: MAX_MUTATION_RATE,
            elite_count,
            mutate_count,
            breed_count,
            diversity_sample_size: ((POPULATION_SIZE as f32 * DIVERSITY_SAMPLE_RATIO).round() as usize).clamp(1, POPULATION_SIZE),
        })
    }

    pub fn evolve(&mut self) -> GenerationStatistics {
        self.adapt_mutation_rate();
        
        // Calculate fitness in parallel
        let mut labelled_population: Vec<_> = self.population
            .into_par_iter()
            .map(|coefficients| {
                (coefficients, self.fixture.play(coefficients))
            })
            .collect();
        labelled_population.sort_by(|(_, s1), (_, s2)| s2.cmp(s1));

        let next_generation = self.next_generation(&labelled_population);

        let results: Vec<_> = labelled_population.into_iter().map(|(_, s1)| s1).collect();
        let coefficient_diversity_sample: Vec<FlatCostCoefficients> = self.population.iter()
            .take(self.diversity_sample_size)
            .map(|&p| p.into())
            .collect();
        let mean_coefficient_std_dev = (0 .. COEFFICIENTS_COUNT).map(|index|
            coefficient_diversity_sample.iter()
                .map(|coefficients| coefficients[index])
                .collect::<Vec<f32>>()
                .std_dev()
        ).collect::<Vec<f32>>().into_iter().sum::<f32>() / COEFFICIENTS_COUNT as f32;

        let stats = GenerationStatistics {
            id: self.generations.len(),
            max: results[0],
            mean: results.iter().copied().sum::<GameResult>() / POPULATION_SIZE,
            median: pre_sorted_median(&results),
            best: self.population[0],
            mutation_rate: self.mutation_rate,
            fitness_diversity: results.iter()
                .take(self.diversity_sample_size)
                .map(|result| result.score() as f32)
                .collect::<Vec<_>>()
                .std_dev(),
            genetic_diversity: mean_coefficient_std_dev,
        };
        self.generations.push(stats.clone());

        self.population = next_generation.try_into().unwrap();

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
                mutated_population.push(coefficient.mutate(self.mutation_rate, &mut self.rng));
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
                    Some(x.merge_with(y).mutate(self.mutation_rate, &mut self.rng))
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

    fn adapt_mutation_rate(&mut self) {
        if let Some(last ) = self.generations.last() {
            if last.genetic_diversity < TARGET_GENETIC_DIVERSITY {
                self.mutation_rate = (self.mutation_rate * (1.0 + MUTATION_RATE_STEP)).min(MAX_MUTATION_RATE);
            } else {
                self.mutation_rate = (self.mutation_rate * (1.0 - MUTATION_RATE_STEP)).max(MIN_MUTATION_RATE);
            }
        }
    }
}

pub trait Merge {
    fn merge_with(&self, other: &Self) -> Self;
}

pub trait Mutate {
    fn mutate<R: Rng + ?Sized>(&self, magnitude: f32, rng: &mut R) -> Self;
}


/// the input must be pre-sorted
fn pre_sorted_median<T: PartialOrd + Copy + Add<Output = T> + Div<usize, Output = T>>(arr: &[T]) -> T {
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
    fn from_ratios(elite_ratio: f32, mutate_ratio: f32, breed_ratio: f32) -> Result<Self, String> {
        let size_f32 = POPULATION_SIZE as f32;
        let elite_count = (size_f32 * elite_ratio).ceil().clamp(0.0, size_f32) as usize;
        let mutation_count = (size_f32 * mutate_ratio).ceil().clamp(0.0, size_f32) as usize;
        let breed_count = (size_f32 * breed_ratio).ceil().clamp(0.0, size_f32) as usize;

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
        new_seed(),
        Duration::from_millis(60000),
        Duration::from_millis(16)
    );
    let mut ga = GeneticAlgorithm::new(fixture, 0.01, 0.4, 0.2)?;

    println!("starting");
    for _ in 0..10_000 {
        let stats = ga.evolve();
        println!("{:?}", stats);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn genetic_algorithm() {
        todo!()
    }
}