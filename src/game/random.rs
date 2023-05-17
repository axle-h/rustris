use std::collections::VecDeque;
use rand::{
    thread_rng,
    distributions::{Distribution, Standard},
    Rng,
};
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
use super::tetromino::TetrominoShape;

pub const PEEK_SIZE: usize = 7;
const ALL_SHAPES: [TetrominoShape; 7] = [
    TetrominoShape::I, TetrominoShape::O, TetrominoShape::T,
    TetrominoShape::S, TetrominoShape::Z,
    TetrominoShape::J, TetrominoShape::L
];

pub enum RandomMode { True, Bag }

impl RandomMode {
    pub fn build(&self) -> Box<dyn RandomTetromino> {
        match self {
            RandomMode::True => Box::new(TrueRandom::new()),
            RandomMode::Bag => Box::new(BagRandom::new())
        }
    }
}

pub trait RandomTetromino {
    fn next(&mut self) -> TetrominoShape;
    fn peek(&self) -> [TetrominoShape; PEEK_SIZE];
}

pub struct TrueRandom {
    queue: VecDeque<TetrominoShape>
}

impl TrueRandom {
    pub fn new() -> Self {
        Self { queue: (0..PEEK_SIZE).map(|_| rand::random()).collect() }
    }
}

impl RandomTetromino for TrueRandom {
    fn next(&mut self) -> TetrominoShape {
        self.queue.push_back(rand::random());
        self.queue.pop_front().unwrap()
    }

    fn peek(&self) -> [TetrominoShape; PEEK_SIZE] {
        self.queue.iter().take(PEEK_SIZE).copied().collect::<Vec<TetrominoShape>>().try_into().unwrap()
    }
}

pub struct BagRandom {
    queue: VecDeque<TetrominoShape>
}

impl BagRandom {
    pub fn new() -> Self {
        let mut result = Self { queue: VecDeque::new() };
        result.assert_bags();
        return result;
    }

    fn assert_bags(&mut self) {
        while self.queue.len() <= PEEK_SIZE {
            let bag: BagOfTetromino = rand::random();
            for shape in bag.contents {
                self.queue.push_back(shape);
            }
        }
    }
}

impl RandomTetromino for BagRandom {
    fn next(&mut self) -> TetrominoShape {
        let result = self.queue.pop_front().unwrap();
        self.assert_bags();
        return result;
    }

    fn peek(&self) -> [TetrominoShape; PEEK_SIZE] {
        self.queue.iter().take(PEEK_SIZE).copied().collect::<Vec<TetrominoShape>>().try_into().unwrap()
    }
}

impl Distribution<TetrominoShape> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TetrominoShape {
        let index = rng.gen_range(0..ALL_SHAPES.len());
        ALL_SHAPES[index]
    }
}

pub struct BagOfTetromino {
    contents: Vec<TetrominoShape>
}

impl Distribution<BagOfTetromino> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BagOfTetromino {
        BagOfTetromino {
            contents: ALL_SHAPES.choose_multiple(rng, ALL_SHAPES.len()).cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    fn next_n(random: &mut dyn RandomTetromino, n: usize) -> Vec<TetrominoShape> {
        (0..n).map(|_| random.next()).collect()
    }

    #[test]
    fn bag_random() {
        let mut random = BagRandom::new();

        // chunk into 3 bags of 7 shapes (arrays make it easier for creating the sets)
        let bags: Vec<[TetrominoShape; 7]> = next_n(&mut random, 21)
            .chunks(7)
            .map(|chunk| chunk.iter().copied().collect::<Vec<TetrominoShape>>().try_into().unwrap())
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
        let mut random = BagRandom::new();
        let peek = random.peek();
        let observed: [TetrominoShape; PEEK_SIZE] = next_n(&mut random, PEEK_SIZE).try_into().unwrap();
        assert_eq!(observed, peek);
    }

    #[test]
    fn true_random() {
        let mut random = TrueRandom::new();
        let observed: [TetrominoShape; 1000] = next_n(&mut random, 1000).try_into().unwrap();
        // should generate all shapes in 1000 tries
        assert_eq!(HashSet::from(observed), HashSet::from(ALL_SHAPES));
    }

    #[test]
    fn true_random_peek() {
        let mut random = TrueRandom::new();
        let peek = random.peek();
        let observed: [TetrominoShape; PEEK_SIZE] = next_n(&mut random, PEEK_SIZE).try_into().unwrap();
        assert_eq!(observed, peek);
    }
}