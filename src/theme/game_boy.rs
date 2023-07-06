use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::block_theme::{BlockTheme, BlockThemeOptions, TetrominoSnips, VISIBLE_BUFFER};
use serde::{Deserialize, Serialize};

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

const ALPHA_PIXELS: u32 = 6;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn block_snip(x: i32, y: i32) -> Rect {
    Rect::new(x, y, BLOCK_PIXELS, BLOCK_PIXELS)
}

fn char_snip(row: i32, col: i32) -> Rect {
    // characters are in row x col with 8 pixels between columns and 7 pixels between rows
    Rect::new(1 + col * 8, 45 + row * 7, ALPHA_PIXELS, ALPHA_PIXELS)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameBoyPalette {
    GameBoyLight,
    GreenSoup,
}

fn game_boy_theme_options(palette: &GameBoyPalette, config: Config) -> BlockThemeOptions {
    BlockThemeOptions::new(
        "gb".to_string(),
        config,
        palette.sprite_sheet_file(),
        palette.background_file(),
        palette.board_file(),
        palette.game_over_file(),
        0x30,
        BLOCK_PIXELS,
        (ALPHA_PIXELS, ALPHA_PIXELS),
        [
            char_snip(3, 0),
            char_snip(3, 1),
            char_snip(3, 2),
            char_snip(3, 3),
            char_snip(3, 4),
            char_snip(3, 5),
            char_snip(3, 6),
            char_snip(3, 7),
            char_snip(3, 8),
            char_snip(3, 9),
        ],
        [
            Rect::new(162, 11 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 48 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 72 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 96 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 120 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(12, 101 + BUFFER_PIXELS as i32, 32, 32),
        (0..6)
            .map(|i| Rect::new(40 - i * 8, 25, ALPHA_PIXELS, ALPHA_PIXELS))
            .collect(),
        (0..4)
            .map(|i| Rect::new(33 - i * 8, 52, ALPHA_PIXELS, ALPHA_PIXELS))
            .collect(),
        (0..4)
            .map(|i| Rect::new(33 - i * 8, 78, ALPHA_PIXELS, ALPHA_PIXELS))
            .collect(),
        false,
        Point::new(55, 0),
        Point::new(8, 0),
        TetrominoSnips::asymmetrical(
            Rect::new(1, 35, BLOCK_PIXELS * 4, BLOCK_PIXELS),
            [
                block_snip(1, 35),
                block_snip(9, 35),
                block_snip(17, 35),
                block_snip(25, 35),
            ],
        ),
        TetrominoSnips::uniform(
            Rect::new(51, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(51, 26),
        ),
        TetrominoSnips::uniform(
            Rect::new(26, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(26, 26),
        ),
        TetrominoSnips::uniform(
            Rect::new(1, 1, BLOCK_PIXELS * 2, BLOCK_PIXELS * 2),
            block_snip(1, 1),
        ),
        TetrominoSnips::uniform(
            Rect::new(43, 1, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(51, 1),
        ),
        TetrominoSnips::uniform(
            Rect::new(1, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(1, 26),
        ),
        TetrominoSnips::uniform(
            Rect::new(18, 1, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(18, 1),
        ),
        block_snip(34, 35),
        palette.white(),
        DestroyAnimationType::Flash,
        GameOverAnimationType::CurtainUp,
    )
}

impl GameBoyPalette {
    fn palette_resource_file(&self, name: &str) -> String {
        let postfix = match self {
            GameBoyPalette::GameBoyLight => "",
            GameBoyPalette::GreenSoup => "-gs",
        };
        format!("{}{}.png", name, postfix)
    }

    fn sprite_sheet_file(&self) -> String {
        self.palette_resource_file("sprites")
    }

    fn board_file(&self) -> String {
        self.palette_resource_file("board")
    }

    fn background_file(&self) -> String {
        self.palette_resource_file("background")
    }

    fn game_over_file(&self) -> String {
        self.palette_resource_file("game-over")
    }

    fn paused_file(&self) -> String {
        self.palette_resource_file("paused")
    }

    // fn black(&self) -> Color {
    //     match self {
    //         GameBoyPalette::GameBoyLight => Color::BLACK,
    //         GameBoyPalette::GreenSoup => Color::RGB(0x00, 0x3f, 0x00),
    //     }
    // }
    //
    // fn dark_grey(&self) -> Color {
    //     match self {
    //         GameBoyPalette::GameBoyLight => Color::RGB(0x55, 0x55, 0x55),
    //         GameBoyPalette::GreenSoup => Color::RGB(0x2e, 0x73, 0x20),
    //     }
    // }
    //
    // fn light_grey(&self) -> Color {
    //     match self {
    //         GameBoyPalette::GameBoyLight => Color::RGB(0xaa, 0xaa, 0xaa),
    //         GameBoyPalette::GreenSoup => Color::RGB(0x8c, 0xbf, 0x0a),
    //     }
    // }

    fn white(&self) -> Color {
        match self {
            GameBoyPalette::GameBoyLight => Color::WHITE,
            GameBoyPalette::GreenSoup => Color::RGB(0xa0, 0xcf, 0x0a),
        }
    }

    pub fn theme<'a>(
        &self,
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        config: Config,
    ) -> Result<BlockTheme<'a>, String> {
        let options = game_boy_theme_options(self, config);
        BlockTheme::new(canvas, texture_creator, options)
    }
}
