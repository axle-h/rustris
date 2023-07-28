use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use crate::config::Config;
use crate::theme::modern::modern_theme;
use crate::theme::nes::nes_theme;
use crate::theme::snes::snes_theme;
use crate::theme::Theme;

pub struct AllThemes<'a> {
    game_boy: Theme<'a>,
    nes: Theme<'a>,
    snes: Theme<'a>,
    modern: Theme<'a>
}

impl<'a> AllThemes<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        ttf: &Sdl2TtfContext,
        config: Config,
        window_height: u32
    ) -> Result<Self, String> {
        let game_boy = config
            .theme
            .game_boy_palette
            .theme(canvas, &texture_creator, config)?;
        let nes = nes_theme(canvas, &texture_creator, config)?;
        let snes = snes_theme(canvas, &texture_creator, config)?;
        let modern = modern_theme(canvas, &texture_creator, ttf, config, window_height)?;
        Ok(Self { game_boy, nes, snes, modern })
    }

    pub fn game_boy(&self) -> &Theme<'a> {
        &self.game_boy
    }

    pub fn nes(&self) -> &Theme<'a> {
        &self.nes
    }

    pub fn snes(&self) -> &Theme<'a> {
        &self.snes
    }

    pub fn modern(&self) -> &Theme<'a> {
        &self.modern
    }

    pub fn all(&self) -> Vec<&Theme<'a>> {
        vec![&self.game_boy, &self.nes, &self.snes, &self.modern]
    }
}
