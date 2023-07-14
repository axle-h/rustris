use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::block_theme::{BlockTheme, BlockThemeOptions};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::geometry::VISIBLE_BUFFER;
use crate::theme::sprite_sheet::TetrominoSpriteSheetMeta;

const ALPHA_PIXELS: u32 = 7;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn mino(i: i32, j: i32) -> Point {
    Point::new(i * BLOCK_PIXELS as i32, j * BLOCK_PIXELS as i32,)
}

fn char_snip(row: i32, col: i32) -> Rect {
    Rect::new(col * 8, 111 + row * 8, ALPHA_PIXELS, ALPHA_PIXELS)
}

pub fn nes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<BlockTheme<'a>, String> {
    let options = BlockThemeOptions::new(
        "nes".to_string(),
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
        "sprites.png".to_string(),
        "background.png".to_string(),
        "board.png".to_string(),
        "game-over.png".to_string(),
        (ALPHA_PIXELS, ALPHA_PIXELS),
        (0..10)
            .map(|i| char_snip(0, i))
            .collect::<Vec<Rect>>()
            .try_into()
            .unwrap(),
        [
            Rect::new(170, 16 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 56 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 80 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 104 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(170, 128 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(16, 127 + BUFFER_PIXELS as i32, 32, 32),
        (0..6)
            .map(|i| {
                Rect::new(
                    8 + i * (ALPHA_PIXELS as i32 + 1),
                    24,
                    ALPHA_PIXELS,
                    ALPHA_PIXELS,
                )
            })
            .rev()
            .collect(),
        (0..3)
            .map(|i| {
                Rect::new(
                    20 + i * (ALPHA_PIXELS as i32 + 1),
                    72,
                    ALPHA_PIXELS,
                    ALPHA_PIXELS,
                )
            })
            .rev()
            .collect(),
        (0..3)
            .map(|i| {
                Rect::new(
                    20 + i * (ALPHA_PIXELS as i32 + 1),
                    91,
                    ALPHA_PIXELS,
                    ALPHA_PIXELS,
                )
            })
            .rev()
            .collect(),
        true,
        Point::new(66, 0),
        Point::new(7, 0),
        Color::RGB(0x74, 0x74, 0x74),
        DestroyAnimationType::Sweep,
        GameOverAnimationType::CurtainDown,
    );
    BlockTheme::new(canvas, texture_creator, options)
}
