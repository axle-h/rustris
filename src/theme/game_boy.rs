use std::cmp::min;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Scancode::O;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};
use crate::game::block::BlockState;
use crate::game::board::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::{Game, GameMetrics};
use crate::game::geometry::Rotation;
use crate::game::tetromino::TetrominoShape;
use super::Theme;

const REFERENCE_I: Color = Color::CYAN;
const REFERENCE_J: Color = Color::BLUE;
const REFERENCE_L: Color = Color::RGBA(0xff, 0x7f, 0x00, 0xff);
const REFERENCE_O: Color = Color::YELLOW;
const REFERENCE_S: Color = Color::GREEN;
const REFERENCE_T: Color = Color::RGBA(0x80, 0x00, 0x80, 0xff);
const REFERENCE_Z: Color = Color::RED;

const ALPHA_PIXELS: u32 = 6;
const ALPHA_GUTTER_PIXELS: u32 = 2;
const BLOCK_PIXELS: u32 = 8;
const GB_VISIBLE_BUFFER: u32 = 2;
const GB_BOARD_HEIGHT: u32 = BOARD_HEIGHT + GB_VISIBLE_BUFFER;
const BOARD_WIDTH_PIXELS: u32 = BOARD_WIDTH * BLOCK_PIXELS;
const BOARD_HEIGHT_PIXELS: u32 = GB_BOARD_HEIGHT * BLOCK_PIXELS;
const GB_BUFFER_PIXELS: u32 = GB_VISIBLE_BUFFER * BLOCK_PIXELS;
const GB_BG_WIDTH: u32 = 200; // TODO read from the texture
const GB_BG_HEIGHT: u32 = 167; // TODO read from the texture
const MAX_SCORE: u32 = 999999;
const MAX_LEVEL: u32 = 9999;
const MAX_LINES: u32 = 9999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameBoyPalette {
    GameBoyLight,
    GreenSoup
}

impl GameBoyPalette {
    fn sprite_sheet_file(&self) -> &str {
        match self {
            GameBoyPalette::GameBoyLight => "./sprites/game-boy.png",
            GameBoyPalette::GreenSoup => "./sprites/game-boy-gs.png"
        }
    }

    fn background_file(&self) -> &str {
        match self {
            GameBoyPalette::GameBoyLight => "./sprites/game-boy-bg-2.png",
            GameBoyPalette::GreenSoup => "./sprites/game-boy-bg-gs.png"
        }
    }

    fn black(&self) -> Color {
        match self {
            GameBoyPalette::GameBoyLight => Color::BLACK,
            GameBoyPalette::GreenSoup => Color::RGB(0x00, 0x3f, 0x00)
        }
    }

    fn dark_grey(&self) -> Color {
        match self {
            GameBoyPalette::GameBoyLight => Color::RGB(0x55, 0x55, 0x55),
            GameBoyPalette::GreenSoup => Color::RGB(0x2e, 0x73, 0x20)
        }
    }

    fn light_grey(&self) -> Color {
        match self {
            GameBoyPalette::GameBoyLight => Color::RGB(0xaa, 0xaa, 0xaa),
            GameBoyPalette::GreenSoup => Color::RGB(0x8c, 0xbf, 0x0a)
        }
    }

    fn white(&self) -> Color {
        match self {
            GameBoyPalette::GameBoyLight => Color::WHITE,
            GameBoyPalette::GreenSoup => Color::RGB(0xa0, 0xcf, 0x0a)
        }
    }
}

fn score_digit_rect(index: usize) -> Rect {
    Rect::new(40 - index as i32 * 8, 25 + GB_BUFFER_PIXELS as i32, ALPHA_PIXELS, ALPHA_PIXELS)
}

fn level_digit_rect(index: usize) -> Rect {
    Rect::new(33 - index as i32 * 8, 52 + GB_BUFFER_PIXELS as i32, ALPHA_PIXELS, ALPHA_PIXELS)
}

fn lines_digit_rect(index: usize) -> Rect {
    Rect::new(33 - index as i32 * 8, 78 + GB_BUFFER_PIXELS as i32, ALPHA_PIXELS, ALPHA_PIXELS)
}

fn block_snip(x: i32, y: i32) -> Rect {
    Rect::new(x, y, BLOCK_PIXELS, BLOCK_PIXELS)
}

fn char_snip(row: i32, col: i32) -> Rect {
    // characters are in row x col with 8 pixels between columns and 7 pixels between rows
    Rect::new(1 + col * 8, 45 + row * 7, ALPHA_PIXELS, ALPHA_PIXELS)
}

fn string_width(length: u32) -> u32 {
    ALPHA_PIXELS * length + ALPHA_GUTTER_PIXELS * (length - 1)
}

pub struct GameBoyTheme<'a> {
    players: u32,
    palette: GameBoyPalette,
    sprites: Texture<'a>,
    sprites_ghost: Texture<'a>,
    background: Texture<'a>,
    paused: Texture<'a>,

    // TODO all this lot could be const
    i: Rect,
    i_blocks: [Rect; 4],
    j: Rect,
    j_block: Rect,
    l: Rect,
    l_block: Rect,
    o: Rect,
    o_block: Rect,
    s: Rect,
    s_block: Rect,
    t: Rect,
    t_block: Rect,
    z: Rect,
    z_block: Rect,
    alpha_snips: [Rect; 26],
    num_snips: [Rect; 10],
    peek: Rect,
    hold: Rect
}

impl<'a> GameBoyTheme<'a> {
    pub fn new(players: u32, canvas: &mut WindowCanvas, texture_creator: &'a TextureCreator<WindowContext>, palette: GameBoyPalette) -> Result<Self, String> {
        if players == 0 || players > 2 {
            return Err("maximum players is 2".to_string());
        }

        let sprites = texture_creator.load_texture(palette.sprite_sheet_file())?;
        let sprites_query = sprites.query();

        // ghost sprites are just lightened sprites
        let mut sprites_ghost = texture_creator
            .create_texture_target(None, sprites_query.width, sprites_query.height)
            .map_err(|e| e.to_string())?;
        sprites_ghost.set_alpha_mod(0x60);
        canvas.with_texture_canvas(&mut sprites_ghost, |texture_canvas| {
            texture_canvas.copy(&sprites, None, None).unwrap();
        }).map_err(|e| e.to_string())?;

        let alpha_snips = [
            char_snip(0, 0), char_snip(0, 1), char_snip(0, 2),
            char_snip(0, 3), char_snip(0, 4), char_snip(0, 5),
            char_snip(0, 6), char_snip(0, 7), char_snip(0, 8),
            char_snip(0, 9),
            char_snip(1, 0), char_snip(1, 1), char_snip(1, 2),
            char_snip(1, 3), char_snip(1, 4), char_snip(1, 5),
            char_snip(1, 6), char_snip(1, 7), char_snip(1, 8),
            char_snip(1, 9),
            char_snip(2, 0), char_snip(2, 1), char_snip(2, 2),
            char_snip(2, 3), char_snip(2, 4), char_snip(2, 5)
        ];

        let mut paused = texture_creator.create_texture_target(None, string_width(6), ALPHA_PIXELS)
            .map_err(|e| e.to_string())?;
        paused.set_blend_mode(BlendMode::Blend);

        let mut theme = Self {
            players,
            palette,
            sprites,
            sprites_ghost,
            background: texture_creator.load_texture(palette.background_file())?,
            paused,
            i: Rect::new(1, 35, BLOCK_PIXELS * 4, BLOCK_PIXELS),
            i_blocks: [
                block_snip(1, 35), block_snip(9, 35),
                block_snip(17, 35), block_snip(25, 35)
            ],
            j: Rect::new(51, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            j_block: block_snip(51, 26),
            l: Rect::new(26, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            l_block: block_snip(26, 26),
            o: Rect::new(1, 1, BLOCK_PIXELS * 2, BLOCK_PIXELS * 2),
            o_block: block_snip(1, 1),
            s: Rect::new(43, 1, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            s_block: block_snip(51, 1),
            t: Rect::new(1, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            t_block: block_snip(1, 26),
            z: Rect::new(18, 1, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            z_block: block_snip(18, 1),
            alpha_snips,
            num_snips: [
                char_snip(3, 0), char_snip(3, 1), char_snip(3, 2),
                char_snip(3, 3), char_snip(3, 4), char_snip(3, 5),
                char_snip(3, 6), char_snip(3, 7), char_snip(3, 8),
                char_snip(3, 9),
            ],
            peek: Rect::new(162, 11 + GB_BUFFER_PIXELS as i32, 32, 32),
            hold: Rect::new(12, 101 + GB_BUFFER_PIXELS as i32, 32, 32)
        };

        let paused_rects = theme.string_rects("PAUSED", 0, 0);
        canvas.with_texture_canvas(&mut theme.paused, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
            texture_canvas.clear();
            for (src, dst) in paused_rects {
                texture_canvas.copy(&theme.sprites, src, dst).unwrap();
            }
        }).map_err(|e| e.to_string())?;

        return Ok(theme);

    }

    fn draw_tetromino(&self, canvas: &mut Canvas<Window>, sprites: &Texture, rect: &Rect, shape: TetrominoShape) -> Result<(), String> {
        let sprite_snip = match shape {
            TetrominoShape::I => self.i,
            TetrominoShape::O => self.o,
            TetrominoShape::T => self.t,
            TetrominoShape::S => self.s,
            TetrominoShape::Z => self.z,
            TetrominoShape::J => self.j,
            TetrominoShape::L => self.l
        };
        let mut dest_rect = Rect::new(0, 0, sprite_snip.width(), sprite_snip.height());
        dest_rect.center_on(rect.center());
        canvas.copy(sprites, sprite_snip, dest_rect)?;
        Ok(())
    }

    fn draw_mino(&self, canvas: &mut Canvas<Window>, sprites: &Texture, x: u32, y: u32, shape: TetrominoShape, rotation: Rotation, mino_id: u32) -> Result<(), String> {
        let dest_rect = Rect::new((x * BLOCK_PIXELS) as i32, (y * BLOCK_PIXELS) as i32, BLOCK_PIXELS as u32, BLOCK_PIXELS as u32);
        let sprite_snip = match shape {
            TetrominoShape::I => {
                // only the I tetromino spite uses rotation on the gb
                canvas.copy_ex(sprites, self.i_blocks[mino_id as usize], dest_rect, -rotation.angle(), None, false, false)?;
                return Ok(());
            },
            TetrominoShape::O => self.o_block,
            TetrominoShape::T => self.t_block,
            TetrominoShape::S => self.s_block,
            TetrominoShape::Z => self.z_block,
            TetrominoShape::J => self.j_block,
            TetrominoShape::L => self.l_block
        };
        canvas.copy(sprites, sprite_snip, dest_rect)?;
        Ok(())
    }

    fn digit_snip(&self, digit: char) -> Rect {
        self.num_snips[(digit as usize) - '0' as usize]
    }

    fn alpha_snip(&self, alpha: char) -> Rect {
        self.alpha_snips[(alpha as usize) - 'A' as usize]
    }

    fn string_rects(&self, s: &str, offset_x: i32, offset_y: i32) -> Vec<(Rect, Rect)> {
        let mut result = vec![];
        let mut x = offset_x;
        for c in s.chars() {
            result.push((self.alpha_snip(c), Rect::new(x, offset_y, ALPHA_PIXELS, ALPHA_PIXELS)));
            x += (ALPHA_PIXELS + ALPHA_GUTTER_PIXELS) as i32;
        }
        return result;
    }
}

impl<'a> Theme for GameBoyTheme<'a> {
    fn background_size(&self) -> (u32, u32) {
        (GB_BG_WIDTH * self.players, GB_BG_HEIGHT + GB_BUFFER_PIXELS)
    }

    fn board_snip(&self, player: u32) -> Rect {
        if player == 0 || player > self.players {
            panic!("bad player id")
        }
        Rect::new(63 + (GB_BG_WIDTH * (player - 1)) as i32, 0, BOARD_WIDTH_PIXELS, BOARD_HEIGHT_PIXELS)
    }

    fn draw_background(&self, canvas: &mut Canvas<Window>, games: &Vec<GameMetrics>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        for game in games.iter() {
            let offset_x = (game.player - 1) * GB_BG_WIDTH;
            canvas.copy(&self.background, None, Rect::new(offset_x as i32, GB_BUFFER_PIXELS as i32,GB_BG_WIDTH, GB_BG_HEIGHT))?;

            let score = format!("{}", min(game.score, MAX_SCORE));
            for (index, char) in score.chars().rev().enumerate() {
                canvas.copy(&self.sprites, self.digit_snip(char), score_digit_rect(index))?;
            }

            let level = format!("{}", min(game.level, MAX_LEVEL));
            for (index, char) in level.chars().rev().enumerate() {
                canvas.copy(&self.sprites, self.digit_snip(char), level_digit_rect(index))?;
            }

            let lines = format!("{}", min(game.lines, MAX_LINES));
            for (index, char) in lines.chars().rev().enumerate() {
                canvas.copy(&self.sprites, self.digit_snip(char), lines_digit_rect(index))?;
            }

            self.draw_tetromino(canvas, &self.sprites, &self.peek, game.queue[0])?;

            match game.hold {
                None => {}
                Some(shape) => {
                    self.draw_tetromino(canvas, &self.sprites, &self.hold, shape)?;
                }
            }
        }
        Ok(())
    }

    fn draw_board(&self, canvas: &mut Canvas<Window>, game: &Game) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.fill_rect(Rect::new(0, (BOARD_HEIGHT * BLOCK_PIXELS) as i32, BOARD_WIDTH_PIXELS, GB_BUFFER_PIXELS))?;
        canvas.set_draw_color(self.palette.white());
        canvas.fill_rect(Rect::new(0, 0, BOARD_WIDTH_PIXELS, BOARD_HEIGHT * BLOCK_PIXELS))?;
        for y in 0..GB_BOARD_HEIGHT {
            for (x, block) in game.row(y).iter().enumerate() {
                match block {
                    BlockState::Empty => {}
                    BlockState::Tetromino(shape, rotation, mino_id) => {
                        self.draw_mino(canvas, &self.sprites, x as u32, y, *shape, *rotation, *mino_id)?;
                    }
                    BlockState::Ghost(shape, rotation, mino_id) => {
                        self.draw_mino(canvas, &self.sprites_ghost, x as u32, y, *shape, *rotation, *mino_id)?;
                    }
                    BlockState::Stack(shape, rotation, mino_id) => {
                        self.draw_mino(canvas, &self.sprites, x as u32, y, *shape, *rotation, *mino_id)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn pause_texture(&self) -> &Texture {
        &self.paused
    }
}