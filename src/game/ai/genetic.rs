use std::array;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Div};
use std::time::Instant;
use itertools::Itertools;
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::board_cost::{AiCoefficients, Genome};
use crate::game::ai::game_result::GameResult;
use crate::game::ai::generation_stats::{GenerationResult, GenerationStatistics};
use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
use crate::game::ai::mutation::{GenomeMutation, RateLimits};


pub struct GeneticAlgorithm {
    population: [Genome; POPULATION_SIZE],
    generations: Vec<GenerationStatistics>,
    fixture: HeadlessGameFixture,
    mutation: GenomeMutation,
    elite_count: usize,
    cached_scores: HashMap<Genome, GameResult>,
    end_game: EndGame,
    max_generations: usize,
}

const POPULATION_SIZE: usize = 200;

impl GeneticAlgorithm {
    // TODO increase game length as games start to get longer
    pub fn new(
        fixture: HeadlessGameFixture,
        elite_count: usize,
        mut mutation: GenomeMutation,
        end_game: EndGame,
        max_generations: usize,
        population_seed: Option<AiCoefficients>
    ) -> Self {
        let genome_seed: Option<Genome> = population_seed.map(|seed| seed.into());
        Self {
            population: array::from_fn(|_| {
                if let Some(genome_seed) = genome_seed {
                    mutation.mutate(genome_seed)
                } else {
                    mutation.random()
                }
            }),
            generations: vec![],
            fixture,
            mutation,
            elite_count,
            cached_scores: HashMap::new(),
            end_game,
            max_generations
        }
    }

    pub fn run(&mut self) -> GenerationStatistics {
        let t0 = Instant::now();
        loop {
            let stats = self.evolve();
            println!("{}", stats);
            if stats.id() >= self.max_generations || self.end_game.is_end_game(stats.max().result(), Instant::now() - t0) {
                return stats
            }
        }
    }
    
    fn evolve(&mut self) -> GenerationStatistics {
        // Calculate fitness in parallel
        let mut population: Vec<_> = self.population
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
        for (genome, result) in population.iter().copied() {
            self.cached_scores.insert(genome, result);
        }

        population.sort_by(|(_, s1), (_, s2)| s2.cmp(s1));

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

        let p95_index = (population.len() as f64 * 0.05).floor() as usize;
        let stats = GenerationStatistics::new(
            self.generations.len() + 1,
            population[0].into(),
            population[p95_index].into(),
            population[population.len() / 2].into(),
            self.mutation.current_mutation_rate(),
            self.mutation.current_crossover_rate()
        );
        self.generations.push(stats);
        self.mutation.add_sample(stats);
        
        self.population = self.next_generation(population).try_into().unwrap();

        stats
    }

    fn next_generation(&mut self, labelled_population: Vec<(Genome, GameResult)>) -> Vec<Genome> {
        // TODO use this same score for classification above
        let surviving_population: Vec<_> = labelled_population.into_iter()
            .map(|(genome, result)| {
                let game_over_penalty = if result.game_over() { 100_000.0 } else { 0.0 };
                let fitness = result.score() as f64 - game_over_penalty;
                (genome, fitness)
            })
            .sorted_by(|(_, f1), (_, f2)| f2.partial_cmp(f1).unwrap())
            .take(POPULATION_SIZE / 2) // TODO configurable survival rate
            .collect();

        let children_count = ((POPULATION_SIZE as f64 - self.elite_count as f64) / 2.0).ceil() as usize; // TODO pre-calculate this
        let parents = self.mutation.parents(&surviving_population, children_count);
        
        let mut next_population: HashSet<_> = parents.iter()
            .flat_map(|[parent1, parent2]| self.mutation.crossover(*parent1, *parent2))
            .collect();

        for (elite, _) in surviving_population.iter().take(self.elite_count).copied() {
            next_population.insert(elite);
        }
        
        // ensure we have enough unique children
        while next_population.len() < POPULATION_SIZE {
            for [parent1, parent2] in parents.iter() {
                let [child1, child2] = self.mutation.crossover(*parent1, *parent2);
                next_population.insert(child1);
                next_population.insert(child2);
                
                if next_population.len() >= POPULATION_SIZE {
                    break
                }
            }
        }

        next_population.into_iter().take(POPULATION_SIZE).collect()
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
        EndGame::of_lines(10_000) // TODO what is the world record?
    );
    let mutation = GenomeMutation::of_max(
        RateLimits::new(0.1 ..= 0.20),
        RateLimits::new(0.1 ..= 0.20),
        5,
        rand::random()
    );
    GeneticAlgorithm::new(fixture, 2, mutation, EndGame::NONE, 10_000, Some(AiCoefficients::default())).run();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::config::Config;
    use crate::game::ai::genetic::GeneticAlgorithm;
    use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
    use crate::game::ai::mutation::{GenomeMutation, RateLimits};

    #[test]
    fn genetic_algorithm() {
        let fixture = HeadlessGameFixture::new(
            Config::default(),
            vec![100.into()],
            HeadlessGameOptions::default(),
            EndGame::of_seconds(2)
        );
        let mutation = GenomeMutation::of_max(RateLimits::default(), RateLimits::default(), 5, 100.into());
        GeneticAlgorithm::new(fixture, 2, mutation, EndGame::NONE, 1, None).run();

        assert!(true);
    }
}