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

const ALPHA_PIXELS: u32 = 7;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn mino(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_PIXELS as i32, j * BLOCK_PIXELS as i32,)
}

fn char_snip(row: i32, col: i32) -> Point {
    Point::new(col * 8, 111 + row * 8)
}

pub fn nes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<RetroTheme<'a>, String> {
    let options = RetroThemeOptions::new(
        "nes",
        config,
        TetrominoSpriteSheetMeta::new(
            "resource/nes/sprites.png",
            BLOCK_PIXELS,
            mino(0, 0),
            mino(2, 0),
            mino(1, 0),
            mino(0, 0),
            mino(2, 0),
            mino(0, 0),
            mino(1, 0),
            mino(0, 0),
            0x50
        ),
        "background.png",
        "board.png",
        "game-over.png",
        [
            Rect::new(170, 16 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 56 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 80 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 104 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 128 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(16, 127 + BUFFER_PIXELS as i32, 32, 32),
        FontRenderOptions::Sprites {
            file: "resource/nes/sprites.png".to_string(),
            sprites: alpha_sprites(
                (0 .. 10).map(|i| char_snip(0, i)).collect::<Vec<Point>>().try_into().unwrap(),
                ALPHA_PIXELS,
                ALPHA_PIXELS
            ),
            spacing: 1
        },
        MetricSnips::zero_fill((8, 24), 999999),
        MetricSnips::zero_fill((20, 72), 999),
        MetricSnips::zero_fill((20, 91), 999),
        Point::new(66, 0),
        Point::new(7, 0),
        Color::RGB(0x74, 0x74, 0x74),
        DestroyAnimationType::Sweep,
        GameOverAnimationType::CurtainDown,
        SoundThemeOptions::default("nes", config.audio).without_stack_drop()
    );
    RetroTheme::new(canvas, texture_creator, options)
}
