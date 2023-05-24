pub mod destroy;
pub mod game_over;
pub mod hard_drop;
pub mod impact;

use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum TextureAnimate {
    Nothing,
    SetAlpha,
    FillAlphaRectangle { width: f64 },
}

pub trait TextureAnimation {
    fn update(&mut self, delta: Duration) -> Option<TextureAnimate>;
    fn current(&self) -> Option<TextureAnimate>;
}
