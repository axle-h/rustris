use std::ops::{Add, Div, Mul, Sub};
use std::time::Duration;

pub struct Timing {
    target_fps: u32,
    frame_duration: Duration
}

impl Timing {
    pub fn new(target_fps: u32) -> Self {
        Self { target_fps, frame_duration: Duration::from_secs(1) / target_fps }
    }

    pub fn frame_duration(&self, frames: u32) -> Duration {
        self.frame_duration * frames
    }
}