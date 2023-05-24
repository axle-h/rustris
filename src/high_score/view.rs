use crate::high_score::table::HighScoreTable;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use std::cmp::max;

struct SizedTexture<'a> {
    texture: Texture<'a>,
    width: u32,
    height: u32,
}

struct HighScoreTableRow<'a> {
    ordinal: SizedTexture<'a>,
    name: SizedTexture<'a>,
    score: SizedTexture<'a>,
}

pub struct HighScoreTableView<'a> {
    rows: Vec<HighScoreTableRow<'a>>,
    texture: Texture<'a>,
    row_height: u32,
    ordinal_column_width: u32,
    padding: u32,
    width: u32,
    rect: Rect,
}

fn create_font_texture<'a>(
    font: &Font,
    texture_creator: &'a TextureCreator<WindowContext>,
    text: &str,
) -> Result<SizedTexture<'a>, String> {
    let surface = font
        .render(text)
        .blended(Color::WHITE)
        .map_err(|e| e.to_string())?;
    let texture = texture_creator
        .create_texture_from_surface(surface)
        .map_err(|e| e.to_string())?;
    let query = texture.query();
    Ok(SizedTexture {
        texture,
        width: query.width,
        height: query.height,
    })
}

impl<'a> HighScoreTableView<'a> {
    pub fn new(
        table: HighScoreTable,
        ttf: &Sdl2TtfContext,
        texture_creator: &'a TextureCreator<WindowContext>,
        window_size: (u32, u32),
    ) -> Result<Self, String> {
        let (window_width, window_height) = window_size;
        let font_size = window_width / 32;
        let font = ttf.load_font("resource/menu/Roboto-Bold.ttf", font_size as u16)?;

        let mut rows = vec![];
        let mut row_height = 0;
        for (i, entry) in table.entries().iter().enumerate() {
            let name_texture = create_font_texture(&font, texture_creator, &entry.name)?;
            let score_texture =
                create_font_texture(&font, texture_creator, &entry.score.to_string())?;
            let ordinal_texture =
                create_font_texture(&font, texture_creator, &(i + 1).to_string())?;
            row_height = max(
                row_height,
                max(
                    name_texture.height,
                    max(score_texture.height, ordinal_texture.height),
                ),
            );
            let row = HighScoreTableRow {
                ordinal: ordinal_texture,
                name: name_texture,
                score: score_texture,
            };
            rows.push(row);
        }

        let n_rows = rows.len() as u32;
        if n_rows == 0 {
            return Err("no high scores".to_string());
        }

        let ordinal_column_width = rows.iter().map(|x| x.ordinal.width).max().unwrap();
        let name_column_width = rows.iter().map(|x| x.name.width).max().unwrap();
        let score_column_width = rows.iter().map(|x| x.score.width).max().unwrap();
        let padding = font_size / 2;
        let width =
            ordinal_column_width + padding + name_column_width + padding + score_column_width;
        let height = n_rows * row_height + (n_rows - 1) * padding;
        let texture = texture_creator
            .create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;
        let rect = Rect::from_center(
            Point::new(window_width as i32 / 2, window_height as i32 / 2),
            width,
            height,
        );

        Ok(Self {
            rows,
            texture,
            row_height,
            width,
            padding,
            ordinal_column_width,
            rect,
        })
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        canvas
            .with_texture_canvas(&mut self.texture, |c| {
                let mut y = 0;
                for row in self.rows.iter() {
                    c.copy(
                        &row.ordinal.texture,
                        None,
                        Rect::new(0, y, row.ordinal.width, row.ordinal.height),
                    )
                    .unwrap();
                    c.copy(
                        &row.name.texture,
                        None,
                        Rect::new(
                            (self.ordinal_column_width + self.padding) as i32,
                            y,
                            row.name.width,
                            row.name.height,
                        ),
                    )
                    .unwrap();
                    c.copy(
                        &row.score.texture,
                        None,
                        Rect::new(
                            self.width as i32 - row.score.width as i32,
                            y,
                            row.score.width,
                            row.score.height,
                        ),
                    )
                    .unwrap();
                    y += self.row_height as i32;
                }
            })
            .map_err(|e| e.to_string())?;
        canvas.copy(&self.texture, None, self.rect)
    }
}
