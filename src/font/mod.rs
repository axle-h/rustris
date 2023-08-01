use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;

const FONT_ROBOTO_REGULAR: &[u8] = include_bytes!("Roboto-Regular.ttf");
const FONT_ROBOTO_BOLD: &[u8] = include_bytes!("Roboto-Bold.ttf");
const FONT_ROBOTO_MONO_REGULAR: &[u8] = include_bytes!("RobotoMono-Regular.ttf");
const FONT_HANDJET: &[u8] = include_bytes!("Handjet.ttf");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FontType {
    Normal,
    Bold,
    Mono,
    Handjet,
}

impl FontType {
    pub fn load<'ttf>(
        &self,
        ttf: &'ttf Sdl2TtfContext,
        size: u32,
    ) -> Result<Font<'ttf, 'ttf>, String> {
        ttf.load_font_from_rwops(RWops::from_bytes(self.bytes())?, size as u16)
    }

    fn bytes(&self) -> &'static [u8] {
        match self {
            FontType::Normal => FONT_ROBOTO_REGULAR,
            FontType::Bold => FONT_ROBOTO_BOLD,
            FontType::Mono => FONT_ROBOTO_MONO_REGULAR,
            FontType::Handjet => FONT_HANDJET,
        }
    }
}

pub struct FontTexture<'a> {
    pub texture: Texture<'a>,
    pub width: u32,
    pub height: u32,
}

impl<'a> FontTexture<'a> {
    pub fn from_string(
        font: &Font,
        texture_creator: &'a TextureCreator<WindowContext>,
        text: &str,
        color: Color,
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
            height: query.height,
        })
    }

    pub fn from_char(
        font: &Font,
        texture_creator: &'a TextureCreator<WindowContext>,
        ch: char,
        color: Color,
    ) -> Result<Self, String> {
        let surface = font
            .render_char(ch)
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
            height: query.height,
        })
    }
}
