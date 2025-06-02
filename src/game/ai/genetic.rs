use std::fmt::Display;
use std::time::Instant;
use rayon::prelude::*;
use crate::config::Config;
use crate::game::ai::action_evaluator::ActionEvaluator;
use crate::game::ai::generation_stats::{GenerationStatistics, Organism};
use crate::game::ai::genome::Genome;
use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
use crate::game::ai::linear::LinearCoefficients;
use crate::game::ai::mutation::{GenomeMutation, RateLimits};
use crate::game::ai::record::GenerationRecord;

#[derive(Debug, Clone, Copy)]
pub struct HyperParameters {
    population_size: usize,
    elite_count: usize, // elites are passed onto the next generation unchanged
    survivor_count: usize, // only survivors are selected to breed
    parent_count: usize, // the number of breeding pairs each generation, the parents are selected from the surviving population weighted by fitness
    end_game: EndGame,
    max_generations: usize,
    generations_per_seed: usize,
}

impl HyperParameters {
    pub fn new(population_size: usize, elite_rate: f64, survival_rate: f64, end_game: EndGame, max_generations: usize, generations_per_seed: usize) -> Self {
        fn rate_to_count(population_size: usize, rate: f64) -> usize {
            assert!(rate >= 0.0 && rate <= 1.0, "rates must be between 0.0 and 1.0");
            (population_size as f64 * rate) as usize
        }

        let elite_count = rate_to_count(population_size, elite_rate);
        let survivor_count = rate_to_count(population_size, survival_rate);

        assert!(elite_count + survivor_count < population_size, "too many elites and survivors");

        Self {
            population_size,
            elite_count,
            survivor_count: rate_to_count(population_size, survival_rate),
            parent_count: ((population_size as f64 - elite_count as f64) / 2.0).ceil() as usize,
            end_game,
            max_generations,
            generations_per_seed
        }
    }
}

impl Default for HyperParameters {
    fn default() -> Self {
        Self::new(
            500,
            0.02,
            0.5,
            EndGame::NONE,
            usize::MAX,
            100
        )
    }
}

pub struct GeneticAlgorithm<const GENOME: usize, F>
where F : Fn(&Genome<GENOME>) -> ActionEvaluator
{
    population: Vec<Organism<GENOME>>,
    generations: Vec<GenerationStatistics<GENOME>>,
    fixture: HeadlessGameFixture,
    mutation: GenomeMutation<GENOME>,
    hyper_parameters: HyperParameters,
    action_evaluator_factory: F,
}

impl<const N: usize, F> GeneticAlgorithm<N, F>
where F : Fn(&Genome<N>) -> ActionEvaluator + Send + Sync
{
    pub fn new(
        fixture: HeadlessGameFixture,
        mut mutation: GenomeMutation<N>,
        hyper_parameters: HyperParameters,
        population_seed: Option<Genome<N>>,
        action_evaluator_fn: F
    ) -> Self {
        let genome_seed: Option<Genome<N>> = population_seed.map(|seed| seed.into());
        let mut population = Vec::with_capacity(hyper_parameters.population_size);
        for _ in 0 .. hyper_parameters.population_size {
            let genome = if let Some(genome_seed) = genome_seed {
                mutation.mutate(genome_seed)
            } else {
                mutation.random()
            };
            population.push(Organism::new(genome));
        }

        Self {
            population,
            generations: vec![],
            fixture,
            mutation,
            hyper_parameters,
            action_evaluator_factory: action_evaluator_fn
        }
    }

    pub fn run(&mut self) -> GenerationStatistics<N> {
        println!("Running genetic algorithm...");

        let mut generations_until_next_seed = self.hyper_parameters.generations_per_seed;
        let mut record = GenerationRecord::new().expect("Failed to create generation record");
        println!("Results saved to {}", record.path().display());

        let t0 = Instant::now();
        loop {
            let stats = self.evolve();
            println!("{}", stats);
            record.add(&stats).expect("Failed to write to generation record");
            if stats.id() >= self.hyper_parameters.max_generations || self.hyper_parameters.end_game.is_end_game(stats.max().result(), Instant::now() - t0) {
                return stats
            }

            if generations_until_next_seed == 0 {
                generations_until_next_seed = self.hyper_parameters.generations_per_seed;
                self.fixture.next_seed();
                println!("Using new seed {}", self.fixture.current_seed())
            } else {
                generations_until_next_seed -= 1;
            }
        }
    }
    
    fn evolve(&mut self) -> GenerationStatistics<N> {
        // Calculate fitness in parallel
        self.population
            .par_iter_mut()
            .for_each(|member| {
                member.set_result(|genome| self.fixture.play((self.action_evaluator_factory)(genome)));
            });
        self.population.sort_by(|s1, s2| s2.result().cmp(&s1.result()));

        let p95_index = (self.hyper_parameters.population_size as f64 * 0.05).floor() as usize;
        let p50_index = self.hyper_parameters.population_size / 2;
        let stats = GenerationStatistics::new(
            self.generations.len() + 1,
            self.fixture.current_seed(),
            self.population[0],
            self.population[p95_index],
            self.population[p50_index],
            self.mutation.current_mutation_rate(),
            self.mutation.current_crossover_rate()
        );
        self.generations.push(stats);
        self.mutation.add_sample(stats);

        self.next_generation();

        stats
    }

    fn next_generation(&mut self) {
        let surviving_population: Vec<_> = self.population.iter()
            .take(self.hyper_parameters.survivor_count)
            .copied()
            .collect();

        self.population.clear();

        for elite in surviving_population.iter().take(self.hyper_parameters.elite_count) {
            // includes cached result
            self.population.push(*elite);
        }

        let parents = self.mutation.parents(&surviving_population, self.hyper_parameters.parent_count);

        let mut required_children = self.hyper_parameters.population_size - self.population.len();
        while required_children > 0 {
            for [parent1, parent2] in parents.iter() {
                let [child1, child2] = self.mutation.crossover(*parent1, *parent2)
                    .map(Organism::new);
                self.population.push(child1);
                required_children -= 1;

                if required_children > 0 {
                    self.population.push(child2);
                    required_children -= 1;
                }

                if required_children == 0 {
                    break;
                }
            }
        }
    }
}

pub fn ga_main_linear() -> Result<(), String> {
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        rand::random(),
        HeadlessGameOptions::default(),
        EndGame::of_lines(10_000) // TODO what is the world record?
    );
    let mutation = GenomeMutation::of_max(
        RateLimits::new(0.1 ..= 0.20),
        RateLimits::new(0.1 ..= 0.20),
        5,
        rand::random()
    );
    GeneticAlgorithm::new(
        fixture,
        mutation,
        HyperParameters::default(),
        Some(LinearCoefficients::default().into()),
        move |&genome| ActionEvaluator::Linear(genome.into())
    ).run();
    
    Ok(())
}

pub fn ga_main_neural() -> Result<(), String> {
    let fixture = HeadlessGameFixture::new(
        Config::default(),
        rand::random(),
        HeadlessGameOptions::default(),
        EndGame::of_lines(10_000) // TODO what is the world record?
    );
    let mutation = GenomeMutation::of_max(
        RateLimits::new(0.1 ..= 0.20),
        RateLimits::new(0.1 ..= 0.20),
        5,
        rand::random()
    );
    GeneticAlgorithm::new(
        fixture,
        mutation,
        HyperParameters::default(),
        None, //Some(TetrisNeuralNetwork::default().into()),
        move |&genome| ActionEvaluator::NeuralNetwork(genome.into())
    ).run();

    Ok(())
}

pub fn ga_main() -> Result<(), String> {
    ga_main_neural()
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::game::ai::action_evaluator::ActionEvaluator;
    use crate::game::ai::genetic::{GeneticAlgorithm, HyperParameters};
    use crate::game::ai::headless_game::{EndGame, HeadlessGameFixture, HeadlessGameOptions};
    use crate::game::ai::mutation::{GenomeMutation, RateLimits};

    #[test]
    fn genetic_algorithm() {
        let fixture = HeadlessGameFixture::new(
            Config::default(),
            100.into(),
            HeadlessGameOptions::default(),
            EndGame::of_seconds(2)
        );
        let mutation: GenomeMutation<9> = GenomeMutation::of_max(RateLimits::default(), RateLimits::default(), 5, 100.into());
        GeneticAlgorithm::new(
            fixture,
            mutation,
            HyperParameters::new(
                10,
                0.01,
                0.5,
                EndGame::NONE,
                1,
                100
            ),
            None,
            move |&genome| ActionEvaluator::Linear(genome.into())
        ).run();

        assert!(true);
    }
}