use std::time::Duration;

const ACCELERATION: f64 = 10.0;
const SPEED: f64 = 5.0;
const LIMIT: f64 = 0.25;

#[derive(Clone, Copy, Debug)]
pub enum State {
    Rest,
    Impacted { acceleration: f64, speed: f64 },
    Return { acceleration: f64, speed: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct ImpactAnimation {
    offset_x: f64,
    offset_y: f64,
    state: State,
}

impl ImpactAnimation {
    pub fn new() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            state: State::Rest,
        }
    }

    pub fn impact(&mut self) {
        self.state = match self.state {
            State::Rest | State::Return { .. } => State::Impacted {
                acceleration: ACCELERATION,
                speed: SPEED,
            },
            State::Impacted {
                acceleration,
                speed,
            } => State::Impacted {
                acceleration: acceleration + ACCELERATION,
                speed: speed + SPEED,
            },
        };
    }

    pub fn next_offset(&mut self, duration: Duration) -> (f64, f64) {
        if duration.is_zero() {
            return (self.offset_x, self.offset_y);
        }
        self.state = match self.state {
            State::Rest => State::Rest,
            State::Impacted {
                acceleration,
                speed,
            } => {
                let duration_secs = duration.as_secs_f64();
                let speed = speed + acceleration * duration_secs;
                self.offset_y += speed * duration_secs;
                if self.offset_y > LIMIT {
                    State::Return {
                        acceleration,
                        speed: 0.0,
                    }
                } else {
                    State::Impacted {
                        acceleration,
                        speed,
                    }
                }
            }
            State::Return {
                acceleration,
                speed,
            } => {
                let duration_secs = duration.as_secs_f64();
                let speed = speed - acceleration * duration_secs;
                self.offset_y += speed * duration_secs;
                if self.offset_y <= 0.0 {
                    self.offset_y = 0.0;
                    State::Rest
                } else {
                    State::Return {
                        acceleration,
                        speed,
                    }
                }
            }
        };
        (self.offset_x, self.offset_y)
    }
}
