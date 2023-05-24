use super::block::BlockState;
use super::geometry::Point;
use super::tetromino::{Tetromino, TetrominoShape};
use crate::game::tetromino::Minos;

use std::fmt::{Display, Formatter};

use std::ops::Range;

pub const BOARD_WIDTH: u32 = 10;
pub const BOARD_HEIGHT: u32 = 20;
const BUFFER_HEIGHT: u32 = 20;
const TOTAL_HEIGHT: u32 = BOARD_HEIGHT + BUFFER_HEIGHT;
const TOTAL_BLOCKS: u32 = BOARD_WIDTH * TOTAL_HEIGHT;

pub const MAX_DESTROYED_LINES: usize = 4;
pub type DestroyLines = [Option<u32>; MAX_DESTROYED_LINES];

pub fn compact_destroy_lines(lines: DestroyLines) -> Vec<u32> {
    lines
        .into_iter()
        .filter(|y| y.is_some())
        .flatten()
        .collect()
}

pub struct Board {
    blocks: [BlockState; TOTAL_BLOCKS as usize],
    tetromino: Option<Tetromino>,
}

fn index_at(x: u32, y: u32) -> usize {
    (y * BOARD_WIDTH + x) as usize
}

fn index(point: Point) -> usize {
    index_at(point.x as u32, point.y as u32)
}

fn row_range(y: u32) -> Range<usize> {
    index_at(0, y)..index_at(0, y + 1)
}

fn rows_range(y_from: u32, y_to: u32) -> Range<usize> {
    assert!(y_to >= y_from);
    index_at(0, y_from)..index_at(0, y_to + 1)
}

impl Board {
    pub fn new() -> Self {
        Self {
            blocks: [BlockState::Empty; TOTAL_BLOCKS as usize],
            tetromino: None,
        }
    }

    pub fn row(&self, y: u32) -> &[BlockState] {
        &self.blocks[row_range(y)]
    }

    fn clear_row(&mut self, y: u32) {
        for i in row_range(y) {
            self.blocks[i] = BlockState::Empty;
        }
    }

    pub fn block(&self, point: Point) -> BlockState {
        self.blocks[index(point)]
    }

    fn set_block(&mut self, point: Point, state: BlockState) {
        self.blocks[index(point)] = state;
    }

    pub fn try_spawn_tetromino(&mut self, shape: TetrominoShape) -> bool {
        let tetromino = Tetromino::new(shape);
        if self.tetromino.is_some() {
            panic!("tetromino already spawned")
        }

        let minos = tetromino.minos();
        let mut success = true;
        for (id, p) in minos.into_iter().enumerate() {
            if self.block(p).collides() {
                success = false;
            } else {
                self.set_block(
                    p,
                    BlockState::Tetromino(tetromino.shape(), tetromino.rotation(), id as u32),
                );
            }
        }
        // regardless of success we have set blocks for this tetromino
        self.tetromino = Some(tetromino);

        if success {
            self.render_ghost();
        }

        success
    }

    /// Moves the current tetromino left if possible
    pub fn left(&mut self) -> bool {
        if self.tetromino.is_none() {
            return false;
        }

        let minos = self.tetromino.unwrap().minos();
        for p in minos {
            if p.x == 0 {
                // collided with the wall
                return false;
            }
            let block_left = self.block(p.translate(-1, 0));
            if block_left.collides() {
                // collided with the stack
                return false;
            }
        }

        self.mutate_tetromino(|t| t.translate(-1, 0));
        true
    }

    /// Moves the current tetromino right if possible
    pub fn right(&mut self) -> bool {
        if self.tetromino.is_none() {
            return false;
        }

        let minos = self.tetromino.unwrap().minos();
        for p in minos {
            if p.x == BOARD_WIDTH as i32 - 1 {
                // collided with the wall
                return false;
            }
            let block_right = self.block(p.translate(1, 0));
            if block_right.collides() {
                // collided with the stack
                return false;
            }
        }

        self.mutate_tetromino(|t| t.translate(1, 0));
        true
    }

    fn mutate_tetromino<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Tetromino),
    {
        // remove from board
        for p in self.tetromino.unwrap().minos() {
            self.set_block(p, BlockState::Empty);
        }

        // update tetromino
        let tetromino = self.tetromino.as_mut().unwrap();
        f(tetromino);

        // add back to board
        let tetromino = self.tetromino.unwrap();
        for (id, p) in tetromino.minos().into_iter().enumerate() {
            self.set_block(
                p,
                BlockState::Tetromino(tetromino.shape(), tetromino.rotation(), id as u32),
            );
        }

        self.render_ghost();
    }

    fn render_ghost(&mut self) {
        // todo test
        // remove all existing ghost blocks
        for i in 0..(TOTAL_BLOCKS as usize) {
            if matches!(self.blocks[i], BlockState::Ghost(_, _, _)) {
                self.blocks[i] = BlockState::Empty;
            }
        }

        if self.tetromino.is_none() {
            // no tetromino, no ghost.
            return;
        }

        let tetromino = self.tetromino.unwrap();
        let mut minos = tetromino.minos();

        while !self.minos_collide(minos) {
            #[allow(clippy::needless_range_loop)]
            for i in 0..minos.len() {
                minos[i] = minos[i].translate(0, -1);
            }
        }

        for (id, p) in minos.into_iter().enumerate() {
            if self.block(p) == BlockState::Empty {
                self.set_block(
                    p,
                    BlockState::Ghost(tetromino.shape(), tetromino.rotation(), id as u32),
                )
            }
        }
    }

    pub fn rotate(&mut self, clockwise: bool) -> bool {
        let wall_kick_id = self.try_rotate(clockwise);
        if wall_kick_id.is_none() {
            return false;
        }
        self.mutate_tetromino(|tetromino| tetromino.rotate(clockwise, wall_kick_id.unwrap()));
        true
    }

    fn try_rotate(&self, clockwise: bool) -> Option<usize> {
        self.tetromino?;

        let next_minos = self
            .tetromino
            .unwrap()
            .possible_minos_after_rotation(clockwise);
        for (id, minos) in next_minos.iter().enumerate() {
            let mut success = true;
            for p in minos {
                if p.x < 0 || p.x >= BOARD_WIDTH as i32 || p.y < 0 || p.y >= TOTAL_HEIGHT as i32 {
                    success = false;
                    break;
                }
                if self.block(*p).collides() {
                    success = false;
                    break;
                }
            }
            if success {
                return Some(id);
            }
        }

        None
    }

    pub fn register_lock_placement(&mut self) -> u32 {
        match self.tetromino.as_mut() {
            None => panic!("no tetromino to register lock movement"),
            Some(tetromino) => tetromino.register_lock_placement(),
        }
    }

    pub fn lock_placements(&self) -> u32 {
        match self.tetromino {
            None => panic!("no tetromino to get lock placements"),
            Some(tetromino) => tetromino.lock_placements(),
        }
    }

    pub fn is_collision(&self) -> bool {
        if self.tetromino.is_none() {
            panic!("no tetromino to test for collision")
        }
        self.minos_collide(self.tetromino.unwrap().minos())
    }

    fn minos_collide(&self, minos: Minos) -> bool {
        for p in minos {
            if p.y == 0 {
                // collided with the floor
                return true;
            }
            let block_down = self.block(p.translate(0, -1));
            if block_down.collides() {
                // collided with the stack
                return true;
            }
        }

        false
    }

    /// Steps down the current tetromino
    /// Returns true if successful
    pub fn step_down(&mut self) -> bool {
        if self.tetromino.is_none() {
            panic!("no tetromino to step down")
        }

        if self.is_collision() {
            return false;
        }

        self.mutate_tetromino(|t| t.translate(0, -1));
        true
    }

    pub fn hard_drop(&mut self) -> Option<(u32, Minos)> {
        self.tetromino?;

        let mut minos = self.tetromino.unwrap().minos();
        let mut hard_dropped_rows = 0;

        while !self.minos_collide(minos) {
            hard_dropped_rows += 1;
            #[allow(clippy::needless_range_loop)]
            for i in 0..minos.len() {
                minos[i] = minos[i].translate(0, -1);
            }
        }

        let original_minos = self.tetromino.unwrap().minos();
        if hard_dropped_rows > 0 {
            self.mutate_tetromino(|t| t.translate(0, -hard_dropped_rows));
            Some((hard_dropped_rows as u32, original_minos))
        } else {
            None
        }
    }

    /// Locks the current tetromino
    pub fn lock(&mut self) {
        if self.tetromino.is_none() {
            return;
        }
        let tetromino = self.tetromino.unwrap();
        for (id, p) in tetromino.minos().into_iter().enumerate() {
            self.set_block(
                p,
                BlockState::Stack(tetromino.shape(), tetromino.rotation(), id as u32),
            );
        }
        self.tetromino = None
    }

    pub fn pattern(&self) -> DestroyLines {
        let mut result: DestroyLines = [None; MAX_DESTROYED_LINES];
        let mut index = 0;
        for y in 0..TOTAL_HEIGHT {
            if self.row(y).iter().all(|b| b.collides()) {
                result[index] = Some(y);
                index += 1;
                if index == MAX_DESTROYED_LINES {
                    break;
                }
            }
        }
        result
    }

    pub fn destroy(&mut self, pattern: DestroyLines) -> bool {
        let mut rows = compact_destroy_lines(pattern);
        rows.sort();
        for y in rows.into_iter().rev() {
            self.clear_row(y);
            if y + 1 == TOTAL_HEIGHT {
                // cannot drop down hte top row
                continue;
            }
            // drop down all rows above the line clear
            for y_drop in (y + 1)..TOTAL_HEIGHT {
                self.blocks
                    .copy_within(row_range(y_drop), index_at(0, y_drop - 1));
            }
            // clear top row
            self.clear_row(TOTAL_HEIGHT - 1);
        }
        true
    }

    pub fn hold(&mut self) -> Option<TetrominoShape> {
        self.tetromino?;

        let tetromino = self.tetromino.unwrap();

        // remove from board
        for p in tetromino.minos() {
            self.set_block(p, BlockState::Empty);
        }
        self.tetromino = None;
        self.render_ghost();

        Some(tetromino.shape())
    }

    pub fn send_garbage(&mut self, hole: u32) {
        // bump up all rows above the garbage
        // not checking if we overflow the buffer since a game over will result on the next update anyway
        for y in (0..(TOTAL_HEIGHT - 1)).rev() {
            self.blocks.copy_within(row_range(y), index_at(0, y + 1));
        }

        let skip_index = index_at(hole, 0);
        for i in row_range(0) {
            if i == skip_index {
                self.blocks[i] = BlockState::Empty;
            } else {
                self.blocks[i] = BlockState::Garbage;
            }
        }
    }

    pub fn is_tetromino_above_skyline(&self) -> bool {
        if self.tetromino.is_none() {
            return false;
        }

        self.tetromino
            .unwrap()
            .minos()
            .into_iter()
            .all(|mino| mino.y >= BOARD_HEIGHT as i32)
    }

    pub fn is_stack_above_skyline(&self) -> bool {
        for block in &self.blocks[rows_range(BOARD_HEIGHT, TOTAL_HEIGHT - 1)] {
            if block.collides() {
                return true;
            }
        }
        false
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "   {}", "-".repeat(BOARD_WIDTH as usize))?;

        for y in (0..TOTAL_HEIGHT).rev() {
            if y == BUFFER_HEIGHT - 1 {
                writeln!(f, "   {}", "-".repeat(BOARD_WIDTH as usize))?;
            }
            write!(f, "{:02}|", y)?;

            for x in 0..BOARD_WIDTH {
                let block = self.block(Point::new(x as i32, y as i32));
                match block {
                    BlockState::Tetromino(_, _, _) => write!(f, "T")?,
                    BlockState::Stack(_, _, _) => write!(f, "S")?,
                    BlockState::Garbage => write!(f, "G")?,
                    _ => write!(f, " ")?,
                }
            }

            writeln!(f, "|")?;
        }
        write!(f, "   {}", "-".repeat(BOARD_WIDTH as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::geometry::Rotation;

    const NO_DESTROYED_LINES: DestroyLines = [None; MAX_DESTROYED_LINES];

    macro_rules! spawn_tests {
        ($($name:ident: $shape:expr => $points:expr),*) => {
            $(
                #[test]
                fn $name() {
                    let mut board = Board::new();
                    can_spawn_tetromino(&mut board, $shape);
                    should_have_tetromino_at(&board, &$points);
                    should_have_n_tetromino_blocks(&board, 4);
                }
            )*
        };
    }

    spawn_tests! {
        spawn_i: TetrominoShape::I => [Point::new(3, 20), Point::new(4, 20), Point::new(5, 20), Point::new(6, 20)],
        spawn_o: TetrominoShape::O => [Point::new(4, 20), Point::new(5, 20), Point::new(4, 21), Point::new(5, 21)],
        spawn_t: TetrominoShape::T => [Point::new(3, 20), Point::new(4, 20), Point::new(5, 20), Point::new(4, 21)],
        spawn_s: TetrominoShape::S => [Point::new(3, 20), Point::new(4, 20), Point::new(4, 21), Point::new(5, 21)],
        spawn_z: TetrominoShape::Z => [Point::new(4, 20), Point::new(5, 20), Point::new(3, 21), Point::new(4, 21)],
        spawn_j: TetrominoShape::J => [Point::new(3, 20), Point::new(4, 20), Point::new(5, 20), Point::new(3, 21)],
        spawn_l: TetrominoShape::L => [Point::new(3, 20), Point::new(4, 20), Point::new(5, 20), Point::new(5, 21)]
    }

    fn can_spawn_tetromino(board: &mut Board, shape: TetrominoShape) {
        assert!(board.try_spawn_tetromino(shape));
    }

    fn should_have_tetromino_at(board: &Board, points: &[Point]) {
        for p in points.iter() {
            assert!(
                matches!(board.block(*p), BlockState::Tetromino(_, _, _)),
                "{}",
                board
            );
        }
    }

    fn should_only_have_stack_at(board: &Board, points: &[Point]) {
        for p in points.iter() {
            assert!(
                matches!(board.block(*p), BlockState::Stack(_, _, _)),
                "{}",
                board
            );
        }
        let observed = board
            .blocks
            .iter()
            .filter(|b| matches!(b, BlockState::Stack(_, _, _)))
            .count();
        assert_eq!(observed, points.len(), "{}", board);
    }

    fn should_have_n_tetromino_blocks(board: &Board, n: u32) {
        let observed = board
            .blocks
            .iter()
            .filter(|b| matches!(b, BlockState::Tetromino(_, _, _)))
            .count() as u32;
        assert_eq!(observed, n, "{}", board);
    }

    fn should_have_empty_board(board: &Board) {
        let observed = board
            .blocks
            .into_iter()
            .filter(|b| b != &BlockState::Empty)
            .count() as u32;
        assert_eq!(observed, 0, "{}", board);
    }

    #[test]
    fn steps_down() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::J);
        assert!(board.step_down());
        should_have_tetromino_at(
            &board,
            &[
                Point::new(3, 19),
                Point::new(4, 19),
                Point::new(5, 19),
                Point::new(3, 20),
            ],
        );
    }

    #[test]
    fn steps_down_to_floor() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::L);
        should_collide_after_step_downs(&mut board, BOARD_HEIGHT);

        should_have_tetromino_at(
            &board,
            &[
                Point::new(3, 0),
                Point::new(4, 0),
                Point::new(5, 0),
                Point::new(5, 1),
            ],
        );
    }

    #[test]
    fn locks() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::L);
        having_step_downs(&mut board, BOARD_HEIGHT);
        board.lock();
        should_only_have_stack_at(
            &board,
            &[
                Point::new(3, 0),
                Point::new(4, 0),
                Point::new(5, 0),
                Point::new(5, 1),
            ],
        );
    }

    #[test]
    fn steps_down_to_stack() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        can_spawn_tetromino(&mut board, TetrominoShape::I);
        should_collide_after_step_downs(&mut board, BOARD_HEIGHT - 1);

        should_have_tetromino_at(
            &board,
            &[
                Point::new(3, 1),
                Point::new(4, 1),
                Point::new(5, 1),
                Point::new(6, 1),
            ],
        );
    }

    #[test]
    fn moves_left() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        having_step_downs(&mut board, 2); // into board
        assert!(board.left(), "{}", board);
        should_have_tetromino_at(
            &board,
            &[
                Point::new(3, 18),
                Point::new(4, 18),
                Point::new(3, 19),
                Point::new(4, 19),
            ],
        );
    }

    #[test]
    fn cannot_move_left_through_wall() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        for i in 0..4 {
            assert!(board.left(), "{}: {}", i, board)
        }
        assert!(!board.left(), "{}", board);
        should_have_tetromino_at(
            &board,
            &[
                Point::new(0, 20),
                Point::new(1, 20),
                Point::new(0, 21),
                Point::new(1, 21),
            ],
        );
    }

    #[test]
    fn cannot_move_left_through_stack() {
        let mut board = Board::new();
        having_stack_col(&mut board, 0);
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        having_step_downs(&mut board, 1); // peeking into board i.e. will only collide with a single mino
        for i in 0..3 {
            assert!(board.left(), "{}: {}", i, board)
        }
        assert!(!board.left(), "{}", board);
        should_have_tetromino_at(
            &board,
            &[
                Point::new(1, 19),
                Point::new(2, 19),
                Point::new(1, 20),
                Point::new(2, 20),
            ],
        );
    }

    #[test]
    fn moves_right() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        having_step_downs(&mut board, 2); // into board
        assert!(board.right(), "{}", board);
        should_have_tetromino_at(
            &board,
            &[
                Point::new(5, 18),
                Point::new(6, 18),
                Point::new(5, 19),
                Point::new(6, 19),
            ],
        );
    }

    #[test]
    fn cannot_move_right_through_wall() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        for i in 0..4 {
            assert!(board.right(), "{}: {}", i, board)
        }
        assert!(!board.right(), "{}", board);
        should_have_tetromino_at(
            &board,
            &[
                Point::new(8, 20),
                Point::new(9, 20),
                Point::new(8, 21),
                Point::new(9, 21),
            ],
        );
    }

    #[test]
    fn cannot_move_right_through_stack() {
        let mut board = Board::new();
        having_stack_col(&mut board, 9);
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        having_step_downs(&mut board, 1); // peeking into board i.e. will only collide with a single mino
        for i in 0..3 {
            assert!(board.right(), "{}: {}", i, board)
        }
        assert!(!board.right(), "{}", board);
        should_have_tetromino_at(
            &board,
            &[
                Point::new(7, 19),
                Point::new(8, 19),
                Point::new(7, 20),
                Point::new(8, 20),
            ],
        );
    }

    fn having_step_downs(board: &mut Board, n: u32) {
        for _i in 0..n {
            board.step_down();
        }
    }

    fn should_collide_after_step_downs(board: &mut Board, n: u32) {
        for i in 0..n {
            assert!(board.step_down(), "{}: {}", i, board);
        }
        assert!(board.is_collision(), "{}", board);
    }

    fn having_stack_at(board: &mut Board, x: u32, y: u32) {
        board.set_block(
            Point::new(x as i32, y as i32),
            BlockState::Stack(TetrominoShape::L, Rotation::North, 0),
        );
    }

    fn having_stack_row(board: &mut Board, y: u32) {
        for x in 0..BOARD_WIDTH {
            having_stack_at(board, x, y);
        }
    }

    fn having_stack_col(board: &mut Board, x: u32) {
        for y in 0..BOARD_HEIGHT {
            having_stack_at(board, x, y);
        }
    }

    #[test]
    fn rotating_o_does_nothing() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        assert!(board.rotate(true));
        should_have_tetromino_at(
            &mut board,
            &[
                Point::new(4, 20),
                Point::new(5, 20),
                Point::new(4, 21),
                Point::new(5, 21),
            ],
        );
        should_have_n_tetromino_blocks(&board, 4);
    }

    #[test]
    fn rotating_l() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::L);
        assert!(board.rotate(true));
        should_have_tetromino_at(
            &mut board,
            &[
                Point::new(4, 21),
                Point::new(4, 20),
                Point::new(4, 19),
                Point::new(5, 19),
            ],
        );
        should_have_n_tetromino_blocks(&board, 4);
    }

    #[test]
    fn rotating_i_off_floor() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::I);
        having_step_downs(&mut board, BOARD_HEIGHT);
        assert!(board.rotate(true));
        should_have_tetromino_at(
            &mut board,
            &[
                Point::new(6, 0),
                Point::new(6, 1),
                Point::new(6, 2),
                Point::new(6, 3),
            ],
        );
        should_have_n_tetromino_blocks(&board, 4);
    }

    #[test]
    fn no_patterns_on_empty_board() {
        let board = Board::new();
        let observed = board.pattern();
        assert_eq!(observed, NO_DESTROYED_LINES);
    }

    #[test]
    fn no_patterns() {
        let mut board = Board::new();
        having_stack_at(&mut board, 0, 0);
        let observed = board.pattern();
        assert_eq!(observed, NO_DESTROYED_LINES);
    }

    #[test]
    fn single_line_pattern() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        having_stack_at(&mut board, 0, 1);
        let observed = board.pattern();
        assert_eq!(observed, [Some(0), None, None, None]);
    }

    #[test]
    fn double_line_pattern() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        having_stack_row(&mut board, 1);
        having_stack_at(&mut board, 0, 2);
        let observed = board.pattern();
        assert_eq!(observed, [Some(0), Some(1), None, None]);
    }

    #[test]
    fn triple_line_pattern() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        having_stack_row(&mut board, 1);
        having_stack_row(&mut board, 2);
        having_stack_at(&mut board, 0, 3);
        let observed = board.pattern();
        assert_eq!(observed, [Some(0), Some(1), Some(2), None]);
    }

    #[test]
    fn triple_line_pattern_with_separation() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        having_stack_row(&mut board, 1);
        having_stack_row(&mut board, 3);
        having_stack_at(&mut board, 0, 2);
        let observed = board.pattern();
        assert_eq!(observed, [Some(0), Some(1), Some(3), None]);
    }

    #[test]
    fn tetris_line_pattern() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        having_stack_row(&mut board, 1);
        having_stack_row(&mut board, 2);
        having_stack_row(&mut board, 3);
        having_stack_at(&mut board, 0, 4);
        let observed = board.pattern();
        assert_eq!(observed, [Some(0), Some(1), Some(2), Some(3)]);
    }

    #[test]
    fn destroy_single_line() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        having_stack_at(&mut board, 0, 1);
        having_stack_at(&mut board, 0, 2);
        board.destroy([Some(0), None, None, None]);
        should_only_have_stack_at(&board, &[Point::new(0, 0), Point::new(0, 1)]);
    }

    #[test]
    fn hard_drops_onto_floor() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        assert_eq!(board.hard_drop().map(|(y, _)| y), Some(20));
        should_have_tetromino_at(
            &board,
            &[
                Point::new(4, 0),
                Point::new(5, 0),
                Point::new(4, 1),
                Point::new(5, 1),
            ],
        );
    }

    #[test]
    fn hard_drops_onto_stack() {
        let mut board = Board::new();
        having_stack_row(&mut board, 0);
        can_spawn_tetromino(&mut board, TetrominoShape::O);
        assert_eq!(board.hard_drop().map(|(y, _)| y), Some(19));
        should_have_tetromino_at(
            &board,
            &[
                Point::new(4, 1),
                Point::new(5, 1),
                Point::new(4, 2),
                Point::new(5, 2),
            ],
        );
    }

    #[test]
    fn holds_tetromino() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::I);
        assert_eq!(board.hold(), Some(TetrominoShape::I));
        should_have_empty_board(&board);
        assert!(board.tetromino.is_none());
    }

    #[test]
    fn holds_nothing() {
        let mut board = Board::new();
        assert_eq!(board.hold(), None);
        should_have_empty_board(&board);
        assert!(board.tetromino.is_none());
    }

    #[test]
    fn sends_garbage() {
        let mut board = Board::new();
        having_stack_at(&mut board, 4, 1);
        having_stack_at(&mut board, 5, 0);
        board.send_garbage(5);
        should_only_have_stack_at(&board, &[Point::new(4, 2), Point::new(5, 1)]);

        for x in 0..BOARD_WIDTH {
            let block = board.blocks[index_at(x, 0)];
            if x == 5 {
                assert_eq!(block, BlockState::Empty);
            } else {
                assert_eq!(block, BlockState::Garbage);
            }
        }
    }

    #[test]
    fn is_tetromino_above_skyline() {
        let mut board = Board::new();
        can_spawn_tetromino(&mut board, TetrominoShape::I);
        assert!(board.is_tetromino_above_skyline(), "{}", board);
        assert!(board.step_down());
        assert!(!board.is_tetromino_above_skyline(), "{}", board);
    }

    #[test]
    fn is_stack_above_skyline() {
        let mut board = Board::new();
        assert!(!board.is_stack_above_skyline(), "{}", board);
        having_stack_row(&mut board, BOARD_HEIGHT);
        assert!(board.is_stack_above_skyline(), "{}", board);
    }
}
