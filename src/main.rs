#![windows_subsystem = "windows"]

mod animation;
mod build_info;
mod config;
mod event;
mod game;
mod game_input;
mod high_score;
mod menu;
mod menu_input;
mod player;
mod scale;
mod theme;
mod theme_context;
mod particles;

extern crate sdl2;

use crate::animation::game_over::GameOverAnimate;
use crate::animation::hard_drop::HardDropAnimation;
use crate::config::{Config, GameConfig, MatchRules, VideoMode};
use crate::event::GameEvent;
use crate::game_input::GameInputKey;
use crate::high_score::entry::HighScoreEntry;
use crate::high_score::table::HighScoreTable;
use crate::high_score::view::HighScoreTableView;
use crate::menu::{Menu, MenuAction};
use crate::menu_input::{MenuInputContext, MenuInputKey};
use crate::player::MatchState;

use game_input::GameInputContext;
use player::Match;
use sdl2::image::{InitFlag as ImageInitFlag, Sdl2ImageContext};
use sdl2::mixer::{InitFlag as MixerInitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::pixels::Color;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::sys::mixer::MIX_CHANNELS;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use sdl2::{AudioSubsystem, EventPump, Sdl};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, SystemTime};
use sdl2::rect::Rect;
use theme_context::{PlayerTextures, TextureMode, ThemeContext};
use crate::particles::geometry::PointF;
use crate::particles::Particles;
use crate::particles::render::ParticleRender;
use crate::particles::source::{ParticleModulation, ParticlePositionSource, ParticleSource};

const MAX_PLAYERS: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMenuAction {
    NewMatch { config: GameConfig },
    ViewHighScores,
}

struct TetrisSdl {
    config: Config,
    _sdl: Sdl,
    ttf: Sdl2TtfContext,
    _image: Sdl2ImageContext,
    canvas: WindowCanvas,
    event_pump: EventPump,
    _audio: AudioSubsystem,
    texture_creator: TextureCreator<WindowContext>
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

        let mut window_builder = video.window("Tetris", width, height);
        match config.video.mode {
            VideoMode::FullScreen { .. } => {
                window_builder.fullscreen();
            }
            VideoMode::FullScreenDesktop => {
                window_builder.fullscreen_desktop();
            }
            _ => {}
        };

        let canvas_builder = window_builder
            .position_centered()
            .opengl()
            .allow_highdpi()
            .build()
            .map_err(|e| e.to_string())?
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

        let texture_creator = canvas.texture_creator();
        let event_pump = sdl.event_pump()?;

        let audio = sdl.audio()?;
        sdl2::mixer::open_audio(44_100, AUDIO_S16LSB, DEFAULT_CHANNELS, 512)?;
        let _mixer_context = sdl2::mixer::init(MixerInitFlag::OGG)?;
        sdl2::mixer::allocate_channels((MAX_PLAYERS * MIX_CHANNELS) as i32);
        sdl2::mixer::Music::set_volume(config.audio.music_volume());

        Ok(Self {
            config,
            _sdl: sdl,
            ttf,
            _image: image,
            canvas,
            event_pump,
            _audio: audio,
            texture_creator,
        })
    }

    pub fn main_menu(&mut self, game_config: Option<GameConfig>) -> Result<Option<MainMenuAction>, String> {
        let mut game_config = match game_config {
            None => GameConfig::new(1, 0, MatchRules::Battle),
            Some(config) => config
        };
        let inputs = MenuInputContext::new(self.config.input);
        let menu_items = vec![
            (
                "players",
                MenuAction::SelectList {
                    items: vec!["1", "2"],
                    current: game_config.players as usize - 1,
                },
            ),
            (
                "mode",
                MenuAction::SelectList {
                    items: vec![
                        "battle",
                        "40 line sprint",
                        "10,000 point sprint",
                        "marathon",
                    ],
                    current: match game_config.rules {
                        MatchRules::Battle => 0,
                        MatchRules::LineSprint { .. } => 1,
                        MatchRules::ScoreSprint { .. } => 2,
                        MatchRules::Marathon => 3
                    },
                },
            ),
            (
                "level",
                MenuAction::SelectList {
                    items: vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
                    current: game_config.level as usize,
                },
            ),
            ("high scores", MenuAction::Select),
            ("start", MenuAction::Select),
            ("quit", MenuAction::Select),
        ];
        let mut menu = Menu::new(
            menu_items,
            &self.ttf,
            &self.texture_creator,
            self.config,
            self.canvas.window().size(),
        )?;
        menu.play_music()?;
        'menu: loop {
            let events = inputs.parse(self.event_pump.poll_iter());
            if events.contains(&MenuInputKey::Quit) {
                return Ok(None);
            }
            if events.contains(&MenuInputKey::Start) {
                break 'menu;
            }
            if !events.is_empty() {
                menu.play_sound()?;
            }

            for key in events.into_iter() {
                match menu.read_key(key) {
                    None => {}
                    Some((name, action)) => match name {
                        "players" => game_config.players = action.parse::<u32>().unwrap(),
                        "mode" => {
                            game_config.rules = match action {
                                "battle" => MatchRules::Battle,
                                "40 line sprint" => MatchRules::LineSprint { lines: 40 },
                                "10,000 point sprint" => MatchRules::ScoreSprint { score: 10_000 },
                                "marathon" => MatchRules::Marathon,
                                _ => unreachable!(),
                            }
                        }
                        "level" => game_config.level = action.parse::<u32>().unwrap(),
                        "high scores" => return Ok(Some(MainMenuAction::ViewHighScores)),
                        "start" => break 'menu,
                        "quit" => return Ok(None),
                        _ => {}
                    },
                }
            }

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            menu.draw(&mut self.canvas)?;

            self.canvas.present();
        }
        Ok(Some(MainMenuAction::NewMatch {
            config: game_config,
        }))
    }

    pub fn view_high_score(&mut self) -> Result<(), String> {
        let inputs = MenuInputContext::new(self.config.input);
        let high_scores = HighScoreTable::load()?;
        if high_scores.entries().is_empty() {
            return Ok(());
        }

        let mut view = HighScoreTableView::new(
            high_scores,
            &self.ttf,
            &self.texture_creator,
            self.canvas.window().size(),
        )?;
        'menu: loop {
            let events = inputs.parse(self.event_pump.poll_iter());
            if !events.is_empty() {
                // any button press
                break 'menu;
            }
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            view.draw(&mut self.canvas)?;

            self.canvas.present();
        }
        Ok(())
    }

    pub fn game(&mut self, game_config: GameConfig) -> Result<(), String> {
        let mut inputs = GameInputContext::new(self.config.input);
        let mut fixture = Match::new(game_config, self.config);
        let window_size = self.canvas.window().size();
        let mut themes = ThemeContext::new(
            &mut self.canvas,
            &self.texture_creator,
            game_config.players,
            window_size,
            self.config,
        )?;

        let mut player_textures = (0..game_config.players)
            .map(|_| {
                PlayerTextures::new(
                    &self.texture_creator,
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

        let particle_scale = particles::scale::Scale::new(self.canvas.window().size());
        let mut particles = ParticleRender::new(Particles::new(10000), &self.texture_creator, particle_scale)?;

        themes.theme_mut().music().play(-1)?;

        let mut high_score_entry: Option<HighScoreEntry> = None;
        let mut player_hard_drop_animations: HashMap<u32, HardDropAnimation> = HashMap::new();
        let mut t0 = SystemTime::now();
        let mut max_level = 0;
        'game: loop {
            let now = SystemTime::now();
            let delta = now.duration_since(t0).map_err(|e| e.to_string())?;
            t0 = now;

            fixture.unset_flags();
            for hard_dropping_player in player_hard_drop_animations.keys() {
                fixture.set_hard_dropping(*hard_dropping_player);
            }
            let events = inputs
                .update(delta, self.event_pump.poll_iter())
                .into_iter()
                .flat_map(|input| match high_score_entry.as_mut() {
                    None => match input {
                        GameInputKey::MoveLeft { player } => fixture.mut_game(player, |g| g.left()),
                        GameInputKey::MoveRight { player } => {
                            fixture.mut_game(player, |g| g.right())
                        }
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
                            _ => Some(GameEvent::Quit),
                        },
                        GameInputKey::Quit => Some(GameEvent::Quit),
                        GameInputKey::NextTheme => Some(GameEvent::NextTheme),
                    },
                    Some(entry) => match input {
                        GameInputKey::MoveLeft { player } if player == entry.player() => {
                            entry.left()
                        }
                        GameInputKey::MoveRight { player } if player == entry.player() => {
                            entry.right()
                        }
                        GameInputKey::RotateClockwise { player } if player == entry.player() => {
                            entry.down()
                        }
                        GameInputKey::RotateAnticlockwise { player }
                            if player == entry.player() =>
                        {
                            entry.up()
                        }
                        GameInputKey::Quit => Some(GameEvent::Quit),
                        _ => None,
                    },
                })
                .collect::<Vec<GameEvent>>();

            for event in events.iter() {
                match event {
                    GameEvent::Quit => break 'game,
                    GameEvent::Paused => sdl2::mixer::Music::pause(),
                    GameEvent::UnPaused => sdl2::mixer::Music::resume(),
                    GameEvent::NextTheme if !fixture.state().is_game_over() => {
                        themes.start_fade(&mut self.canvas)?;
                        themes.next();

                        // handle music
                        match fixture.state() {
                            MatchState::Normal => {
                                themes.theme_mut().music().fade_in(-1, 1000)?;
                            }
                            MatchState::Paused => {
                                // switch music but pause it immediately
                                themes.theme_mut().music().play(-1)?;
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
                        let mino_rects = theme.mino_rects(*player_id, *minos);
                        let dropped_pixels = theme.rows_to_pixels(*dropped_rows);
                        let hard_drop_animation = HardDropAnimation::new(
                            &self.canvas,
                            &self.texture_creator,
                            mino_rects,
                            dropped_pixels,
                        )?;
                        player_hard_drop_animations.insert(*player_id, hard_drop_animation);
                    }
                    GameEvent::HighScoreEntry if high_score_entry.is_some() => {
                        match high_score_entry.unwrap().to_high_score() {
                            None => {}
                            Some(high_score) => fixture.save_high_score(high_score)?,
                        }
                        break 'game;
                    }
                    _ => {}
                }

                themes.theme_mut().receive_event(*event)?;
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
                    match maybe_high_score {
                        Some(high_score) if game_over_done => {
                            // start high score entry
                            fixture.set_high_score_entry();
                            high_score_entry = Some(HighScoreEntry::new(
                                high_score,
                                &self.texture_creator,
                                themes.theme(),
                                themes.scale(),
                            )?);
                        }
                        _ => {}
                    }
                }
                MatchState::Normal if !themes.is_fading() => {
                    let mut garbage: Vec<(u32, u32)> = vec![];
                    let mut new_game_over: Option<u32> = None;
                    for player in fixture.players.iter_mut() {
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
                        match event.unwrap() {
                            GameEvent::GameOver(_) => {
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
                                if level_up {
                                    let level = player.game.level();
                                    if level > max_level {
                                        // todo option to disable this in config
                                        max_level = level;
                                        themes.start_fade(&mut self.canvas)?;
                                        themes.next();
                                        themes.theme_mut().music().fade_in(-1, 1000)?;
                                    }
                                }

                                if send_garbage_lines > 0 {
                                    garbage.push((player.player, send_garbage_lines));
                                }
                            }
                            _ => {}
                        }
                        themes.theme_mut().receive_event(event.unwrap())?;
                    }

                    // maybe start game over
                    if let Some(winner) = fixture.check_for_winning_player() {
                        sdl2::mixer::Music::halt();
                        fixture.set_winner(winner, themes.theme().game_over_animation_type());
                        themes.theme_mut().receive_event(GameEvent::Victory)?;
                    } else if new_game_over.is_some() {
                        if let Some(loser) = new_game_over {
                            sdl2::mixer::Music::halt();
                            fixture.set_game_over(loser, themes.theme().game_over_animation_type());
                        } else {
                            unreachable!();
                        }
                    } else {
                        // maybe send garbage
                        for (from_player, send_garbage_lines) in garbage {
                            fixture.send_garbage(from_player, send_garbage_lines);
                        }
                    }
                }
                _ => {}
            }

            // draw the game
            self.canvas
                .set_draw_color(themes.theme().background_color());
            self.canvas.clear();
            self.canvas
                .with_multiple_texture_canvas(
                    texture_refs.iter(),
                    |texture_canvas, texture_mode| match texture_mode {
                        TextureMode::PlayerBackground(player_id)
                            if !player_hard_drop_animations.contains_key(player_id) =>
                        {
                            let player = fixture.player(*player_id);
                            themes
                                .theme_mut()
                                .draw_background(texture_canvas, &player.game)
                                .unwrap();
                        }
                        TextureMode::PlayerBoard(player_id)
                            if !player_hard_drop_animations.contains_key(player_id) =>
                        {
                            let player = fixture.player(*player_id);
                            themes
                                .theme_mut()
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


            // particles
            for player in fixture.players.iter() {
                for (j, emit) in player.current_particles() {
                    let line_snip = themes.player_line_snip(player.player, j);
                    let source = emit.build_rect_source(&particle_scale, line_snip);
                    particles.add_source(source);
                };
            }

            if !fixture.state().is_paused() {
                particles.update(delta);
            }
            particles.draw(&mut self.canvas)?;

            let mut remove_hard_drop_animations: Vec<u32> = vec![];
            for (player_id, animation) in player_hard_drop_animations.iter_mut() {
                if !animation.update(&mut self.canvas, delta)? {
                    remove_hard_drop_animations.push(*player_id);
                }
            }
            for player_id in remove_hard_drop_animations {
                player_hard_drop_animations.remove(&player_id);
                fixture.player_mut(player_id).impact()
            }

            if fixture.state().is_paused() {
                themes.draw_paused(&mut self.canvas)?;
            }

            match high_score_entry.as_mut() {
                None => {}
                Some(entry) => {
                    themes.draw_hide_game(&mut self.canvas)?;
                    entry.draw(&mut self.canvas, themes.theme())?;
                }
            }

            self.canvas.present();
        }

        Ok(())
    }

    fn particle_demo(&mut self) -> Result<(), String> {
        let particle_scale = particles::scale::Scale::new(self.canvas.window().size());
        let mut particles = ParticleRender::new(Particles::new(1000), &self.texture_creator, particle_scale)?;

        let (window_width, window_height) = self.canvas.window().size();
        let rect = Rect::from_center((window_width as i32 / 2, window_height as i32 / 2), 200, 50);

        particles.add_source(
            ParticleSource::new(
                particle_scale.rect_lattice(rect),
                ParticleModulation::Constant { count: 200, step: Duration::from_secs(3) }
            ).with_velocity((PointF::new(0.0, -0.4), PointF::new(0.1, 0.1)))
                .with_gravity(1.5)
                .with_anchor(Duration::from_millis(500))
                .with_fade_in(Duration::from_millis(500))
        );

        let mut inputs = GameInputContext::new(self.config.input);
        let mut t0 = SystemTime::now();
        'game: loop {
            let now = SystemTime::now();
            let delta = now.duration_since(t0).map_err(|e| e.to_string())?;
            t0 = now;

            for key in inputs.update(delta, self.event_pump.poll_iter()).into_iter() {
                match key {
                    GameInputKey::Quit => {
                        break 'game
                    }
                    _ => {}
                }
            }

            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            particles.update(delta);
            particles.draw(&mut self.canvas)?;

            self.canvas.present();
        }

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut last_game_config: Option<GameConfig> = None;
    let mut tetris = TetrisSdl::new()?;
    loop {
        match tetris.main_menu(last_game_config)? {
            Some(MainMenuAction::NewMatch { config }) => {
                last_game_config = Some(config);
                tetris.game(config)?;
            },
            Some(MainMenuAction::ViewHighScores) => tetris.view_high_score()?,
            _ => {
                break;
            }
        }
    }
    Ok(())

    // let mut tetris = TetrisSdl::new()?;
    // tetris.particle_demo()
}
