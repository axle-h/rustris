use crate::particles::color::ParticleColor;
use crate::particles::geometry::PointF;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    position: PointF,
    velocity: PointF,
    acceleration: PointF,
    max_alpha: f64,
    alpha: f64,
    color: ParticleColor,
    time_to_live: Option<f64>,
}

impl Particle {
    pub fn new(position: PointF, velocity: PointF, acceleration: PointF, max_alpha: f64, alpha: f64, color: ParticleColor, time_to_live: Option<f64>) -> Self {
        Self { position, velocity, acceleration, max_alpha, alpha, color, time_to_live }
    }

    /// checks if the particle is out of bounds (0-1) and trajectory will not bring it back
    pub fn is_escaped(&self) -> bool {
        (self.position.x() > 1.0 && self.velocity.x() >= 0.0 && self.acceleration.x() >= 0.0)
            || (self.position.x() < 0.0 && self.velocity.x() <= 0.0 && self.acceleration.x() <= 0.0)
            || (self.position.y() > 1.0 && self.velocity.y() >= 0.0 && self.acceleration.y() >= 0.0)
            || (self.position.y() < 0.0 && self.velocity.y() <= 0.0 && self.acceleration.y() <= 0.0)
    }

    pub fn update(&mut self, delta_time: f64) {
        self.velocity += self.acceleration * delta_time;
        self.position += self.velocity * delta_time;
    }

    pub fn position(&self) -> PointF {
        self.position
    }
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    pub fn color(&self) -> ParticleColor {
        self.color
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroup {
    lifetime: f64,
    anchor_for: Option<f64>,
    fade_in: Option<f64>,
    fade_out: bool,
    particles: Vec<Particle>,
}

impl ParticleGroup {
    pub fn new(anchor_for: Option<f64>, fade_in: Option<f64>, fade_out: bool, particles: Vec<Particle>) -> Self {
        Self { lifetime: 0.0, anchor_for, fade_in, fade_out, particles }
    }

    pub fn update_life(&mut self, delta_time: f64) {
        self.lifetime += delta_time;

        // remove dead particles
        let mut to_remove = vec![];
        for (index, particle) in self.particles.iter().enumerate() {
            if particle.is_escaped() {
                to_remove.push(index);
            } else if let Some(time_to_live) = particle.time_to_live {
                if self.lifetime >= time_to_live {
                    to_remove.push(index);
                }
            }
        }
        for index in to_remove.into_iter().rev() {
            self.particles.remove(index);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.particles.len()
    }

    pub fn update_particles(&mut self, delta_time: f64) {
        // spatial
        if let Some(anchor_for) = self.anchor_for {
            self.anchor_for = if delta_time >= anchor_for {
                None
            } else {
                Some(anchor_for - delta_time)
            }
        } else {
            for particle in self.particles.iter_mut() {
                particle.update(delta_time);
            }
        }

        // alpha
        if let Some(fade_in) = self.fade_in {
            if self.lifetime >= fade_in {
                self.fade_in = None;
            } else {
                for particle in self.particles.iter_mut() {
                    particle.alpha = particle.max_alpha * self.lifetime.min(fade_in) / fade_in;
                }
            }
        }

        if self.fade_out {
            for particle in self.particles.iter_mut() {
                if let Some(ttl) = particle.time_to_live {
                    particle.alpha = particle.max_alpha * (1.0 - self.lifetime.min(ttl) / ttl);
                }
            }
        }
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles.as_slice()
    }
}
