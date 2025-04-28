use super::TextureAnimation;
use crate::animation::TextureAnimate;
use crate::game::board::DestroyLines;

use crate::particles::prescribed::PrescribedParticles;
use sdl2::pixels::Color;
use std::time::Duration;

const MAX_FLASHES: u32 = 3;
const FLASH_DURATION: Duration = Duration::from_millis(250);
pub const SWEEP_DURATION: Duration = Duration::from_millis(750);
const PARTICLE_FADE_IN_DURATION: Duration = Duration::from_millis(750);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestroyAnimationType {
    Flash,
    Sweep,
    Particles { color: Color },
}

#[derive(Clone, Copy, Debug)]
enum State {
    Finished,
    Nothing(Duration),
    Animate(Duration, TextureAnimate),
}

impl State {
    fn with_delta(self, delta: Duration) -> Self {
        match self {
            State::Finished => State::Finished,
            State::Nothing(duration) => State::Nothing(duration + delta),
            State::Animate(duration, animate) => State::Animate(duration + delta, animate),
        }
    }
}

fn first_state(destroy_type: DestroyAnimationType) -> State {
    match destroy_type {
        DestroyAnimationType::Flash => State::Animate(Duration::ZERO, TextureAnimate::SetAlpha),
        DestroyAnimationType::Sweep => State::Animate(
            Duration::ZERO,
            TextureAnimate::FillAlphaRectangle { width: 0.0 },
        ),
        DestroyAnimationType::Particles { .. } => State::Nothing(Duration::ZERO),
    }
}

pub struct DestroyAnimation {
    destroy_type: DestroyAnimationType,
    lines: DestroyLines,
    state: State,
    count: u32,
}

impl DestroyAnimation {
    pub fn new(destroy_type: DestroyAnimationType, lines: DestroyLines) -> Self {
        Self {
            destroy_type,
            lines,
            state: first_state(destroy_type),
            count: 0,
        }
    }

    pub fn lines(&self) -> DestroyLines {
        self.lines
    }

    fn next_flash(&mut self, duration: Duration) -> State {
        if duration < FLASH_DURATION {
            self.state
        } else {
            self.count += 1;
            if self.count >= MAX_FLASHES {
                State::Finished
            } else if self.count % 2 == 0 {
                State::Animate(Duration::ZERO, TextureAnimate::SetAlpha)
            } else {
                State::Nothing(Duration::ZERO)
            }
        }
    }

    fn next_sweep(&mut self, duration: Duration) -> State {
        let width = duration.as_secs_f64() / SWEEP_DURATION.as_secs_f64();
        if width >= 1.0 {
            State::Finished
        } else {
            State::Animate(duration, TextureAnimate::FillAlphaRectangle { width })
        }
    }

    fn next_particles(&mut self, color: Color) -> State {
        match self.state {
            State::Nothing(_) if self.count == 0 => {
                self.count += 1;
                let particles = PrescribedParticles::FadeInLatticeBurstAndFall {
                    fade_in: PARTICLE_FADE_IN_DURATION,
                    color,
                };
                State::Animate(Duration::ZERO, TextureAnimate::EmitParticles(particles))
            }
            State::Animate(_, _) => State::Nothing(Duration::ZERO),
            State::Nothing(duration) if duration < PARTICLE_FADE_IN_DURATION => {
                State::Nothing(duration)
            }
            _ => State::Finished,
        }
    }

    fn next(&mut self, duration: Duration) -> State {
        match self.destroy_type {
            DestroyAnimationType::Flash => self.next_flash(duration),
            DestroyAnimationType::Sweep => self.next_sweep(duration),
            DestroyAnimationType::Particles { color } => self.next_particles(color),
        }
    }
}

impl TextureAnimation for DestroyAnimation {
    fn update(&mut self, delta: Duration) -> Option<TextureAnimate> {
        self.state = self.state.with_delta(delta);

        let state = match self.state {
            State::Finished => State::Finished,
            State::Nothing(duration) => self.next(duration),
            State::Animate(duration, _) => self.next(duration),
        };
        self.state = state;
        self.current()
    }

    fn current(&self) -> Option<TextureAnimate> {
        match self.state {
            State::Finished => None,
            State::Nothing(_) => Some(TextureAnimate::Nothing),
            State::Animate(_, animate) => Some(animate),
        }
    }
}
