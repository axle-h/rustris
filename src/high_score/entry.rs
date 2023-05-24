use crate::event::GameEvent;
use crate::high_score::table::HighScore;
use crate::high_score::NewHighScore;
use crate::scale::Scale;
use crate::theme::Theme;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::cmp::{max, min};

const NAME_CHARACTERS: usize = 5;
const VERTICAL_GUTTER: u32 = 4;
const NAME_CHAR_GUTTER: u32 = 3;
const CHAR_RECT_PADDING: u32 = 1;

pub struct HighScoreEntry<'a> {
    title: String,
    high_score: NewHighScore,
    name: [char; NAME_CHARACTERS],
    current_char: usize,
    texture: Texture<'a>,
    snip: Rect,
    title_point: Point,
    score_point: Point,
    name_rects: [Rect; NAME_CHARACTERS],
}

fn line_x(total_width: u32, line_width: u32) -> u32 {
    total_width / 2 - line_width / 2
}

impl<'a> HighScoreEntry<'a> {
    pub fn new(
        high_score: NewHighScore,
        texture_creator: &'a TextureCreator<WindowContext>,
        theme: &dyn Theme,
        scale: &Scale,
    ) -> Result<Self, String> {
        let title = format!("new high score player {}", high_score.player);

        let (title_width, title_height) = theme.string_size(title.len() as u32);
        let (score_width, score_height) =
            theme.string_size(high_score.score.to_string().len() as u32);
        let (name_width, name_height) = theme.string_size(NAME_CHARACTERS as u32);
        let width = max(title_width, max(score_width, name_width));
        let height = title_height
            + VERTICAL_GUTTER
            + score_height
            + 2 * VERTICAL_GUTTER
            + name_height
            + CHAR_RECT_PADDING;
        let mut texture = texture_creator
            .create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        let snip = scale.scaled_window_center_rect(width, height);

        let title_point = Point::new(line_x(width, title_width) as i32, 0);
        let score_point = Point::new(
            line_x(width, score_width) as i32,
            (title_height + VERTICAL_GUTTER) as i32,
        );

        let (char_width, char_height) = theme.string_size(1);
        let name_rects: [Rect; NAME_CHARACTERS] = (0..NAME_CHARACTERS as u32)
            .map(|i| {
                Rect::new(
                    (line_x(width, name_width) + i * (char_width + NAME_CHAR_GUTTER)) as i32,
                    (title_height + VERTICAL_GUTTER + score_height + 2 * VERTICAL_GUTTER) as i32,
                    char_width,
                    char_height,
                )
            })
            .collect::<Vec<Rect>>()
            .try_into()
            .unwrap();

        Ok(Self {
            title,
            high_score,
            name: [' '; NAME_CHARACTERS],
            current_char: 0,
            texture,
            snip,
            title_point,
            score_point,
            name_rects,
        })
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, theme: &dyn Theme) -> Result<(), String> {
        canvas
            .with_texture_canvas(&mut self.texture, |c| {
                c.set_draw_color(Color::RGBA(0, 0, 0, 0));
                c.clear();

                theme.draw_string(c, self.title_point, &self.title).unwrap();
                theme
                    .draw_string(c, self.score_point, &self.high_score.score.to_string())
                    .unwrap();

                for i in 0..NAME_CHARACTERS {
                    let rect = self.name_rects[i];
                    theme
                        .draw_string(c, rect.top_left(), &self.name[i].to_string())
                        .unwrap();

                    if self.current_char == i {
                        c.set_draw_color(Color::RED); // TODO take from theme
                        let select_rect = Rect::new(
                            rect.top_left().x() - CHAR_RECT_PADDING as i32,
                            rect.top_left().y() - CHAR_RECT_PADDING as i32,
                            rect.width() + CHAR_RECT_PADDING * 2,
                            rect.height() + CHAR_RECT_PADDING * 2,
                        );
                        c.draw_rect(select_rect).unwrap();
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        canvas.copy(&self.texture, None, self.snip)
    }

    pub fn player(&self) -> u32 {
        self.high_score.player
    }

    pub fn to_high_score(&self) -> Option<HighScore> {
        let name = self.name.iter().collect::<String>().trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some(HighScore::from_string(name, self.high_score.score))
        }
    }

    pub fn up(&mut self) -> Option<GameEvent> {
        self.move_char(-1)
    }

    pub fn down(&mut self) -> Option<GameEvent> {
        self.move_char(1)
    }

    pub fn right(&mut self) -> Option<GameEvent> {
        self.current_char = min(self.current_char + 1, NAME_CHARACTERS);
        if self.current_char == NAME_CHARACTERS {
            Some(GameEvent::HighScoreEntry)
        } else {
            Some(GameEvent::HighScoreNextChar)
        }
    }

    pub fn left(&mut self) -> Option<GameEvent> {
        if self.current_char > 0 {
            self.current_char -= 1;
            Some(GameEvent::HighScorePreviousChar)
        } else {
            None
        }
    }

    fn move_char(&mut self, value: i32) -> Option<GameEvent> {
        let current = self.name[self.current_char];
        let mut next = if current == ' ' {
            if value > 0 {
                'A'
            } else {
                'Z'
            }
        } else {
            char::from_u32((current as i32 + value) as u32).unwrap()
        };
        if next > 'Z' {
            next = 'A';
        } else if next < 'A' {
            next = 'Z';
        }
        self.name[self.current_char] = next;
        Some(GameEvent::HighScoreMoveChar)
    }
}
