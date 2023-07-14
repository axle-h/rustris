use std::cmp::min;
use std::rc::Rc;
use std::time::Duration;
use crate::particles::color::ParticleColor;
use crate::particles::geometry::PointF;
use crate::particles::quantity::VariableQuantity;
use crate::particles::source::{ParticleSource, ParticlePositionSource};

pub mod geometry;
pub mod source;
pub mod render;
pub mod scale;
pub mod quantity;
pub mod prescribed;
pub mod color;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Particle {
    position: PointF,
    velocity: PointF,
    acceleration: PointF,
    alpha: f64,
    color: ParticleColor
}

impl Particle {
    fn new(source: &ParticleSource, position: PointF) -> Self {
        Self {
            position,
            velocity: source.velocity().next(),
            acceleration: source.acceleration().next(),
            alpha: if source.fade_in().is_some() { 0.0 } else { 1.0 },
            color: source.color().next()
        }
    }

    /// checks if the particle is out of bounds (0-1) and trajectory will not bring it back
    fn is_escaped(&self) -> bool {
        (self.position.x() > 1.0 && self.velocity.x() >= 0.0 && self.acceleration.x() >= 0.0)
            || (self.position.x() < 0.0 && self.velocity.x() <= 0.0 && self.acceleration.x() <= 0.0)
            || (self.position.y() > 1.0 && self.velocity.y() >= 0.0 && self.acceleration.y() >= 0.0)
            || (self.position.y() < 0.0 && self.velocity.y() <= 0.0 && self.acceleration.y() <= 0.0)
    }
}

struct ParticleGroup {
    lifetime: f64,
    time_to_live: Option<f64>,
    anchor_for: Option<f64>,
    fade_in: Option<f64>,
    particles: Vec<Particle>,
}

impl ParticleGroup {
    fn new(source: &ParticleSource, positions: Vec<PointF>) -> Self {
        Self {
            lifetime: 0.0,
            time_to_live: if let Some(lifetime) = &source.lifetime_secs() { Some(lifetime.next()) } else { None },
            anchor_for: source.anchor_for().map(|d| d.as_secs_f64()),
            fade_in: source.fade_in().map(|d| d.as_secs_f64()),
            particles: positions.into_iter().map(|p| Particle::new(source, p)).collect()
        }
    }

    fn remove_escaped_particles(&mut self) {
        let mut to_remove = vec![];
        for (index, particle) in self.particles.iter().enumerate() {
            if particle.is_escaped() {
                to_remove.push(index);
            }
        }
        for index in to_remove.into_iter().rev() {
            self.particles.remove(index);
        }
    }

    fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    fn len(&self) -> usize {
        self.particles.len()
    }
}

pub struct Particles {
    particles: Vec<ParticleGroup>,
    sources: Vec<ParticleSource>,
    max_particles: usize
}

impl Particles {
    pub fn new(max_particles: usize) -> Self {
        Self { sources: vec![], particles: vec![], max_particles }
    }

    pub fn particles(&self) -> Vec<&Particle> {
        self.particles.iter().flat_map(|g| g.particles.iter()).collect()
    }

    pub fn add_source(&mut self, source: ParticleSource) {
        self.sources.push(source);
    }

    pub fn update(&mut self, delta: Duration) {
        let delta_time = delta.as_secs_f64();
        self.update_life(delta_time);
        self.update_particle(delta_time);
        self.emit_particles(delta);
        // todo update color size rotation
    }

    fn update_life(&mut self, delta_time: f64) {
        let mut to_remove = vec![];
        for (i, group) in self.particles.iter_mut().enumerate() {
            group.lifetime += delta_time;

            if let Some(time_to_live) = group.time_to_live {
                if group.lifetime >= time_to_live {
                    to_remove.push(i);
                }
            }

            group.remove_escaped_particles();
            if group.is_empty() {
                to_remove.push(i);
            }
        }
        for i in to_remove.into_iter().rev() {
            self.particles.remove(i);
        }
    }

    fn update_particle(&mut self, delta_time: f64) {
        for group in self.particles.iter_mut() {
            // spatial
            if let Some(anchor_for) = group.anchor_for {
                group.anchor_for = if delta_time >= anchor_for {
                    None
                } else {
                    Some(anchor_for - delta_time)
                }
            } else {
                for particle in group.particles.iter_mut() {
                    particle.velocity += particle.acceleration * delta_time;
                    particle.position += particle.velocity * delta_time;
                }
            }

            // alpha
            if let Some(fade_in) = group.fade_in {
                if group.lifetime >= fade_in {
                    group.fade_in = None;
                } else {
                    for particle in group.particles.iter_mut() {
                        particle.alpha = 1.0_f64.min(group.lifetime / fade_in);
                    }
                }
            }
        }
    }

    fn emit_particles(&mut self, delta: Duration) {
        let current_particles = self.particles.iter().map(|g| g.len()).sum::<usize>() as i32;
        let mut max_particles = self.max_particles as i32 - current_particles;

        let mut to_remove = vec![];
        for (index, source) in self.sources.iter_mut().enumerate() {
            if max_particles <= 0 {
                return;
            }

            let positions = source.update(delta, max_particles as u32);
            let group = ParticleGroup::new(source, positions);
            max_particles -= group.len() as i32;
            self.particles.push(group);

            if source.is_complete() {
                to_remove.push(index);
            }
        }
        for index in to_remove.into_iter().rev() {
            self.sources.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}