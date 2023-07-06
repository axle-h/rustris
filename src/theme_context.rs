use crate::config::Config;
use crate::game::tetromino::Minos;
use crate::scale::Scale;
use crate::theme::nes::nes_theme;
use crate::theme::snes::snes_theme;
use crate::theme::Theme;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::time::Duration;

const THEME_FADE_DURATION: Duration = Duration::from_millis(1000);
const THEMES: usize = 3;

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
    pub fn new(player: u32, theme: &dyn Theme, scale: Scale) -> Self {
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
    theme: Box<dyn Theme + 'a>,
    bg_source_snip: Rect,
    board_source_snip: Rect,
    player_themes: Vec<ThemedPlayer>,
    pause_snip: Rect,
    scale: Scale,
}

impl<'a> ScaledTheme<'a> {
    fn new(theme: Box<dyn Theme + 'a>, players: u32, window_size: (u32, u32)) -> Self {
        let scale = Scale::new(players, theme.background_size(), window_size);
        let (theme_width, theme_height) = theme.background_size();
        let bg_source_snip = Rect::new(0, 0, theme_width, theme_height);
        let board_rect = theme.board_snip();
        let board_source_snip = Rect::new(0, 0, board_rect.width(), board_rect.height());
        let player_themes = (0..players)
            .map(|pid| ThemedPlayer::new(pid + 1, theme.as_ref(), scale))
            .collect::<Vec<ThemedPlayer>>();
        let paused_query = theme.pause_texture().query();
        let pause_snip = scale.scaled_window_center_rect(paused_query.width, paused_query.height);
        Self {
            theme,
            bg_source_snip,
            board_source_snip,
            player_themes,
            pause_snip,
            scale,
        }
    }

    pub fn mino_rects(&self, player_id: u32, minos: Minos) -> [Rect; 4] {
        let rects = self.theme.mino_rects(minos);
        let player_board = &self.player_themes[player_id as usize - 1].board_snip;
        rects.map(|rect| {
            self.scale
                .scale_and_offset_rect(rect, player_board.x(), player_board.y())
        })
    }

    pub fn rows_to_pixels(&self, value: u32) -> u32 {
        let raw_pixels = self.theme.block_size() * value;
        self.scale.scale_length(raw_pixels)
    }
}

pub struct ThemeContext<'a> {
    current: usize,
    themes: [ScaledTheme<'a>; THEMES],
    lighten_screen: Texture<'a>,
    fade_buffer: Texture<'a>,
    fade_duration: Option<Duration>,
}

impl<'a> ThemeContext<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        players: u32,
        window_size: (u32, u32),
        config: Config,
    ) -> Result<Self, String> {
        let game_boy = config
            .theme
            .game_boy_palette
            .theme(canvas, texture_creator, config)?;
        //let game_boy_green = GameBoyPalette::GreenSoup.theme(canvas, texture_creator, config)?;
        let nes = nes_theme(canvas, texture_creator, config)?;
        let snes = snes_theme(canvas, texture_creator, config)?;

        let (window_width, window_height) = window_size;
        let mut lighten_screen = texture_creator
            .create_texture_target(None, window_width, window_height)
            .map_err(|e| e.to_string())?;
        lighten_screen.set_blend_mode(BlendMode::Blend);
        canvas
            .with_texture_canvas(&mut lighten_screen, |c| {
                c.set_draw_color(Color::RGBA(255, 255, 255, 150));
                c.clear();
            })
            .map_err(|e| e.to_string())?;

        let mut fade_buffer = texture_creator
            .create_texture_target(None, window_width, window_height)
            .map_err(|e| e.to_string())?;
        fade_buffer.set_blend_mode(BlendMode::Blend);

        Ok(Self {
            current: 0,
            themes: [game_boy, nes, snes]
                .map(|theme| ScaledTheme::new(Box::new(theme), players, window_size)),
            lighten_screen,
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

    pub fn theme_mut(&mut self) -> &mut dyn Theme {
        self.themes[self.current].theme.as_mut()
    }

    pub fn theme(&self) -> &dyn Theme {
        self.themes[self.current].theme.as_ref()
    }

    pub fn scale(&self) -> &Scale {
        &self.themes[self.current].scale
    }

    pub fn player_line_snip(&self, player: u32, j: u32) -> Rect {
        let theme = &self.themes[self.current];
        let player = theme.player_themes.get(player as usize - 1).unwrap();
        theme.scale.scale_and_offset_rect(
            theme.theme.line_snip(j),
            player.board_snip.x(),
            player.board_snip.y()
        )
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
                            .offset_scaled_rect(player.board_snip, offset_x, offset_y);
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

    pub fn draw_paused(&self, canvas: &mut WindowCanvas) -> Result<(), String> {
        let current = self.current();
        canvas.copy(&self.lighten_screen, None, None)?;
        canvas.copy(current.theme.pause_texture(), None, current.pause_snip)
    }
}
