use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::block_theme::{BlockTheme, BlockThemeOptions, TetrominoSnips, VISIBLE_BUFFER};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

const ALPHA_WIDTH: u32 = 7;
const ALPHA_HEIGHT: u32 = 8;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn block_snip(i: i32, j: i32) -> Rect {
    Rect::new(
        i * BLOCK_PIXELS as i32,
        j * BLOCK_PIXELS as i32,
        BLOCK_PIXELS,
        BLOCK_PIXELS,
    )
}

fn char_snip(row: i32, col: i32) -> Rect {
    Rect::new(col * 8, 35 + row * 9, ALPHA_WIDTH, ALPHA_HEIGHT)
}

pub fn snes_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<BlockTheme<'a>, String> {
    let options = BlockThemeOptions::new(
        "snes".to_string(),
        config,
        "sprites.png".to_string(),
        "background.png".to_string(),
        "board.png".to_string(),
        "game-over.png".to_string(),
        "paused.png".to_string(),
        0x40,
        BLOCK_PIXELS,
        (ALPHA_WIDTH, ALPHA_HEIGHT),
        (0..10)
            .map(|i| char_snip(0, i))
            .collect::<Vec<Rect>>()
            .try_into()
            .unwrap(),
        [
            Rect::new(168, 17 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 58 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 82 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 106 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(168, 130 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(19, 133 + BUFFER_PIXELS as i32, 32, 32),
        (0..6)
            .map(|i| {
                Rect::new(
                    7 + i * (ALPHA_WIDTH as i32 + 1),
                    22,
                    ALPHA_WIDTH,
                    ALPHA_WIDTH,
                )
            })
            .rev()
            .collect(),
        (0..3)
            .map(|i| {
                Rect::new(
                    23 + i * (ALPHA_WIDTH as i32 + 1),
                    62,
                    ALPHA_WIDTH,
                    ALPHA_WIDTH,
                )
            })
            .rev()
            .collect(),
        (0..3)
            .map(|i| {
                Rect::new(
                    23 + i * (ALPHA_WIDTH as i32 + 1),
                    98,
                    ALPHA_WIDTH,
                    ALPHA_WIDTH,
                )
            })
            .rev()
            .collect(),
        true,
        Point::new(62, 0),
        Point::new(8, 0),
        TetrominoSnips::uniform_stack(
            Rect::new(143, 26, BLOCK_PIXELS * 4, BLOCK_PIXELS),
            block_snip(1, 1),
            block_snip(1, 0),
        ),
        TetrominoSnips::uniform_stack(
            Rect::new(118, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(3, 1),
            block_snip(3, 0),
        ),
        TetrominoSnips::uniform_stack(
            Rect::new(93, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(2, 1),
            block_snip(2, 0),
        ),
        TetrominoSnips::uniform_stack(
            Rect::new(1, 18, BLOCK_PIXELS * 2, BLOCK_PIXELS * 2),
            block_snip(0, 1),
            block_snip(0, 0),
        ),
        TetrominoSnips::uniform_stack(
            Rect::new(43, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(2, 1),
            block_snip(2, 0),
        ),
        TetrominoSnips::uniform_stack(
            Rect::new(68, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(0, 1),
            block_snip(0, 0),
        ),
        TetrominoSnips::uniform_stack(
            Rect::new(18, 18, BLOCK_PIXELS * 3, BLOCK_PIXELS * 2),
            block_snip(3, 1),
            block_snip(3, 0),
        ),
        block_snip(0, 0),
        Color::RGB(0x74, 0x74, 0x74),
        DestroyAnimationType::Sweep,
        GameOverAnimationType::CurtainDown,
    );
    BlockTheme::new(canvas, texture_creator, options)
}
