use std::array;
use std::ops::{Add, Div};
use std::time::Duration;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;
use rand_chacha::ChaChaRng;
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::board_cost::CostCoefficients;
use crate::game::ai::game_result::GameResult;
use crate::game::ai::headless::HeadlessGameFixture;
use crate::game::random::new_seed;

const POPULATION_SIZE: usize = 100;

#[derive(Clone, Debug, PartialEq)]
pub struct GenerationStatistics {
    id: usize,
    max: GameResult,
    mean: GameResult,
    median: GameResult,
    best: CostCoefficients
}

pub struct GeneticAlgorithm {
    population: [CostCoefficients; POPULATION_SIZE],
    generations: Vec<GenerationStatistics>,
    fixture: HeadlessGameFixture,
    rng: ChaChaRng,
    mutation_rate: f32,
    counts: PopulationCounts,
}

impl GeneticAlgorithm {
    pub fn new(mutation_rate: f32, counts: PopulationCounts) -> Self {
        let mut rng = ChaChaRng::from_os_rng();
        Self {
            population: array::from_fn(|_| rng.random()),
            generations: vec![],
            fixture: HeadlessGameFixture::new(
                Config::default(),
                new_seed(),
                Duration::from_millis(30000),
                Duration::from_millis(16)
            ),
            rng,
            mutation_rate,
            counts,
        }
    }

    pub fn next_generation(&mut self) -> GenerationStatistics {
        // Calculate fitness in parallel
        let mut fitness_population: Vec<_> = self.population
            .into_par_iter()
            .map(|coefficients| {
                (coefficients, self.fixture.play(coefficients))
            })
            .collect();

        fitness_population.sort_by(|(_, s1), (_, s2)| s2.cmp(s1));

        let mut elite_population: Vec<CostCoefficients> = vec![];
        let mut mutated_population: Vec<CostCoefficients> = vec![];
        let mut breeding_population: Vec<CostCoefficients> = vec![];
        for (coefficient, _) in fitness_population.iter() {
            let mut added = false;
            if elite_population.len() < self.counts.elite {
                elite_population.push(*coefficient);
                added = true;
            }
            if mutated_population.len() < self.counts.mutate {
                mutated_population.push(coefficient.mutate(self.mutation_rate, &mut self.rng));
                added = true;
            }
            if breeding_population.len() < self.counts.breed {
                breeding_population.push(*coefficient);
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
        self.population = next_generation.try_into().unwrap();

        let scores: Vec<_> = fitness_population.into_iter().map(|(_, s1)| s1).collect();
        let stats = GenerationStatistics {
            id: self.generations.len(),
            max: scores[0],
            mean: scores.iter().copied().sum::<GameResult>() / POPULATION_SIZE,
            median: pre_sorted_median(&scores),
            best: self.population[0]
        };
        self.generations.push(stats.clone());
        stats
    }
}

pub trait Merge {
    fn merge_with(&self, other: &Self) -> Self;
}

pub trait Mutate {
    fn mutate(&self, magnitude: f32, rng: &mut ChaChaRng) -> Self;
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
    fn from_ratios(elite_ratio: f32, mutation_ratio: f32, breed_ratio: f32) -> Result<Self, String> {
        let size_f32 = POPULATION_SIZE as f32;
        let elite_count = (size_f32 * elite_ratio).ceil().clamp(0.0, size_f32) as usize;
        let mutation_count = (size_f32 * mutation_ratio).ceil().clamp(0.0, size_f32) as usize;
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
    let mut ga = GeneticAlgorithm::new(
        0.01,
        PopulationCounts::from_ratios(0.02, 0.7, 0.2)?
    );

    println!("starting");
    for _ in 0..10_000 {
        let stats = ga.next_generation();
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