mod game;
mod theme;
mod events;

extern crate sdl2;

use std::cmp::min;
use std::fmt::Debug;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::InitFlag;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator};
use sdl2::video::WindowContext;
use events::Events;
use game::Game;
use game::block::BlockState;
use game::board::{BOARD_HEIGHT, BOARD_WIDTH};
use game::timing::Timing;
use crate::events::GameEventKey;
use crate::game::{GameMetrics, GameState};
use crate::game::random::RandomMode;
use crate::theme::Theme;
use crate::theme::game_boy::{GameBoyTheme, GameBoyPalette};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const PLAYERS: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextureMode {
    Player(u32, Rect),
    Background
}

fn scale_rect(rect: Rect, scale: u32) -> Rect {
    Rect::new(
        rect.x * scale as i32, rect.y * scale as i32,
        rect.width() * scale, rect.height() * scale
    )
}

fn scale_and_offset_rect(rect: Rect, scale: u32, offset_x: i32, offset_y: i32) -> Rect {
    Rect::new(
        rect.x * scale as i32 + offset_x, rect.y * scale as i32 + offset_y,
        rect.width() * scale, rect.height() * scale
    )
}

struct Player<'a> {
    player: u32,
    board_texture: Texture<'a>,
    board_snip: Rect
}

impl<'a> Player<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>, player: u32, theme: &dyn Theme) -> Result<Self, String> {
        let mut board_snip = theme.board_snip(player);
        let mut texture = texture_creator
            .create_texture_target(None, board_snip.width(), board_snip.height())
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);
        Ok(
            Self {
                player,
                board_texture: texture,
                board_snip
            }
        )
    }
}

struct Games {
    games: Vec<Game>,
    paused: bool
}

impl Games {
    fn new(players: u32) -> Self {
        if players == 0 {
            panic!("must have at least one player")
        }
        Self {
            games: (1..=players)
                .map(|player| Game::new(player, 0, RandomMode::Bag))
                .collect::<Vec<Game>>(),
            paused: false
        }
    }

    fn metrics(&self) -> Vec<GameMetrics> {
        self.games.iter().map(|g| g.metrics()).collect()
    }
    
    fn unset_soft_drop(&mut self) {
        for game in self.games.iter_mut() {
            game.set_soft_drop(false);
        }
    }

    fn toggle_paused(&mut self) -> Option<bool> {
        self.paused = !self.paused;
        Some(self.paused)
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn mut_game<F>(&mut self, player: u32, mut f: F) -> Option<bool>
            where F: FnMut(&mut Game) -> bool {
        debug_assert!(player > 0);
        if self.paused {
            None
        } else {
            self.games.get_mut(player as usize - 1).map(f)
        }
    }

    fn game(&self, player: u32) -> &Game {
        debug_assert!(player > 0);
        self.games.get(player as usize - 1).unwrap()
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    sdl2::image::init(InitFlag::PNG)?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Tetris", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let (window_width, window_height) = window.size();
    let mut canvas = window
        .into_canvas()
        .target_texture()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut fps = sdl2::gfx::framerate::FPSManager::new();
    fps.set_framerate(60)?;

    let timing = Timing::new(fps.get_framerate() as u32);
    let mut events = Events::new();

    let texture_creator = canvas.texture_creator();
    let theme = GameBoyTheme::new(PLAYERS, &mut canvas, &texture_creator, GameBoyPalette::GameBoyLight)?;
    let (bg_width, bg_height) = theme.background_size();

    // find best integer scale
    let scale = min(window_width / bg_width, window_height / bg_height);

    let mut bg_rect = scale_rect(Rect::new(0, 0, bg_width, bg_height), scale);
    bg_rect.center_on((window_width as i32 / 2, window_height as i32 / 2));
    let mut bg_texture = texture_creator
        .create_texture_target(None, bg_width, bg_height)
        .map_err(|e| e.to_string())?;
    bg_texture.set_blend_mode(BlendMode::Blend);

    let mut players = (1..=PLAYERS)
        .map(|player| Player::new(&texture_creator, player, &theme).unwrap())
        .collect::<Vec<Player>>();

    let mut games = Games::new(PLAYERS);

    // push mut refs of all textures and their render modes into a single vector so we can render to texture in one loop
    let mut texture_refs: Vec<(&mut Texture, TextureMode)> = vec![
        (&mut bg_texture, TextureMode::Background)
    ];

    for player in players.iter_mut() {
        let scaled_board_rect = scale_and_offset_rect(player.board_snip, scale, bg_rect.x(), bg_rect.y());
        texture_refs.push(
            (&mut player.board_texture, TextureMode::Player(player.player, scaled_board_rect))
        )
    }

    let mut lighten_screen = texture_creator
        .create_texture_target(None, bg_rect.width(), bg_rect.height())
        .map_err(|e| e.to_string())?;
    lighten_screen.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut lighten_screen, |texture_canvas| {
        texture_canvas.set_draw_color(Color::RGBA(255, 255, 255, 150));
        texture_canvas.fill_rect(Rect::new(0, 0, bg_rect.width(), bg_rect.height())).unwrap();
    }).map_err(|e| e.to_string())?;


    canvas.set_draw_color(Color::WHITE); // todo take from theme
    canvas.clear();
    canvas.present();

    'main: loop {
        let delta = timing.frame_duration(1);

        games.unset_soft_drop();

        for event in events.update(delta, event_pump.poll_iter()) {
            // TODO emit the events here so we can play sound effects for them in the theme
            let success = match event {
                GameEventKey::MoveLeft { player } => games.mut_game(player, |g| g.left()),
                GameEventKey::MoveRight { player } => games.mut_game(player, |g| g.right()),
                GameEventKey::SoftDrop { player } => games.mut_game(player, |g| g.set_soft_drop(true)),
                GameEventKey::HardDrop { player } => games.mut_game(player, |g| g.hard_drop()),
                GameEventKey::RotateClockwise { player } => games.mut_game(player, |g| g.rotate(true)),
                GameEventKey::RotateAnticlockwise { player } => games.mut_game(player, |g| g.rotate(false)),
                GameEventKey::Hold { player } => games.mut_game(player, |g| g.hold()),
                GameEventKey::Pause => games.toggle_paused(),
                GameEventKey::Quit => { break 'main; }
            }.unwrap_or(false);
        }

        if !games.paused {
            for game in games.games.iter_mut() {
                match game.update(delta) {
                    GameState::GameOver(_) => {
                        // TODO game over screen
                        println!("GAME OVER player {}", game.player());
                        break 'main;
                    },
                    GameState::Destroy(pattern) => {
                        // TODO send garbage
                    }
                    _ => {}
                }
            }
        }

        let game_metrics = games.metrics();

        // draw the game
        canvas.set_draw_color(Color::WHITE); // todo take from theme
        canvas.clear();
        canvas.with_multiple_texture_canvas(texture_refs.iter(), |texture_canvas, texture_mode| {
            match texture_mode {
                TextureMode::Player(player, _) => {
                    theme.draw_board(texture_canvas, games.game(*player)).unwrap();
                }
                TextureMode::Background => {
                    theme.draw_background(texture_canvas, &game_metrics).unwrap();
                }
            }
        }).map_err(|e| e.to_string())?;

        for (texture, texture_mode) in texture_refs.iter_mut() {
            match texture_mode {
                TextureMode::Player(_, rect) => {
                    // the game board is always rendered upside down as the game impl has a reversed y coordinate system to sdl
                    canvas.copy_ex(texture, None, *rect, 0.0, None, false, true)?;
                },
                TextureMode::Background => {
                   canvas.copy(texture, None, bg_rect)?;
                }
            }
        }

        if games.is_paused() {
            canvas.copy(&lighten_screen, None, bg_rect)?;

            let texture = theme.pause_texture();
            let query = texture.query();
            let mut paused_rect = scale_rect(Rect::new(0, 0, query.width, query.height), scale);
            paused_rect.center_on(bg_rect.center());
            canvas.copy(texture, None, paused_rect)?;
        }

        canvas.present();
        fps.delay(); // todo probably need to update fps timing
    }

    Ok(())
}
