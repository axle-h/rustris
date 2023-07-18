use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::retro::{RetroTheme, RetroThemeOptions};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::font::{alpha_sprites, FontRenderOptions, MetricSnips};
use crate::theme::geometry::VISIBLE_BUFFER;
use crate::theme::sound::SoundThemeOptions;
use crate::theme::sprite_sheet::TetrominoSpriteSheetMeta;

const ALPHA_WIDTH: u32 = 7;
const ALPHA_HEIGHT: u32 = 8;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn mino(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_PIXELS as i32, j * BLOCK_PIXELS as i32)
}

fn char_snip(row: i32, col: i32) -> Point {
    Point::new(col * 8, 35 + row * 9)
}

pub fn snes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<RetroTheme<'a>, String> {
    let options = RetroThemeOptions::new(
        "snes",
        config,
        TetrominoSpriteSheetMeta::new(
            "resource/snes/sprites.png",
            BLOCK_PIXELS,
            (
                mino(1, 1),
                mino(1, 0),
            ),
            (
                mino(3, 1),
                mino(3, 0),
            ),
            (
                mino(2, 1),
                mino(2, 0),
            ),
            (
                mino(0, 1),
                mino(0, 0),
            ),
            (
                mino(2, 1),
                mino(2, 0),
            ),
            (
                mino(0, 1),
                mino(0, 0),
            ),
            (
                mino(3, 1),
                mino(3, 0),
            ),
            mino(0, 0),
            0x50
        ),
        "background.png",
        "board.png",
        "game-over.png",
        [
            Rect::new(168, 17 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 58 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 82 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 106 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 130 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(19, 133 + BUFFER_PIXELS as i32, 32, 32),
        FontRenderOptions::Sprites {
            file: "resource/snes/sprites.png".to_string(),
            sprites: alpha_sprites(
                (0 .. 10).map(|i| char_snip(0, i)).collect::<Vec<Point>>().try_into().unwrap(),
                ALPHA_WIDTH,
                ALPHA_HEIGHT
            ),
            spacing: 1
        },
        MetricSnips::zero_fill((7, 22), 999999),
        MetricSnips::zero_fill((23, 62), 999),
        MetricSnips::zero_fill((23, 98), 999),
        Point::new(62, 0),
        Point::new(8, 0),
        Color::RGB(0x74, 0x74, 0x74),
        DestroyAnimationType::Sweep,
        GameOverAnimationType::CurtainDown,
        SoundThemeOptions::default("snes", config.audio)
    );
    RetroTheme::new(canvas, texture_creator, options)
}
