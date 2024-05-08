#![windows_subsystem = "windows"]

mod animation;
mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
mod config;
mod event;
mod font;
mod frame_rate;
mod game;
mod game_input;
mod high_score;
mod menu;
mod menu_input;
mod particles;
mod paused;
mod player;
mod scale;
mod theme;
mod theme_context;
mod icon;

extern crate sdl2;

use crate::animation::game_over::GameOverAnimate;
use crate::animation::hard_drop::HardDropAnimation;
use crate::config::{Config, GameConfig, MatchRules, MatchThemes, VideoMode};
use crate::event::{GameEvent, HighScoreEntryEvent};
use crate::game_input::GameInputKey;
use crate::high_score::render::HighScoreRender;
use crate::high_score::table::HighScoreTable;
use crate::menu::{Menu, MenuItem};
use crate::menu_input::{MenuInputContext, MenuInputKey};
use crate::player::MatchState;

use crate::frame_rate::FrameRate;
use crate::high_score::NewHighScore;

use crate::particles::prescribed::{
    prescribed_fireworks, prescribed_orbit, prescribed_tetromino_race,
};
use crate::particles::render::ParticleRender;
use crate::particles::source::ParticleSource;
use crate::particles::Particles;
use crate::paused::PausedScreen;
use crate::theme::all::AllThemes;

use game_input::GameInputContext;
use player::Match;
use sdl2::image::{InitFlag as ImageInitFlag, Sdl2ImageContext};
use sdl2::mixer::{InitFlag as MixerInitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::sys::mixer::MIX_CHANNELS;
use sdl2::ttf::Sdl2TtfContext;

use sdl2::{AudioSubsystem, EventPump, Sdl};
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use crate::menu::sound::MenuSound;
use theme_context::{PlayerTextures, TextureMode, ThemeContext};
use crate::icon::app_icon;

#[cfg(not(feature = "retro_handheld"))]
const MAX_PLAYERS: u32 = 2;

#[cfg(feature = "retro_handheld")]
const MAX_PLAYERS: u32 = 1;

const MAX_PARTICLES_PER_PLAYER: usize = 100000;
const MAX_BACKGROUND_PARTICLES: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMenuAction {
    Start,
    ViewHighScores,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostGameAction {
    NewHighScore(NewHighScore),
    ReturnToMenu,
    Quit,
}

struct TetrisSdl {
    config: Config,
    _sdl: Sdl,
    ttf: Sdl2TtfContext,
    _image: Sdl2ImageContext,
    canvas: WindowCanvas,
    event_pump: EventPump,
    _audio: AudioSubsystem,
    particle_scale: particles::scale::Scale,
    menu_sound: MenuSound,
    game_config: GameConfig
}

impl TetrisSdl {
    pub fn new() -> Result<Self, String> {
        let config = Config::load()?;
        let sdl = sdl2::init()?;
        let image = sdl2::image::init(ImageInitFlag::PNG)?;
        let video = sdl.video()?;
        let ttf = sdl2::ttf::init().map_err(|e| e.to_string())?;

        // let resolutions: BTreeSet<(i32, i32)> = (0..video.num_display_modes(0)?)
        //     .into_iter()
        //     .map(|i| video.display_mode(0, i).unwrap())
        //     .map(|mode| (mode.w, mode.h))
        //     .collect();

        if config.video.disable_screensaver && video.is_screen_saver_enabled() {
            video.disable_screen_saver();
        }

        let (width, height) = match config.video.mode {
            VideoMode::Window { width, height } => (width, height),
            VideoMode::FullScreen { width, height } => (width, height),
            _ => (1, 1),
        };

        let mut window_builder = video.window(build_info::PKG_NAME, width, height);
        match config.video.mode {
            VideoMode::FullScreen { .. } => {
                window_builder.fullscreen();
            }
            VideoMode::FullScreenDesktop => {
                window_builder.fullscreen_desktop();
            }
            _ => {}
        };

        let mut window = window_builder
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        window.set_icon(app_icon()?);

        let canvas_builder = window
            .into_canvas()
            .target_texture()
            .accelerated();

        let canvas = if config.video.vsync {
            canvas_builder.present_vsync()
        } else {
            canvas_builder
        }
        .build()
        .map_err(|e| e.to_string())?;

        let (width, height) = canvas.window().size();

        let event_pump = sdl.event_pump()?;

        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(44_100, AUDIO_S16LSB, DEFAULT_CHANNELS, 512)?;
        let _mixer_context = sdl2::mixer::init(MixerInitFlag::OGG)?;
        sdl2::mixer::allocate_channels((MAX_PLAYERS * MIX_CHANNELS) as i32);
        sdl2::mixer::Music::set_volume(config.audio.music_volume());
        let menu_sound = MenuSound::new(config.audio)?;

        Ok(Self {
            config,
            _sdl: sdl,
            ttf,
            _image: image,
            canvas,
            event_pump,
            _audio: audio,
            particle_scale: particles::scale::Scale::new((width, height)),
            menu_sound,
            game_config: Default::default(),
        })
    }

    fn orbit_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_orbit(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    fn tetromino_race_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_tetromino_race(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    fn fireworks_particle_source(&self) -> Box<dyn ParticleSource> {
        let (window_width, window_height) = self.canvas.window().size();
        prescribed_fireworks(
            Rect::new(0, 0, window_width, window_height),
            &self.particle_scale,
        )
    }

    pub fn main_menu(&mut self, particles: &mut ParticleRender) -> Result<MainMenuAction, String> {
        const PLAYERS: &str = "players";
        const THEMES: &str = "themes";
        const MODE: &str = "mode";
        const LEVEL: &str = "level";
        const HIGH_SCORES: &str = "high scores";
        const START: &str = "start";
        const QUIT: &str = "quit";

        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let modes = MatchRules::DEFAULT_MODES;

        let mut menu_items = vec![
            MenuItem::select_list(
                THEMES,
                MatchThemes::names().into_iter().map(|s| s.to_string()).collect(),
                self.game_config.themes as usize,
            ),
            MenuItem::select_list(
                MODE,
                modes.iter().map(|m| m.name()).collect(),
                modes.iter().position(|&m| m == self.game_config.rules).unwrap()
            ),
            MenuItem::select_list(
                LEVEL,
                (0..10).map(|i| i.to_string()).collect(),
                self.game_config.level as usize,
            ),
            MenuItem::select(HIGH_SCORES),
            MenuItem::select(START),
            MenuItem::select(QUIT),
        ];

        if MAX_PLAYERS > 1 {
            menu_items.insert(
                0,
                MenuItem::select_list(
                    PLAYERS,
                    (1..=MAX_PLAYERS)
                        .into_iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>(),
                    self.game_config.players as usize - 1,
                )
            )
        }

        let mut menu = Menu::new(
            menu_items,
            &mut self.canvas,
            &self.ttf,
            &texture_creator,
            build_info::PKG_NAME.to_uppercase(),
            None
        )?;

        particles.clear();
        particles.add_source(self.tetromino_race_particle_source());

        let mut frame_rate = FrameRate::new();

        self.menu_sound.play_main_menu_music()?;
        loop {
            let delta = frame_rate.update()?;

            for key in inputs.parse(self.event_pump.poll_iter()).into_iter() {
                if key == MenuInputKey::Quit {
                    return Ok(MainMenuAction::Quit);
                }
                match menu.read_key(key) {
                    None => match key {
                        MenuInputKey::Start => {
                            self.menu_sound.play_chime()?;
                            return Ok(MainMenuAction::Start);
                        }
                        _ => {}
                    },
                    Some((name, action)) => match name {
                        PLAYERS => self.game_config.players = action.parse::<u32>().unwrap(),
                        THEMES => self.game_config.themes = MatchThemes::from_str(action).unwrap(),
                        MODE => {
                            let mode_index =
                                modes.iter().position(|&m| m.name() == action).unwrap();
                            self.game_config.rules = modes[mode_index];
                        }
                        LEVEL => self.game_config.level = action.parse::<u32>().unwrap(),
                        HIGH_SCORES => return Ok(MainMenuAction::ViewHighScores),
                        START => return Ok(MainMenuAction::Start),
                        QUIT => return Ok(MainMenuAction::Quit),
                        _ => {}
                    },
                }

                self.menu_sound.play_chime()?;
            }

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // particles
            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            // menu
            menu.draw(&mut self.canvas)?;

            self.canvas.present();
        }
    }

    pub fn view_high_score(&mut self, particles: &mut ParticleRender) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load()?;
        if high_scores.entries().is_empty() {
            return Ok(());
        }

        let mut view = HighScoreRender::new(
            high_scores,
            &self.ttf,
            &texture_creator,
            self.canvas.window().size(),
            None,
        )?;

        particles.clear();
        particles.add_source(self.fireworks_particle_source());

        let mut frame_rate = FrameRate::new();

        self.menu_sound.play_high_score_music()?;
        'menu: loop {
            let delta = frame_rate.update()?;

            let events = inputs.parse(self.event_pump.poll_iter());
            if !events.is_empty() {
                // any button press
                break 'menu;
            }
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // particles
            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            view.draw(&mut self.canvas)?;

            self.canvas.present();
        }
        Ok(())
    }

    pub fn new_high_score(
        &mut self,
        new_high_score: NewHighScore,
        particles: &mut ParticleRender,
    ) -> Result<(), String> {
        let texture_creator = self.canvas.texture_creator();
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load()?;
        if high_scores.entries().is_empty() {
            return Ok(());
        }

        let mut table = HighScoreRender::new(
            high_scores,
            &self.ttf,
            &texture_creator,
            self.canvas.window().size(),
            Some(new_high_score),
        )?;

        particles.clear();
        particles.add_source(self.fireworks_particle_source());

        let mut frame_rate = FrameRate::new();

        self.menu_sound.play_high_score_music()?;
        'menu: loop {
            let delta = frame_rate.update()?;

            for key in inputs.parse(self.event_pump.poll_iter()) {
                let event = match key {
                    MenuInputKey::Up => table.up(),
                    MenuInputKey::Down => table.down(),
                    MenuInputKey::Left => table.left(),
                    MenuInputKey::Right => table.right(),
                    MenuInputKey::Start => break 'menu,
                    MenuInputKey::Quit => return Ok(()),
                    _ => None,
                };
                if event.is_none() {
                    continue;
                }
                match event.unwrap() {
                    HighScoreEntryEvent::Finished => break 'menu,
                    _ => self.menu_sound.play_chime()?,
                }
            }

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // particles
            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            table.draw(&mut self.canvas)?;

            self.canvas.present();
        }

        if let Some(new_entry) = table.new_entry() {
            let mut high_scores = HighScoreTable::load().unwrap();
            high_scores.add_high_score(new_entry);
            high_scores.save()
        } else {
            Ok(())
        }
    }

    pub fn game(
        &mut self,
        all_themes: &AllThemes,
        bg_particles: &mut ParticleRender,
        fg_particles: &mut ParticleRender,
    ) -> Result<PostGameAction, String> {
        let texture_creator = self.canvas.texture_creator();
        let mut inputs = GameInputContext::new(self.config.input);
        let mut fixture = Match::new(self.game_config, self.config);
        let window_size = self.canvas.window().size();
        let mut themes = ThemeContext::new(all_themes, &texture_creator, self.game_config, window_size)?;

        let mut player_textures = (0..self.game_config.players)
            .map(|_| {
                PlayerTextures::new(
                    &texture_creator,
                    themes.max_background_size(),
                    themes.max_board_size(),
                )
                .unwrap()
            })
            .collect::<Vec<PlayerTextures>>();

        // push mut refs of all textures and their render modes into a single vector so we can render to texture in one loop
        let mut texture_refs: Vec<(&mut Texture, TextureMode)> = vec![];
        for (player_index, textures) in player_textures.iter_mut().enumerate() {
            let player = player_index as u32 + 1;
            texture_refs.push((
                &mut textures.background,
                TextureMode::PlayerBackground(player),
            ));
            texture_refs.push((&mut textures.board, TextureMode::PlayerBoard(player)));
        }

        fg_particles.clear();
        bg_particles.clear();
        bg_particles.add_source(self.orbit_particle_source());

        themes.theme().music().play(-1)?;
        let paused_screen =
            PausedScreen::new(&mut self.canvas, &self.ttf, &texture_creator, window_size)?;

        let mut player_hard_drop_animations: HashMap<u32, HardDropAnimation> = HashMap::new();
        let mut max_level = 0;
        let mut frame_rate = FrameRate::new();

        loop {
            let delta = frame_rate.update()?;

            let mut to_emit_particles = vec![];

            fixture.unset_flags();
            for hard_dropping_player in player_hard_drop_animations.keys() {
                fixture.set_hard_dropping(*hard_dropping_player);
            }

            let events = inputs
                .update(delta, self.event_pump.poll_iter())
                .into_iter()
                .flat_map(|input| match input {
                    GameInputKey::MoveLeft { player } => fixture.mut_game(player, |g| g.left()),
                    GameInputKey::MoveRight { player } => fixture.mut_game(player, |g| g.right()),
                    GameInputKey::SoftDrop { player } => {
                        fixture.mut_game(player, |g| g.set_soft_drop(true))
                    }
                    GameInputKey::HardDrop { player } => {
                        fixture.mut_game(player, |g| g.hard_drop())
                    }
                    GameInputKey::RotateClockwise { player } => {
                        fixture.mut_game(player, |g| g.rotate(true))
                    }
                    GameInputKey::RotateAnticlockwise { player } => {
                        fixture.mut_game(player, |g| g.rotate(false))
                    }
                    GameInputKey::Hold { player } => fixture.mut_game(player, |g| g.hold()),
                    GameInputKey::Pause => match fixture.state() {
                        MatchState::Normal | MatchState::Paused => fixture.toggle_paused(),
                        _ => None,
                    },
                    GameInputKey::Quit => Some(GameEvent::Quit),
                    GameInputKey::ReturnToMenu => Some(GameEvent::ReturnToMenu),
                    GameInputKey::NextTheme => Some(GameEvent::NextTheme),
                })
                .collect::<Vec<GameEvent>>();

            for event in events.into_iter() {
                match event {
                    GameEvent::Quit => return Ok(PostGameAction::Quit),
                    GameEvent::ReturnToMenu => return Ok(PostGameAction::ReturnToMenu), // even if high score?!
                    GameEvent::Paused => sdl2::mixer::Music::pause(),
                    GameEvent::UnPaused => sdl2::mixer::Music::resume(),
                    GameEvent::NextTheme if !fixture.state().is_game_over() => {
                        themes.start_fade(&mut self.canvas)?;
                        themes.next();

                        // handle music
                        match fixture.state() {
                            MatchState::Normal => {
                                themes.theme().music().fade_in(-1, 1000)?;
                            }
                            MatchState::Paused => {
                                // switch music but pause it immediately
                                themes.theme().music().play(-1)?;
                                sdl2::mixer::Music::pause();
                            }
                            _ => {}
                        }
                    }
                    GameEvent::HardDrop {
                        player: player_id,
                        minos,
                        dropped_rows,
                    } => {
                        let theme = themes.current();
                        let mino_rects = theme.mino_rects(player_id, minos);
                        let dropped_pixels = theme.rows_to_pixels(dropped_rows);
                        let hard_drop_animation = HardDropAnimation::new(
                            &self.canvas,
                            &texture_creator,
                            mino_rects,
                            dropped_pixels,
                        )?;
                        player_hard_drop_animations.insert(player_id, hard_drop_animation);
                    }
                    _ => {}
                }

                themes.theme().play_sound_effects(event)?;
                if let Some(emit) = themes.theme().emit_particles(event) {
                    to_emit_particles.push(emit);
                }
            }

            match fixture.state() {
                MatchState::GameOver {
                    high_score: maybe_high_score,
                } => {
                    let mut game_over_done = true;
                    for player in fixture.players.iter_mut() {
                        match player.update_game_over_animation(delta) {
                            Some(animation) if animation != GameOverAnimate::Finished => {
                                game_over_done = false
                            }
                            _ => {}
                        }
                    }
                    if let Some(high_score) = maybe_high_score {
                        // start high score entry
                        if game_over_done {
                            return Ok(PostGameAction::NewHighScore(high_score));
                        }
                    }
                }
                MatchState::Normal if !themes.is_fading() => {
                    let mut garbage: Vec<(u32, u32)> = vec![];
                    let mut new_game_over: Option<u32> = None;
                    let mut next_theme = false;
                    for player in fixture.players.iter_mut() {
                        if let Some(emit) = player.current_particles() {
                            to_emit_particles.push(emit);
                        }

                        if player.update_destroy_animation(delta) {
                            continue;
                        }
                        if player_hard_drop_animations.contains_key(&player.player) {
                            continue;
                        }

                        let event = player.game.update(delta);
                        if event.is_none() {
                            continue;
                        }

                        let event = event.unwrap();
                        match event {
                            GameEvent::GameOver { .. } => {
                                new_game_over = Some(player.player);
                            }
                            GameEvent::Destroy(lines) => {
                                if lines[0].is_some() {
                                    player.animate_destroy(
                                        themes.theme().destroy_animation_type(),
                                        lines,
                                    );
                                }
                            }
                            GameEvent::Destroyed {
                                level_up,
                                send_garbage_lines,
                                ..
                            } => {
                                // if playing with all themes then the theme is auto switched after each level
                                if self.game_config.themes == MatchThemes::All && level_up {
                                    let level = player.game.level();
                                    if level > max_level {
                                        next_theme = true;
                                        max_level = level;
                                    }
                                }

                                if send_garbage_lines > 0 {
                                    garbage.push((player.player, send_garbage_lines));
                                }
                            }
                            _ => {}
                        }
                        themes.theme().play_sound_effects(event)?;
                        if let Some(emit) = themes.theme().emit_particles(event) {
                            to_emit_particles.push(emit);
                        }
                    }

                    // maybe start game over
                    if let Some(winner) = fixture.check_for_winning_player() {
                        sdl2::mixer::Music::halt();
                        fixture.set_winner(winner, themes.theme().game_over_animation_type());
                        let victory = GameEvent::Victory { player: winner };
                        themes.theme().play_sound_effects(victory)?;
                        if let Some(emit) = themes.theme().emit_particles(victory) {
                            to_emit_particles.push(emit);
                        }
                    } else if new_game_over.is_some() {
                        if let Some(loser) = new_game_over {
                            sdl2::mixer::Music::halt();
                            fixture.set_game_over(loser, themes.theme().game_over_animation_type());
                            let winners = fixture
                                .players
                                .iter()
                                .map(|p| p.player)
                                .filter(|p| *p != loser);
                            for winner in winners {
                                let victory = GameEvent::Victory { player: winner };
                                if let Some(emit) = themes.theme().emit_particles(victory) {
                                    to_emit_particles.push(emit);
                                }
                            }
                        }
                    } else {
                        // maybe send garbage
                        for (from_player, send_garbage_lines) in garbage {
                            fixture.send_garbage(from_player, send_garbage_lines);
                        }

                        // maybe change the theme
                        if next_theme {
                            themes.start_fade(&mut self.canvas)?;
                            themes.next();
                            themes.theme().music().fade_in(-1, 1000)?;
                        }
                    }
                }
                _ => {}
            }

            // update particles
            if !fixture.state().is_paused() {
                fg_particles.update(delta);

                if themes.render_bg_particles() {
                    bg_particles.update(delta);
                }
            }
            for emit in to_emit_particles.into_iter() {
                fg_particles.add_source(emit.into_source(&themes, &self.particle_scale));
            }

            // clear
            self.canvas
                .set_draw_color(themes.theme().background_color());
            self.canvas.clear();

            // draw bg particles
            if themes.render_bg_particles() {
                bg_particles.draw(&mut self.canvas)?;
            }

            // draw the game
            self.canvas
                .with_multiple_texture_canvas(
                    texture_refs.iter(),
                    |texture_canvas, texture_mode| match texture_mode {
                        TextureMode::PlayerBackground(player_id)
                            if !player_hard_drop_animations.contains_key(player_id) =>
                        {
                            let player = fixture.player(*player_id);
                            themes
                                .theme()
                                .draw_background(texture_canvas, &player.game)
                                .unwrap();
                        }
                        TextureMode::PlayerBoard(player_id)
                            if !player_hard_drop_animations.contains_key(player_id) =>
                        {
                            let player = fixture.player(*player_id);
                            themes
                                .theme()
                                .draw_board(
                                    texture_canvas,
                                    &player.game,
                                    player.current_destroy_animation(),
                                    player.current_game_over_animation(),
                                )
                                .unwrap();
                        }
                        _ => {}
                    },
                )
                .map_err(|e| e.to_string())?;

            let offsets: Vec<(f64, f64)> = fixture
                .players
                .iter_mut()
                .map(|p| p.next_impact_offset(delta))
                .collect();
            themes.draw_current(&mut self.canvas, &mut texture_refs, delta, offsets)?;

            // fg particles
            fg_particles.draw(&mut self.canvas)?;

            let mut remove_hard_drop_animations: Vec<u32> = vec![];
            for (player_id, animation) in player_hard_drop_animations.iter_mut() {
                if !animation.update(&mut self.canvas, delta)? {
                    remove_hard_drop_animations.push(*player_id);
                }
            }
            for player_id in remove_hard_drop_animations {
                player_hard_drop_animations.remove(&player_id);
                fixture.player_mut(player_id).impact();
            }

            if fixture.state().is_paused() {
                paused_screen.draw(&mut self.canvas)?;
            }

            self.canvas.present();
        }
    }
}

fn main() -> Result<(), String> {
    let mut rustris = TetrisSdl::new()?;
    let texture_creator = rustris.canvas.texture_creator();
    let (_, window_height) = rustris.canvas.window().size();
    let all_themes = AllThemes::new(
        &mut rustris.canvas,
        &texture_creator,
        &rustris.ttf,
        rustris.config,
        window_height,
    )?;
    let mut fg_particles = ParticleRender::new(
        &mut rustris.canvas,
        Particles::new(MAX_PARTICLES_PER_PLAYER * MAX_PLAYERS as usize),
        &texture_creator,
        rustris.particle_scale,
        vec![],
    )?;

    let mut bg_particles = ParticleRender::new(
        &mut rustris.canvas,
        Particles::new(MAX_BACKGROUND_PARTICLES),
        &texture_creator,
        rustris.particle_scale,
        all_themes.all(),
    )?;

    loop {
        match rustris.main_menu(&mut bg_particles)? {
            MainMenuAction::Start => {
                match rustris.game(&all_themes, &mut fg_particles, &mut bg_particles)? {
                    PostGameAction::NewHighScore(high_score) => {
                        rustris.new_high_score(high_score, &mut bg_particles)?
                    }
                    PostGameAction::ReturnToMenu => (),
                    PostGameAction::Quit => return Ok(()),
                }
            }
            MainMenuAction::ViewHighScores => rustris.view_high_score(&mut bg_particles)?,
            MainMenuAction::Quit => break
        }
    }
    Ok(())
}
