use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, WindowCanvas};
use sdl2::video::Window;
use crate::game::{Game, GameMetrics};

pub mod game_boy;

pub trait Theme {
    fn background_size(&self) -> (u32, u32);
    fn board_snip(&self, player: u32) -> Rect;
    fn draw_background(&self, canvas: &mut Canvas<Window>, games: &Vec<GameMetrics>) -> Result<(), String>;
    fn draw_board(&self, canvas: &mut Canvas<Window>, game: &Game) -> Result<(), String>;
    fn pause_texture(&self) -> &Texture;
}
