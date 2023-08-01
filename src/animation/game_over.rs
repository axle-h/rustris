use crate::game::board::BOARD_HEIGHT;
use std::ops::Range;
use std::time::Duration;

const CURTAIN_LINE_DELAY: Duration = Duration::from_millis(30);
const CURTAIN_CLOSED_FOR: Duration = Duration::from_millis(2000);
const CURTAIN_OPEN_FOR: Duration = Duration::from_millis(2000);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOverAnimationType {
    CurtainUp,
    CurtainDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameOverAnimate {
    CurtainClosing(Range<u32>),
    CurtainOpening(Range<u32>),
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    CloseCurtain(Duration, u32),
    CurtainClosed(Duration),
    OpenCurtain(Duration, u32),
    CurtainOpen(Duration),
    Finished,
}

pub struct GameOverAnimation {
    animation_type: GameOverAnimationType,
    state: State,
}

impl GameOverAnimation {
    pub fn new(animation_type: GameOverAnimationType) -> Self {
        Self {
            animation_type,
            state: State::CloseCurtain(Duration::ZERO, 0),
        }
    }

    pub fn update(&mut self, delta: Duration) -> GameOverAnimate {
        self.state = match self.state {
            State::Finished => State::Finished,
            State::CloseCurtain(duration, count) => self.close_curtain(duration + delta, count),
            State::CurtainClosed(duration) => self.curtain_closed(duration + delta),
            State::OpenCurtain(duration, count) => self.open_curtain(duration + delta, count),
            State::CurtainOpen(duration) => self.curtain_open(duration + delta),
        };
        self.current()
    }

    pub fn current(&self) -> GameOverAnimate {
        match self.state {
            State::CloseCurtain(_, count) => {
                let range = match self.animation_type {
                    GameOverAnimationType::CurtainUp => 0..count,
                    GameOverAnimationType::CurtainDown => (BOARD_HEIGHT - count)..BOARD_HEIGHT,
                };
                GameOverAnimate::CurtainClosing(range)
            }
            State::CurtainClosed(_) => GameOverAnimate::CurtainClosing(0..BOARD_HEIGHT),
            State::OpenCurtain(_, count) => {
                let range = match self.animation_type {
                    GameOverAnimationType::CurtainUp => 0..(BOARD_HEIGHT - count),
                    GameOverAnimationType::CurtainDown => count..BOARD_HEIGHT,
                };
                GameOverAnimate::CurtainOpening(range)
            }
            State::CurtainOpen(_) => GameOverAnimate::CurtainOpening(0..0),
            State::Finished => GameOverAnimate::Finished,
        }
    }

    fn close_curtain(&self, duration: Duration, count: u32) -> State {
        if duration < CURTAIN_LINE_DELAY {
            State::CloseCurtain(duration, count)
        } else {
            let count = count + 1;
            if count > BOARD_HEIGHT {
                State::CurtainClosed(Duration::ZERO)
            } else {
                State::CloseCurtain(Duration::ZERO, count)
            }
        }
    }

    fn curtain_closed(&self, duration: Duration) -> State {
        if duration < CURTAIN_CLOSED_FOR {
            State::CurtainClosed(duration)
        } else {
            State::OpenCurtain(Duration::ZERO, 0)
        }
    }

    fn open_curtain(&self, duration: Duration, count: u32) -> State {
        if duration < CURTAIN_LINE_DELAY {
            State::OpenCurtain(duration, count)
        } else {
            let count = count + 1;
            if count > BOARD_HEIGHT {
                State::CurtainOpen(Duration::ZERO)
            } else {
                State::OpenCurtain(Duration::ZERO, count)
            }
        }
    }

    fn curtain_open(&self, duration: Duration) -> State {
        if duration < CURTAIN_OPEN_FOR {
            State::CurtainOpen(duration)
        } else {
            State::Finished
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curtain_up_init() {
        let subject = GameOverAnimation::new(GameOverAnimationType::CurtainUp);
        assert_eq!(subject.current(), GameOverAnimate::CurtainClosing(0..0))
    }

    #[test]
    fn curtain_down_init() {
        let subject = GameOverAnimation::new(GameOverAnimationType::CurtainDown);
        assert_eq!(
            subject.current(),
            GameOverAnimate::CurtainClosing(BOARD_HEIGHT..BOARD_HEIGHT)
        )
    }

    #[test]
    fn curtain_up_closes_curtain() {
        let mut subject = GameOverAnimation::new(GameOverAnimationType::CurtainUp);
        for _ in 0..BOARD_HEIGHT {
            subject.update(CURTAIN_LINE_DELAY);
        }
        assert_eq!(
            subject.current(),
            GameOverAnimate::CurtainClosing(0..BOARD_HEIGHT)
        )
    }

    #[test]
    fn curtain_down_closes_curtain() {
        let mut subject = GameOverAnimation::new(GameOverAnimationType::CurtainDown);
        for _ in 0..BOARD_HEIGHT {
            subject.update(CURTAIN_LINE_DELAY);
        }
        assert_eq!(
            subject.current(),
            GameOverAnimate::CurtainClosing(0..BOARD_HEIGHT)
        )
    }

    #[test]
    fn curtain_up_opens_after_closing() {
        let mut subject = GameOverAnimation::new(GameOverAnimationType::CurtainUp);
        for _ in 0..(BOARD_HEIGHT + 1) {
            subject.update(CURTAIN_LINE_DELAY);
        }
        subject.update(CURTAIN_CLOSED_FOR);
        assert_eq!(
            subject.current(),
            GameOverAnimate::CurtainOpening(0..BOARD_HEIGHT)
        )
    }

    #[test]
    fn curtain_down_opens_after_closing() {
        let mut subject = GameOverAnimation::new(GameOverAnimationType::CurtainDown);
        for _ in 0..(BOARD_HEIGHT + 1) {
            subject.update(CURTAIN_LINE_DELAY);
        }
        subject.update(CURTAIN_CLOSED_FOR);
        assert_eq!(
            subject.current(),
            GameOverAnimate::CurtainOpening(0..BOARD_HEIGHT)
        )
    }

    #[test]
    fn curtain_up_finished_after_curtain_open() {
        let mut subject = GameOverAnimation::new(GameOverAnimationType::CurtainUp);
        for _ in 0..(BOARD_HEIGHT + 1) {
            subject.update(CURTAIN_LINE_DELAY);
        }
        subject.update(CURTAIN_CLOSED_FOR);
        for _ in 0..(BOARD_HEIGHT + 1) {
            subject.update(CURTAIN_LINE_DELAY);
        }
        subject.update(CURTAIN_OPEN_FOR);
        assert_eq!(subject.current(), GameOverAnimate::Finished)
    }

    #[test]
    fn curtain_down_finished_after_curtain_open() {
        let mut subject = GameOverAnimation::new(GameOverAnimationType::CurtainDown);
        for _ in 0..(BOARD_HEIGHT + 1) {
            subject.update(CURTAIN_LINE_DELAY);
        }
        subject.update(CURTAIN_CLOSED_FOR);
        for _ in 0..(BOARD_HEIGHT + 1) {
            subject.update(CURTAIN_LINE_DELAY);
        }
        subject.update(CURTAIN_OPEN_FOR);
        assert_eq!(subject.current(), GameOverAnimate::Finished)
    }
}
