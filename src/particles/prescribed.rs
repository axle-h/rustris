use std::time::Duration;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use crate::game::board::{compact_destroy_lines, DestroyLines};
use crate::game::tetromino::Minos;
use crate::particles::color::ParticleColor;
use crate::particles::geometry::PointF;
use crate::particles::scale::Scale;
use crate::particles::source::{AggregateParticleSource, ParticleModulation, ParticleSource, RandomParticleSource};
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
    BurstDown { color: Color },
    PerimeterBurst { color: Color },
    PerimeterSpray { color: Color }
}

impl PrescribedParticles {
    pub fn into_targeted(self, player: u32, target: PlayerParticleTarget) -> PlayerTargetedParticles {
        PlayerTargetedParticles { player, target, particles: self }
    }

    fn into_source(self, scale: &Scale, rects: &[Rect]) -> Box<dyn ParticleSource> {
        match self {
            PrescribedParticles::FadeInLatticeBurstAndFall { fade_in, color } =>
                RandomParticleSource::new(scale.rect_lattice(rects), ParticleModulation::Cascade)
                    .with_color(ParticleColor::from_sdl(color))
                    .with_velocity((PointF::new(0.0, -0.4), PointF::new(0.1, 0.1)))
                    .with_gravity(1.5)
                    .with_anchor(fade_in)
                    .with_fade_in(fade_in)
                    .with_alpha((0.9, 0.1))
                    .into_box(),
            PrescribedParticles::LightBurstUpAndOut { color } =>
                RandomParticleSource::burst(
                    scale.rect_lattice(rects),
                    ParticleColor::from_sdl(color),
                    (PointF::new(0.0, -0.1), PointF::new(0.2, 0.2)),
                    (1.0, 0.1),
                        (0.4, 0.1)
                ).into_box(),
            PrescribedParticles::BurstUp { color } =>
                RandomParticleSource::burst(
                    scale.rect_lattice(rects),
                    ParticleColor::from_sdl(color),
                    (PointF::new(0.0, -0.2), PointF::new(0.05, 0.1)),
                    (1.0, 0.1),
                    (0.7, 0.3)
                ).into_box(),
            PrescribedParticles::BurstDown { color } =>
                RandomParticleSource::burst(
                    scale.rect_lattice(rects),
                    ParticleColor::from_sdl(color),
                    (PointF::new(0.0, 0.2), PointF::new(0.1, 0.1)),
                    (1.0, 0.1),
                    (0.7, 0.3)
                ).into_box(),
            PrescribedParticles::PerimeterBurst { color } => {
                let color = ParticleColor::from_sdl(color);
                let sources = rects.into_iter().flat_map(|r| perimeter_sources(scale, *r, color)).collect();
                AggregateParticleSource::new(sources).into_box()
            },
            PrescribedParticles::PerimeterSpray { color } => {
                let color = ParticleColor::from_sdl(color);
                let sources = rects.into_iter()
                    .flat_map(|r| perimeter_sources(scale, *r, color))
                    .map(|s| s.with_modulation(ParticleModulation::Constant { count: u32::MAX, step: Duration::from_millis(750) }))
                    .collect();
                AggregateParticleSource::new(sources).into_box()
            },
        }
    }
}

fn perimeter_sources(scale: &Scale, rect: Rect, color: ParticleColor) -> [RandomParticleSource; 4] {
    const V: f64 = 0.2;
    const FADE_OUT: (f64, f64) = (1.0, 0.1);
    const ALPHA: (f64, f64) = (0.7, 0.3);
    let [top, right, bottom, left] = scale.perimeter_lattices(rect);
    [
        RandomParticleSource::burst(top, color, (PointF::new(0.0, -V), PointF::new(0.2, 0.1)), FADE_OUT, ALPHA),
        RandomParticleSource::burst(right, color, (PointF::new(V, 0.0), PointF::new(0.1, 0.2)), FADE_OUT, ALPHA),
        RandomParticleSource::burst(bottom, color, (PointF::new(0.0, V), PointF::new(0.2, 0.1)), FADE_OUT, ALPHA),
        RandomParticleSource::burst(left, color, (PointF::new(-V, 0.0), PointF::new(0.1, 0.2)), FADE_OUT, ALPHA),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerTargetedParticles {
    player: u32,
    target: PlayerParticleTarget,
    particles: PrescribedParticles
}

impl PlayerTargetedParticles {
    pub fn into_source(self, themes: &ThemeContext, particle_scale: &Scale) -> Box<dyn ParticleSource> {
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