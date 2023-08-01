pub mod destroy;
pub mod game_over;
pub mod hard_drop;
pub mod impact;

use crate::particles::prescribed::PrescribedParticles;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum TextureAnimate {
    Nothing,
    SetAlpha,
    FillAlphaRectangle { width: f64 },
    EmitParticles(PrescribedParticles),
}

impl TextureAnimate {
    pub fn is_emit_particles(&self) -> bool {
        matches!(self, TextureAnimate::EmitParticles(_))
    }
}

pub trait TextureAnimation {
    fn update(&mut self, delta: Duration) -> Option<TextureAnimate>;
    fn current(&self) -> Option<TextureAnimate>;
}
