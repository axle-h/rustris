use super::tetromino::TetrominoShape;
use crate::game::board::BOARD_WIDTH;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use rand_chacha::{ChaCha8Rng, ChaChaRng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const PEEK_SIZE: usize = 7;
const ALL_SHAPES: [TetrominoShape; 7] = [
    TetrominoShape::I,
    TetrominoShape::O,
    TetrominoShape::T,
    TetrominoShape::S,
    TetrominoShape::Z,
    TetrominoShape::J,
    TetrominoShape::L,
];

fn rand_shape<R: Rng>(rng: &mut R) -> TetrominoShape {
    ALL_SHAPES[rng.gen_range(0..ALL_SHAPES.len())]
}

type Seed = <ChaCha8Rng as SeedableRng>::Seed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RandomMode {
    /// Random tetromino every time
    True,
    /// All tetrominoes placed in a shuffled "bag" and drawn until the bag is empty, after which a new bag is shuffled
    Bag,
}

impl RandomMode {
    pub fn build(self, count: usize, min_garbage_per_hole: u32) -> Vec<RandomTetromino> {
        let mut seed: Seed = Default::default();
        thread_rng().fill(&mut seed);
        (0..count)
            .map(|_| RandomTetromino::new(self, min_garbage_per_hole, seed))
            .collect()
    }
}

pub struct RandomTetromino {
    random_mode: RandomMode,
    min_garbage_per_hole: u32, // move the garbage hole every n garbage
    garbage_since_last_hole: u32,
    current_garbage_hole: u32,
    rng: ChaChaRng,
    queue: VecDeque<TetrominoShape>,
}

impl RandomTetromino {
    pub fn new(random_mode: RandomMode, min_garbage_per_hole: u32, seed: Seed) -> Self {
        let mut rng = ChaChaRng::from_seed(seed);
        let current_garbage_hole = rng.gen_range(0..BOARD_WIDTH);
        match random_mode {
            RandomMode::True => {
                let queue = (0..PEEK_SIZE)
                    .map(|_| rand_shape(&mut rng))
                    .collect::<VecDeque<TetrominoShape>>();
                Self {
                    random_mode,
                    min_garbage_per_hole,
                    garbage_since_last_hole: 0,
                    current_garbage_hole,
                    rng,
                    queue,
                }
            }
            RandomMode::Bag => {
                let mut result = Self {
                    random_mode,
                    min_garbage_per_hole,
                    garbage_since_last_hole: 0,
                    current_garbage_hole,
                    rng,
                    queue: VecDeque::new(),
                };
                result.assert_bags();
                result
            }
        }
    }

    pub fn next_garbage_hole(&mut self) -> u32 {
        let result = self.current_garbage_hole;
        self.garbage_since_last_hole += 1;
        if self.garbage_since_last_hole >= self.min_garbage_per_hole {
            self.garbage_since_last_hole = 0;
            self.current_garbage_hole = self.rng.gen_range(0..BOARD_WIDTH);
        }
        result
    }

    pub fn next(&mut self) -> TetrominoShape {
        match self.random_mode {
            RandomMode::True => self.next_true(),
            RandomMode::Bag => self.next_bag(),
        }
    }

    fn next_true(&mut self) -> TetrominoShape {
        self.queue.push_back(rand_shape(&mut self.rng));
        self.queue.pop_front().unwrap()
    }

    fn next_bag(&mut self) -> TetrominoShape {
        let result = self.queue.pop_front().unwrap();
        self.assert_bags();
        result
    }

    pub fn peek(&self) -> [TetrominoShape; PEEK_SIZE] {
        self.queue
            .iter()
            .take(PEEK_SIZE)
            .copied()
            .collect::<Vec<TetrominoShape>>()
            .try_into()
            .unwrap()
    }

    fn assert_bags(&mut self) {
        while self.queue.len() <= PEEK_SIZE {
            let bag = ALL_SHAPES
                .choose_multiple(&mut self.rng, ALL_SHAPES.len())
                .cloned()
                .collect::<Vec<TetrominoShape>>();
            for shape in bag {
                self.queue.push_back(shape);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn next_n(random: &mut RandomTetromino, n: usize) -> Vec<TetrominoShape> {
        (0..n).map(|_| random.next()).collect()
    }

    fn next_n_holes(random: &mut RandomTetromino, n: usize) -> Vec<u32> {
        (0..n).map(|_| random.next_garbage_hole()).collect()
    }

    #[test]
    fn bag_random() {
        let mut random = RandomMode::Bag.build(1, 10).pop().unwrap();

        // chunk into 3 bags of 7 shapes (arrays make it easier for creating the sets)
        let bags: Vec<[TetrominoShape; 7]> = next_n(&mut random, 21)
            .chunks(7)
            .map(|chunk| {
                chunk
                    .iter()
                    .copied()
                    .collect::<Vec<TetrominoShape>>()
                    .try_into()
                    .unwrap()
            })
            .collect();

        // each bag should not be in same order
        assert_ne!(bags[0], bags[1]);
        assert_ne!(bags[1], bags[2]);

        // but should all contain all the shapes
        let all_shapes = HashSet::from(ALL_SHAPES);
        assert_eq!(HashSet::from(bags[0]), all_shapes);
        assert_eq!(HashSet::from(bags[1]), all_shapes);
        assert_eq!(HashSet::from(bags[2]), all_shapes);
    }

    #[test]
    fn bag_random_peek() {
        let mut random = RandomMode::Bag.build(1, 10).pop().unwrap();
        let peek = random.peek();
        let observed: [TetrominoShape; PEEK_SIZE] =
            next_n(&mut random, PEEK_SIZE).try_into().unwrap();
        assert_eq!(observed, peek);
    }

    #[test]
    fn true_random() {
        let mut random = RandomMode::True.build(1, 10).pop().unwrap();
        let observed: [TetrominoShape; 1000] = next_n(&mut random, 1000).try_into().unwrap();
        // should generate all shapes in 1000 tries
        assert_eq!(HashSet::from(observed), HashSet::from(ALL_SHAPES));
    }

    #[test]
    fn true_random_peek() {
        let mut random = RandomMode::True.build(1, 10).pop().unwrap();
        let peek = random.peek();
        let observed: [TetrominoShape; PEEK_SIZE] =
            next_n(&mut random, PEEK_SIZE).try_into().unwrap();
        assert_eq!(observed, peek);
    }

    #[test]
    fn static_garbage_hole() {
        let mut random = RandomMode::True.build(1, 100).pop().unwrap();
        let observed: [u32; 100] = next_n_holes(&mut random, 100).try_into().unwrap();
        assert_eq!(HashSet::from(observed).len(), 1);
    }

    #[test]
    fn dynamic_garbage_hole() {
        let mut random = RandomMode::True.build(1, 1).pop().unwrap();
        let observed: [u32; 100] = next_n_holes(&mut random, 100).try_into().unwrap();
        assert!(HashSet::from(observed).len() > 1);
    }
}
