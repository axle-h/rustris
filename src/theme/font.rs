use crate::font::{FontTexture, FontType};
use num_format::{Locale, ToFormattedString};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FontAlign {
    Left { zero_fill: bool },
    Right,
}

impl FontAlign {
    fn is_right(&self) -> bool {
        self == &FontAlign::Right
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MetricSnips {
    point: Point,
    max_value: u32,
    max_chars: u32,
    align: FontAlign,
}

fn char_length(value: u32) -> u32 {
    format!("{}", value).len() as u32
}

impl MetricSnips {
    pub fn right<P: Into<Point>>(point: P, max_value: u32) -> Self {
        Self {
            point: point.into(),
            max_value,
            max_chars: char_length(max_value),
            align: FontAlign::Right,
        }
    }

    pub fn zero_fill<P: Into<Point>>(point: P, max_value: u32) -> Self {
        Self {
            point: point.into(),
            max_value,
            max_chars: char_length(max_value),
            align: FontAlign::Left { zero_fill: true },
        }
    }

    pub fn left<P: Into<Point>>(point: P, max_value: u32) -> Self {
        Self {
            point: point.into(),
            max_value,
            max_chars: char_length(max_value),
            align: FontAlign::Left { zero_fill: false },
        }
    }

    pub fn offset(&self, x: i32, y: i32) -> Self {
        Self {
            point: self.point.offset(x, y),
            max_value: self.max_value,
            max_chars: self.max_chars,
            align: self.align,
        }
    }

    fn zero_fill_chars(&self) -> Option<u32> {
        match self.align {
            FontAlign::Left { zero_fill } if zero_fill => Some(self.max_chars),
            _ => None,
        }
    }

    pub fn point(&self) -> Point {
        self.point
    }

    pub fn max_value(&self) -> u32 {
        self.max_value
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct FontSprite {
    value: char,
    snip: Rect,
}

impl From<(char, Rect)> for FontSprite {
    fn from((value, snip): (char, Rect)) -> Self {
        Self { value, snip }
    }
}

impl FontSprite {
    pub fn new(value: char, snip: Rect) -> Self {
        Self { value, snip }
    }
}

pub enum FontRenderOptions {
    Sprites {
        file_bytes: &'static [u8],
        sprites: Vec<FontSprite>,
        spacing: u32,
    },
}

impl FontRenderOptions {
    pub fn build<'a>(
        &self,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> Result<FontRender<'a>, String> {
        match self {
            FontRenderOptions::Sprites {
                file_bytes,
                sprites,
                spacing,
            } => FontRender::from_sprites(texture_creator, file_bytes, sprites.clone(), *spacing),
        }
    }
}

pub fn alpha_sprites(snips: [Point; 10], width: u32, height: u32) -> Vec<FontSprite> {
    snips
        .iter()
        .enumerate()
        .map(|(i, p)| {
            FontSprite::new(
                char::from_u32('0' as u32 + i as u32).unwrap(),
                Rect::new(p.x(), p.y(), width, height),
            )
        })
        .collect()
}

pub struct FontRender<'a> {
    texture: Texture<'a>,
    sprites: HashMap<char, Rect>,
    spacing: u32,
}

impl<'a> FontRender<'a> {
    pub fn from_sprites(
        texture_creator: &'a TextureCreator<WindowContext>,
        sprite_file: &'static [u8],
        sprites: Vec<FontSprite>,
        spacing: u32,
    ) -> Result<Self, String> {
        let mut texture = texture_creator.load_texture_bytes(sprite_file)?;
        texture.set_blend_mode(BlendMode::Blend);
        let sprites = sprites.iter().map(|&x| (x.value, x.snip)).collect();
        Ok(Self {
            texture,
            sprites,
            spacing,
        })
    }

    pub fn from_font(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf: &Sdl2TtfContext,
        font_type: FontType,
        size: u32,
        color: Color,
    ) -> Result<Self, String> {
        let font = font_type.load(ttf, size)?;

        let chars = ('A'..='Z')
            .chain('a'..='z')
            .chain('0'..='9')
            .chain([' ', ',', '.'])
            .map(|c| {
                (
                    c,
                    FontTexture::from_char(&font, texture_creator, c, color).unwrap(),
                )
            })
            .collect::<Vec<(char, FontTexture)>>();

        let width: u32 = chars.iter().map(|(_, t)| t.width).sum();
        let height = chars.iter().map(|(_, t)| t.height).max().unwrap();

        let mut texture = texture_creator
            .create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);

        let mut sprites = HashMap::new();
        let mut x = 0;
        canvas
            .with_texture_canvas(&mut texture, |c| {
                c.set_draw_color(Color::RGBA(0, 0, 0, 0));
                c.clear();
                for (ch, font_texture) in chars {
                    let snip = Rect::new(x, 0, font_texture.width, font_texture.height);
                    x += font_texture.width as i32;
                    c.copy(&font_texture.texture, None, snip).unwrap();
                    sprites.insert(ch, snip);
                }
            })
            .map_err(|e| e.to_string())?;

        Ok(Self {
            texture,
            sprites,
            spacing: 0,
        })
    }

    pub fn render_string(
        &self,
        canvas: &mut WindowCanvas,
        dest: Point,
        value: &str,
    ) -> Result<(), String> {
        let mut dest = dest;
        for ch in value.chars() {
            let snip = self.sprite(ch);
            let rect = Rect::new(dest.x(), dest.y(), snip.width(), snip.height());
            canvas.copy(&self.texture, snip, rect)?;
            dest += Point::new((snip.width() + self.spacing) as i32, 0);
        }
        Ok(())
    }

    pub fn render_number(
        &self,
        canvas: &mut WindowCanvas,
        meta: MetricSnips,
        value: u32,
    ) -> Result<(), String> {
        if meta.align.is_right() {
            self.render_number_right(canvas, value, meta)
        } else {
            self.render_number_left(canvas, value, meta)
        }
    }

    fn render_number_left(
        &self,
        canvas: &mut WindowCanvas,
        value: u32,
        meta: MetricSnips,
    ) -> Result<(), String> {
        let chars = self.format_number(value, meta.max_value, meta.zero_fill_chars());
        self.render_string(canvas, meta.point, &chars)
    }

    fn render_number_right(
        &self,
        canvas: &mut WindowCanvas,
        value: u32,
        meta: MetricSnips,
    ) -> Result<(), String> {
        let chars = self.format_number(value, meta.max_value, None);
        let mut dest = meta.point;
        for ch in chars.chars().rev() {
            let snip = self.sprite(ch);
            dest -= Point::new(snip.width() as i32, 0);
            let rect = Rect::new(dest.x(), dest.y(), snip.width(), snip.height());
            canvas.copy(&self.texture, snip, rect)?;
            dest -= Point::new(self.spacing as i32, 0);
        }
        Ok(())
    }

    pub fn number_size(&self, value: u32) -> (u32, u32) {
        let chars = self.format_number(value, u32::MAX, None);
        self.string_size(&chars)
    }

    pub fn string_size(&self, value: &str) -> (u32, u32) {
        if value.is_empty() {
            return (0, 0);
        }
        let sprites = value
            .chars()
            .map(|ch| self.sprite(ch))
            .collect::<Vec<Rect>>();
        let total_spacing = (value.len() - 1) as u32 * self.spacing;
        let width = sprites.iter().map(|s| s.width()).sum::<u32>();
        let height = sprites.iter().map(|s| s.height()).max().unwrap();
        (total_spacing + width, height)
    }

    fn sprite(&self, ch: char) -> Rect {
        *self
            .sprites
            .get(&ch)
            .unwrap_or_else(|| panic!("{} is not supported by this font render", ch))
    }

    fn format_number(&self, value: u32, max_value: u32, zero_fill: Option<u32>) -> String {
        let value = value.min(max_value);
        let chars = if self.sprites.contains_key(&',') {
            value.to_formatted_string(&Locale::en)
        } else {
            format!("{}", value)
        };
        if let Some(max_chars) = zero_fill {
            let fill_len = max_chars - chars.len() as u32;
            let mut result: String = (0..fill_len).map(|_| '0').collect();
            result.push_str(&chars);
            result
        } else {
            chars
        }
    }
}
