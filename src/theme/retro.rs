use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::theme::sound::SoundThemeOptions;
use crate::theme::{TetrominoScaleType, Theme, VISIBLE_PEEK};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::theme::font::{FontRenderOptions, MetricSnips};
use crate::theme::geometry::BoardGeometry;
use crate::theme::sprite_sheet::{MinoType, TetrominoSpriteSheet, TetrominoSpriteSheetMeta};

pub struct RetroThemeOptions {
    name: String,
    config: Config,
    block_size: u32,
    sprite_sheet_meta: TetrominoSpriteSheetMeta,
    background_file: String,
    board_file: String,
    game_over_file: String,
    geometry: BoardGeometry,
    peek_snips: [Rect; VISIBLE_PEEK],
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
}


pub fn plus_buffer<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    buffer_height: u32,
    filename: String
) -> Result<(Texture<'a>, u32, u32), String> {
    let raw = texture_creator.load_texture(filename)?;
    let query = raw.query();
    let mut texture = texture_creator
        .create_texture_target(None, query.width, query.height + buffer_height)
        .map_err(|e| e.to_string())?;
    texture.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut texture, |c| {
        c.copy(&raw, None, Rect::new(0, buffer_height as i32, query.width, query.height)).unwrap();
    }).map_err(|e| e.to_string())?;
    Ok((texture, query.width, query.height + buffer_height))
}

pub fn retro_theme<'a>(canvas: &mut WindowCanvas,
                   texture_creator: &'a TextureCreator<WindowContext>,
                   options: RetroThemeOptions,
) -> Result<Theme<'a>, String> {
    let sprite_sheet = TetrominoSpriteSheet::new(canvas, texture_creator, options.sprite_sheet_meta.clone(), options.block_size)?;

    let (board_texture, board_width, board_height) = plus_buffer(
        canvas,
        texture_creator,
        options.geometry.buffer_height(),
        options.board_file()
    )?;
    let board_snip = Rect::new(
        options.board_point.x(),
        options.board_point.y(),
        board_width,
        board_height,
    );

    let (background_texture, bg_width, bg_height) = plus_buffer(canvas, texture_creator, options.geometry.buffer_height(), options.background_file())?;

    let font = options.font_options.build(texture_creator)?;

    let game_over = texture_creator.load_texture(options.game_over_file())?;
    let sound = options.sound.clone().build()?;

    Ok(Theme {
        sprite_sheet,
        geometry: options.geometry,
        board_texture,
        board_snip,
        background_texture,
        background_size: (bg_width, bg_height),
        score_snip: options.score,
        level_snip: options.levels,
        lines_snip: options.lines,
        peek_snips: options.peek_snips,
        hold_snip: options.hold_snip,
        game_over,
        font,
        sound,
        background_color: options.background_color,
        destroy_animation: options.destroy_animation,
        game_over_animation: options.game_over_animation,
        ghost_mino_type: MinoType::Ghost,
        tetromino_scale_type: TetrominoScaleType::Center,
        particle_color: None
    })
}