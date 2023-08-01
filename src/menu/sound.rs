use crate::config::AudioConfig;
use crate::theme::sound::{load_sound, play_sound};
use sdl2::mixer::{Chunk, Music};

const CHIME: &[u8] = include_bytes!("chime.ogg");
const MAIN_MENU_MUSIC: &[u8] = include_bytes!("main-menu.ogg");
const HIGH_SCORE_MUSIC: &[u8] = include_bytes!("high-score.ogg");

pub struct MenuSound {
    chime: Chunk,
    main_menu_music: Music<'static>,
    high_score_music: Music<'static>,
}

impl MenuSound {
    pub fn new(config: AudioConfig) -> Result<Self, String> {
        Ok(Self {
            chime: load_sound(CHIME, config)?,
            main_menu_music: Music::from_static_bytes(MAIN_MENU_MUSIC)?,
            high_score_music: Music::from_static_bytes(HIGH_SCORE_MUSIC)?,
        })
    }

    pub fn play_chime(&self) -> Result<(), String> {
        play_sound(&self.chime)
    }

    pub fn play_main_menu_music(&self) -> Result<(), String> {
        self.main_menu_music.play(-1)
    }

    pub fn play_high_score_music(&self) -> Result<(), String> {
        self.high_score_music.play(-1)
    }
}
