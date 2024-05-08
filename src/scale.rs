use std::cmp::min;
use sdl2::rect::Rect;
use crate::config::VideoConfig;
use crate::theme::ThemeName;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    players: u32,
    scale: f64,
    window_width: u32,
    window_height: u32,
    block_size: f64,
    integer_scale: Option<u32>
}

impl Scale {
    pub fn new(
        players: u32,
        game_size: (u32, u32),
        window_size: (u32, u32),
        block_size: u32,
        config: VideoConfig,
        theme: ThemeName
    ) -> Self {
        let (window_width, window_height) = window_size;
        let (bg_width, bg_height) = game_size;

        // the modern theme does it's own scaling
        let is_integer_scale = theme == ThemeName::Modern || config.integer_scale;

        let (scale, integer_scale) = if is_integer_scale {
            let scale = min(window_width / (bg_width * players), window_height / bg_height);
            (scale as f64, Some(scale))
        } else {
            let padded_window_width = window_width as f64 - (2.0 * config.screen_padding_pct() * window_width as f64);
            let scale_x = padded_window_width / (bg_width as f64 * players as f64);

            let padded_window_height = window_height as f64 - (2.0 * config.screen_padding_pct() * window_height as f64);
            let scale_y = padded_window_height / bg_height as f64;

            let scale = scale_x.min(scale_y);
            (scale, None)
        };

        Self {
            players,
            scale,
            window_width,
            window_height,
            block_size: block_size as f64 * scale,
            integer_scale
        }
    }

    /// splits the entire window up into horizontally stacked chunks equally between players
    pub fn player_window(&self, player: u32) -> Rect {
        let player_chunk_width = self.window_width / self.players;
        let x = player_chunk_width * (player - 1);
        Rect::new(x as i32, 0, player_chunk_width, self.window_height)
    }

    pub fn scale_and_offset_rect(&self, rect: Rect, offset_x: i32, offset_y: i32) -> Rect {
        Rect::new(
            self.scale_coordinate(rect.x) + offset_x,
            self.scale_coordinate(rect.y) + offset_y,
            self.scale_length(rect.width()),
            self.scale_length(rect.height()),
        )
    }

    pub fn scale_rect(&self, rect: Rect) -> Rect {
        self.scale_and_offset_rect(rect, 0, 0)
    }

    pub fn scale_length(&self, value: u32) -> u32 {
        if let Some(integer_scale) = self.integer_scale {
            value * integer_scale
        } else {
            (value as f64 * self.scale).round() as u32
        }
    }

    fn scale_coordinate(&self, value: i32) -> i32 {
        if let Some(integer_scale) = self.integer_scale {
            value * integer_scale as i32
        } else {
            (value as f64 * self.scale).round() as i32
        }
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
