use std::ops::{Add, Div};
use std::cmp::Ordering;
use std::iter::Sum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct GameResult {
    score: u32,
    lines: u32,
    level: u32
}

impl GameResult {
    pub fn new(score: u32, lines: u32, level: u32) -> Self {
        Self { score, lines, level }
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
}

impl Add for GameResult {
    type Output = GameResult;

    fn add(self, rhs: Self) -> Self::Output {
        GameResult {
            score: self.score + rhs.score,
            lines: self.lines + rhs.lines,
            level: self.level + rhs.level,
        }
    }
}

impl Sum for GameResult {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        let mut sum = GameResult { score: 0, lines: 0, level: 0 };
        for result in iter {
            sum = result + sum;
        }
        sum
    }
}

impl Div<usize> for GameResult {
    type Output = GameResult;

    fn div(self, rhs: usize) -> Self::Output {
        let rhs_f32 = rhs as f32;
        GameResult {
            score: (self.score as f32 / rhs_f32).round() as u32,
            lines: (self.lines as f32 / rhs_f32).round() as u32,
            level: (self.level as f32 / rhs_f32).round() as u32,
        }
    }
}

impl Ord for GameResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}