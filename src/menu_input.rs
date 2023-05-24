use crate::config::InputConfig;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuInputKey {
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
    Quit,
}

pub struct MenuInputContext {
    mapping: HashMap<Keycode, MenuInputKey>,
}

impl MenuInputContext {
    pub fn new(config: InputConfig) -> Self {
        Self {
            mapping: config.menu_map(),
        }
    }

    pub fn parse<I>(&self, sdl_events: I) -> Vec<MenuInputKey>
    where
        I: Iterator<Item = Event>,
    {
        let mut result: Vec<MenuInputKey> = vec![];
        for event in sdl_events {
            let maybe_key = match event {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => self.mapping.get(&keycode).copied(),
                Event::Quit { .. } => Some(MenuInputKey::Quit),
                _ => None,
            };
            if let Some(key) = maybe_key {
                result.push(key)
            }
        }
        result
    }
}
