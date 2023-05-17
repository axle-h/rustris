use std::collections::HashMap;
use std::cmp::min;
use std::thread::spawn;
use std::time::Duration;
use sdl2::init;
use board::Board;
use tetromino::TetrominoShape;
use geometry::Point;
use timing::Timing;
use crate::game::block::BlockState;
use crate::game::random::{PEEK_SIZE, RandomMode, RandomTetromino};
use super::game::board::DestroyPattern;

pub mod board;
pub mod tetromino;
pub mod geometry;
pub mod block;
pub mod timing;
pub mod random;

const LINES_PER_LEVEL: u32 = 10;
const SOFT_DROP_STEP_FACTOR: u32 = 20;
const SOFT_DROP_SPAWN_FACTOR: u32 = 10;
const MIN_SPAWN_DELAY: Duration = Duration::from_millis(500);
const LOCK_DURATION: Duration = Duration::from_millis(500);
const SOFT_DROP_LOCK_DURATION: Duration = Duration::from_millis(500 / 20);
const MAX_LOCK_PLACEMENTS: u32 = 15;

const SINGLE_POINTS: u32 = 100;
const DOUBLE_POINTS: u32 = 300;
const TRIPLE_POINTS: u32 = 500;
const TETRIS_POINTS: u32 = 800;
const COMBO_POINTS: u32 = 50;
const DIFFICULT_MULTIPLIER: f64 = 1.5;
const SOFT_DROP_POINTS_PER_ROW: u32 = 1;
const HARD_DROP_POINTS_PER_ROW: u32 = 2;

// pre-calculated step durations in ms: 1000 * (0.8 - (level as f64 * 0.007)).powi(level as i32)
// doing it like this as hashmaps cannot be constant and fp logic is not yet supported at compile time
const STEP_0: Duration = Duration::from_millis(1000);
const STEP_1: Duration = Duration::from_millis(793);
const STEP_2: Duration = Duration::from_millis(618);
const STEP_3: Duration = Duration::from_millis(473);
const STEP_4: Duration = Duration::from_millis(355);
const STEP_5: Duration = Duration::from_millis(262);
const STEP_6: Duration = Duration::from_millis(190);
const STEP_7: Duration = Duration::from_millis(135);
const STEP_8: Duration = Duration::from_millis(94);
const STEP_9: Duration = Duration::from_millis(64);
const STEP_10: Duration = Duration::from_millis(43);
const STEP_11: Duration = Duration::from_millis(28);
const STEP_12: Duration = Duration::from_millis(18);
const STEP_13: Duration = Duration::from_millis(11);
const STEP_14: Duration = Duration::from_millis(7);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOverCondition {
    /// Top Out: An opponent’s Line Attacks force existing Blocks past the top of the Buffer Zone
    TopOut,
    /// Lock Out: The player locks a whole Tetrimino down above the Skyline
    LockOut,
    /// Block Out: One of the starting cells of the Next Tetrimino is blocked by an existing Block
    BlockOut
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Spawn(Duration, TetrominoShape),
    Fall(Duration),
    Lock(Duration),
    Pattern, // check the board for patterns to destroy e.g. lines
    Destroy(DestroyPattern), // destroy marked patterns
    GameOver(GameOverCondition)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combo {
    count: u32,
    difficult: bool
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HoldState {
    shape: TetrominoShape,
    locked: bool
}

pub struct Game {
    player: u32,
    board: Board,
    random: Box<dyn RandomTetromino>,
    level: u32,
    lines: u32,
    score: u32,
    combo: Option<Combo>,
    state: GameState,
    soft_drop: bool,
    skip_next_spawn_delay: bool,
    hold: Option<HoldState>,

}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameMetrics {
    pub player: u32,
    pub level: u32,
    pub lines: u32,
    pub score: u32,
    pub combo: Option<Combo>,
    pub queue: [TetrominoShape; PEEK_SIZE],
    pub hold: Option<TetrominoShape>
}

impl Game {
    pub fn new(player: u32, level: u32, random_mode: RandomMode) -> Game {
        let mut random = random_mode.build();
        let first_shape = random.next();
        Game {
            player,
            board: Board::new(),
            random,
            level,
            lines: 0,
            score: 0,
            combo: None,
            state: GameState::Spawn(Duration::ZERO, first_shape),
            soft_drop: false,
            skip_next_spawn_delay: false,
            hold: None
        }
    }

    pub fn hold(&mut self) -> bool {
        if !(matches!(self.state, GameState::Fall(_))
            || matches!(self.state, GameState::Lock(duration) if duration < LOCK_DURATION)) ||
            matches!(self.hold, Some(HoldState { locked: true, .. })) {
            // hold is blocked
            return false;
        }

        let held_shape = match self.board.hold() {
            None => return false,
            Some(shape) => shape
        };

        let next_shape = match self.hold {
            None => self.random.next(), // just spawn next random shape
            Some(HoldState { shape, .. }) => shape,
        };

        self.state = GameState::Spawn(MIN_SPAWN_DELAY, next_shape);
        self.hold = Some(HoldState { locked: true, shape: held_shape });
        return true;
    }

    pub fn set_soft_drop(&mut self, soft_drop: bool) -> bool {
        self.soft_drop = soft_drop;
        soft_drop
    }

    pub fn hard_drop(&mut self) -> bool {
        match self.board.hard_drop() {
            None => false,
            Some(hard_dropped_rows) => {
                self.state = GameState::Lock(LOCK_DURATION);
                self.score += hard_dropped_rows * HARD_DROP_POINTS_PER_ROW;
                self.skip_next_spawn_delay = true;
                true
            }
        }
    }

    pub fn player(&self) -> u32 {
        self.player
    }

    pub fn metrics(&self) -> GameMetrics {
        GameMetrics {
            player: self.player,
            level: self.level,
            lines: self.lines,
            score: self.score,
            combo: self.combo,
            queue: self.random.peek(),
            hold: self.hold.map(|h| h.shape)
        }
    }

    pub fn left(&mut self) -> bool {
        self.with_checking_lock(|board| board.left())
    }

    pub fn right(&mut self) -> bool {
        self.with_checking_lock(|board| board.right())
    }

    pub fn rotate(&mut self, clockwise: bool) -> bool {
        self.with_checking_lock(|board| board.rotate(clockwise))
    }

    fn with_checking_lock<F>(&mut self, mut f: F) -> bool where F: FnMut(&mut Board) -> bool {
        match self.state {
            GameState::Lock(lock_duration) => {
                // 1. check if the lock is already breached (we send movements before a lock update)
                if lock_duration > LOCK_DURATION {
                    return false;
                }
                // 2. check if this tetromino used all it's lock movements for this altitude
                if self.board.lock_placements() >= MAX_LOCK_PLACEMENTS {
                    // the tetromino has already run out of lock movements, lock it asap
                    self.state = GameState::Lock(LOCK_DURATION);
                    return false;
                }
                // 3. check the movement was blocked by the board
                if !f(&mut self.board) {
                    return false;
                }
                if self.board.register_lock_placement() < MAX_LOCK_PLACEMENTS {
                    // movement is allowed under lock, lock is reset
                    self.state = GameState::Lock(Duration::ZERO);
                } else {
                    // the tetromino just ran out of lock movements, lock it asap
                    self.state = GameState::Lock(LOCK_DURATION);
                }
                return true;
            }
            _ => f(&mut self.board) // not in lock state, pass through closure
        }
    }

    pub fn update(&mut self, delta: Duration) -> GameState {
        let state = match self.state {
            GameState::Spawn(duration, shape) => self.spawn(duration + delta, shape),
            GameState::Fall(duration) => self.fall(duration + delta),
            GameState::Lock(duration) => self.lock(duration + delta),
            GameState::Pattern => self.pattern(),
            GameState::Destroy(pattern) => self.destroy(pattern),
            GameState::GameOver(condition) => GameState::GameOver(condition)
        };
        self.state = state;
        state
    }

    fn spawn(&mut self, duration: Duration, shape: TetrominoShape) -> GameState {
        if !self.skip_next_spawn_delay && duration < self.spawn_delay() {
            return GameState::Spawn(duration, shape);
        }

        if self.board.try_spawn_tetromino(shape) {
            GameState::Fall(Duration::ZERO)
        } else {
            // cannot spawn a tetromino is a game over event
            GameState::GameOver(GameOverCondition::BlockOut)
        }
    }

    fn fall(&mut self, duration: Duration) -> GameState {
        if duration < self.step_delay() {
            return GameState::Fall(duration);
        }

        if !self.board.step_down() {
            // cannot step down, start lock
            return GameState::Lock(Duration::ZERO)
        }

        // has stepped down one row, update score if soft dropping
        if self.soft_drop {
            self.score += SOFT_DROP_POINTS_PER_ROW;
        }

        if self.board.is_collision() {
            // step has caused a collision, start a lock
            if self.board.lock_placements() >= MAX_LOCK_PLACEMENTS {
                // lock asap
                GameState::Lock(LOCK_DURATION)
            } else {
                GameState::Lock(Duration::ZERO)
            }
        } else {
            // no collisions, start a new fall step
            GameState::Fall(Duration::ZERO)
        }
    }

    fn lock(&mut self, duration: Duration) -> GameState {
        let max_lock_duration = if self.soft_drop { SOFT_DROP_LOCK_DURATION } else { LOCK_DURATION };
        if duration < max_lock_duration {
            GameState::Lock(duration)
        } else if self.board.is_collision() {
            // lock timeout and still colliding so lock the piece now
            self.board.lock();
            // maybe unlock hold
            match self.hold {
                Some(HoldState { locked, shape }) if locked => {
                    self.hold = Some(HoldState { locked: false, shape });
                },
                _ => {}
            }
            // todo check for LockOut game over pattern here
            GameState::Pattern
        } else {
            // otherwise must've moved over empty space so start a new fall
            GameState::Fall(Duration::ZERO)
        }
    }

    fn pattern(&mut self) -> GameState {
        GameState::Destroy(self.board.pattern())
    }

    fn destroy(&mut self, pattern: DestroyPattern) -> GameState {
        self.board.destroy(pattern);
        self.update_score(pattern);
        GameState::Spawn(Duration::ZERO, self.random.next())
    }

    fn update_score(&mut self, pattern: DestroyPattern) {
        // TODO test
        // todo t-spin

        let (action_score, lines, action_difficult) = match pattern {
            DestroyPattern::None => (0, 0, false),
            DestroyPattern::Single(_) => (SINGLE_POINTS, 1, false),
            DestroyPattern::Double(_) => (DOUBLE_POINTS, 2, false),
            DestroyPattern::Triple(_) => (TRIPLE_POINTS, 3, false),
            DestroyPattern::Tetris(_) => (TETRIS_POINTS, 4, true)
        };

        if action_score == 0 {
            self.combo = None;
            return;
        }

        // update combo
        self.combo = match self.combo {
            None => Some(Combo { count: 0, difficult: action_difficult }),
            Some(Combo { count, difficult }) => Some(Combo { count: count + 1, difficult: difficult && action_difficult }),
        };

        // calculate score delta
        let level_multiplier = self.level + 1;
        let difficult_multiplier = match self.combo {
            // back to back difficult clears get a 1.5x multiplier
            Some(Combo { count, difficult} ) if count > 0 && difficult => DIFFICULT_MULTIPLIER,
            _ => 1.0
        };
        let combo_score = match self.combo {
            Some (Combo { count, .. }) if count > 0 => COMBO_POINTS * count,
            _ => 0
        };
        let score_delta = action_score as f64 * level_multiplier as f64 * difficult_multiplier + combo_score as f64;

        // update score
        self.score += score_delta.round() as u32;

        // update level
        self.lines += lines;
        let line_level = self.lines / LINES_PER_LEVEL;
        if line_level > self.level {
            self.level = line_level;
        }
    }

    pub fn row(&self, y: u32) -> &[BlockState] {
        self.board.row(y)
    }

    fn spawn_delay(&self) -> Duration {
        min(self.base_delay(SOFT_DROP_SPAWN_FACTOR), MIN_SPAWN_DELAY)
    }

    fn step_delay(&self) -> Duration {
        self.base_delay(SOFT_DROP_STEP_FACTOR)
    }

    fn base_delay(&self, soft_drop_factor: u32) -> Duration {
        let base = match self.level {
            0 => STEP_0,
            1 => STEP_1,
            2 => STEP_2,
            3 => STEP_3,
            4 => STEP_4,
            5 => STEP_5,
            6 => STEP_6,
            7 => STEP_7,
            8 => STEP_8,
            9 => STEP_9,
            10 => STEP_10,
            11 => STEP_11,
            12 => STEP_12,
            13 => STEP_13,
            _ => STEP_14,
        };
        if self.soft_drop { base / soft_drop_factor } else { base }
    }
}
