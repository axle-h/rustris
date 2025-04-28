use std::collections::{HashSet, VecDeque};
use crate::game::board::{compact_destroy_lines, Board, BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::geometry::Point;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct StackStats {
    open_holes: u32,
    closed_holes: u32,
    max_heights: [u32; BOARD_WIDTH as usize],
    sum_delta_height: u32,
    max_delta_height: u32,
    cleared_lines: u32,
}

impl StackStats {
    pub fn open_holes(&self) -> u32 {
        self.open_holes
    }

    pub fn closed_holes(&self) -> u32 {
        self.closed_holes
    }
    
    pub fn max_heights(&self) -> [u32; BOARD_WIDTH as usize] {
        self.max_heights
    }

    pub fn sum_delta_height(&self) -> u32 {
        self.sum_delta_height
    }
    
    pub fn max_delta_height(&self) -> u32 {
        self.max_delta_height
    }

    pub fn cleared_lines(&self) -> u32 {
        self.cleared_lines
    }
}

pub trait BoardFeatures {
    fn stack_stats(&self) -> StackStats;
}

impl BoardFeatures for Board {
    fn stack_stats(&self) -> StackStats {
        let mut board = *self; // copy to apply the patterns if any

        let patterns = board.pattern();
        let cleared_lines = compact_destroy_lines(patterns).len() as u32;
        board.destroy(patterns);

        let mut holes: HashSet<Point> = HashSet::new();
        let mut max_heights = [0; BOARD_WIDTH as usize];

        for x in 0..BOARD_WIDTH as i32 {
            let mut in_stack = false;
            for y in (0..BOARD_HEIGHT as i32).rev() {
                let point = Point::new(x, y);
                if !board.block(point).is_empty() {
                    if !in_stack {
                        max_heights[x as usize] = y as u32 + 1;
                    }
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
                if neighbour.x < 0 || neighbour.x >= BOARD_WIDTH as i32 || !board.block(neighbour).is_empty() {
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

        let mut last_height: Option<u32> = None;
        let mut max_delta_height = 0;
        let mut sum_delta_height = 0;

        for height in max_heights {
            let next_delta_height = last_height.map(|h| h.abs_diff(height)).unwrap_or(0);
            max_delta_height = max_delta_height.max(next_delta_height);
            sum_delta_height += next_delta_height;
            last_height = Some(height);
        }

        StackStats {
            open_holes: open_holes.len() as u32,
            closed_holes: (holes.len() - open_holes.len()) as u32,
            max_heights,
            sum_delta_height,
            max_delta_height,
            cleared_lines
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
        let stats = Board::new().stack_stats();
        assert_eq!(stats, StackStats::default());
    }

    #[test]
    fn no_holes() {
        let stats = Board::new().having_stack_at(&[(0, 0)]).stack_stats();
        assert_eq!(stats.closed_holes, 0);
        assert_eq!(stats.open_holes, 0);
    }

    #[test]
    fn one_closed_hole() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1),
            (0, 0),         (2, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 1);
        assert_eq!(stats.open_holes, 0);
    }

    #[test]
    fn one_closed_hole_against_the_edge() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1),
                    (1, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 1);
        assert_eq!(stats.open_holes, 0);
    }

    #[test]
    fn double_closed_hole() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1), (3, 1),
            (0, 0),                 (3, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 2);
        assert_eq!(stats.open_holes, 0);
    }

    #[test]
    fn two_closed_holes() {
        let stats = Board::new().having_stack_at(&[
            (0, 3), (1, 3), (2, 3),
            (0, 2),         (2, 2),
            (0, 1), (1, 1), (2, 1),
            (0, 0),         (2, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 2);
        assert_eq!(stats.open_holes, 0);
    }

    #[test]
    fn one_open_hole_right() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1),
            (0, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 0);
        assert_eq!(stats.open_holes, 1);
    }

    #[test]
    fn double_open_hole_right() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1),
            (0, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 0);
        assert_eq!(stats.open_holes, 2);
    }

    #[test]
    fn one_open_hole_left() {
        let stats = Board::new().having_stack_at(&[
            (8, 1), (9, 1),
                    (9, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 0);
        assert_eq!(stats.open_holes, 1);
    }

    #[test]
    fn double_open_hole_left() {
        let stats = Board::new().having_stack_at(&[
            (7, 1), (8, 1), (9, 1),
                            (9, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 0);
        assert_eq!(stats.open_holes, 2);
    }

    #[test]
    fn big_open_holes() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1), (3, 1), (4, 1),     (6, 1), (7, 1), (8, 1), (9, 1),
            (0, 0),                                                             (9, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 0);
        assert_eq!(stats.open_holes, 7);
    }

    #[test]
    fn kitchen_sink_of_holes() {
        let stats = Board::new().having_stack_at(&[
                            (2, 3), (3, 3), (4, 3),     (6, 3), (7, 3), (8, 3), (9, 3),
            (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),     (6, 2),         (8, 2),
            (0, 1), (1, 1),         (3, 1), (4, 1),     (6, 1), (7, 1), (8, 1),
            (0, 0),                                     (6, 0),                 (9, 0)
        ]).stack_stats();
        assert_eq!(stats.closed_holes, 6);
        assert_eq!(stats.open_holes, 4);
    }

    #[test]
    fn stack_stats_single_block_change() {
        let stats = Board::new().having_stack_at(&[
            (0, 0)
        ]).stack_stats();
        assert_eq!(stats.max_heights, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(stats.sum_delta_height, 1);
        assert_eq!(stats.max_delta_height, 1);
    }

    #[test]
    fn stack_stats_double_block_change() {
        let stats = Board::new().having_stack_at(&[
            (1, 0)
        ]).stack_stats();
        assert_eq!(stats.max_heights, [0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(stats.sum_delta_height, 2);
        assert_eq!(stats.max_delta_height, 1);
    }

    #[test]
    fn stack_stats_multi_level_change() {
        let stats = Board::new().having_stack_at(&[
            (1, 1),
            (1, 0)
        ]).stack_stats();

        assert_eq!(stats.max_heights, [0, 2, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(stats.sum_delta_height, 4);
        assert_eq!(stats.max_delta_height, 2);
    }


    #[test]
    fn stack_stats_based_on_post_line_clear() {
        let stats = Board::new().having_stack_at(&[
            (0, 1),
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)
        ]).stack_stats();

        assert_eq!(
            stats,
            StackStats {
                open_holes: 0,
                closed_holes: 0,
                max_heights: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                sum_delta_height: 1,
                cleared_lines: 1,
                max_delta_height: 1
            }
        );
    }

    #[test]
    fn stack_stats_kitchen_sink() {
        let stats = Board::new().having_stack_at(&[
                            (2, 3),         (4, 3),     (6, 3),         (8, 3),
            (0, 2), (1, 2), (2, 2), (3, 2), (4, 2),     (6, 2),         (8, 2),
            (0, 1), (1, 1),         (3, 1), (4, 1),     (6, 1), (7, 1), (8, 1),
            (0, 0),                                     (6, 0),                 (9, 0)
        ]).stack_stats();

        assert_eq!(stats.max_heights, [3, 3, 4, 3, 4, 0, 4, 2, 4, 1]);
        assert_eq!(stats.sum_delta_height, 18);
        assert_eq!(stats.max_delta_height, 4);
    }

    #[test]
    fn cleared_lines_1() {
        let stats = Board::new().having_stack_at(&[
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)
        ]).stack_stats();
        assert_eq!(stats.cleared_lines, 1);
    }

    #[test]
    fn cleared_lines_2() {
        let stats = Board::new().having_stack_at(&[
            (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1), (9, 1),
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0),
        ]).stack_stats();
        assert_eq!(stats.cleared_lines, 2);
    }

    #[test]
    fn cleared_lines_3() {
        let stats = Board::new().having_stack_at(&[
            (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2), (7, 2), (8, 2), (9, 2),
            (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1), (9, 1),
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0),
        ]).stack_stats();
        assert_eq!(stats.cleared_lines, 3);
    }

    #[test]
    fn cleared_lines_4() {
        let stats = Board::new().having_stack_at(&[
            (0, 3), (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3), (9, 3),
            (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2), (7, 2), (8, 2), (9, 2),
            (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (8, 1), (9, 1),
            (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0),
        ]).stack_stats();
        assert_eq!(stats.cleared_lines, 4);
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