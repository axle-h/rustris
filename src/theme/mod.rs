use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::{GameOverAnimate, GameOverAnimationType};
use crate::animation::TextureAnimate;
use crate::event::GameEvent;

use crate::game::board::BOARD_WIDTH;
use crate::game::tetromino::TetrominoShape;
use crate::game::Game;
use crate::particles::prescribed::{
    PlayerParticleTarget, PlayerTargetedParticles, PrescribedParticles,
};
use crate::theme::font::{FontRender, MetricSnips};
use crate::theme::geometry::BoardGeometry;
use crate::theme::sound::SoundTheme;
use crate::theme::sprite_sheet::{MinoType, TetrominoSpriteSheet};
use sdl2::mixer::Music;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::helper::{CanvasRenderer, TextureFactory};

pub mod all;
pub mod font;
pub mod gb;
pub mod geometry;
pub mod modern;
pub mod nes;
mod retro;
pub mod snes;
pub mod sound;
pub mod sprite_sheet;
pub mod helper;

const VISIBLE_PEEK: usize = 5;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ThemeName {
    GameBoy,
    Nes,
    Snes,
    #[default]
    Modern,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TetrominoScaleType {
    /// Fill the snip with scaled
    Fill {
        default_scale: f64,
        peek0_scale: f64,
    },

    /// Centered in the snip
    Center,
}

/// copies a texture with blend mode = none
pub fn create_mask_texture<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    texture: &Texture,
) -> Result<Texture<'a>, String> {
    let query = texture.query();
    let mut mask_texture = texture_creator.create_texture_target_blended(query.width, query.height)?;
    canvas
        .with_texture_canvas(&mut mask_texture, |c| {
            c.clear_0();
            c.copy(texture, None, None).unwrap();
        })
        .map_err(|e| e.to_string())?;
    Ok(mask_texture)
}

pub struct Theme<'a> {
    name: ThemeName,
    sprite_sheet: TetrominoSpriteSheet<'a>,
    geometry: BoardGeometry,
    board_texture: Texture<'a>,
    board_mask_texture: Texture<'a>, // same as board texture but with blend mode set to none
    board_snip: Rect,
    background_texture: Texture<'a>,
    background_size: (u32, u32),
    score_snip: MetricSnips,
    level_snip: MetricSnips,
    lines_snip: MetricSnips,
    peek_snips: [Rect; VISIBLE_PEEK],
    hold_snip: Rect,
    font: FontRender<'a>,
    game_over: Texture<'a>,
    sound: SoundTheme,
    background_color: Color,
    destroy_animation: DestroyAnimationType,
    game_over_animation: GameOverAnimationType,
    ghost_mino_type: MinoType,
    tetromino_scale_type: TetrominoScaleType,
    particle_color: Option<Color>,
}

impl<'a> Theme<'a> {
    pub fn name(&self) -> ThemeName {
        self.name
    }

    pub fn geometry(&self) -> &BoardGeometry {
        &self.geometry
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn background_size(&self) -> (u32, u32) {
        self.background_size
    }

    pub fn board_snip(&self) -> Rect {
        self.board_snip
    }

    pub fn draw_background(&self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String> {
        let metrics = game.metrics();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        // background
        let (width, height) = self.background_size;
        canvas.copy(
            &self.background_texture,
            None,
            Rect::new(0, 0, width, height),
        )?;

        if let Some(hold_shape) = metrics.hold {
            self.draw_tetromino(canvas, hold_shape, self.hold_snip, false)?;
        }

        for (index, (peek_shape, peek_rect)) in metrics
            .queue
            .iter()
            .copied()
            .zip(self.peek_snips)
            .enumerate()
        {
            self.draw_tetromino(canvas, peek_shape, peek_rect, index == 0)?;
        }

        self.font
            .render_number(canvas, self.score_snip, metrics.score)?;
        self.font
            .render_number(canvas, self.level_snip, metrics.level)?;
        self.font
            .render_number(canvas, self.lines_snip, metrics.lines)?;

        Ok(())
    }

    pub fn draw_board(
        &self,
        canvas: &mut WindowCanvas,
        game: &Game,
        animate_lines: Vec<(u32, TextureAnimate)>,
        animate_game_over: Option<GameOverAnimate>,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        canvas.copy(
            &self.board_texture,
            None,
            Rect::new(0, 0, self.board_snip.width(), self.board_snip.height()),
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
            self.sprite_sheet
                .draw_board(canvas, game, &self.geometry, self.ghost_mino_type)?;

            // animations
            for (j, animate) in animate_lines {
                let line_snip = self.geometry.line_snip(j);
                match animate {
                    TextureAnimate::SetAlpha => {
                        // simulate alpha by copying over the board background
                        canvas.copy(&self.board_mask_texture, line_snip, line_snip)?;
                    }
                    TextureAnimate::FillAlphaRectangle { width } => {
                        // simulate alpha by copying over the board background
                        let row_rect = self.geometry.line_snip(j);
                        let rect_width = (row_rect.width() as f64 * width).round() as u32;
                        let dst_rect =
                            Rect::from_center(row_rect.center(), rect_width, row_rect.height());
                        let src_rect =
                            Rect::from_center(line_snip.center(), rect_width, row_rect.height());
                        canvas.copy(&self.board_mask_texture, src_rect, dst_rect)?;
                    }
                    _ => {}
                }
            }
        } else {
            let game_over_query = self.game_over.query();
            let game_snip = self.geometry.game_snip();
            let game_over_snip = Rect::from_center(
                game_snip.center(),
                game_over_query.width,
                game_over_query.height,
            );
            canvas.copy(&self.game_over, None, game_over_snip)?;
        }

        for j in curtain_range {
            for i in 0..BOARD_WIDTH {
                let point = self.geometry.mino_point(i, j);
                self.sprite_sheet.draw_garbage(canvas, point)?;
            }
        }

        Ok(())
    }

    pub fn destroy_animation_type(&self) -> DestroyAnimationType {
        self.destroy_animation
    }

    pub fn game_over_animation_type(&self) -> GameOverAnimationType {
        self.game_over_animation
    }

    pub fn music(&self) -> &Music {
        self.sound.music()
    }

    pub fn play_sound_effects(&self, event: GameEvent) -> Result<(), String> {
        self.sound.receive_event(event)
    }

    pub fn emit_particles(&self, event: GameEvent) -> Option<PlayerTargetedParticles> {
        if let Some(color) = self.particle_color {
            match event {
                GameEvent::Spawn { player, minos } => {
                    let target = PlayerParticleTarget::Minos(minos);
                    let particles = PrescribedParticles::LightBurstUpAndOut { color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::HardDrop { player, minos, .. } => {
                    let target = PlayerParticleTarget::Minos(minos);
                    let particles = PrescribedParticles::BurstUp { color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::Lock {
                    player,
                    minos,
                    hard_or_soft_dropped,
                } if hard_or_soft_dropped => {
                    let target = PlayerParticleTarget::Minos(minos);
                    let particles = PrescribedParticles::BurstDown { color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::ReceivedGarbageLine { player, line } => {
                    let target = PlayerParticleTarget::Line(line);
                    let particles = PrescribedParticles::BurstDown { color };
                    Some(particles.into_targeted(player, target))
                }
                GameEvent::Destroyed {
                    player, level_up, ..
                } if level_up => Some(
                    PrescribedParticles::PerimeterBurst { color }
                        .into_targeted(player, PlayerParticleTarget::Board),
                ),
                GameEvent::Victory { player } => Some(
                    PrescribedParticles::PerimeterSpray { color }
                        .into_targeted(player, PlayerParticleTarget::Board),
                ),
                _ => None,
            }
        } else {
            None
        }
    }

    fn draw_tetromino(
        &self,
        canvas: &mut WindowCanvas,
        shape: TetrominoShape,
        snip: Rect,
        is_peek0: bool,
    ) -> Result<(), String> {
        match self.tetromino_scale_type {
            TetrominoScaleType::Fill { peek0_scale, .. } if is_peek0 => self
                .sprite_sheet
                .draw_tetromino_fill(canvas, shape, MinoType::Normal, snip, peek0_scale),
            TetrominoScaleType::Fill { default_scale, .. } => self
                .sprite_sheet
                .draw_tetromino_fill(canvas, shape, MinoType::Normal, snip, default_scale),
            TetrominoScaleType::Center => self.sprite_sheet.draw_tetromino_in_center(
                canvas,
                shape,
                MinoType::Normal,
                snip.center(),
            ),
        }
    }

    pub fn particle_color(&self) -> Option<Color> {
        self.particle_color
    }

    pub fn sprite_sheet(&self) -> &TetrominoSpriteSheet<'a> {
        &self.sprite_sheet
    }
}
