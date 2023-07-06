use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::{GameOverAnimate, GameOverAnimationType};
use crate::animation::TextureAnimate;
use crate::event::GameEvent;

use crate::game::tetromino::Minos;
use crate::game::Game;
use sdl2::mixer::Music;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};

mod block_theme;
pub mod game_boy;
pub mod nes;
pub mod snes;
pub mod sound;
pub mod perimeter;

pub trait Theme {
    fn max_score(&self) -> u32;
    fn block_size(&self) -> u32;
    fn background_color(&self) -> Color;
    fn background_size(&self) -> (u32, u32);
    fn board_snip(&self) -> Rect;
    fn draw_background(&mut self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String>;
    fn draw_board(
        &mut self,
        canvas: &mut WindowCanvas,
        game: &Game,
        animate_lines: Vec<(u32, TextureAnimate)>,
        animate_game_over: Option<GameOverAnimate>,
    ) -> Result<(), String>;

    /// get the rect of the line within the board_snip
    fn line_snip(&self, j: u32) -> Rect;

    fn destroy_animation_type(&self) -> DestroyAnimationType;
    fn game_over_animation_type(&self) -> GameOverAnimationType;
    fn music(&self) -> &Music;
    fn receive_event(&mut self, event: GameEvent) -> Result<(), String>;
    fn mino_rects(&self, minos: Minos) -> [Rect; 4];
}

// const REFERENCE_I: Color = Color::CYAN;
// const REFERENCE_J: Color = Color::BLUE;
// const REFERENCE_L: Color = Color::RGBA(0xff, 0x7f, 0x00, 0xff);
// const REFERENCE_O: Color = Color::YELLOW;
// const REFERENCE_S: Color = Color::GREEN;
// const REFERENCE_T: Color = Color::RGBA(0x80, 0x00, 0x80, 0xff);
// const REFERENCE_Z: Color = Color::RED;
