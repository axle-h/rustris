use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::theme::font::{FontRenderOptions, MetricSnips};
use crate::theme::geometry::BoardGeometry;
use crate::theme::sound::SoundThemeOptions;
use crate::theme::sprite_sheet::{MinoType, TetrominoSpriteSheet, TetrominoSpriteSheetMeta};
use crate::theme::{create_mask_texture, TetrominoScaleType, Theme, ThemeName, VISIBLE_PEEK};
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

pub struct RetroThemeOptions {
    name: ThemeName,
    block_size: u32,
    sprite_sheet_meta: TetrominoSpriteSheetMeta,
    background_file: &'static [u8],
    board_file: &'static [u8],
    game_over_file: &'static [u8],
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
    sound: SoundThemeOptions,
}

impl RetroThemeOptions {
    pub fn new(
        name: ThemeName,
        sprite_sheet_meta: TetrominoSpriteSheetMeta,
        background_file: &'static [u8],
        board_file: &'static [u8],
        game_over_file: &'static [u8],
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
        sound: SoundThemeOptions,
    ) -> Self {
        let block_size = sprite_sheet_meta.block_size();
        let geometry = BoardGeometry::new(block_size, game_point);
        let buffer_height = geometry.buffer_height() as i32;
        Self {
            name,
            block_size,
            sprite_sheet_meta,
            background_file,
            board_file,
            game_over_file,
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
            sound,
        }
    }
}

pub fn plus_buffer<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    buffer_height: u32,
    file_bytes: &'static [u8],
) -> Result<(Texture<'a>, u32, u32), String> {
    let raw = texture_creator.load_texture_bytes(file_bytes)?;
    let query = raw.query();
    let mut texture = texture_creator
        .create_texture_target(None, query.width, query.height + buffer_height)
        .map_err(|e| e.to_string())?;
    texture.set_blend_mode(BlendMode::Blend);
    canvas
        .with_texture_canvas(&mut texture, |c| {
            c.copy(
                &raw,
                None,
                Rect::new(0, buffer_height as i32, query.width, query.height),
            )
            .unwrap();
        })
        .map_err(|e| e.to_string())?;
    Ok((texture, query.width, query.height + buffer_height))
}

pub fn retro_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    options: RetroThemeOptions,
) -> Result<Theme<'a>, String> {
    let sprite_sheet = TetrominoSpriteSheet::new(
        canvas,
        texture_creator,
        options.sprite_sheet_meta.clone(),
        options.block_size,
    )?;

    let (board_texture, board_width, board_height) = plus_buffer(
        canvas,
        texture_creator,
        options.geometry.buffer_height(),
        options.board_file,
    )?;

    let board_mask_texture = create_mask_texture(canvas, texture_creator, &board_texture)?;

    let board_snip = Rect::new(
        options.board_point.x(),
        options.board_point.y(),
        board_width,
        board_height,
    );

    let (background_texture, bg_width, bg_height) = plus_buffer(
        canvas,
        texture_creator,
        options.geometry.buffer_height(),
        options.background_file,
    )?;

    let font = options.font_options.build(texture_creator)?;

    let game_over = texture_creator.load_texture_bytes(options.game_over_file)?;
    let sound = options.sound.clone().build()?;

    Ok(Theme {
        name: options.name,
        sprite_sheet,
        geometry: options.geometry,
        board_texture,
        board_mask_texture,
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
        particle_color: None,
    })
}
