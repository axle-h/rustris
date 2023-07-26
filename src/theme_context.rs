use crate::config::{Config, GameConfig, MatchThemes};
use crate::game::tetromino::Minos;
use crate::scale::Scale;
use crate::theme::nes::nes_theme;
use crate::theme::snes::snes_theme;
use crate::theme::Theme;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::time::Duration;
use sdl2::ttf::Sdl2TtfContext;
use crate::theme::modern::modern_theme;

const THEME_FADE_DURATION: Duration = Duration::from_millis(1000);
const THEMES: usize = 4;

pub struct PlayerTextures<'a> {
    pub background: Texture<'a>,
    pub board: Texture<'a>,
}

impl<'a> PlayerTextures<'a> {
    pub fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        background_size: (u32, u32),
        board_size: (u32, u32),
    ) -> Result<Self, String> {
        let (bg_width, bg_height) = background_size;
        let mut background = texture_creator
            .create_texture_target(None, bg_width, bg_height)
            .map_err(|e| e.to_string())?;
        background.set_blend_mode(BlendMode::Blend);

        let (board_width, board_height) = board_size;
        let mut board = texture_creator
            .create_texture_target(None, board_width, board_height)
            .map_err(|e| e.to_string())?;
        board.set_blend_mode(BlendMode::Blend);

        Ok(Self { background, board })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureMode {
    PlayerBackground(u32),
    PlayerBoard(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThemedPlayer {
    player: u32,
    bg_snip: Rect,
    board_snip: Rect,
}

impl ThemedPlayer {
    pub fn new(player: u32, theme: &Theme, scale: Scale) -> Self {
        let (theme_width, theme_height) = theme.background_size();
        let mut bg_snip = scale.scale_rect(Rect::new(0, 0, theme_width, theme_height));
        bg_snip.center_on(scale.player_window(player).center());
        let board_snip = scale.scale_and_offset_rect(theme.board_snip(), bg_snip.x(), bg_snip.y());
        Self {
            player,
            bg_snip,
            board_snip,
        }
    }
}

pub struct ScaledTheme<'a> {
    theme: Theme<'a>,
    bg_source_snip: Rect,
    board_source_snip: Rect,
    player_themes: Vec<ThemedPlayer>,
    scale: Scale,
}

impl<'a> ScaledTheme<'a> {
    fn new(theme: Theme<'a>, players: u32, window_size: (u32, u32)) -> Self {
        let scale = Scale::new(players, theme.background_size(), window_size, theme.geometry().block_size());
        let (theme_width, theme_height) = theme.background_size();
        let bg_source_snip = Rect::new(0, 0, theme_width, theme_height);
        let board_rect = theme.board_snip();
        let board_source_snip = Rect::new(0, 0, board_rect.width(), board_rect.height());
        let player_themes = (0..players)
            .map(|pid| ThemedPlayer::new(pid + 1, &theme, scale))
            .collect::<Vec<ThemedPlayer>>();
        Self {
            theme,
            bg_source_snip,
            board_source_snip,
            player_themes,
            scale,
        }
    }

    pub fn mino_rects(&self, player_id: u32, minos: Minos) -> [Rect; 4] {
        let rects = self.theme.geometry().mino_rects(minos);
        let player_board = &self.player_themes[player_id as usize - 1].board_snip;
        rects.map(|rect| {
            self.scale
                .scale_and_offset_rect(rect, player_board.x(), player_board.y())
        })
    }

    pub fn rows_to_pixels(&self, value: u32) -> u32 {
        let raw_pixels = self.theme.geometry().block_size() * value;
        self.scale.scale_length(raw_pixels)
    }
}

pub struct ThemeContext<'a> {
    current: usize,
    themes: [ScaledTheme<'a>; THEMES],
    fade_buffer: Texture<'a>,
    fade_duration: Option<Duration>,
}

impl<'a> ThemeContext<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf: &'a Sdl2TtfContext,
        game_config: GameConfig,
        window_size: (u32, u32),
        config: Config,
    ) -> Result<Self, String> {
        let (window_width, window_height) = window_size;
        let game_boy = config
            .theme
            .game_boy_palette
            .theme(canvas, texture_creator, config)?;
        let nes = nes_theme(canvas, texture_creator, config)?;
        let snes = snes_theme(canvas, texture_creator, config)?;
        let modern = modern_theme(canvas, texture_creator, ttf, config, window_height)?;

        let mut fade_buffer = texture_creator
            .create_texture_target(None, window_width, window_height)
            .map_err(|e| e.to_string())?;
        fade_buffer.set_blend_mode(BlendMode::Blend);


        let themes: [Theme; THEMES] = [game_boy, nes, snes, modern];
        let current = match game_config.themes {
            MatchThemes::All | MatchThemes::GameBoy => 0,
            MatchThemes::Nes => 1,
            MatchThemes::Snes => 2,
            MatchThemes::Modern => 3
        };

        Ok(Self {
            current,
            themes: themes
                .map(|theme| ScaledTheme::new(theme, game_config.players, window_size)),
            fade_buffer,
            fade_duration: None,
        })
    }

    pub fn max_background_size(&self) -> (u32, u32) {
        let sizes = self
            .themes
            .iter()
            .map(|theme| theme.theme.background_size());
        let width = sizes.clone().map(|(w, _)| w).max().unwrap();
        let height = sizes.clone().map(|(_, h)| h).max().unwrap();
        (width, height)
    }

    pub fn max_board_size(&self) -> (u32, u32) {
        let rects = self.themes.iter().map(|theme| theme.theme.board_snip());
        let width = rects.clone().map(|r| r.width()).max().unwrap();
        let height = rects.clone().map(|r| r.height()).max().unwrap();
        (width, height)
    }

    pub fn theme_mut(&mut self) -> &mut Theme<'a> {
        &mut self.themes[self.current].theme
    }

    pub fn theme(&self) -> &Theme<'a> {
        &self.themes[self.current].theme
    }

    pub fn scale(&self) -> &Scale {
        &self.themes[self.current].scale
    }

    pub fn player_line_snip(&self, player: u32, j: u32) -> Rect {
        let theme = &self.themes[self.current];
        let player = theme.player_themes.get(player as usize - 1).unwrap();
        theme.scale.scale_and_offset_rect(
            theme.theme.geometry().line_snip(j),
            player.board_snip.x(),
            player.board_snip.y()
        )
    }

    pub fn player_mino_snips(&self, player: u32, minos: Minos) -> [Rect; 4] {
        let theme = &self.themes[self.current];
        let player = theme.player_themes.get(player as usize - 1).unwrap();
        theme.theme.geometry().mino_rects(minos)
            .map(|r| theme.scale.scale_and_offset_rect(
                r, player.board_snip.x(), player.board_snip.y()))
    }

    pub fn player_board_snip(&self, player: u32) -> Rect {
        let theme = &self.themes[self.current];
        theme.player_themes.get(player as usize - 1).unwrap().board_snip
    }

    pub fn current(&self) -> &ScaledTheme {
        &self.themes[self.current]
    }

    pub fn next(&mut self) {
        self.current = (self.current + 1) % THEMES;
    }

    pub fn start_fade(&mut self, canvas: &mut WindowCanvas) -> Result<(), String> {
        self.fade_duration = Some(Duration::ZERO);

        let query = self.fade_buffer.query();
        let pixels = canvas.read_pixels(None, query.format)?;
        self.fade_buffer
            .update(
                None,
                pixels.as_slice(),
                query.format.byte_size_per_pixel() * query.width as usize,
            )
            .map_err(|e| e.to_string())
    }

    pub fn is_fading(&self) -> bool {
        self.fade_duration.is_some()
    }

    pub fn render_bg_particles(&self) -> bool {
        self.current().theme.particle_color().is_some()
    }

    pub fn draw_current(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_refs: &mut [(&mut Texture, TextureMode)],
        delta: Duration,
        offsets: Vec<(f64, f64)>,
    ) -> Result<(), String> {
        let current = self.current();
        for (texture, texture_mode) in texture_refs.iter_mut() {
            match texture_mode {
                TextureMode::PlayerBackground(pid) => {
                    let player = current.player_themes[*pid as usize - 1];
                    canvas.copy(texture, current.bg_source_snip, player.bg_snip)?;
                }
                TextureMode::PlayerBoard(pid) => {
                    let (offset_x, offset_y) = offsets[*pid as usize - 1];
                    let player = current.player_themes[*pid as usize - 1];
                    let dst =
                        current
                            .scale
                            .offset_proportional_to_block_size(player.board_snip, offset_x, offset_y);
                    canvas.copy(texture, current.board_source_snip, dst)?;
                }
            }
        }

        // check if we should be fading out the previous theme
        match self.fade_duration {
            None => {}
            Some(duration) => {
                let duration = duration + delta;
                if duration > THEME_FADE_DURATION {
                    self.fade_duration = None;
                } else {
                    let alpha = 255.0 * duration.as_millis() as f64
                        / THEME_FADE_DURATION.as_millis() as f64;
                    self.fade_buffer.set_alpha_mod(255 - alpha as u8);
                    canvas.copy(&self.fade_buffer, None, None)?;
                    self.fade_duration = Some(duration);
                }
            }
        }

        Ok(())
    }
}
