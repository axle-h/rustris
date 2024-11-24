use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum::RGBA8888;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

pub trait TextureQuery {
    fn size(&self) -> (u32, u32);
}

impl TextureQuery for Texture<'_> {
    fn size(&self) -> (u32, u32) {
        let query = self.query();
        (query.width, query.height)
    }
}

pub trait TextureFactory {
    fn create_texture_target_blended(&self, width: u32, height: u32) -> Result<Texture, String>;
    fn load_texture_bytes_blended(&self, buf: &[u8]) -> Result<Texture, String>;
}

impl TextureFactory for TextureCreator<WindowContext> {
    fn create_texture_target_blended(&self, width: u32, height: u32) -> Result<Texture, String> {
        let mut texture = self
            .create_texture_target(RGBA8888, width, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        Ok(texture)
    }

    fn load_texture_bytes_blended(&self, buf: &[u8]) -> Result<Texture, String> {
        let mut texture = self.load_texture_bytes(buf)?;
        texture.set_blend_mode(BlendMode::Blend);
        Ok(texture)
    }
}

pub trait CanvasRenderer {
    fn clear_0(&mut self);
}

impl CanvasRenderer for Canvas<Window> {
    fn clear_0(&mut self) {
        let draw_color = self.draw_color();
        self.set_draw_color(Color::RGBA(0, 0, 0, 0));
        self.clear();
        self.set_draw_color(draw_color);
    }
}