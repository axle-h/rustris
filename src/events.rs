use std::collections::HashMap;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEventKey {
    MoveLeft { player: u32 },
    MoveRight { player: u32 },
    SoftDrop { player: u32 },
    HardDrop { player: u32 },
    RotateClockwise { player: u32 },
    RotateAnticlockwise { player: u32 },
    Hold { player: u32 },
    Pause,
    Quit
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
struct GameEvent {
    key: GameEventKey,
    duration: Duration,
    repeating: bool
}

impl GameEvent {
    fn new(key: GameEventKey) -> Self {
        Self { key, duration: Duration::ZERO, repeating: false }
    }
}

type KeyMapping = HashMap<Keycode, GameEventKey>;

pub struct Events {
    mapping: KeyMapping,
    current: HashMap<GameEventKey, GameEvent>
}

enum MaybeKey {
    Down(GameEventKey),
    Up(GameEventKey),
    None
}

const AUTO_REPEAT_DELAY: Duration = Duration::from_millis(300);
const AUTO_REPEAT_ITERATION: Duration = Duration::from_millis(25);

impl Events {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::from([
                (Keycode::Escape, GameEventKey::Quit),
                (Keycode::Left, GameEventKey::MoveLeft { player: 1 }),
                (Keycode::Right, GameEventKey::MoveRight { player: 1 }),
                (Keycode::Down, GameEventKey::SoftDrop { player: 1 }),
                (Keycode::Up, GameEventKey::HardDrop { player: 1 }),
                (Keycode::Z, GameEventKey::RotateAnticlockwise { player: 1 }),
                (Keycode::X, GameEventKey::RotateClockwise { player: 1 }),
                (Keycode::F1, GameEventKey::Pause),
                (Keycode::RShift, GameEventKey::Hold { player: 1 }),
                (Keycode::LShift, GameEventKey::Hold { player: 1 }),
            ]),
            current: HashMap::new()
        }
    }

    pub fn update<I>(&mut self, delta: Duration, sdl_events: I) -> Vec<GameEventKey>
        where I: Iterator<Item = Event> {
        let mut result: Vec<GameEventKey> = vec![];

        // update any keys that might still be held with the delta
        for event in self.current.values_mut() {
            event.duration += delta;
        }

        for sdl_event in sdl_events {
            match self.map_from_sdl_event(sdl_event) {
                MaybeKey::None => {},
                MaybeKey::Down(key) => {
                    let event = GameEvent::new(key);
                    self.current.insert(key, event);
                    result.push(key);
                },
                MaybeKey::Up(key) => {
                    self.current.remove(&key);
                }
            };
        }

        // check for any held keys that have triggered a repeat
        for event in self.current.values_mut() {
            match event.key {
                GameEventKey::MoveLeft { .. } | GameEventKey::MoveRight { .. } => {
                    // check auto-repeat
                    if event.repeating {
                        if event.duration >= AUTO_REPEAT_ITERATION {
                            event.duration = Duration::ZERO;
                            result.push(event.key);
                        }
                    } else if event.duration >= AUTO_REPEAT_DELAY {
                        event.duration = Duration::ZERO;
                        event.repeating = true;
                        result.push(event.key);
                    }
                }
                GameEventKey::SoftDrop { player } => {
                    result.push(GameEventKey::SoftDrop { player });
                }
                _ => {}
            }
        }

        return result;
    }

    fn map_from_sdl_event(&self, event: Event) -> MaybeKey {
        match event {
            Event::Quit { .. } => MaybeKey::Down(GameEventKey::Quit),
            Event::KeyDown {
                keycode: Some(keycode),
                repeat: false,
                ..
            } => {
                match self.mapping.get(&keycode) {
                    None => MaybeKey::None,
                    Some(key) => MaybeKey::Down(*key)
                }
            },
            Event::KeyUp {
                keycode: Some(keycode),
                repeat: false,
                ..
            } => {
                match self.mapping.get(&keycode) {
                    None => MaybeKey::None,
                    Some(key) => MaybeKey::Up(*key)
                }
            },
            _ => MaybeKey::None
        }
    }
}