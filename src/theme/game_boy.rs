use std::convert::TryInto;
use std::iter::Iterator;
use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::retro::{RetroTheme, RetroThemeOptions};
use serde::{Deserialize, Serialize};

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::font::{alpha_sprites, FontRenderOptions, MetricSnips};
use crate::theme::geometry::VISIBLE_BUFFER;
use crate::theme::sound::SoundThemeOptions;
use crate::theme::sprite_sheet::TetrominoSpriteSheetMeta;

const ALPHA_PIXELS: u32 = 6;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn char_snip(row: i32, col: i32) -> Point {
    // characters are in row x col with 8 pixels between columns and 7 pixels between rows
    Point::new(1 + col * 8, 45 + row * 7)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameBoyPalette {
    GameBoyLight,
    GreenSoup,
}

fn game_boy_theme_options(palette: &GameBoyPalette, config: Config) -> RetroThemeOptions {
    RetroThemeOptions::new(
        "gb",
        config,
        TetrominoSpriteSheetMeta::new(
            &format!("resource/gb/{}", palette.sprite_sheet_file()),
            BLOCK_PIXELS,
            [Point::new(1, 35), Point::new(9, 35), Point::new(17, 35), Point::new(25, 35)],
            Point::new(51, 26),
            Point::new(26, 26),
            Point::new(1, 1),
            Point::new(51, 1),
            Point::new(1, 26),
            Point::new(18, 1),
            (34, 35),
            0x30
        ),
        &palette.background_file(),
        &palette.board_file(),
        &palette.game_over_file(),
        [
            Rect::new(162, 11 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 48 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 72 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 96 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 120 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(12, 101 + BUFFER_PIXELS as i32, 32, 32),
        FontRenderOptions::Sprites {
            file: format!("resource/gb/{}", palette.sprite_sheet_file()),
            sprites: alpha_sprites(
                (0 .. 10).map(|i| char_snip(3, i)).collect::<Vec<Point>>().try_into().unwrap(),
                ALPHA_PIXELS,
                ALPHA_PIXELS
            ),
            spacing: 2
        },
        MetricSnips::right((46, 25), 999999),
        MetricSnips::right((39, 52), 999),
        MetricSnips::right((39, 78), 999),
        Point::new(55, 0),
        Point::new(8, 0),
        palette.white(),
        DestroyAnimationType::Flash,
        GameOverAnimationType::CurtainUp,
        SoundThemeOptions::default("gb", config.audio)
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
    ) -> Result<RetroTheme<'a>, String> {
        let options = game_boy_theme_options(self, config);
        RetroTheme::new(canvas, texture_creator, options)
    }
}
