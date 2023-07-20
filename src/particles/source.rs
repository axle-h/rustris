use std::cmp::min;
use std::time::Duration;
use crate::particles::color::ParticleColor;
use crate::particles::geometry::{PointF, RectF};
use crate::particles::{Particle, ParticleGroup};
use crate::particles::quantity::VariableQuantity;

#[derive(Clone, Debug, PartialEq)]
pub enum ParticlePositionSource {
    /// All particles are emitted from one point
    Static(PointF),

    /// Emitted randomly within a rectangle
    Rect(RectF),

    Lattice(Vec<PointF>)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleModulation {
    /// All available particles are emitted as soon as possible
    Cascade,

    /// A maximum number of particles are emitted
    CascadeLimit { count: u32 },

    /// A maximum number of particles are emitted at a constant time step
    Constant { count: u32, step: Duration },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParticleSourceState {
    Complete,
    Emit,
    Delay(Duration)
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParticleSource {
    state: ParticleSourceState,
    position_source: ParticlePositionSource,
    modulation: ParticleModulation,
    anchor_for: Option<Duration>,
    fade_in: Option<Duration>,
    fade_out: bool,
    lifetime_secs: Option<VariableQuantity<f64>>,
    velocity: VariableQuantity<PointF>,
    acceleration: VariableQuantity<PointF>,
    color: VariableQuantity<ParticleColor>,
    alpha: VariableQuantity<f64>,
}


// todo trait this out so I can have an aggregate particle source
impl ParticleSource {
    pub fn new(position_source: ParticlePositionSource, modulation: ParticleModulation) -> Self  {
        Self {
            state: ParticleSourceState::Emit,
            position_source, modulation,
            anchor_for: None,
            fade_in: None,
            fade_out: false,
            lifetime_secs: None,
            velocity: VariableQuantity::new(PointF::ZERO, PointF::ZERO),
            acceleration: VariableQuantity::new(PointF::ZERO, PointF::ZERO),
            color: VariableQuantity::new(ParticleColor::WHITE, ParticleColor::ZERO),
            alpha: VariableQuantity::new(1.0, 0.0)
        }
    }

    pub fn with_alpha<A : Into<VariableQuantity<f64>>>(mut self, value: A) -> Self {
        self.alpha = value.into();
        self
    }


    pub fn with_color<C : Into<VariableQuantity<ParticleColor>>>(mut self, value: C) -> Self {
        self.color = value.into();
        self
    }

    pub fn with_anchor(mut self, value: Duration) -> Self {
        self.anchor_for = Some(value);
        self
    }

    pub fn with_fade_in(mut self, value: Duration) -> Self {
        self.fade_in = Some(value);
        self
    }

    pub fn with_fade_out<L : Into<VariableQuantity<f64>>>(mut self, value: L) -> Self {
        self.fade_out = true;
        self.lifetime_secs = Some(value.into());
        self
    }

    pub fn with_velocity<V : Into<VariableQuantity<PointF>>>(mut self, value: V) -> Self {
        self.velocity = value.into();
        self
    }

    pub fn with_acceleration<A: Into<VariableQuantity<PointF>>>(mut self, value: A) -> Self {
        self.acceleration = value.into();
        self
    }

    pub fn with_gravity(mut self, value: f64) -> Self {
        self.with_acceleration(PointF::new(0.0, value))
    }

    pub fn is_complete(&self) -> bool {
        self.state == ParticleSourceState::Complete
    }

    pub fn update(&mut self, delta_time: Duration, max_particles: u32) -> Option<ParticleGroup> {
        if self.state == ParticleSourceState::Complete {
            return None;
        }
        let emit_particles = match self.modulation {
            ParticleModulation::Cascade => self.cascade(max_particles),
            ParticleModulation::CascadeLimit { count } => self.cascade(count),
            ParticleModulation::Constant { count, step } => self.constant(count, step, delta_time)
        }.min(max_particles);

        if emit_particles == 0 {
            return None;
        }

        let particles = match &self.position_source {
            ParticlePositionSource::Lattice(points) => points.iter().take(emit_particles as usize).copied().collect::<Vec<PointF>>(),
            _ => (0..emit_particles).map(|_| self.next_position()).collect()
        }.into_iter().map(|p| self.next_particle(p)).collect();

        Some(
            ParticleGroup {
                lifetime: 0.0,
                anchor_for: self.anchor_for.map(|d| d.as_secs_f64()),
                fade_in: self.fade_in.map(|d| d.as_secs_f64()),
                fade_out: self.fade_out,
                particles
            }
        )
    }

    fn cascade(&mut self, count: u32) -> u32 {
        self.state = ParticleSourceState::Complete;
        count
    }

    fn constant(&mut self, count: u32, step: Duration, delta_time: Duration) -> u32 {
        match self.state {
            ParticleSourceState::Emit => {
                self.state = ParticleSourceState::Delay(step);
                count
            },
            ParticleSourceState::Delay(delay) => {
                let delta_time_nanos = delay.as_nanos() as u64 + delta_time.as_nanos() as u64;
                let step_nanos = step.as_nanos() as u64;
                let n_steps = delta_time_nanos / step_nanos;
                self.state = ParticleSourceState::Delay(Duration::from_nanos(delta_time_nanos % step_nanos));
                n_steps as u32 * count
            },
            _ => unreachable!()
        }
    }

    fn next_position(&self) -> PointF {
        match self.position_source {
            ParticlePositionSource::Static(point) => point,
            ParticlePositionSource::Rect(rect) => {
                let x = rect.x() + rect.width() * rand::random::<f64>();
                let y = rect.y() + rect.height() * rand::random::<f64>();
                PointF::new(x, y)
            },
            _ => unreachable!()
        }
    }

    fn next_particle(&self, position: PointF) -> Particle {
        let max_alpha = self.alpha.next();
        Particle {
            position,
            velocity: self.velocity.next(),
            acceleration: self.acceleration.next(),
            alpha: if self.fade_in.is_some() { 0.0 } else { max_alpha },
            max_alpha,
            color: self.color.next(),
            time_to_live: if let Some(lifetime) = &self.lifetime_secs { Some(lifetime.next()) } else { None },
        }
    }
}
