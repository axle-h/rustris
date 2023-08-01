use crate::animation::destroy::{DestroyAnimation, DestroyAnimationType};
use crate::animation::game_over::{GameOverAnimate, GameOverAnimation, GameOverAnimationType};
use crate::animation::impact::ImpactAnimation;
use crate::animation::{TextureAnimate, TextureAnimation};
use crate::config::{Config, GameConfig, MatchRules};
use crate::event::GameEvent;
use crate::game::board::{compact_destroy_lines, DestroyLines};
use crate::game::random::RandomTetromino;
use crate::game::{Game, GameMetrics};
use crate::high_score::table::HighScoreTable;
use crate::high_score::NewHighScore;

use rand::Rng;

use crate::particles::prescribed::{PlayerParticleTarget, PlayerTargetedParticles};
use std::time::Duration;

pub struct Player {
    pub player: u32,
    pub game: Game,
    pub destroy_animation: Option<DestroyAnimation>,
    pub game_over_animation: Option<GameOverAnimation>,
    pub impact_animation: ImpactAnimation,
    pub is_hard_dropping: bool,
}

impl Player {
    pub fn new(player: u32, random: RandomTetromino, level: u32) -> Self {
        Self {
            player,
            game: Game::new(player, level, random),
            destroy_animation: None,
            game_over_animation: None,
            impact_animation: ImpactAnimation::new(),
            is_hard_dropping: false,
        }
    }

    pub fn animate_game_over(&mut self, game_over_type: GameOverAnimationType) {
        self.game_over_animation = Some(GameOverAnimation::new(game_over_type));
    }

    pub fn animate_destroy(&mut self, destroy_type: DestroyAnimationType, lines: DestroyLines) {
        self.destroy_animation = Some(DestroyAnimation::new(destroy_type, lines));
    }

    pub fn update_game_over_animation(&mut self, delta: Duration) -> Option<GameOverAnimate> {
        self.game_over_animation.as_mut().map(|a| a.update(delta))
    }

    pub fn current_game_over_animation(&self) -> Option<GameOverAnimate> {
        self.game_over_animation.as_ref().map(|a| a.current())
    }

    pub fn update_destroy_animation(&mut self, delta: Duration) -> bool {
        let result = match &mut self.destroy_animation {
            None => false,
            Some(animation) => animation.update(delta).is_some(),
        };
        if !result && self.destroy_animation.is_some() {
            self.destroy_animation = None;
        }
        result
    }

    pub fn impact(&mut self) {
        self.impact_animation.impact();
    }

    pub fn next_impact_offset(&mut self, delta: Duration) -> (f64, f64) {
        self.impact_animation.next_offset(delta)
    }

    pub fn current_destroy_animation(&self) -> Vec<(u32, TextureAnimate)> {
        match &self.destroy_animation {
            None => vec![],
            Some(animation) => match animation.current() {
                Some(animate) if !animate.is_emit_particles() => {
                    compact_destroy_lines(animation.lines())
                        .into_iter()
                        .map(|y| (y, animate))
                        .collect()
                }
                _ => vec![],
            },
        }
    }

    pub fn current_particles(&self) -> Option<PlayerTargetedParticles> {
        self.destroy_animation
            .as_ref()
            .and_then(|animation| {
                animation
                    .current()
                    .map(|animate| (animate, animation.lines()))
            })
            .and_then(|(animate, lines)| {
                if let TextureAnimate::EmitParticles(particles) = animate {
                    let target = PlayerParticleTarget::DestroyedLines(lines);
                    Some(particles.into_targeted(self.player, target))
                } else {
                    None
                }
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchState {
    Normal,
    Paused,
    GameOver { high_score: Option<NewHighScore> },
}

impl MatchState {
    pub fn is_paused(&self) -> bool {
        self == &MatchState::Paused
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self, MatchState::GameOver { .. })
    }
}

pub struct Match {
    pub players: Vec<Player>,
    high_scores: HighScoreTable,
    state: MatchState,
    rules: MatchRules,
}

impl Match {
    pub fn new(game_config: GameConfig, config: Config) -> Self {
        if game_config.players == 0 {
            panic!("must have at least one player")
        }

        let randoms = config.game.random_mode.build(
            game_config.players as usize,
            config.game.min_garbage_per_hole,
        );

        Self {
            players: randoms
                .into_iter()
                .enumerate()
                .map(|(pid, rand)| Player::new(pid as u32 + 1, rand, game_config.level))
                .collect::<Vec<Player>>(),
            high_scores: HighScoreTable::load().unwrap(),
            state: MatchState::Normal,
            rules: game_config.rules,
        }
    }

    pub fn unset_flags(&mut self) {
        for player in self.players.iter_mut() {
            player.game.set_soft_drop(false);
            player.is_hard_dropping = false;
        }
    }

    pub fn set_hard_dropping(&mut self, player: u32) {
        self.player_mut(player).is_hard_dropping = true;
    }

    pub fn toggle_paused(&mut self) -> Option<GameEvent> {
        match self.state {
            MatchState::Normal => {
                self.state = MatchState::Paused;
                Some(GameEvent::Paused)
            }
            MatchState::Paused => {
                self.state = MatchState::Normal;
                Some(GameEvent::UnPaused)
            }
            _ => None,
        }
    }

    pub fn state(&self) -> MatchState {
        self.state
    }

    pub fn check_for_winning_player(&self) -> Option<u32> {
        match self.rules {
            MatchRules::ScoreSprint {
                score: sprint_score,
            } => {
                let best_game = self.highest_score();
                if best_game.score >= sprint_score {
                    Some(best_game.player)
                } else {
                    None
                }
            }
            MatchRules::LineSprint {
                lines: sprint_lines,
            } => {
                let best_game = self.most_lines();
                if best_game.lines >= sprint_lines {
                    Some(best_game.player)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn set_winner(&mut self, player: u32, animation_type: GameOverAnimationType) {
        for losing_player in self.players.iter_mut().filter(|p| p.player != player) {
            losing_player.animate_game_over(animation_type);
        }
        self.state = MatchState::GameOver { high_score: None };
    }

    pub fn set_game_over(&mut self, player: u32, animation_type: GameOverAnimationType) {
        let best_game = self.highest_score();

        let high_score = if self.high_scores.is_high_score(best_game.score) {
            Some(NewHighScore::new(best_game.player, best_game.score))
        } else {
            None
        };

        self.state = MatchState::GameOver { high_score };
        self.players
            .get_mut(player as usize - 1)
            .unwrap()
            .animate_game_over(animation_type);
    }

    pub fn mut_game<F>(&mut self, player: u32, mut f: F) -> Option<GameEvent>
    where
        F: FnMut(&mut Game) -> Option<GameEvent>,
    {
        debug_assert!(player > 0);

        match self.state {
            MatchState::Normal => match self.players.get_mut(player as usize - 1) {
                Some(player) if !player.is_hard_dropping => f(&mut player.game),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn player(&self, player: u32) -> &Player {
        debug_assert!(player > 0);
        self.players.get(player as usize - 1).unwrap()
    }

    pub fn player_mut(&mut self, player: u32) -> &mut Player {
        debug_assert!(player > 0);
        self.players.get_mut(player as usize - 1).unwrap()
    }

    pub fn send_garbage(&mut self, from_player: u32, garbage_lines: u32) {
        debug_assert!(from_player > 0);
        if self.players.len() < 2 || !self.rules.garbage_enabled() {
            return;
        }

        let other_players = self
            .players
            .iter()
            .map(|p| p.player)
            .filter(|p| *p != from_player)
            .collect::<Vec<u32>>();

        let pid = if other_players.len() == 1 {
            other_players[0]
        } else {
            other_players[rand::thread_rng().gen_range(0..other_players.len())]
        } as usize;
        self.players
            .get_mut(pid - 1)
            .unwrap()
            .game
            .send_garbage(garbage_lines);
    }

    fn highest_score(&self) -> GameMetrics {
        self.players
            .iter()
            .map(|p| p.game.metrics())
            .max_by(|x, y| x.score.cmp(&y.score))
            .unwrap()
    }

    fn most_lines(&self) -> GameMetrics {
        self.players
            .iter()
            .map(|p| p.game.metrics())
            .max_by(|x, y| x.lines.cmp(&y.lines))
            .unwrap()
    }
}
