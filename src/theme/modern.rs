use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::GameOverAnimationType;
use crate::config::Config;
use crate::font::FontType;
use crate::particles::prescribed::{PlayerParticleTarget, PlayerTargetedParticles, PrescribedParticles};
use crate::theme::font::{FontRender, MetricSnips};
use crate::theme::geometry::{BoardGeometry, VISIBLE_BOARD_HEIGHT};
use crate::theme::sound::SoundThemeOptions;
use crate::theme::sprite_sheet::{MinoType, TetrominoSpriteSheet, TetrominoSpriteSheetMeta};
use crate::theme::{TetrominoScaleType, Theme, VISIBLE_PEEK};

const MIN_VERTICAL_BUFFER_PCT: f64 = 0.1;
const BOARD_BORDER_PCT_OF_BLOCK: f64 = 0.5;
const BOARD_BOARDER_SHADOW: u8 = 0x99;
const TETROMINO_PCT_OF_BLOCK: f64 = 1.5;
const BIG_TETROMINO_PCT_OF_BLOCK: f64 = 2.5;

// 3 blocks is good as most are 3 blocks wide, then we scale I down and O up to 3.
const TETROMINO_PREFERRED_SCALE: f64 = TETROMINO_PCT_OF_BLOCK / 3.0;
const BIG_TETROMINO_PREFERRED_SCALE: f64 = BIG_TETROMINO_PCT_OF_BLOCK / 3.0;

const VERTICAL_GUTTER_PCT_OF_BLOCK: f64 = 0.2;

const SCORE_HEADER: &str = "SCORE";
const MAX_SCORE: u32 = 999999;

const LEVEL_HEADER: &str = "LEVEL";
const MAX_LEVEL: u32 = 999;

const LINES_HEADER: &str = "LINES";
const MAX_LINES: u32 = 999;

fn block(row: i32, col: i32) -> Point {
    Point::new(4 + 56 * col, 4 + 56 * row)
}

fn mino(col: i32) -> (Point, Point) {
    // (normal block, stack block)
    (block(0, col), block(1, col))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameMetricType {
    Score,
    Level,
    Lines
}

impl GameMetricType {
    fn label(&self) -> &str {
        match self {
            GameMetricType::Score => "SCORE",
            GameMetricType::Level => "LEVEL",
            GameMetricType::Lines => "LINES"
        }
    }
}

struct GameMetricsRow {
    metric: GameMetricType,
    value: MetricSnips,
    label: Point,
    label_width: u32,
    value_width: u32
}

impl GameMetricsRow {
    fn width(&self) -> u32 {
        self.value_width.max(self.label_width)
    }
}

struct GameMetricsTable {
    rows: Vec<GameMetricsRow>
}

impl GameMetricsTable {
    const VERTICAL_SPACING: u32 = 2;

    fn new(geometry: &BoardGeometry, font: &FontRender, font_bold: &FontRender, labelled_max: &[(GameMetricType, u32)]) -> Self {
        let mut y = geometry.visible_height() as i32; // start from the bottom
        let x = 0;
        let rows = labelled_max.iter().rev().copied().map(|(metric, max)| {
            let (value_width, value_height) = font.number_size(max);
            let (label_width, label_height) = font_bold.string_size(metric.label());
            y -= value_height as i32;
            let value = MetricSnips::left((x, y), max);
            y -= (label_height + Self::VERTICAL_SPACING) as i32;
            let label = Point::new(x, y);
            GameMetricsRow { metric, value, label, value_width, label_width }
        }).collect();

        Self { rows }
    }

    fn into_right_aligned(self) -> Self {
        let width = self.width() as i32;
        let rows = self.rows.into_iter()
            .map(|r|
                GameMetricsRow {
                    value: MetricSnips::right((width, r.value.point().y()), r.value.max_value()),
                    label: Point::new(width - r.label_width as i32, r.label.y()),
                    metric: r.metric,
                    label_width: r.label_width,
                    value_width: r.value_width
                }
            ).collect();
        Self { rows }
    }

    fn offset_x(&mut self, x: i32) {
        for row in self.rows.iter_mut() {
            row.value = row.value.offset(x, 0);
            row.label = row.label.offset(x, 0);
        }
    }

    fn width(&self) -> u32 {
        self.rows.iter().map(|r| r.width()).sum()
    }
}

pub fn modern_theme<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    ttf: &Sdl2TtfContext,
    config: Config,
    window_height: u32
) -> Result<Theme<'a>, String> {
    let block_size = (window_height as f64 - (2.0 * window_height as f64 * MIN_VERTICAL_BUFFER_PCT)) / VISIBLE_BOARD_HEIGHT as f64;
    let border_weight = (block_size * BOARD_BORDER_PCT_OF_BLOCK).round() as u32;
    let vertical_gutter = (VERTICAL_GUTTER_PCT_OF_BLOCK * block_size).round() as u32;
    let tetromino_size = (TETROMINO_PCT_OF_BLOCK * block_size).round() as u32;
    let big_tetromino_size = (BIG_TETROMINO_PCT_OF_BLOCK * block_size).round() as u32;
    let block_size = block_size.round() as u32;

    let geometry = BoardGeometry::new(block_size, (border_weight as i32, 0));

    let font_size = 3 * block_size / 4;
    let font = FontRender::from_font(canvas, texture_creator, ttf, FontType::Normal, font_size, Color::WHITE)?;
    let font_bold = FontRender::from_font(canvas, texture_creator, ttf, FontType::Bold, font_size, Color::WHITE)?;

    let metrics_left = GameMetricsTable::new(
        &geometry, &font, &font_bold,
        &[(GameMetricType::Score, MAX_SCORE)]
    ).into_right_aligned();
    let left_gutter_width = metrics_left.width().max(tetromino_size) + vertical_gutter;

    let board_snip = Rect::new(
        left_gutter_width as i32, 0,
        geometry.width() + 2 * border_weight,
        geometry.visible_height() + border_weight
    );

    let mut metrics_right = GameMetricsTable::new(
        &geometry, &font, &font_bold,
        &[(GameMetricType::Level, MAX_LEVEL), (GameMetricType::Lines, MAX_LINES)]
    );
    metrics_right.offset_x(board_snip.right() + vertical_gutter as i32);

    let right_gutter_width = metrics_right.width().max(big_tetromino_size) + vertical_gutter;

    let background_width = left_gutter_width + board_snip.width() + right_gutter_width;
    let background_height = board_snip.height();

    let hold_snip = Rect::new(
        (left_gutter_width - tetromino_size - vertical_gutter) as i32, geometry.buffer_height() as i32,
        tetromino_size, tetromino_size
    );

    let peek_offset_x = (big_tetromino_size - tetromino_size) / 2;
    let peek_snips = (0..VISIBLE_PEEK)
        .map(|i| {
            let (size, offset_x, offset_y) = if i == 0 { (big_tetromino_size, 0, 0) } else { (tetromino_size, peek_offset_x, block_size) };
            Rect::new(
                board_snip.x() + board_snip.width() as i32 + vertical_gutter as i32 + offset_x as i32,
                geometry.buffer_height() as i32 + i as i32 * (vertical_gutter + size) as i32 + offset_y as i32,
                size,
                size
            )
        })
        .collect::<Vec<Rect>>()
        .try_into()
        .unwrap();

    let sprite_sheet_meta = TetrominoSpriteSheetMeta::new(
        "resource/modern/sprites.png",
        48,
        mino(6),
        mino(1),
        mino(3),
        mino(7),
        mino(2),
        mino(4),
        mino(5),
        block(0, 0),
        0x50
    );
    let mut borders = vec![];

    let step = BOARD_BOARDER_SHADOW / border_weight as u8;
    for i in 0..border_weight {
        let j = border_weight - i - 1;
        let alpha = if j > 0 { BOARD_BOARDER_SHADOW - j as u8 * step } else { 0xff };
        let rect = Rect::new(
             i as i32, geometry.buffer_height() as i32,
            geometry.width() - 2 * i + 2 * border_weight, geometry.height() - i + border_weight
        );
        borders.push((rect, alpha))
    }

    let sound = SoundThemeOptions::default("modern", config.audio)
        .with_alt_send_garbage()
        .with_hard_drop()
        .with_hold()
        .with_distinct_clear()
        .build()?;

    let all_metrics = metrics_left.rows.into_iter()
        .chain(metrics_right.rows.into_iter())
        .collect::<Vec<GameMetricsRow>>();

    let mut board_texture = texture_creator
        .create_texture_target(None, board_snip.width(), board_snip.height())
        .map_err(|e| e.to_string())?;
    board_texture.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut board_texture, |c| {
        c.set_draw_color(Color::RGBA(0, 0, 0, 0));
        c.clear();
        for (r, color) in borders.iter().copied() {
            c.set_draw_color(Color::RGBA(color, color, color, color));
            c.draw_rect(r).unwrap();
        }
        // re-clear the board to get rid of the top of the border
        c.set_draw_color(Color::RGBA(0, 0, 0, 0));
        c.fill_rect(Rect::new(border_weight as i32, 0, geometry.width(), geometry.visible_height())).unwrap();
    }).map_err(|e| e.to_string())?;

    let mut bg_texture = texture_creator
        .create_texture_target(None, background_width, background_height)
        .map_err(|e| e.to_string())?;
    bg_texture.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut bg_texture, |c| {
        for row in all_metrics.iter() {
            font_bold.render_string(c, row.label, &row.metric.label()).unwrap();
        }
    }).map_err(|e| e.to_string())?;


    let game_over_font = FontRender::from_font(canvas, texture_creator, ttf, FontType::Bold, font_size * 2, Color::WHITE)?;
    let (game_text_width, game_text_height) = game_over_font.string_size("GAME");
    let (over_text_width, over_text_height) = game_over_font.string_size("OVER");
    let game_over_width = game_text_width.max(over_text_width);
    let game_over_height = game_text_height + vertical_gutter + over_text_height;
    let mut game_over = texture_creator
        .create_texture_target(None, game_over_width, game_over_height)
        .map_err(|e| e.to_string())?;
    game_over.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut game_over, |c| {
        let top_center = Rect::new(0, 0, game_over_width, game_text_height).center();
        let game_text_rect = Rect::from_center(top_center, game_text_width, game_text_height);
        game_over_font.render_string(c, game_text_rect.top_left(), "GAME").unwrap();
        let bottom_center = Rect::new(0, game_text_height as i32, game_over_width, over_text_height).center();
        let over_text_rect = Rect::from_center(bottom_center, over_text_width, over_text_height);
        game_over_font.render_string(c, over_text_rect.top_left(), "OVER").unwrap();
    }).map_err(|e| e.to_string())?;

    Ok(
        Theme {
            sprite_sheet: TetrominoSpriteSheet::new(canvas, texture_creator, sprite_sheet_meta, block_size)?,
            board_texture,
            background_texture: bg_texture,
            geometry,
            background_size: (background_width, background_height),
            board_snip,
            hold_snip,
            peek_snips,
            font,
            score_snip: all_metrics.iter().find(|r| r.metric == GameMetricType::Score).unwrap().value,
            level_snip: all_metrics.iter().find(|r| r.metric == GameMetricType::Level).unwrap().value,
            lines_snip: all_metrics.iter().find(|r| r.metric == GameMetricType::Lines).unwrap().value,
            game_over,
            sound,
            background_color: Color::BLACK,
            destroy_animation: DestroyAnimationType::Particles { color: Color::WHITE },
            game_over_animation: GameOverAnimationType::CurtainUp,
            ghost_mino_type: MinoType::Perimeter,
            tetromino_scale_type: TetrominoScaleType::Fill {
                default_scale: TETROMINO_PREFERRED_SCALE,
                peek0_scale: BIG_TETROMINO_PREFERRED_SCALE
            },
            particle_color: Some(Color::WHITE)
        }
    )
}
