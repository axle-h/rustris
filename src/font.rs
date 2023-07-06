use std::collections::HashMap;
use sdl2::ttf::{Font, Sdl2TtfContext};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FontType {
    Normal,
    Bold,
    Mono
}

impl FontType {
    pub fn load<'ttf>(&self, ttf: &'ttf Sdl2TtfContext, size: u32) -> Result<Font<'ttf, 'ttf>, String> {
        ttf.load_font(self.path(), size as u16)
    }

    fn path(&self) -> &str {
        match self {
            FontType::Normal => "resource/font/Roboto-Regular.ttf",
            FontType::Bold => "resource/font/Roboto-Bold.ttf",
            FontType::Mono => "resource/font/RobotoMono-Regular.ttf"
        }
    }
}