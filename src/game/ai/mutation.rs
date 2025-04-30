use std::ops::RangeInclusive;
use rand::Rng;
use std::collections::VecDeque;
use crate::game::ai::board_cost::{CostCoefficients};
use crate::game::ai::generation_stats::GenerationStatistics;

#[derive(Debug, Clone)]
pub struct MutationRateLimits {
    range: RangeInclusive<f64>,
    step: f64
}

impl MutationRateLimits {
    pub fn new(range: RangeInclusive<f64>, step: f64) -> Self {
        Self { range, step }
    }

    pub fn mid(&self) -> f64 {
        (self.range.start() + self.range.end()) / 2.0
    }
}

impl Default for MutationRateLimits {
    fn default() -> Self {
        Self::new(0.1 ..= 1.0, 0.1)
    }
}

pub struct MutationRate {
    limits: MutationRateLimits,
    samples: VecDeque<f64>,
    stat_fn: fn(GenerationStatistics) -> f64,
    current: f64,
}

impl MutationRate {
    fn new(limits: MutationRateLimits, max_samples: usize, stat_fn: fn(GenerationStatistics) -> f64) -> Self {
        Self { current: limits.mid(), limits, samples: VecDeque::with_capacity(max_samples), stat_fn }
    }
    
    pub fn current_rate(&self) -> f64 {
        self.current
    }

    pub fn of_median(limits: MutationRateLimits, max_samples: usize) -> Self {
        Self::new(
            limits,
            max_samples,
            |stats| stats.median().score() as f64
        )
    }

    pub fn of_max(limits: MutationRateLimits, max_samples: usize) -> Self {
        Self::new(
            limits,
            max_samples,
            |stats| stats.max().score() as f64
        )
    }

    pub fn add_sample(&mut self, stats: GenerationStatistics) {
        let value = (self.stat_fn)(stats);
        if self.samples.len() >= self.samples.capacity() {
            self.samples.pop_front(); // Remove oldest sample if at capacity
        }
        self.samples.push_back(value);

        if self.samples.len() < self.samples.capacity() {
            return;
        }

        // Calculate trend by comparing latest half of samples to earlier half
        let mid = self.samples.len() / 2;
        let older: f64 = self.samples.iter().take(mid).sum::<f64>() / mid as f64;
        let newer: f64 = self.samples.iter().skip(mid).sum::<f64>() / (self.samples.len() - mid) as f64;
        
        // Adjust rate based on trend
        if newer > older {
            self.current = (self.current - self.limits.step).max(*self.limits.range.start());
        } else {
            self.current = (self.current + self.limits.step).min(*self.limits.range.end());
        }
    }
    
    pub fn mutate(&mut self, factor: f64, stats: CostCoefficients, rng: &mut impl Rng) -> CostCoefficients {
        stats.mutate(self.current * factor, rng)
    }
}

