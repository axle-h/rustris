use std::collections::{HashSet, VecDeque};
use crate::game::board::{Board, BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::geometry::Point;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Holes {
    open: u32,
    closed: u32
}

impl Holes {
    pub fn open(&self) -> u32 {
        self.open
    }

    pub fn closed(&self) -> u32 {
        self.closed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct StackStats {
    max_heights: [u32; BOARD_WIDTH as usize],
    delta_height: u32
}

impl StackStats {
    pub fn max_heights(&self) -> [u32; BOARD_WIDTH as usize] {
        self.max_heights
    }

    pub fn delta_height(&self) -> u32 {
        self.delta_height
    }
}

pub trait BoardFeatures {
    fn holes(&self) -> Holes;
    fn stack_stats(&self) -> StackStats;
}

impl BoardFeatures for Board {
    fn holes(&self) -> Holes {
        let mut holes: HashSet<Point> = HashSet::new();

        for x in 0..BOARD_WIDTH as i32 {
            let mut in_stack = false;
            for y in (0..BOARD_HEIGHT as i32).rev() {
                let point = Point::new(x, y);
                if !self.block(point).is_empty() {
                    in_stack = true;
                } else if in_stack {
                    holes.insert(point);
                }
            }
        }

        let mut open_holes: HashSet<Point> = HashSet::new();
        for p in holes.iter() {
            let mut to_visit = VecDeque::from([p.translate(-1, 0), p.translate(1, 0)]);
            while let Some(neighbour) = to_visit.pop_front() {
                if neighbour.x < 0 || neighbour.x >= BOARD_WIDTH as i32 || !self.block(neighbour).is_empty() {
                    continue
                }

                if open_holes.contains(&neighbour) || !holes.contains(&neighbour) {
                    // a neighbour is an open hole this hole must also be open
                    // OR a neighbour is empty but not a hole then this must be an open hole
                    open_holes.insert(*p);
                    break
                }

                // neighbour is a hole, but we're not sure if it's an open hole yet so check the 2nd order neighbour
                let neighbour2 = if neighbour.x > p.x {
                    neighbour.translate(1, 0)
                } else {
                    neighbour.translate(-1, 0)
                };
                to_visit.push_back(neighbour2);
            }
        }

        Holes {
            open: open_holes.len() as u32,
            closed: (holes.len() - open_holes.len()) as u32
        }
    }

    fn stack_stats(&self) -> StackStats {
        let mut max_heights = [0; BOARD_WIDTH as usize];

        for x in 0..BOARD_WIDTH as i32 {
            for y in (0..BOARD_HEIGHT as i32).rev() {
                if self.block((x, y)).is_empty() {
                    continue;
                }
                max_heights[x as usize] = y as u32 + 1;
                break;
            }
        }

        let mut last_height: Option<u32> = None;
        let mut delta_height = 0;

        for height in max_heights {
            delta_height += last_height.map(|h| h.abs_diff(height)).unwrap_or(0);
            last_height = Some(height);
        }

        StackStats {
            max_heights,
            delta_height
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::block::BlockState;
    use crate::game::geometry::Rotation;
    use crate::game::tetromino::TetrominoShape;
    use super::*;

    #[test]
    fn empty_board() {
        let holes = Board::new().holes();
        assert_eq!(holes, Holes::default());
    }

    #[test]
    fn no_holes() {
        let holes = Board::new().having_stack_at(&[(0, 0)]).holes();
        assert_eq!(holes, Holes::default());
    }

    #[test]
    fn one_closed_hole() {
        let holes = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1),
            (0, 0),         (2, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 1, open: 0 });
    }

    #[test]
    fn one_closed_hole_against_the_edge() {
        let holes = Board::new().having_stack_at(&[
            (0, 1), (1, 1),
                    (1, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 1, open: 0 });
    }

    #[test]
    fn double_closed_hole() {
        let holes = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1), (3, 1),
            (0, 0),                 (3, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 2, open: 0 });
    }

    #[test]
    fn two_closed_holes() {
        let holes = Board::new().having_stack_at(&[
            (0, 3), (1, 3), (2, 3),
            (0, 2),         (2, 2),
            (0, 1), (1, 1), (2, 1),
            (0, 0),         (2, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 2, open: 0 });
    }

    #[test]
    fn one_open_hole_right() {
        let holes = Board::new().having_stack_at(&[
            (0, 1), (1, 1),
            (0, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 0, open: 1 });
    }

    #[test]
    fn double_open_hole_right() {
        let holes = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1),
            (0, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 0, open: 2 });
    }

    #[test]
    fn one_open_hole_left() {
        let holes = Board::new().having_stack_at(&[
            (8, 1), (9, 1),
                    (9, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 0, open: 1 });
    }

    #[test]
    fn double_open_hole_left() {
        let holes = Board::new().having_stack_at(&[
            (7, 1), (8, 1), (9, 1),
                            (9, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 0, open: 2 });
    }

    #[test]
    fn big_open_holes() {
        let holes = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1), (3, 1), (4, 1),     (6, 1), (7, 1), (8, 1), (9, 1),
            (0, 0),                                                             (9, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 0, open: 7 });
    }

    #[test]
    fn kitchen_sink_of_holes() {
        let holes = Board::new().having_stack_at(&[
                            (2, 3), (3, 3), (4, 3),     (6, 3), (7, 3), (8, 3), (9, 3),
            (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),     (6, 2),         (8, 2),
            (0, 1), (1, 1),         (3, 1), (4, 1),     (6, 1), (7, 1), (8, 1),
            (0, 0),                                     (6, 0),                 (9, 0)
        ]).holes();
        assert_eq!(holes, Holes { closed: 6, open: 4 });
    }

    #[test]
    fn stack_stats_empty_board() {
        let stack_stats = Board::new().having_stack_at(&[]).stack_stats();
        assert_eq!(stack_stats, StackStats::default());
    }

    #[test]
    fn stack_stats_flat_board() {
        let stack_stats = Board::new().having_stack_at(&[
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)
        ]).stack_stats();
        assert_eq!(
            stack_stats,
            StackStats {
                max_heights: [1; BOARD_WIDTH as usize],
                delta_height: 0
            }
        );
    }

    #[test]
    fn stack_stats_single_block_change() {
        let stack_stats = Board::new().having_stack_at(&[
            (0, 0)
        ]).stack_stats();
        assert_eq!(
            stack_stats,
            StackStats {
                max_heights: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                delta_height: 1
            }
        );
    }

    #[test]
    fn stack_stats_double_block_change() {
        let stack_stats = Board::new().having_stack_at(&[
            (1, 0)
        ]).stack_stats();
        assert_eq!(
            stack_stats,
            StackStats {
                max_heights: [0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                delta_height: 2
            }
        );
    }

    #[test]
    fn stack_stats_multi_level_change() {
        let stack_stats = Board::new().having_stack_at(&[
            (1, 1),
            (1, 0)
        ]).stack_stats();

        assert_eq!(
            stack_stats,
            StackStats {
                max_heights: [0, 2, 0, 0, 0, 0, 0, 0, 0, 0],
                delta_height: 4
            }
        );
    }

    #[test]
    fn stack_stats_kitchen_sink() {
        let stack_stats = Board::new().having_stack_at(&[
                            (2, 3),         (4, 3),     (6, 3),         (8, 3),
            (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),     (6, 2),         (8, 2),
            (0, 1), (1, 1),         (3, 1), (4, 1),     (6, 1), (7, 1), (8, 1),
            (0, 0),                                     (6, 0),                 (9, 0)
        ]).stack_stats();

        assert_eq!(
            stack_stats,
            StackStats {
                max_heights: [3, 3, 4, 3, 4, 0, 4, 2, 4, 1],
                delta_height: 18
            }
        );
    }

    trait BoardHarness {
        fn having_stack_at(&mut self, points: &[(u32, u32)]) -> &mut Self;
    }

    impl BoardHarness for Board {
        fn having_stack_at(&mut self, points: &[(u32, u32)]) -> &mut Self {
            for point in points.into_iter() {
                self.set_block(*point, BlockState::Stack(TetrominoShape::L, Rotation::North, 0));
            }
            self
        }
    }
}