use std::time::Duration;
use crate::animation::destroy::SWEEP_DURATION;
use crate::config::Config;
use crate::event::GameEvent;
use crate::game::ai::agent::AiAgent;
use crate::game::ai::board_cost::{BoardCost, AiCoefficients};
use crate::game::ai::game_result::GameResult;
use crate::game::Game;
use crate::game::random::{RandomTetromino, Seed};

pub struct HeadlessGame {
    agent: AiAgent,
    game: Game,
    options: HeadlessGameOptions,
    duration: Duration,
    game_over: bool,
}

impl HeadlessGame {
    pub fn new(
        rng: RandomTetromino,
        agent: AiAgent,
        options: HeadlessGameOptions,
    ) -> Self {
        Self {
            agent,
            game: Game::new(1, 0, rng),
            duration: Duration::ZERO,
            game_over: false,
            options
        }
    }
    
    pub fn play(&mut self) -> GameResult {
        while self.update() {}
        GameResult::new(self.game.score, self.game.lines, self.game.level, self.game_over, self.duration)
    }

    fn update(&mut self) -> bool {
        if self.game_over {
            return false
        }
        
        // TODO have dynamic end goals e.g. score, lines, levels, tetris count
        self.duration += self.options.step;
        if self.duration > self.options.max_duration {
            return false;
        }

        self.agent.act(&mut self.game);
        let mut events = self.game.empty_event_buffer();

        if let Some(event) = self.game.update(self.options.step) {
            events.push(event);
        }

        for event in events {
            match event {
                GameEvent::GameOver { .. } => {
                    self.game_over = true;
                    return false;
                },
                GameEvent::Destroy(_) => {
                    // simulate line clear animation
                    self.duration += self.options.line_clear_delay;
                }
                _ => ()
            }
        }

        true
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeadlessGameOptions {
    max_duration: Duration,
    line_clear_delay: Duration,
    step: Duration
}

impl HeadlessGameOptions {
    pub fn new(max_duration: Duration, line_clear_delay: Duration, step: Duration) -> Self {
        Self {
            max_duration,
            line_clear_delay,
            step
        }
    }
}

impl Default for HeadlessGameOptions {
    fn default() -> Self {
        Self {
            step: Duration::from_millis(16), // 60hz
            max_duration: Duration::from_millis(600_000),
            line_clear_delay: SWEEP_DURATION
        }   
    }
}

pub struct HeadlessGameFixture {
    config: Config,
    seeds: Vec<Seed>,
    game_options: HeadlessGameOptions,
    look_ahead: usize
}

impl HeadlessGameFixture {
    pub fn new(config: Config, seeds: Vec<Seed>, game_options: HeadlessGameOptions, look_ahead: usize) -> Self {
        Self { config, seeds, game_options, look_ahead }
    }
    
    pub fn play(&self, coefficients: AiCoefficients) -> GameResult {
        let mut sum_result = GameResult::default();
        for seed in self.seeds.iter() {
            let rng = RandomTetromino::new(
                self.config.game.random_mode,
                self.config.game.min_garbage_per_hole,
                *seed
            );
            let agent = AiAgent::new(BoardCost::new(coefficients), self.look_ahead);
            let result = HeadlessGame::new(rng, agent, self.game_options).play();
            sum_result += result;
        }
        
        if self.seeds.len() > 1 {
            sum_result / self.seeds.len()
        } else {
            sum_result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fixture() -> HeadlessGameFixture {
        HeadlessGameFixture::new(
            Config::default(),
            vec![100.into(), 101.into()],
            HeadlessGameOptions::new(
                Duration::from_millis(5_000),
                Duration::from_millis(100),
                Duration::from_millis(16),
            ),
            0
        )
    }
    
    #[test]
    fn runs_headless_game() {
        let fixture = test_fixture();
        let result = fixture.play(AiCoefficients::default());
        assert!(result.score() > 0);
    }

    #[test]
    fn same_score_for_the_same_inputs() {
        let fixture = test_fixture();
        let result1 = fixture.play(AiCoefficients::default());
        let result2 = fixture.play(AiCoefficients::default());
        assert_eq!(result1, result2);
    }
}