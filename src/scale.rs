use sdl2::rect::{Point, Rect};
use std::cmp::min;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    players: u32,
    scale: u32,
    window_width: u32,
    window_height: u32,
}

impl Scale {
    pub fn new(players: u32, game_size: (u32, u32), window_size: (u32, u32)) -> Self {
        let (window_width, window_height) = window_size;
        let (bg_width, bg_height) = game_size;
        let scale = min(
            window_width / (bg_width * players),
            window_height / bg_height,
        );
        Self {
            players,
            scale,
            window_width,
            window_height,
        }
    }

    pub fn player_window(&self, player: u32) -> Rect {
        let player_chunk_width = self.window_width / self.players;
        let x = player_chunk_width * (player - 1);
        Rect::new(x as i32, 0, player_chunk_width, self.window_height)
    }

    pub fn scale_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x * self.scale as i32,
            rect.y * self.scale as i32,
            rect.width() * self.scale,
            rect.height() * self.scale,
        )
    }

    pub fn scaled_window_center_rect(&self, width: u32, height: u32) -> Rect {
        Rect::from_center(
            Point::new(self.window_width as i32 / 2, self.window_height as i32 / 2),
            width * self.scale,
            height * self.scale,
        )
    }

    pub fn scale_length(&self, value: u32) -> u32 {
        value * self.scale
    }

    pub fn scale_and_offset_rect(&self, rect: Rect, offset_x: i32, offset_y: i32) -> Rect {
        Rect::new(
            rect.x * self.scale as i32 + offset_x,
            rect.y * self.scale as i32 + offset_y,
            rect.width() * self.scale,
            rect.height() * self.scale,
        )
    }

    pub fn offset_scaled_rect(&self, rect: Rect, offset_x: f64, offset_y: f64) -> Rect {
        Rect::new(
            (rect.x as f64 + offset_x * self.scale as f64).round() as i32,
            (rect.y as f64 + offset_y * self.scale as f64).round() as i32,
            rect.width(),
            rect.height(),
        )
    }
}
