use crate::config::Config;
use crate::menu_input::MenuInputKey;
use crate::theme::sound::{load_sound, play_sound};

use crate::build_info::BUILD_INFO;
use sdl2::image::LoadTexture;
use sdl2::mixer::Chunk;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use std::cmp::max;
use std::collections::HashMap;
use crate::font::FontType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction<'a> {
    Select,
    SelectList { items: Vec<&'a str>, current: usize },
}

pub type MenuItem<'a> = (&'a str, MenuAction<'a>);

pub struct Menu<'a> {
    menu_items: Vec<MenuItem<'a>>,
    vertical_gutter: u32,
    horizontal_gutter: u32,
    caret_gutter: u32,
    current_row_id: usize,
    string_textures: HashMap<&'a str, Texture<'a>>,
    body_texture: Texture<'a>,
    caret_texture: Texture<'a>,
    watermark_texture: Texture<'a>,
    watermark_rect: Rect,
    caret_size: u32,
    name_width: u32,
    row_height: u32,
    menu_sound: Chunk,
}

fn maybe_add_texture<'a>(
    font: &Font,
    text: &'a str,
    texture_creator: &'a TextureCreator<WindowContext>,
    string_textures: &mut HashMap<&'a str, Texture<'a>>,
) -> Result<(u32, u32), String> {
    let query = match string_textures.get(text) {
        None => {
            let surface = font
                .render(text)
                .blended(Color::WHITE)
                .map_err(|e| e.to_string())?;
            let texture = texture_creator
                .create_texture_from_surface(surface)
                .map_err(|e| e.to_string())?;
            let query = texture.query();
            string_textures.insert(text, texture);
            query
        }
        Some(texture) => texture.query(),
    };
    Ok((query.width, query.height))
}

impl<'a> Menu<'a> {
    pub fn new(
        menu_items: Vec<MenuItem<'a>>,
        ttf: &Sdl2TtfContext,
        texture_creator: &'a TextureCreator<WindowContext>,
        config: Config,
        window_size: (u32, u32),
    ) -> Result<Self, String> {
        assert!(!menu_items.is_empty());

        let mut caret_texture = texture_creator.load_texture("resource/menu/caret.png")?;
        caret_texture.set_blend_mode(BlendMode::Blend);

        let mut string_textures: HashMap<&'a str, Texture<'a>> = HashMap::new();
        let (window_width, window_height) = window_size;
        let font_size = window_width / 32;
        let font = FontType::Bold.load(ttf, font_size)?;

        let vertical_gutter = font_size / 3;
        let horizontal_gutter = font_size;
        let caret_gutter = font_size / 3;

        let mut max_action_width = 0;
        let mut max_name_width = 0;
        let mut row_height = 0;
        for (name, action) in menu_items.iter() {
            let (name_width, name_height) =
                maybe_add_texture(&font, name, texture_creator, &mut string_textures)?;
            let (action_width, action_height) = match action {
                MenuAction::Select => (0, 0),
                MenuAction::SelectList { items, .. } => {
                    let mut width = 0;
                    let mut height = 0;
                    for text in items.iter() {
                        let (w, h) =
                            maybe_add_texture(&font, text, texture_creator, &mut string_textures)?;
                        width = max(width, w);
                        height = max(height, h);
                    }
                    (width, height)
                }
            };
            max_name_width = max(max_name_width, name_width);
            max_action_width = max(max_action_width, action_width);
            row_height = max(row_height, max(name_height, action_height));
        }

        let caret_size = max(caret_texture.query().width, row_height) / 2;
        let body_height =
            row_height * menu_items.len() as u32 + vertical_gutter * (menu_items.len() - 1) as u32;
        let body_width =
            caret_size + caret_gutter + max_name_width + horizontal_gutter + max_action_width;
        let mut body_texture = texture_creator
            .create_texture_target(None, body_width, body_height)
            .map_err(|e| e.to_string())?;
        body_texture.set_blend_mode(BlendMode::Blend);

        let watermark_font_size = font_size / 2;
        let watermark_font = FontType::Bold.load(ttf, watermark_font_size)?;
        let watermark_surface = watermark_font
            .render(BUILD_INFO)
            .blended(Color::WHITE)
            .map_err(|e| e.to_string())?;
        let watermark_rect = Rect::new(
            (window_width - watermark_surface.width() - watermark_font_size) as i32,
            (window_height - watermark_surface.height() - watermark_font_size) as i32,
            watermark_surface.width(),
            watermark_surface.height(),
        );
        let mut watermark_texture = texture_creator
            .create_texture_from_surface(watermark_surface)
            .map_err(|e| e.to_string())?;
        watermark_texture.set_blend_mode(BlendMode::Blend);

        let result = Self {
            menu_items,
            vertical_gutter,
            horizontal_gutter,
            caret_gutter,
            current_row_id: 0,
            string_textures,
            body_texture,
            caret_texture,
            watermark_texture,
            watermark_rect,
            caret_size,
            name_width: max_name_width,
            row_height,
            menu_sound: load_sound("menu", "chime", config)?
        };
        Ok(result)
    }

    pub fn play_sound(&self) -> Result<(), String> {
        play_sound(&self.menu_sound)
    }

    pub fn up(&mut self) {
        self.current_row_id = match self.current_row_id {
            0 => self.menu_items.len() - 1,
            _ => self.current_row_id - 1,
        };
    }

    pub fn down(&mut self) {
        self.current_row_id += 1;
        self.current_row_id %= self.menu_items.len();
    }

    pub fn left(&mut self) -> Option<(&str, &str)> {
        self.direction(-1)
    }

    pub fn right(&mut self) -> Option<(&str, &str)> {
        self.direction(1)
    }

    pub fn read_key(&mut self, key: MenuInputKey) -> Option<(&str, &str)> {
        match key {
            MenuInputKey::Up => {
                self.up();
                None
            }
            MenuInputKey::Down => {
                self.down();
                None
            }
            MenuInputKey::Left => self.left(),
            MenuInputKey::Right => self.right(),
            MenuInputKey::Select => self.select(),
            _ => None,
        }
    }

    fn direction(&mut self, direction: i32) -> Option<(&str, &str)> {
        let (name, action) = self.menu_items.get(self.current_row_id).unwrap();
        let name = *name;
        let maybe_result = match action {
            MenuAction::SelectList { items, current } => {
                let current_plus = *current as i32 + direction;
                let next_current = if current_plus < 0 {
                    items.len() - 1
                } else {
                    (current_plus as usize) % items.len()
                };
                let next_action = MenuAction::SelectList {
                    items: items.to_vec(),
                    current: next_current,
                };
                Some((next_action, items[next_current]))
            }
            _ => None,
        };
        match maybe_result {
            None => None,
            Some((next_action, result)) => {
                self.menu_items[self.current_row_id] = (name, next_action);
                Some((name, result))
            }
        }
    }

    pub fn select(&mut self) -> Option<(&str, &str)> {
        let (name, action) = self.menu_items.get(self.current_row_id).unwrap();
        let name = *name;
        let (next_action, result) = match action {
            MenuAction::Select => (MenuAction::Select, ""),
            MenuAction::SelectList { items, current } => {
                let next_current = (*current + 1) % items.len();
                (
                    MenuAction::SelectList {
                        items: items.to_vec(),
                        current: next_current,
                    },
                    items[next_current],
                )
            }
        };
        self.menu_items[self.current_row_id] = (name, next_action);
        Some((name, result))
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas
            .with_texture_canvas(&mut self.body_texture, |tc| {
                tc.set_draw_color(Color::RGBA(0, 0, 0, 0));
                tc.clear();
                let mut y = 0;
                for (row_id, (name, action)) in self.menu_items.iter().enumerate() {
                    let mut x = 0u32;
                    if row_id == self.current_row_id {
                        let caret_rect =
                            Rect::new(x as i32, y as i32, self.caret_size, self.row_height);
                        let rect = Rect::from_center(
                            caret_rect.center(),
                            self.caret_size,
                            self.caret_size,
                        );
                        tc.copy(&self.caret_texture, None, rect).unwrap();
                    }
                    x += self.caret_size + self.caret_gutter;

                    let name_texture = self.string_textures.get(name).unwrap();
                    let name_query = name_texture.query();
                    tc.copy(
                        name_texture,
                        None,
                        Rect::new(x as i32, y as i32, name_query.width, name_query.height),
                    )
                    .unwrap();
                    x += self.name_width + self.horizontal_gutter;

                    if let MenuAction::SelectList { items, current } = action {
                        let selected = items[*current];
                        let texture = self.string_textures.get(selected).unwrap();
                        let query = texture.query();
                        let rect = Rect::new(x as i32, y as i32, query.width, query.height);
                        tc.copy(texture, None, rect).unwrap();
                    }

                    y += self.row_height + self.vertical_gutter;
                }
            })
            .map_err(|e| e.to_string())?;

        let (window_width, window_height) = canvas.window().size();
        let window_center = Rect::new(0, 0, window_width, window_height).center();
        let query = self.body_texture.query();
        let rect = Rect::from_center(window_center, query.width, query.height);
        canvas.copy(&self.body_texture, None, rect)?;

        canvas.copy(&self.watermark_texture, None, self.watermark_rect)
    }
}
