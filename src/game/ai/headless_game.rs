use std::time::Duration;
use crate::animation::destroy::SWEEP_DURATION;
use crate::config::Config;
use crate::event::GameEvent;
use crate::game::ai::agent::AiAgent;
use crate::game::ai::board_cost::{BoardCost, AiCoefficients};
use crate::game::ai::game_result::GameResult;
use crate::game::Game;
use crate::game::random::{RandomTetromino, Seed, PEEK_SIZE};

pub struct HeadlessGame {
    agent: AiAgent,
    game: Game,
    end_game: EndGame,
    options: HeadlessGameOptions,
    duration: Duration,
    game_over: bool,
}

impl HeadlessGame {
    pub fn new(
        rng: RandomTetromino,
        agent: AiAgent,
        options: HeadlessGameOptions,
        end_game: EndGame,
    ) -> Self {
        Self {
            agent,
            game: Game::new(1, 0, rng),
            duration: Duration::ZERO,
            game_over: false,
            options,
            end_game
        }
    }
    
    pub fn play(&mut self) -> GameResult {
        loop {
            if let Some(result) = self.update() {
                return result;
            }
        }
    }

    fn update(&mut self) -> Option<GameResult> {
        self.duration += self.options.step;
        
        let result = GameResult::new(self.game.score, self.game.lines, self.game.level, self.game_over, self.duration);
        if self.game_over || self.end_game.is_end_game(result, self.duration) {
            return Some(result);
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
                    return Some(result);
                },
                GameEvent::Destroy(_) => {
                    // simulate line clear animation
                    self.duration += self.options.line_clear_delay;
                }
                _ => ()
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeadlessGameOptions {
    line_clear_delay: Duration,
    step: Duration,
    look_ahead: usize
}

impl Default for HeadlessGameOptions {
    fn default() -> Self {
        Self {
            step: Duration::from_millis(16), // 60hz
            line_clear_delay: SWEEP_DURATION,
            look_ahead: 2
        }   
    }
}

pub struct HeadlessGameFixture {
    config: Config,
    seeds: Vec<Seed>,
    game_options: HeadlessGameOptions,
    end_game: EndGame,
}

impl HeadlessGameFixture {
    pub fn new(config: Config, seeds: Vec<Seed>, game_options: HeadlessGameOptions, end_game: EndGame) -> Self {
        Self { config, seeds, game_options, end_game }
    }
    
    pub fn play(&self, coefficients: AiCoefficients) -> GameResult {
        let mut sum_result = GameResult::default();
        for seed in self.seeds.iter() {
            let rng = RandomTetromino::new(
                self.config.game.random_mode,
                self.config.game.min_garbage_per_hole,
                *seed
            );
            let agent = AiAgent::new(BoardCost::new(coefficients), self.game_options.look_ahead);
            let result = HeadlessGame::new(rng, agent, self.game_options, self.end_game).play();
            sum_result += result;
        }
        
        if self.seeds.len() > 1 {
            sum_result / self.seeds.len()
        } else {
            sum_result
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EndGame {
    pub score: u32,
    pub lines: u32,
    pub duration: Duration,
}

impl Default for EndGame {
    fn default() -> Self {
        Self::NONE
    }
}

impl EndGame {
    pub const NONE: Self = Self {
        score: u32::MAX,
        lines: u32::MAX,
        duration: Duration::MAX
    };

    pub fn of_score(score: u32) -> Self {
        Self {
            score,
            ..Default::default()
        }
    }

    pub fn of_lines(lines: u32) -> Self {
        Self {
            lines,
            ..Default::default()
        }
    }

    pub fn of_seconds(seconds: u64) -> Self {
        Self {
            duration: Duration::from_secs(seconds),
            ..Default::default()
        }
    }
    
    pub fn is_end_game(&self, result: GameResult, duration: Duration) -> bool {
        result.score() >= self.score
            || result.lines() >= self.lines
            || duration >= self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fixture() -> HeadlessGameFixture {
        HeadlessGameFixture::new(
            Config::default(),
            vec![100.into(), 101.into()],
            HeadlessGameOptions::default(),
            EndGame::of_seconds(5)
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