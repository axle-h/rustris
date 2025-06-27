use std::cmp::Ordering;
use std::ops::Add;
use itertools::Itertools;
use crate::game::geometry::{Point, Pose, Rotation};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputSequence(Vec<Translation>);

impl InputSequence {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn from_slice(seq: &[Translation]) -> Self {
        Self(seq.to_vec())
    }

    pub fn after_soft_drop(&self) -> Option<Self> {
        self.0.iter().position(|&t| t == Translation::SoftDrop).map(|i| Self::from_slice(&self.0[i + 1..]))
    }

    pub fn new(sequence: Vec<Translation>) -> Self {
        Self(sequence)
    }

    pub fn translations(&self) -> &[Translation] {
        &self.0
    }
    
    pub fn pop(&mut self) -> Option<Translation> {
        self.0.pop()
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn push(&mut self, translation: Translation) {
        if let Some(last) = self.0.last() {
            if *last == translation.opposite().unwrap_or(translation) {
                // If the translation is the opposite of the last one, remove the last one
                self.0.pop();
            } else {
                self.0.push(translation);
            }
        } else {
            self.0.push(translation);
        }
    }

    pub fn resolve(&self) -> ResolvedInputSequence {
        let mut translate = 0;
        let mut rotation = Rotation::North;
        for translation in self.0.iter() {
            match translation {
                Translation::Left => translate -= 1,
                Translation::Right => translate += 1,
                Translation::RotateClockwise => rotation = rotation.rotate(true),
                Translation::RotateAnticlockwise => rotation = rotation.rotate(false),
                _ => {}
            }
        }
        ResolvedInputSequence { rotation, translate }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ResolvedInputSequence {
    rotation: Rotation,
    translate: i32,
}

impl ResolvedInputSequence {
    pub fn into_sequence(self) -> InputSequence {
        let mut seq = vec![];
        let translation = if self.translate > 0 {
            Translation::Right
        } else {
            Translation::Left
        };
        for _ in 0..self.translate.abs() {
            seq.push(translation);
        }
        match self.rotation {
            Rotation::North => {}
            Rotation::East => {
                seq.push(Translation::RotateClockwise);
            }
            Rotation::South => {
                seq.push(Translation::RotateClockwise);
                seq.push(Translation::RotateClockwise);
            }
            Rotation::West => {
                seq.push(Translation::RotateAnticlockwise);
            }
        }
        InputSequence(seq)
    }
}

impl Add<Translation> for ResolvedInputSequence {
    type Output = Self;

    fn add(self, rhs: Translation) -> Self::Output {
        match rhs {
            Translation::Left => Self { translate: self.translate - 1, ..self },
            Translation::Right => Self { translate: self.translate + 1, ..self },
            Translation::RotateClockwise => Self { rotation: self.rotation.rotate(true), ..self },
            Translation::RotateAnticlockwise => Self { rotation: self.rotation.rotate(false), ..self },
            _ => self
        }
    }
}

impl Add<Translation> for InputSequence {
    type Output = Self;

    fn add(mut self, rhs: Translation) -> Self::Output {
        self.push(rhs);
        self
    }
}

impl Default for InputSequence {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialOrd for InputSequence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InputSequence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Translation {
    Left = 1,
    Right = 2,
    RotateClockwise = 3,
    RotateAnticlockwise = 4,
    HardDrop = 5,
    SoftDrop = 6
}

impl Translation {
    pub fn into_index(self) -> usize {
        self as usize
    }
    
    pub fn opposite(self) -> Option<Self> {
        match self {
            Translation::Left => Some(Translation::Right),
            Translation::Right => Some(Translation::Left),
            Translation::RotateClockwise => Some(Translation::RotateAnticlockwise),
            Translation::RotateAnticlockwise => Some(Translation::RotateClockwise),
            _ => None, // HardDrop and SoftDrop do not have opposites
        }
    }
    
    pub fn apply(self, pose: Pose) -> Pose {
        match self {
            Translation::Left => pose + Point::new(-1, 0),
            Translation::Right => pose + Point::new(1, 0),
            Translation::RotateClockwise => pose.rotate(true),
            Translation::RotateAnticlockwise => pose.rotate(false),
            Translation::HardDrop | Translation::SoftDrop => panic!("undefined translation for pose"),
        }
    }
}

impl PartialOrd for Translation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Translation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_index().cmp(&other.into_index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn rotates_clockwise() {
        let seq = InputSequence::new(vec![Translation::RotateClockwise, Translation::RotateClockwise]).resolve();
        assert_eq!(seq, ResolvedInputSequence { translate: 0, rotation: Rotation::South });
    }

    #[test]
    fn rotates_anticlockwise() {
        let seq = InputSequence::new(vec![Translation::RotateAnticlockwise, Translation::RotateAnticlockwise]).resolve();
        assert_eq!(seq, ResolvedInputSequence { translate: 0, rotation: Rotation::South });
    }

    #[test]
    fn rotates_360() {
        let seq = InputSequence::new(vec![
            Translation::RotateClockwise, Translation::RotateClockwise,
            Translation::RotateClockwise, Translation::RotateClockwise
        ]).resolve();
        
        assert_eq!(seq, ResolvedInputSequence { translate: 0, rotation: Rotation::North });
    }

    #[test]
    fn left() {
        let seq = InputSequence::new(vec![Translation::Left, Translation::Left]).resolve();
        assert_eq!(seq, ResolvedInputSequence { translate: -2, rotation: Rotation::North });
    }

    #[test]
    fn right() {
        let seq = InputSequence::new(vec![Translation::Right, Translation::Right]).resolve();
        assert_eq!(seq, ResolvedInputSequence { translate: 2, rotation: Rotation::North });
    }

    #[test]
    fn deduplicates_movement() {
        let seq = InputSequence::new(vec![
            Translation::Left, Translation::Left, Translation::Right
        ]).resolve();
        assert_eq!(seq, ResolvedInputSequence { translate: -1, rotation: Rotation::North });
    }
}