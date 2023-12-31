use sdl2::rect::Rect;
use std::cmp::min;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    players: u32,
    scale: u32,
    window_width: u32,
    window_height: u32,
    block_size: u32,
}

impl Scale {
    pub fn new(
        players: u32,
        game_size: (u32, u32),
        window_size: (u32, u32),
        block_size: u32,
    ) -> Self {
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
            block_size: block_size * scale,
        }
    }

    /// splits the entire window up into horizontally stacked chunks equally between players
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

    pub fn offset_proportional_to_block_size(
        &self,
        rect: Rect,
        offset_x: f64,
        offset_y: f64,
    ) -> Rect {
        let block_size = self.block_size as f64;
        Rect::new(
            (rect.x as f64 + offset_x * block_size).round() as i32,
            (rect.y as f64 + offset_y * block_size).round() as i32,
            rect.width(),
            rect.height(),
        )
    }
}
