use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use crate::particles::color::ParticleColor;
use crate::particles::geometry::PointF;
use crate::particles::scale::Scale;
use crate::particles::source::{ParticleModulation, ParticleSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrescribedParticles {
    FadeInLatticeBurstAndFall { fade_in: Duration, color: Color }
 }

impl PrescribedParticles {
    pub fn build_rect_source(&self, scale: &Scale, rect: Rect) -> ParticleSource {
        match self {
            PrescribedParticles::FadeInLatticeBurstAndFall { fade_in, color } => {
                ParticleSource::new(scale.rect_lattice(rect), ParticleModulation::Cascade)
                    .with_color(ParticleColor::from_sdl(*color))
                    .with_velocity((PointF::new(0.0, -0.4), PointF::new(0.1, 0.1)))
                    .with_gravity(1.5)
                    .with_anchor(*fade_in)
                    .with_fade_in(*fade_in)
            }
        }
    }
}

