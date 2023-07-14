use sdl2::mixer::Music;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::{GameOverAnimate, GameOverAnimationType};
use crate::animation::TextureAnimate;
use crate::event::GameEvent;
use crate::font::FontType;
use crate::game::block::BlockState;
use crate::game::Game;
use crate::game::geometry::Rotation;
use crate::game::tetromino::{Minos, TetrominoShape};
use crate::theme::geometry::{BoardGeometry, VISIBLE_BOARD_HEIGHT};
use crate::theme::sprite_sheet::{TetrominoSpriteSheet, TetrominoSpriteSheetMeta};
use crate::theme::Theme;

const MIN_VERTICAL_BUFFER_PCT: f64 = 0.1;
const BOARD_BORDER_PCT_OF_BLOCK: f64 = 0.5;
const BOARD_BOARDER_SHADOW: u8 = 0x99;
const TETROMINO_PCT_OF_BLOCK: f64 = 1.5;
// 3 blocks is good as most are 3 blocks wide, then we scale I down and O up to 3.
const TETROMINO_PREFERRED_SCALE: f64 = TETROMINO_PCT_OF_BLOCK / 3.0;
const VERTICAL_GUTTER_PCT_OF_BLOCK: f64 = 0.2;
const VISIBLE_PEEK: usize = 5;

pub struct VectorTheme<'a> {
    music: Music<'a>,
    sprite_sheet: TetrominoSpriteSheet<'a>,
    geometry: BoardGeometry,
    borders: Vec<(Rect, u8)>,
    border_weight: u32,
    background_size: (u32, u32),
    board_snip: Rect,
    hold_snip: Rect,
    peek_snips: [Rect; VISIBLE_PEEK],

}

fn block(row: i32, col: i32) -> Point {
    Point::new(4 + 56 * col, 4 + 56 * row)
}

fn mino(col: i32) -> (Point, Point) {
    // (normal block, stack block)
    (block(0, col), block(1, col))
}

impl<'a> VectorTheme<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf: &Sdl2TtfContext,
        window_height: u32
    ) -> Result<Self, String> {
        let block_size = (window_height as f64 - (2.0 * window_height as f64 * MIN_VERTICAL_BUFFER_PCT)) / VISIBLE_BOARD_HEIGHT as f64;
        let border_weight = (block_size * BOARD_BORDER_PCT_OF_BLOCK).round() as u32;
        let vertical_gutter = (VERTICAL_GUTTER_PCT_OF_BLOCK * block_size).round() as u32;
        let tetromino_size = (TETROMINO_PCT_OF_BLOCK * block_size).round() as u32;
        let block_size = block_size.round() as u32;

        let font = FontType::Normal.load(ttf, block_size / 2)?;
        // todo render digits

        let geometry = BoardGeometry::new(block_size, (border_weight as i32, 0));

        let board_snip = Rect::new(
            (tetromino_size + vertical_gutter) as i32, 0,
            geometry.width() + 2 * border_weight,
            geometry.visible_height() + border_weight
        );

        let background_width = board_snip.width() + 2 * (tetromino_size + vertical_gutter);
        let background_height = board_snip.height();

        let hold_snip = Rect::new(0, geometry.buffer_height() as i32, tetromino_size, tetromino_size);
        let peek_snips = (0..VISIBLE_PEEK)
            .map(|i| Rect::new(
                board_snip.x() + board_snip.width() as i32 + vertical_gutter as i32,
                 geometry.buffer_height() as i32 + i as i32 * (vertical_gutter + tetromino_size) as i32,
                tetromino_size,
                tetromino_size
            ))
            .collect::<Vec<Rect>>()
            .try_into()
            .unwrap();

        let sprite_sheet_meta = TetrominoSpriteSheetMeta::new(
            "resource/modern/sprites.png",
            48,
            mino(6),
            mino(1),
            mino(3),
            mino(7),
            mino(2),
            mino(4),
            mino(5),
            block(0, 0),
            0x50
        );
        let mut borders = vec![];

        let step = BOARD_BOARDER_SHADOW / border_weight as u8;
        for i in 0..border_weight {
            let j = border_weight - i - 1;
            let alpha = if j > 0 { BOARD_BOARDER_SHADOW - j as u8 * step } else { 0xff };
            let rect = Rect::new(
                 i as i32, geometry.buffer_height() as i32,
                geometry.width() - 2 * i + 2 * border_weight, geometry.height() - i + border_weight
            );
            borders.push((rect, alpha))
        }

        Ok(
            Self {
                music: Music::from_file("resource/gb/music.ogg")?, // TODO
                sprite_sheet: TetrominoSpriteSheet::new(canvas, texture_creator, sprite_sheet_meta, block_size)?,
                geometry,
                borders,
                border_weight,
                background_size: (background_width, background_height),
                board_snip,
                hold_snip,
                peek_snips
            }
        )
    }
}

impl<'a> Theme for VectorTheme<'a> {
    fn geometry(&self) -> &BoardGeometry {
        &self.geometry
    }

    fn background_color(&self) -> Color {
        Color::BLACK
    }

    fn background_size(&self) -> (u32, u32) {
        self.background_size
    }

    fn board_snip(&self) -> Rect {
        self.board_snip
    }

    fn draw_background(&mut self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        let metrics = game.metrics();
        if let Some(hold_shape) = metrics.hold {
            self.sprite_sheet.draw_tetromino_fill(canvas, hold_shape, self.hold_snip, TETROMINO_PREFERRED_SCALE)?;
        }

        for (peek_shape, peek_rect) in metrics.queue.iter().zip(self.peek_snips) {
            self.sprite_sheet.draw_tetromino_fill(canvas, *peek_shape, peek_rect, TETROMINO_PREFERRED_SCALE)?;
        }

        Ok(())
    }

    fn draw_board(&mut self, canvas: &mut WindowCanvas, game: &Game, animate_lines: Vec<(u32, TextureAnimate)>, animate_game_over: Option<GameOverAnimate>) -> Result<(), String> {
        for (r, c) in self.borders.iter().copied() {
            canvas.set_draw_color(Color::RGBA(c, c, c, c));
            canvas.draw_rect(r)?;
        }

        // clear the board
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.fill_rect(Rect::new(self.border_weight as i32, 0, self.geometry.width(), self.geometry.visible_height()))?;

        canvas.set_draw_color(Color::WHITE);

        for j in 0..VISIBLE_BOARD_HEIGHT {
            for (i, block) in game.row(j).iter().enumerate() {
                let point = self.geometry.mino_point(i as u32, j);
                match block {
                    BlockState::Empty => {}
                    BlockState::Tetromino(shape, rotation, mino_id) => {
                        self.sprite_sheet.draw_mino(canvas, *shape, *rotation, *mino_id, point)?;
                    }
                    BlockState::Ghost(shape, rotation, mino_id) => {
                        self.sprite_sheet.draw_perimeter(canvas, *shape, *rotation, *mino_id, point)?;
                    }
                    BlockState::Stack(shape, rotation, mino_id) => {
                        self.sprite_sheet.draw_stack(canvas, *shape, *rotation, *mino_id, point)?;
                    }
                    BlockState::Garbage => {
                        self.sprite_sheet.draw_garbage(canvas, point)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn destroy_animation_type(&self) -> DestroyAnimationType {
        DestroyAnimationType::Particles { color: Color::WHITE }
    }

    fn game_over_animation_type(&self) -> GameOverAnimationType {
        GameOverAnimationType::CurtainUp
    }

    fn music(&self) -> &Music {
        &self.music
    }

    fn receive_event(&mut self, event: GameEvent) -> Result<(), String> {
        // TODO
        Ok(())
    }
}