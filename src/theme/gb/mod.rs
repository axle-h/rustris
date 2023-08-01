use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::retro::{retro_theme, RetroThemeOptions};
use std::convert::TryInto;
use std::iter::Iterator;

use crate::theme::font::{alpha_sprites, FontRenderOptions, MetricSnips};
use crate::theme::geometry::VISIBLE_BUFFER;
use crate::theme::sound::SoundThemeOptions;
use crate::theme::sprite_sheet::TetrominoSpriteSheetMeta;
use crate::theme::{Theme, ThemeName};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

const SPRITES: &[u8] = include_bytes!("sprites.png");
const BACKGROUND_FILE: &[u8] = include_bytes!("background.png");
const BOARD_FILE: &[u8] = include_bytes!("board.png");
const GAME_OVER_FILE: &[u8] = include_bytes!("game-over.png");

const GAME_OVER_SOUND: &[u8] = include_bytes!("game-over.ogg");
const LEVEL_UP_SOUND: &[u8] = include_bytes!("level-up.ogg");
const CLEAR_SOUND: &[u8] = include_bytes!("line-clear.ogg");
const LOCK_SOUND: &[u8] = include_bytes!("lock.ogg");
const MOVE_SOUND: &[u8] = include_bytes!("move.ogg");
const MUSIC: &[u8] = include_bytes!("music.ogg");
const PAUSE_SOUND: &[u8] = include_bytes!("pause.ogg");
const ROTATE_SOUND: &[u8] = include_bytes!("rotate.ogg");
const SEND_GARBAGE_SOUND: &[u8] = include_bytes!("send-garbage.ogg");
const STACK_DROP_SOUND: &[u8] = include_bytes!("stack-drop.ogg");
const TETRIS_SOUND: &[u8] = include_bytes!("tetris.ogg");
const VICTORY_SOUND: &[u8] = include_bytes!("victory.ogg");

const ALPHA_PIXELS: u32 = 6;
const BLOCK_PIXELS: u32 = 8;
const BUFFER_PIXELS: u32 = VISIBLE_BUFFER * BLOCK_PIXELS;

fn char_snip(row: i32, col: i32) -> Point {
    // characters are in row x col with 8 pixels between columns and 7 pixels between rows
    Point::new(1 + col * 8, 45 + row * 7)
}

pub fn game_boy_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    config: Config,
) -> Result<Theme<'a>, String> {
    let options = RetroThemeOptions::new(
        ThemeName::GameBoy,
        TetrominoSpriteSheetMeta::new(
            SPRITES,
            BLOCK_PIXELS,
            [
                Point::new(1, 35),
                Point::new(9, 35),
                Point::new(17, 35),
                Point::new(25, 35),
            ],
            Point::new(51, 26),
            Point::new(26, 26),
            Point::new(1, 1),
            Point::new(51, 1),
            Point::new(1, 26),
            Point::new(18, 1),
            (34, 35),
            0x30,
        ),
        BACKGROUND_FILE,
        BOARD_FILE,
        GAME_OVER_FILE,
        [
            Rect::new(162, 11 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 48 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 72 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 96 + BUFFER_PIXELS as i32, 32, 32),
            Rect::new(162, 120 + BUFFER_PIXELS as i32, 32, 32),
        ],
        Rect::new(12, 101 + BUFFER_PIXELS as i32, 32, 32),
        FontRenderOptions::Sprites {
            file_bytes: SPRITES,
            sprites: alpha_sprites(
                (0..10)
                    .map(|i| char_snip(3, i))
                    .collect::<Vec<Point>>()
                    .try_into()
                    .unwrap(),
                ALPHA_PIXELS,
                ALPHA_PIXELS,
            ),
            spacing: 2,
        },
        MetricSnips::right((46, 25), 999999),
        MetricSnips::right((39, 52), 999),
        MetricSnips::right((39, 78), 999),
        Point::new(55, 0),
        Point::new(8, 0),
        Color::WHITE,
        DestroyAnimationType::Flash,
        GameOverAnimationType::CurtainUp,
        SoundThemeOptions::default(
            config.audio,
            MUSIC,
            MOVE_SOUND,
            ROTATE_SOUND,
            LOCK_SOUND,
            SEND_GARBAGE_SOUND,
            [CLEAR_SOUND, CLEAR_SOUND, CLEAR_SOUND, TETRIS_SOUND],
            LEVEL_UP_SOUND,
            GAME_OVER_SOUND,
            PAUSE_SOUND,
            VICTORY_SOUND,
        )
        .with_stack_drop(STACK_DROP_SOUND),
    );

    retro_theme(canvas, texture_creator, options)
}
