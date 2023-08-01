use crate::event::{GameEvent, GameOverCondition};
use crate::game::block::BlockState;
use crate::game::board::DestroyLines;
use crate::game::random::{RandomTetromino, PEEK_SIZE};
use board::Board;

use std::cmp::{max, min};

use std::time::Duration;
use tetromino::TetrominoShape;

pub mod block;
pub mod board;
pub mod geometry;
pub mod random;
pub mod tetromino;

const LINES_PER_LEVEL: u32 = 10;
const SOFT_DROP_STEP_FACTOR: u32 = 20;
const SOFT_DROP_SPAWN_FACTOR: u32 = 10;
const MIN_SPAWN_DELAY: Duration = Duration::from_millis(500);
const LOCK_DURATION: Duration = Duration::from_millis(500);
const SOFT_DROP_LOCK_DURATION: Duration = Duration::from_millis(500 / 2);
const MAX_LOCK_PLACEMENTS: u32 = 15;
const GARBAGE_WAIT: Duration = Duration::from_millis(50);

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
pub enum GameState {
    Spawn(Duration, TetrominoShape),
    Fall(Duration),
    Lock(Duration),
    HardDropLock,
    Pattern,               // check the board for patterns to destroy e.g. lines
    Destroy(DestroyLines), // destroy marked patterns
    GameOver,
    SpawnGarbage {
        duration: Duration,
        next_shape: TetrominoShape,
        spawned: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combo {
    count: u32,
    difficult: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HoldState {
    shape: TetrominoShape,
    locked: bool,
}

pub struct Game {
    player: u32,
    board: Board,
    random: RandomTetromino,
    level: u32,
    lines: u32,
    score: u32,
    combo: Option<Combo>,
    state: GameState,
    soft_drop: bool,
    skip_next_spawn_delay: bool,
    hold: Option<HoldState>,
    garbage_buffer: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameMetrics {
    pub player: u32,
    pub level: u32,
    pub lines: u32,
    pub score: u32,
    pub combo: Option<Combo>,
    pub queue: [TetrominoShape; PEEK_SIZE],
    pub hold: Option<TetrominoShape>,
}

impl Game {
    pub fn new(player: u32, level: u32, mut random: RandomTetromino) -> Game {
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
            hold: None,
            garbage_buffer: 0,
        }
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn hold(&mut self) -> Option<GameEvent> {
        if !(matches!(self.state, GameState::Fall(_))
            || matches!(self.state, GameState::Lock(duration) if duration < LOCK_DURATION))
            || matches!(self.hold, Some(HoldState { locked: true, .. }))
        {
            // hold is blocked
            return None;
        }

        let held_shape = match self.board.hold() {
            None => return None,
            Some(shape) => shape,
        };

        let next_shape = match self.hold {
            None => self.random.next(), // just spawn next random shape
            Some(HoldState { shape, .. }) => shape,
        };

        self.state = GameState::Spawn(MIN_SPAWN_DELAY, next_shape);
        self.hold = Some(HoldState {
            locked: true,
            shape: held_shape,
        });
        Some(GameEvent::Hold)
    }

    pub fn set_soft_drop(&mut self, soft_drop: bool) -> Option<GameEvent> {
        self.soft_drop = soft_drop;
        if soft_drop {
            Some(GameEvent::SoftDrop)
        } else {
            None
        }
    }

    pub fn hard_drop(&mut self) -> Option<GameEvent> {
        self.board.hard_drop().map(|(hard_dropped_rows, minos)| {
            self.state = GameState::HardDropLock;
            self.score += hard_dropped_rows * HARD_DROP_POINTS_PER_ROW;
            self.skip_next_spawn_delay = true;
            GameEvent::HardDrop {
                player: self.player,
                minos,
                dropped_rows: hard_dropped_rows,
            }
        })
    }

    pub fn metrics(&self) -> GameMetrics {
        GameMetrics {
            player: self.player,
            level: self.level,
            lines: self.lines,
            score: self.score,
            combo: self.combo,
            queue: self.random.peek(),
            hold: self.hold.map(|h| h.shape),
        }
    }

    pub fn left(&mut self) -> Option<GameEvent> {
        if self.with_checking_lock(|board| board.left()) {
            Some(GameEvent::Move)
        } else {
            None
        }
    }

    pub fn right(&mut self) -> Option<GameEvent> {
        if self.with_checking_lock(|board| board.right()) {
            Some(GameEvent::Move)
        } else {
            None
        }
    }

    pub fn rotate(&mut self, clockwise: bool) -> Option<GameEvent> {
        if self.with_checking_lock(|board| board.rotate(clockwise)) {
            Some(GameEvent::Rotate)
        } else {
            None
        }
    }

    pub fn send_garbage(&mut self, rows: u32) {
        self.garbage_buffer += rows;
    }

    fn with_checking_lock<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Board) -> bool,
    {
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
                    self.state = GameState::Fall(Duration::ZERO);
                } else {
                    // the tetromino just ran out of lock movements, lock it asap
                    self.state = GameState::Lock(LOCK_DURATION);
                }
                true
            }
            _ => f(&mut self.board), // not in lock state, pass through closure
        }
    }

    pub fn update(&mut self, delta: Duration) -> Option<GameEvent> {
        let (state, event) = match self.state {
            GameState::Spawn(duration, shape) => self.spawn(duration + delta, shape),
            GameState::Fall(duration) => self.fall(duration + delta),
            GameState::Lock(duration) => self.lock(duration + delta, false),
            GameState::HardDropLock => self.lock(LOCK_DURATION, true),
            GameState::Pattern => self.pattern(),
            GameState::Destroy(pattern) => self.destroy(pattern),
            GameState::SpawnGarbage {
                duration,
                next_shape,
                spawned,
            } => self.spawn_garbage(duration + delta, next_shape, spawned),
            GameState::GameOver => (GameState::GameOver, None),
        };
        self.state = state;
        event
    }

    fn spawn(
        &mut self,
        duration: Duration,
        shape: TetrominoShape,
    ) -> (GameState, Option<GameEvent>) {
        if self.garbage_buffer > 0 {
            return (
                GameState::SpawnGarbage {
                    duration: Duration::ZERO,
                    next_shape: shape,
                    spawned: 0,
                },
                Some(GameEvent::ReceivedGarbage {
                    player: self.player,
                    lines: self.garbage_buffer,
                }),
            );
        }

        if !self.skip_next_spawn_delay && duration < self.spawn_delay() {
            return (GameState::Spawn(duration, shape), None);
        }

        self.skip_next_spawn_delay = false;
        if let Some(minos) = self.board.try_spawn_tetromino(shape) {
            (
                GameState::Fall(Duration::ZERO),
                Some(GameEvent::Spawn {
                    player: self.player,
                    minos,
                }),
            )
        } else {
            // cannot spawn a tetromino is a game over event
            (
                GameState::GameOver,
                Some(GameEvent::GameOver {
                    player: self.player,
                    condition: GameOverCondition::BlockOut,
                }),
            )
        }
    }

    fn fall(&mut self, duration: Duration) -> (GameState, Option<GameEvent>) {
        if duration < self.step_delay() {
            return (GameState::Fall(duration), None);
        }

        if !self.board.step_down() {
            // cannot step down, start lock
            return (GameState::Lock(Duration::ZERO), None);
        }

        // has stepped down one row, update score if soft dropping
        if self.soft_drop {
            self.score += SOFT_DROP_POINTS_PER_ROW;
        }

        if self.board.is_collision() {
            // step has caused a collision, start a lock
            let state = if self.board.lock_placements() >= MAX_LOCK_PLACEMENTS {
                // lock asap
                GameState::Lock(LOCK_DURATION)
            } else {
                GameState::Lock(Duration::ZERO)
            };
            (state, Some(GameEvent::Fall))
        } else {
            // no collisions, start a new fall step
            (GameState::Fall(Duration::ZERO), Some(GameEvent::Fall))
        }
    }

    fn lock(&mut self, duration: Duration, hard_dropped: bool) -> (GameState, Option<GameEvent>) {
        let max_lock_duration = if self.soft_drop {
            SOFT_DROP_LOCK_DURATION
        } else {
            LOCK_DURATION
        };
        if !hard_dropped && duration < max_lock_duration {
            (GameState::Lock(duration), None)
        } else if self.board.is_collision() {
            // lock timeout and still colliding so lock the piece now
            // but before locking, need to check for a game over event.
            let is_lock_out = self.board.is_tetromino_above_skyline();

            let minos = self.board.lock();
            // maybe unlock hold
            match self.hold {
                Some(HoldState { locked, shape }) if locked => {
                    self.hold = Some(HoldState {
                        locked: false,
                        shape,
                    });
                }
                _ => {}
            }

            if is_lock_out {
                (
                    GameState::GameOver,
                    Some(GameEvent::GameOver {
                        player: self.player,
                        condition: GameOverCondition::LockOut,
                    }),
                )
            } else {
                (
                    GameState::Pattern,
                    Some(GameEvent::Lock {
                        player: self.player,
                        minos: minos.expect("we must've locked"),
                        hard_or_soft_dropped: hard_dropped || self.soft_drop,
                    }),
                )
            }
        } else {
            // otherwise must've moved over empty space so start a new fall
            (GameState::Fall(Duration::ZERO), None)
        }
    }

    fn pattern(&mut self) -> (GameState, Option<GameEvent>) {
        // TODO t-spin garbage
        let lines = self.board.pattern();
        (GameState::Destroy(lines), Some(GameEvent::Destroy(lines)))
    }

    fn destroy(&mut self, lines: DestroyLines) -> (GameState, Option<GameEvent>) {
        self.board.destroy(lines);
        (
            GameState::Spawn(Duration::ZERO, self.random.next()),
            self.update_score_and_get_garbage_to_send(lines),
        )
    }

    fn spawn_garbage(
        &mut self,
        duration: Duration,
        next_shape: TetrominoShape,
        spawned: u32,
    ) -> (GameState, Option<GameEvent>) {
        if duration < GARBAGE_WAIT {
            return (
                GameState::SpawnGarbage {
                    duration,
                    next_shape,
                    spawned,
                },
                None,
            );
        }

        let hole = self.random.next_garbage_hole();
        self.board.send_garbage(hole);

        if self.board.is_stack_above_skyline() {
            // TopOut
            return (
                GameState::GameOver,
                Some(GameEvent::GameOver {
                    player: self.player,
                    condition: GameOverCondition::TopOut,
                }),
            );
        }

        self.garbage_buffer -= 1;
        let event = GameEvent::ReceivedGarbageLine {
            player: self.player,
            line: spawned,
        };

        if self.garbage_buffer == 0 {
            self.skip_next_spawn_delay = true;
            (GameState::Spawn(Duration::ZERO, next_shape), Some(event))
        } else {
            (
                GameState::SpawnGarbage {
                    duration: Duration::ZERO,
                    next_shape,
                    spawned: spawned + 1,
                },
                Some(event),
            )
        }
    }

    fn update_score_and_get_garbage_to_send(&mut self, pattern: DestroyLines) -> Option<GameEvent> {
        // TODO test
        // todo t-spin
        // todo perfect clear

        let line_count = pattern.iter().filter(|y| y.is_some()).count() as u32;

        let (action_score, action_difficult, garbage_lines) = match line_count {
            0 => {
                self.combo = None;
                return None;
            }
            1 => (SINGLE_POINTS, false, 0),
            2 => (DOUBLE_POINTS, false, 1),
            3 => (TRIPLE_POINTS, false, 2),
            4 => (TETRIS_POINTS, true, 4),
            _ => unreachable!(),
        };

        // update combo
        self.combo = match self.combo {
            None => Some(Combo {
                count: 0,
                difficult: action_difficult,
            }),
            Some(Combo { count, difficult }) => Some(Combo {
                count: count + 1,
                difficult: difficult && action_difficult,
            }),
        };

        // calculate score delta
        let level_multiplier = self.level + 1;
        let (difficult_score_multiplier, difficult_garbage_lines) = match self.combo {
            // back to back difficult clears get a 1.5x multiplier
            Some(Combo { count, difficult }) if count > 0 && difficult => (DIFFICULT_MULTIPLIER, 1),
            _ => (1.0, 0),
        };
        let combo_score = match self.combo {
            Some(Combo { count, .. }) if count > 0 => COMBO_POINTS * count,
            _ => 0,
        };
        let score_delta =
            action_score as f64 * level_multiplier as f64 * difficult_score_multiplier
                + combo_score as f64;

        // update score
        self.score += score_delta.round() as u32;

        // update level
        self.lines += line_count;
        let line_level = self.lines / LINES_PER_LEVEL;
        let level_up = line_level > self.level;
        if level_up {
            self.level = line_level;
        }

        Some(GameEvent::Destroyed {
            player: self.player,
            lines: pattern,
            send_garbage_lines: garbage_lines + difficult_garbage_lines,
            level_up,
        })
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

        if self.soft_drop {
            max(base / soft_drop_factor, STEP_14)
        } else {
            base
        }
    }
}
