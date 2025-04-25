use std::iter::Sum;
use std::ops::{Add, Div};
use std::time::Duration;
use crate::config::Config;
use crate::event::GameEvent;
use crate::game::ai::agent::AiAgent;
use crate::game::ai::board_cost::{BoardCost, CostCoefficients};
use crate::game::ai::game_result::GameResult;
use crate::game::Game;
use crate::game::random::{new_seed, RandomTetromino};

pub struct HeadlessGame {
    agent: AiAgent,
    game: Game,
    max_duration: Duration,
    duration: Duration,
    game_over: bool
}

impl HeadlessGame {
    pub fn new(config: Config, seed: u64, max_duration: Duration, coefficients: CostCoefficients) -> Self {
        let rng = RandomTetromino::new(
            config.game.random_mode,
            config.game.min_garbage_per_hole,
            seed
        );
        Self {
            agent: AiAgent::new(BoardCost::new(coefficients)),
            game: Game::new(1, 0, rng),
            max_duration,
            duration: Duration::ZERO,
            game_over: false
        }
    }

    pub fn result(&self) -> GameResult {
        GameResult::new(self.game.score, self.game.lines, self.game.level)
    }

    pub fn update(&mut self, delta: Duration) -> bool {
        if self.game_over {
            return false
        }
        self.duration += delta;
        if self.duration > self.max_duration {
            self.game_over = true;
            return false;
        }

        self.agent.act(&mut self.game);
        self.game.empty_event_buffer();

        if let Some(GameEvent::GameOver { .. }) = self.game.update(delta) {
            self.game_over = true;
        }

        true
    }
}

pub struct HeadlessGameFixture {
    config: Config,
    seed: u64,
    max_duration: Duration,
    step: Duration
}

impl HeadlessGameFixture {
    pub fn new(config: Config, seed: u64, max_duration: Duration, step: Duration) -> Self {
        Self { config, seed, max_duration, step }
    }

    pub fn play(&self, coefficients: CostCoefficients) -> GameResult {
        let mut game = HeadlessGame::new(self.config, self.seed, self.max_duration, coefficients);
        while game.update(self.step) {}
        game.result()
    }
}

impl Default for HeadlessGameFixture {
    fn default() -> Self {
        Self::new(
            Config::default(),
            new_seed(),
            Duration::from_millis(30000),
            Duration::from_millis(16)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_headless_game() {
        let fixture = HeadlessGameFixture::default();
        let result = fixture.play(CostCoefficients::SENSIBLE_DEFAULTS);
        assert!(result.score() > 0);
    }

    #[test]
    fn same_score_for_the_same_inputs() {
        let fixture = HeadlessGameFixture::default();
        let result1 = fixture.play(CostCoefficients::SENSIBLE_DEFAULTS);
        let result2 = fixture.play(CostCoefficients::SENSIBLE_DEFAULTS);
        assert_eq!(result1, result2);
    }
}