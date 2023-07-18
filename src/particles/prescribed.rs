use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use crate::game::board::{compact_destroy_lines, DestroyLines};
use crate::game::tetromino::Minos;
use crate::particles::color::ParticleColor;
use crate::particles::geometry::PointF;
use crate::particles::scale::Scale;
use crate::particles::source::{ParticleModulation, ParticleSource};
use crate::theme_context::ThemeContext;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerParticleTarget {
    DestroyedLines(DestroyLines),
    Minos(Minos),
    Line(u32),
    Lines { from: u32, to: u32 },
    Board
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrescribedParticles {
    FadeInLatticeBurstAndFall { fade_in: Duration, color: Color },
    LightBurstUpAndOut { color: Color },
    BurstUp { color: Color },
    BurstDown { color: Color }
}

impl PrescribedParticles {
    pub fn into_targeted(self, player: u32, target: PlayerParticleTarget) -> PlayerTargetedParticles {
        PlayerTargetedParticles { player, target, particles: self }
    }

    fn into_source(self, scale: &Scale, rects: &[Rect]) -> ParticleSource {
        match self {
            PrescribedParticles::FadeInLatticeBurstAndFall { fade_in, color } =>
                ParticleSource::new(scale.rect_lattice(rects), ParticleModulation::Cascade)
                    .with_color(ParticleColor::from_sdl(color))
                    .with_velocity((PointF::new(0.0, -0.4), PointF::new(0.1, 0.1)))
                    .with_gravity(1.5)
                    .with_anchor(fade_in)
                    .with_fade_in(fade_in)
                    .with_alpha((0.9, 0.1)),
            PrescribedParticles::LightBurstUpAndOut { color } =>
                ParticleSource::new(scale.rect_lattice(rects), ParticleModulation::Cascade)
                    .with_color(ParticleColor::from_sdl(color))
                    .with_velocity((PointF::new(0.0, -0.1), PointF::new(0.2, 0.2)))
                    .with_fade_out((1.0, 0.1))
                    .with_alpha((0.4, 0.1)),
            PrescribedParticles::BurstUp { color } =>
                ParticleSource::new(scale.rect_lattice(rects), ParticleModulation::Cascade)
                    .with_color(ParticleColor::from_sdl(color))
                    .with_velocity((PointF::new(0.0, -0.2), PointF::new(0.05, 0.1)))
                    .with_fade_out((1.0, 0.1))
                    .with_alpha((0.7, 0.3)),
            PrescribedParticles::BurstDown { color } =>
                ParticleSource::new(scale.rect_lattice(rects), ParticleModulation::Cascade)
                    .with_color(ParticleColor::from_sdl(color))
                    .with_velocity((PointF::new(0.0, 0.1), PointF::new(0.2, 0.2)))
                    .with_fade_out((1.0, 0.1))
                    .with_alpha((0.7, 0.3))

        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerTargetedParticles {
    player: u32,
    target: PlayerParticleTarget,
    particles: PrescribedParticles
}

impl PlayerTargetedParticles {
    pub fn into_source(self, themes: &ThemeContext, particle_scale: &Scale) -> ParticleSource {
        let target_rects = match self.target {
            PlayerParticleTarget::DestroyedLines(lines) => compact_destroy_lines(lines)
                .into_iter()
                .map(|j| themes.player_line_snip(self.player, j))
                .collect(),
            PlayerParticleTarget::Minos(minos) => themes.player_mino_snips(self.player, minos).to_vec(),
            PlayerParticleTarget::Board => vec![themes.player_board_snip(self.player)],
            PlayerParticleTarget::Line(j) => vec![themes.player_line_snip(self.player, j)],
            PlayerParticleTarget::Lines { from, to } => (from..=to).map(|j| themes.player_line_snip(self.player, j)).collect()
        };

        self.particles.into_source(&particle_scale, target_rects.as_slice())
    }
}