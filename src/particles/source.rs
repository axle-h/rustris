use std::cmp::min;
use std::time::Duration;
use crate::particles::color::ParticleColor;
use crate::particles::geometry::{PointF, RectF};
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

    pub fn with_alpha<A : Into<VariableQuantity<f64>>>(&self, value: A) -> Self {
        let mut result = self.clone();
        result.alpha = value.into();
        result
    }


    pub fn with_color<C : Into<VariableQuantity<ParticleColor>>>(&self, value: C) -> Self {
        let mut result = self.clone();
        result.color = value.into();
        result
    }

    pub fn with_anchor(&self, value: Duration) -> Self {
        let mut result = self.clone();
        result.anchor_for = Some(value);
        result
    }

    pub fn with_fade_in(&self, value: Duration) -> Self {
        let mut result = self.clone();
        result.fade_in = Some(value);
        result
    }

    pub fn with_fade_out<L : Into<VariableQuantity<f64>>>(&self, value: L) -> Self {
        let mut result = self.clone();
        result.fade_out = true;
        result.lifetime_secs = Some(value.into());
        result
    }

    pub fn with_velocity<V : Into<VariableQuantity<PointF>>>(&self, value: V) -> Self {
        let mut result = self.clone();
        result.velocity = value.into();
        result
    }

    pub fn with_acceleration<A: Into<VariableQuantity<PointF>>>(&self, value: A) -> Self {
        let mut result = self.clone();
        result.acceleration = value.into();
        result
    }

    pub fn with_gravity(&self, value: f64) -> Self {
        self.with_acceleration(PointF::new(0.0, value))
    }

    pub fn is_complete(&self) -> bool {
        self.state == ParticleSourceState::Complete
    }

    pub fn update(&mut self, delta_time: Duration, max_particles: u32) -> Vec<PointF> {
        if self.state == ParticleSourceState::Complete {
            return vec![];
        }
        let emit_particles = match self.modulation {
            ParticleModulation::Cascade => self.cascade(max_particles),
            ParticleModulation::CascadeLimit { count } => self.cascade(count),
            ParticleModulation::Constant { count, step } => self.constant(count, step, delta_time)
        }.min(max_particles);

        if let ParticlePositionSource::Lattice(points) = &self.position_source {
            return points.iter().take(emit_particles as usize).copied().collect()
        }

        (0 .. emit_particles)
            .map(|_| self.next_position())
            .collect()
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



    pub fn anchor_for(&self) -> Option<Duration> {
        self.anchor_for
    }

    pub fn fade_in(&self) -> Option<Duration> {
        self.fade_in
    }

    pub fn fade_out(&self) -> bool {
        self.fade_out
    }

    pub fn lifetime_secs(&self) -> &Option<VariableQuantity<f64>> {
        &self.lifetime_secs
    }

    pub fn velocity(&self) -> &VariableQuantity<PointF> {
        &self.velocity
    }

    pub fn acceleration(&self) -> &VariableQuantity<PointF> {
        &self.acceleration
    }

    pub fn color(&self) -> &VariableQuantity<ParticleColor> {
        &self.color
    }

    pub fn alpha(&self) -> &VariableQuantity<f64> {
        &self.alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn point_particle_source() {
    //     let source = ParticlePositionSource::Static(PointF::new(10.0, 10.0)).build();
    //     assert_eq!(source.next(), PointF::new(10.0, 10.0));
    // }
    //
    // #[test]
    // fn rect_particle_source() {
    //     let bounds = RectF::new(10.0, 10.0, 100.0, 100.0);
    //     let source = ParticlePositionSource::Rect(bounds).build();
    //
    //     let mut last_point = PointF::ZERO;
    //     for _ in 1..10 {
    //         let observed = source.next();
    //         assert!(bounds.contains_point(observed));
    //         assert_ne!(last_point, observed);
    //     }
    // }
}