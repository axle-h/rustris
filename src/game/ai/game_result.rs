use std::ops::{Add, AddAssign, Div, Sub};
use std::cmp::Ordering;
use std::iter::Sum;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameResult {
    score: u32,
    lines: u32,
    level: u32,
    game_over: bool
}

impl GameResult {
    pub fn new(score: u32, lines: u32, level: u32, game_over: bool) -> Self {
        Self { score, lines, level, game_over }
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn lines(&self) -> u32 {
        self.lines
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn game_over(&self) -> bool {
        self.game_over
    }
}

impl Default for GameResult {
    fn default() -> Self {
        Self::new(0, 0, 0, false)
    }
}

impl Add for GameResult {
    type Output = GameResult;

    fn add(self, rhs: Self) -> Self::Output {
        GameResult {
            score: self.score + rhs.score,
            lines: self.lines + rhs.lines,
            level: self.level + rhs.level,
            game_over: self.game_over || rhs.game_over,
        }
    }
}

impl AddAssign for GameResult {
    fn add_assign(&mut self, rhs: Self) {
        self.score += rhs.score;
        self.lines += rhs.lines;
        self.level += rhs.level;
        self.game_over |= rhs.game_over;
    }
}

impl Sub for GameResult {
    type Output = f64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.score as f64 - rhs.score as f64
    }
}

impl Div<usize> for GameResult {
    type Output = GameResult;

    fn div(self, rhs: usize) -> Self::Output {
        let rhs_f64 = rhs as f64;
        GameResult {
            score: (self.score as f64 / rhs_f64).round() as u32,
            lines: (self.lines as f64 / rhs_f64).round() as u32,
            level: (self.level as f64 / rhs_f64).round() as u32,
            game_over: self.game_over,
        }
    }
}

impl Ord for GameResult {
    fn cmp(&self, other: &Self) -> Ordering {
        other.game_over.cmp(&self.game_over)
            .then_with(|| self.score.cmp(&other.score))
    }
}

impl PartialOrd for GameResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))   
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_result_ordering() {
        let result1 = GameResult::new(10, 10, 10, false);
        let result2 = GameResult::new(10, 10, 10, true);
        assert!(result1 > result2);
    }
}