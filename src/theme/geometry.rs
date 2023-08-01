use crate::game::board::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::tetromino::Minos;
use sdl2::rect::{Point, Rect};

pub const VISIBLE_BUFFER: u32 = 2;
pub const VISIBLE_BOARD_HEIGHT: u32 = BOARD_HEIGHT + VISIBLE_BUFFER;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BoardGeometry {
    block_size: u32,
    visible_height: u32,
    buffer_height: u32,
    height: u32,
    width: u32,
    offset: Point,
}

impl BoardGeometry {
    pub fn new<P: Into<Point>>(block_size: u32, offset: P) -> Self {
        let visible_height = block_size * VISIBLE_BOARD_HEIGHT;
        let buffer_height = block_size * VISIBLE_BUFFER;
        let height = block_size * BOARD_HEIGHT;
        let width = block_size * BOARD_WIDTH;
        Self {
            block_size,
            visible_height,
            buffer_height,
            height,
            width,
            offset: offset.into(),
        }
    }

    fn j_to_y(&self, j: u32) -> u32 {
        self.visible_height - ((j + 1) * self.block_size) + self.offset.y() as u32
    }

    fn i_to_x(&self, i: u32) -> u32 {
        i * self.block_size + self.offset.x() as u32
    }

    pub fn mino_point(&self, i: u32, j: u32) -> Point {
        Point::new(self.i_to_x(i) as i32, self.j_to_y(j) as i32)
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn visible_height(&self) -> u32 {
        self.visible_height
    }

    pub fn buffer_height(&self) -> u32 {
        self.buffer_height
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn line_snip(&self, j: u32) -> Rect {
        Rect::new(
            self.i_to_x(0) as i32,
            self.j_to_y(j) as i32,
            self.width,
            self.block_size,
        )
    }

    pub fn mino_rect(&self, i: u32, j: u32) -> Rect {
        Rect::new(
            self.i_to_x(i) as i32,
            self.j_to_y(j) as i32,
            self.block_size,
            self.block_size,
        )
    }

    pub fn mino_rects(&self, minos: Minos) -> [Rect; 4] {
        minos.map(|mino| self.mino_rect(mino.x as u32, mino.y as u32))
    }

    pub fn game_snip(&self) -> Rect {
        Rect::new(self.offset.x(), self.offset.y(), self.width, self.height)
    }
}
