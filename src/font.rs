use std::collections::HashMap;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;

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

pub struct FontTexture<'a> {
    pub texture: Texture<'a>,
    pub width: u32,
    pub height: u32
}

impl<'a> FontTexture<'a> {
    pub fn new(
        font: &Font,
        texture_creator: &'a TextureCreator<WindowContext>,
        text: &str,
        color: Color
    ) -> Result<Self, String> {
        let surface = font
            .render(text)
            .blended(color)
            .map_err(|e| e.to_string())?;
        let mut texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        let query = texture.query();

        Ok(Self {
            texture,
            width: query.width,
            height: query.height
        })
    }
}