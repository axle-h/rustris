use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::{GameOverAnimate, GameOverAnimationType};
use crate::animation::TextureAnimate;
use crate::config::Config;
use crate::event::GameEvent;
use crate::game::block::BlockState;
use crate::game::board::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::geometry::Rotation;
use crate::game::random::PEEK_SIZE;
use crate::game::tetromino::{Minos, TetrominoShape};
use crate::game::Game;
use crate::theme::sound::{load_sound, play_sound, SoundTheme, SoundThemeOptions};
use crate::theme::Theme;
use sdl2::image::LoadTexture;
use sdl2::mixer::{Chunk, Music};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::cmp::min;
use crate::particles::prescribed::PlayerTargetedParticles;
use crate::theme::font::{FontRender, FontRenderOptions, MetricSnips};
use crate::theme::geometry::{BoardGeometry, VISIBLE_BOARD_HEIGHT};
use crate::theme::sprite_sheet::{TetrominoSpriteSheet, TetrominoSpriteSheetMeta};

pub struct RetroThemeOptions {
    name: String,
    config: Config,
    block_size: u32,
    sprite_sheet_meta: TetrominoSpriteSheetMeta,
    background_file: String,
    board_file: String,
    game_over_file: String,
    geometry: BoardGeometry,
    peek_snips: [Rect; 5],
    hold_snip: Rect,
    font_options: FontRenderOptions,
    score: MetricSnips,
    levels: MetricSnips,
    lines: MetricSnips,
    board_point: Point,
    background_color: Color,
    destroy_animation: DestroyAnimationType,
    game_over_animation: GameOverAnimationType,
    sound: SoundThemeOptions
}

impl RetroThemeOptions {
    pub fn new(
        name: &str,
        config: Config,
        sprite_sheet_meta: TetrominoSpriteSheetMeta,
        background_file: &str,
        board_file: &str,
        game_over_file: &str,
        peek_snips: [Rect; 5],
        hold_snip: Rect,
        font_options: FontRenderOptions,
        score: MetricSnips,
        levels: MetricSnips,
        lines: MetricSnips,
        board_point: Point,
        game_point: Point,
        background_color: Color,
        destroy_animation: DestroyAnimationType,
        game_over_animation: GameOverAnimationType,
        sound: SoundThemeOptions
    ) -> Self {
        let block_size = sprite_sheet_meta.block_size();
        let geometry = BoardGeometry::new(block_size, game_point);
        let buffer_height = geometry.buffer_height() as i32;
        Self {
            name: name.to_string(),
            config,
            block_size,
            sprite_sheet_meta,
            background_file: background_file.to_string(),
            board_file: board_file.to_string(),
            game_over_file: game_over_file.to_string(),
            geometry,
            peek_snips,
            hold_snip,
            font_options,
            score: score.offset(0, buffer_height),
            levels: levels.offset(0, buffer_height),
            lines: lines.offset(0, buffer_height),
            board_point,
            background_color,
            destroy_animation,
            game_over_animation,
            sound
        }
    }

    fn resource(&self, name: &str) -> String {
        format!("resource/{}/{}", self.name, name)
    }

    fn background_file(&self) -> String {
        self.resource(&self.background_file)
    }

    fn board_file(&self) -> String {
        self.resource(&self.board_file)
    }

    fn game_over_file(&self) -> String {
        self.resource(&self.game_over_file)
    }

    /// get the rect of the line in the board texture, which has no buffer
    /// TODO move this into geometry
    fn src_row_rect(&self, j: u32) -> Rect {
        let capped_j = min(j, BOARD_HEIGHT); // the src has no buffer so protect against copying from it
        let y = self.geometry.height() - ((capped_j + 1) * self.geometry.block_size());
        Rect::new(
            self.geometry.offset().x(),
            y as i32,
            self.geometry.width(),
            self.geometry.block_size(),
        )
    }
}

pub struct RetroTheme<'a> {
    options: RetroThemeOptions,
    sprite_sheet: TetrominoSpriteSheet<'a>,
    font: FontRender<'a>,
    game_over: Texture<'a>,
    board_texture: Texture<'a>,
    board_texture_size: (u32, u32),
    bg_texture: Texture<'a>,
    bg_rect: Rect,
    sound: SoundTheme<'a>
}

impl<'a> RetroTheme<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        options: RetroThemeOptions,
    ) -> Result<Self, String> {
        let sprite_sheet = TetrominoSpriteSheet::new(canvas, texture_creator, options.sprite_sheet_meta.clone(), options.block_size)?;
        let board_texture = texture_creator.load_texture(options.board_file())?;
        let board_query = board_texture.query();

        let bg_texture = texture_creator.load_texture(options.background_file())?;
        let bg_query = bg_texture.query();
        let bg_rect = Rect::new(
            0,
            options.geometry.buffer_height() as i32,
            bg_query.width,
            bg_query.height,
        );

        let font = options.font_options.build(texture_creator)?;

        let game_over = texture_creator.load_texture(options.game_over_file())?;
        let sound = options.sound.clone().build()?;

        Ok(Self {
            options,
            sprite_sheet,
            game_over,
            font,
            board_texture,
            board_texture_size: (board_query.width, board_query.height),
            bg_texture,
            bg_rect,
            sound,
        })
    }
}

impl<'a> Theme for RetroTheme<'a> {
    fn geometry(&self) -> &BoardGeometry {
        &self.options.geometry
    }

    fn background_color(&self) -> Color {
        self.options.background_color
    }

    fn background_size(&self) -> (u32, u32) {
        (
            self.bg_rect.width(),
            self.bg_rect.height() + self.options.geometry.buffer_height(),
        )
    }

    fn board_snip(&self) -> Rect {
        let (w, h) = self.board_texture_size;
        Rect::new(
            self.options.board_point.x(),
            self.options.board_point.y(),
            w,
            h + self.options.geometry.buffer_height(),
        )
    }

    fn draw_background(&mut self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String> {
        let metrics = game.metrics();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        // background
        canvas.copy(&self.bg_texture, None, self.bg_rect)?;

        self.font.render_number(canvas, self.options.score, metrics.score)?;
        self.font.render_number(canvas, self.options.levels, metrics.level)?;
        self.font.render_number(canvas, self.options.lines, metrics.lines)?;

        for i in 0..(min(PEEK_SIZE, self.options.peek_snips.len())) {
            self.sprite_sheet.draw_tetromino_in_center(canvas, metrics.queue[i], self.options.peek_snips[i])?;
        }

        match metrics.hold {
            None => {}
            Some(shape) => {
                self.sprite_sheet.draw_tetromino_in_center(canvas, shape, self.options.hold_snip)?;
            }
        }
        Ok(())
    }

    fn draw_board(
        &mut self,
        canvas: &mut WindowCanvas,
        game: &Game,
        animate_lines: Vec<(u32, TextureAnimate)>,
        animate_game_over: Option<GameOverAnimate>,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        let (board_width, board_height) = self.board_texture_size;
        canvas.copy(
            &self.board_texture,
            None,
            Rect::new(
                0,
                self.options.geometry.buffer_height() as i32,
                board_width,
                board_height,
            ),
        )?;

        let (curtain_range, render_board) = match animate_game_over {
            Some(animate) => match animate {
                GameOverAnimate::CurtainClosing(range) => (range, true),
                GameOverAnimate::CurtainOpening(range) => (range, false),
                GameOverAnimate::Finished => (0..0, false),
            },
            _ => (0..0, true),
        };

        if render_board {
            for j in 0..VISIBLE_BOARD_HEIGHT {
                for (i, block) in game.row(j).iter().enumerate() {
                    let point = self.options.geometry.mino_point(i as u32, j);
                    match block {
                        BlockState::Empty => {}
                        BlockState::Tetromino(shape, rotation, mino_id) => {
                            self.sprite_sheet.draw_mino(canvas, *shape, *rotation, *mino_id, point)?;
                        }
                        BlockState::Ghost(shape, rotation, mino_id) => {
                            // TODO maybe some themes may like a perimeter ghost?
                            self.sprite_sheet.draw_ghost(canvas, *shape, *rotation, *mino_id, point)?;
                        }
                        BlockState::Stack(shape, rotation, mino_id) => {
                            self.sprite_sheet.draw_stack(canvas, *shape, *rotation, *mino_id, point)?;
                        }
                        BlockState::Garbage => {
                            self.sprite_sheet.draw_garbage(canvas, point)?;
                        }
                    }
                }

                // post draw animate
                let animate_line = animate_lines
                    .iter()
                    .find(|(line, _)| *line == j)
                    .map(|(_, animate)| animate);
                match animate_line {
                    None => {}
                    Some(animate) => {
                        match animate {
                            TextureAnimate::SetAlpha => {
                                // simulate alpha by copying over the board background
                                canvas.copy(
                                    &self.board_texture,
                                    self.options.src_row_rect(j),
                                    self.options.geometry.line_snip(j),
                                )?;
                            }
                            TextureAnimate::FillAlphaRectangle { width } => {
                                // simulate alpha by copying over the board background
                                let row_rect = self.options.geometry.line_snip(j);
                                let rect_width = (row_rect.width() as f64 * width).round() as u32;
                                let dst_rect = Rect::from_center(
                                    row_rect.center(),
                                    rect_width,
                                    row_rect.height(),
                                );
                                let src_rect = Rect::from_center(
                                    self.options.src_row_rect(j).center(),
                                    rect_width,
                                    row_rect.height(),
                                );
                                canvas.copy(&self.board_texture, src_rect, dst_rect)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            let game_over_query = self.game_over.query();
            let game_snip = self.options.geometry.game_snip();
            let game_over_snip = Rect::from_center(
                game_snip.center(),
                game_over_query.width,
                game_over_query.height,
            );
            canvas.copy(&self.game_over, None, game_over_snip)?;
        }

        for j in curtain_range {
            for i in 0..BOARD_WIDTH {
                let point = self.options.geometry.mino_point(i as u32, j);
                self.sprite_sheet.draw_garbage(canvas, point)?;
            }
        }

        Ok(())
    }

    fn destroy_animation_type(&self) -> DestroyAnimationType {
        self.options.destroy_animation
    }

    fn game_over_animation_type(&self) -> GameOverAnimationType {
        self.options.game_over_animation
    }

    fn music(&self) -> &Music {
        self.sound.music()
    }

    fn play_sound_effects(&mut self, event: GameEvent) -> Result<(), String> {
        self.sound.receive_event(event)
    }

    fn emit_particles(&self, event: GameEvent) -> Option<PlayerTargetedParticles> {
        None
    }
}
