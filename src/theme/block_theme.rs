use crate::animation::destroy::DestroyAnimationType;
use crate::animation::game_over::{GameOverAnimate, GameOverAnimationType};
use crate::animation::TextureAnimate;
use crate::config::Config;
use crate::event::GameEvent;
use crate::game::block::BlockState;
use crate::game::board::{BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::geometry::Rotation;
use crate::game::random::PEEK_SIZE;
use crate::game::tetromino::{Minos, TetrominoShape};
use crate::game::Game;
use crate::theme::sound::{load_sound, play_sound};
use crate::theme::Theme;
use sdl2::image::LoadTexture;
use sdl2::mixer::{Chunk, Music};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use std::cmp::min;
use crate::theme::perimeter::PerimeterRender;

pub const VISIBLE_BUFFER: u32 = 2;
pub const VISIBLE_BOARD_HEIGHT: u32 = BOARD_HEIGHT + VISIBLE_BUFFER;

pub struct TetrominoSnips {
    pub snip: Rect,
    is_uniform: bool,
    mino_blocks: [Rect; 4],
    stack_blocks: [Rect; 4],
}

impl TetrominoSnips {
    pub fn uniform(snip: Rect, mino_block: Rect) -> Self {
        Self {
            snip,
            is_uniform: true,
            mino_blocks: [mino_block; 4],
            stack_blocks: [mino_block; 4],
        }
    }

    pub fn uniform_stack(snip: Rect, mino_block: Rect, stack_block: Rect) -> Self {
        Self {
            snip,
            is_uniform: true,
            mino_blocks: [mino_block; 4],
            stack_blocks: [stack_block; 4],
        }
    }

    pub fn asymmetrical(snip: Rect, mino_blocks: [Rect; 4]) -> Self {
        Self {
            snip,
            is_uniform: false,
            mino_blocks,
            stack_blocks: mino_blocks,
        }
    }

    // pub fn asymmetrical_stack(snip: Rect, mino_blocks: [Rect; 4], stack_blocks: [Rect; 4]) -> Self {
    //     Self {
    //         snip,
    //         is_uniform: false,
    //         mino_blocks,
    //         stack_blocks,
    //     }
    // }

    fn mino(&self, id: u32) -> Rect {
        self.mino_blocks[id as usize]
    }

    fn stack(&self, id: u32) -> Rect {
        self.stack_blocks[id as usize]
    }

    fn requires_rotation(&self) -> bool {
        !self.is_uniform
    }
}

pub struct MetricSnips {
    digits: Vec<Rect>,
    max_value: u32,
    zero_fill: bool,
}

impl MetricSnips {
    fn new(digits: Vec<Rect>, zero_fill: bool) -> Self {
        let max_string: String = (0..digits.len()).map(|_| '9').collect();
        Self {
            digits,
            max_value: max_string.parse().unwrap(),
            zero_fill,
        }
    }

    fn digit(&self, id: usize) -> Rect {
        self.digits[id]
    }

    fn format(&self, value: u32) -> String {
        let s = format!("{}", value);
        if self.zero_fill {
            let fill_len = self.digits.len() - s.len();
            let mut result: String = (0..fill_len).map(|_| '0').collect();
            result.push_str(&s);
            result
        } else {
            s
        }
    }
}

pub struct BlockThemeOptions {
    name: String,
    config: Config,
    sprite_file: String,
    background_file: String,
    board_file: String,
    game_over_file: String,
    ghost_alpha_mod: u8,
    block_size: u32,
    char_size: (u32, u32),
    num_snips: [Rect; 10],
    peek_snips: [Rect; 5],
    hold_snip: Rect,
    score: MetricSnips,
    levels: MetricSnips,
    lines: MetricSnips,
    board_point: Point,
    game_point: Point,
    board_width: u32,
    buffer_height: u32,
    board_height: u32,
    i: TetrominoSnips,
    j: TetrominoSnips,
    l: TetrominoSnips,
    o: TetrominoSnips,
    s: TetrominoSnips,
    t: TetrominoSnips,
    z: TetrominoSnips,
    garbage_block: Rect,
    background_color: Color,
    destroy_animation: DestroyAnimationType,
    game_over_animation: GameOverAnimationType,
}

fn translate_rects(rects: Vec<Rect>, dx: i32, dy: i32) -> Vec<Rect> {
    rects
        .into_iter()
        .map(|r| Rect::new(r.x + dx, r.y + dy, r.width(), r.height()))
        .collect()
}

impl BlockThemeOptions {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        config: Config,
        sprite_file: String,
        background_file: String,
        board_file: String,
        game_over_file: String,
        ghost_alpha_mod: u8,
        block_size: u32,
        char_size: (u32, u32),
        num_snips: [Rect; 10],
        peek_snips: [Rect; 5],
        hold_snip: Rect,
        score_snips: Vec<Rect>,
        level_snips: Vec<Rect>,
        lines_snips: Vec<Rect>,
        zero_fill: bool,
        board_point: Point,
        game_point: Point,
        i: TetrominoSnips,
        j: TetrominoSnips,
        l: TetrominoSnips,
        o: TetrominoSnips,
        s: TetrominoSnips,
        t: TetrominoSnips,
        z: TetrominoSnips,
        garbage_block: Rect,
        background_color: Color,
        destroy_animation: DestroyAnimationType,
        game_over_animation: GameOverAnimationType,
    ) -> Self {
        let buffer_height = VISIBLE_BUFFER * block_size;
        Self {
            name,
            config,
            sprite_file,
            background_file,
            board_file,
            game_over_file,
            ghost_alpha_mod,
            block_size,
            char_size,
            num_snips,
            peek_snips,
            hold_snip,
            score: MetricSnips::new(
                translate_rects(score_snips, 0, buffer_height as i32),
                zero_fill,
            ),
            levels: MetricSnips::new(
                translate_rects(level_snips, 0, buffer_height as i32),
                zero_fill,
            ),
            lines: MetricSnips::new(
                translate_rects(lines_snips, 0, buffer_height as i32),
                zero_fill,
            ),
            board_point,
            game_point,
            board_width: BOARD_WIDTH * block_size,
            board_height: BOARD_HEIGHT * block_size,
            buffer_height: VISIBLE_BUFFER * block_size,
            i,
            j,
            l,
            o,
            s,
            t,
            z,
            garbage_block,
            background_color,
            destroy_animation,
            game_over_animation,
        }
    }

    fn visible_height(&self) -> u32 {
        self.board_height + self.buffer_height
    }

    fn resource(&self, name: &str) -> String {
        format!("resource/{}/{}", self.name, name)
    }

    fn sprite_file(&self) -> String {
        self.resource(&self.sprite_file)
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

    fn load_music<'a>(&self) -> Result<Music<'a>, String> {
        Music::from_file(self.resource("music.ogg"))
    }

    fn load_sound(&self, name: &str) -> Result<Chunk, String> {
        load_sound(&self.name, name, self.config)
    }

    fn digit_snip(&self, digit: char) -> Rect {
        assert!(digit.is_ascii_digit());
        self.num_snips[(digit as usize) - '0' as usize]
    }

    fn j_to_y(&self, j: u32) -> u32 {
        self.visible_height() - ((j + 1) * self.block_size)
    }

    fn row_rect(&self, j: u32) -> Rect {
        Rect::new(
            self.game_point.x(),
            self.j_to_y(j) as i32,
            BOARD_WIDTH * self.block_size,
            self.block_size,
        )
    }

    fn src_row_rect(&self, j: u32) -> Rect {
        let capped_j = min(j, BOARD_HEIGHT); // the src has no buffer so protect against copying from it
        let y = self.board_height - ((capped_j + 1) * self.block_size);
        Rect::new(
            self.game_point.x(),
            y as i32,
            BOARD_WIDTH * self.block_size,
            self.block_size,
        )
    }

    fn shape(&self, shape: TetrominoShape) -> &TetrominoSnips {
        match shape {
            TetrominoShape::I => &self.i,
            TetrominoShape::O => &self.o,
            TetrominoShape::T => &self.t,
            TetrominoShape::S => &self.s,
            TetrominoShape::Z => &self.z,
            TetrominoShape::J => &self.j,
            TetrominoShape::L => &self.l,
        }
    }

    fn sprite_snip(&self, shape: TetrominoShape) -> Rect {
        self.shape(shape).snip
    }

    fn mino_dest_rect(&self, i: u32, j: u32) -> Rect {
        Rect::new(
            (i * self.block_size) as i32 + self.game_point.x(),
            self.j_to_y(j) as i32,
            self.block_size,
            self.block_size,
        )
    }

    /// returns snip of the mino in the sprite sheet and whether rotation is required
    fn mino_snip(&self, shape: TetrominoShape, mino_id: u32, is_stack: bool) -> (Rect, bool) {
        let snips = self.shape(shape);
        (
            if is_stack {
                snips.stack(mino_id)
            } else {
                snips.mino(mino_id)
            },
            snips.requires_rotation(),
        )
    }
}

pub struct BlockTheme<'a> {
    options: BlockThemeOptions,
    sprites: Texture<'a>,
    sprites_ghost: Texture<'a>,
    // perimeter: PerimeterRender<'a>,
    game_over: Texture<'a>,
    board_texture: Texture<'a>,
    board_texture_size: (u32, u32),
    bg_texture: Texture<'a>,
    bg_rect: Rect,
    music: Music<'a>,
    move_sound: Chunk,
    rotate_sound: Chunk,
    lock_sound: Chunk,
    send_garbage_sound: Chunk,
    stack_drop_sound: Option<Chunk>,
    line_clear_sound: Chunk,
    level_up_sound: Chunk,
    game_over_sound: Chunk,
    tetris_sound: Chunk,
    pause_sound: Chunk,
    victory_sound: Chunk,
}

impl<'a> BlockTheme<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        options: BlockThemeOptions,
    ) -> Result<Self, String> {
        let sprites = texture_creator.load_texture(options.sprite_file())?;

        // ghost resource are just lightened resource
        let sprites_query = sprites.query();
        let mut sprites_ghost = texture_creator
            .create_texture_target(None, sprites_query.width, sprites_query.height)
            .map_err(|e| e.to_string())?;
        sprites_ghost.set_blend_mode(BlendMode::Blend);
        sprites_ghost.set_alpha_mod(options.ghost_alpha_mod);
        canvas
            .with_texture_canvas(&mut sprites_ghost, |c| {
                c.copy(&sprites, None, None).unwrap();
            })
            .map_err(|e| e.to_string())?;

        // let perimeter = PerimeterRender::new(canvas, texture_creator, options.block_size)?;

        let board_texture = texture_creator.load_texture(options.board_file())?;
        let board_query = board_texture.query();

        let bg_texture = texture_creator.load_texture(options.background_file())?;
        let bg_query = bg_texture.query();
        let bg_rect = Rect::new(
            0,
            options.buffer_height as i32,
            bg_query.width,
            bg_query.height,
        );

        let game_over = texture_creator.load_texture(options.game_over_file())?;

        let music = options.load_music()?;
        let move_sound = options.load_sound("move")?;
        let rotate_sound = options.load_sound("rotate")?;
        let lock_sound = options.load_sound("lock")?;
        let send_garbage_sound = options.load_sound("send-garbage")?;
        let stack_drop_sound = options.load_sound("stack-drop").ok(); // optional
        let line_clear_sound = options.load_sound("line-clear")?;
        let level_up_sound = options.load_sound("level-up")?;
        let game_over_sound = options.load_sound("game-over")?;
        let tetris_sound = options.load_sound("tetris")?;
        let pause_sound = options.load_sound("pause")?;
        let victory_sound = options.load_sound("victory")?;

        Ok(Self {
            options,
            sprites,
            sprites_ghost,
            game_over,
            board_texture,
            board_texture_size: (board_query.width, board_query.height),
            bg_texture,
            bg_rect,
            music,
            move_sound,
            rotate_sound,
            lock_sound,
            send_garbage_sound,
            stack_drop_sound,
            line_clear_sound,
            level_up_sound,
            game_over_sound,
            tetris_sound,
            pause_sound,
            victory_sound,
        })
    }

    fn draw_tetromino(
        &self,
        canvas: &mut WindowCanvas,
        sprites: &Texture,
        rect: &Rect,
        shape: TetrominoShape,
    ) -> Result<(), String> {
        let sprite_snip = self.options.sprite_snip(shape);
        let dest_rect = Rect::from_center(rect.center(), sprite_snip.width(), sprite_snip.height());
        canvas.copy(sprites, sprite_snip, dest_rect)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_mino(
        &self,
        canvas: &mut WindowCanvas,
        sprites: &Texture,
        i: u32,
        j: u32,
        shape: TetrominoShape,
        rotation: Rotation,
        mino_id: u32,
        is_stack: bool,
    ) -> Result<(), String> {
        let dest_rect = self.options.mino_dest_rect(i, j);
        let (mino_snip, rotate) = self.options.mino_snip(shape, mino_id, is_stack);
        if rotate {
            canvas.copy_ex(
                sprites,
                mino_snip,
                dest_rect,
                rotation.angle(),
                None,
                false,
                false,
            )
        } else {
            canvas.copy(sprites, mino_snip, dest_rect)
        }
    }

    fn draw_garbage(&self, canvas: &mut WindowCanvas, i: u32, j: u32) -> Result<(), String> {
        let dest_rect = self.options.mino_dest_rect(i, j);
        canvas.copy(&self.sprites, self.options.garbage_block, dest_rect)
    }
}

impl<'a> Theme for BlockTheme<'a> {
    fn max_score(&self) -> u32 {
        self.options.score.max_value
    }

    fn block_size(&self) -> u32 {
        self.options.block_size
    }

    fn background_color(&self) -> Color {
        self.options.background_color
    }

    fn background_size(&self) -> (u32, u32) {
        (
            self.bg_rect.width(),
            self.bg_rect.height() + self.options.buffer_height,
        )
    }

    fn board_snip(&self) -> Rect {
        let (w, h) = self.board_texture_size;
        Rect::new(
            self.options.board_point.x(),
            self.options.board_point.y(),
            w,
            h + self.options.buffer_height,
        )
    }

    fn draw_background(&mut self, canvas: &mut WindowCanvas, game: &Game) -> Result<(), String> {
        let metrics = game.metrics();
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        // background
        canvas.copy(&self.bg_texture, None, self.bg_rect)?;

        let score = self.options.score.format(metrics.score);
        for (index, char) in score.chars().rev().enumerate() {
            canvas.copy(
                &self.sprites,
                self.options.digit_snip(char),
                self.options.score.digit(index),
            )?;
        }

        let level = self.options.levels.format(metrics.level);
        for (index, char) in level.chars().rev().enumerate() {
            canvas.copy(
                &self.sprites,
                self.options.digit_snip(char),
                self.options.levels.digit(index),
            )?;
        }

        let lines = self.options.lines.format(metrics.lines);
        for (index, char) in lines.chars().rev().enumerate() {
            canvas.copy(
                &self.sprites,
                self.options.digit_snip(char),
                self.options.lines.digit(index),
            )?;
        }

        for i in 0..(min(PEEK_SIZE, self.options.peek_snips.len())) {
            self.draw_tetromino(
                canvas,
                &self.sprites,
                &self.options.peek_snips[i],
                metrics.queue[i],
            )?;
        }

        match metrics.hold {
            None => {}
            Some(shape) => {
                self.draw_tetromino(canvas, &self.sprites, &self.options.hold_snip, shape)?;
            }
        }
        Ok(())
    }

    fn draw_board(
        &mut self,
        canvas: &mut WindowCanvas,
        game: &Game,
        animate_lines: Vec<(u32, TextureAnimate)>,
        animate_game_over: Option<GameOverAnimate>,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        let (board_width, board_height) = self.board_texture_size;
        canvas.copy(
            &self.board_texture,
            None,
            Rect::new(
                0,
                self.options.buffer_height as i32,
                board_width,
                board_height,
            ),
        )?;

        let (curtain_range, render_board) = match animate_game_over {
            Some(animate) => match animate {
                GameOverAnimate::CurtainClosing(range) => (range, true),
                GameOverAnimate::CurtainOpening(range) => (range, false),
                GameOverAnimate::Finished => (0..0, false),
            },
            _ => (0..0, true),
        };

        if render_board {
            for j in 0..VISIBLE_BOARD_HEIGHT {
                for (i, block) in game.row(j).iter().enumerate() {
                    match block {
                        BlockState::Empty => {}
                        BlockState::Tetromino(shape, rotation, mino_id) => {
                            self.draw_mino(
                                canvas,
                                &self.sprites,
                                i as u32,
                                j,
                                *shape,
                                *rotation,
                                *mino_id,
                                false,
                            )?;
                        }
                        BlockState::Ghost(shape, rotation, mino_id) => {
                            // self.perimeter.draw_mino(canvas, *shape, *rotation, *mino_id, self.options.mino_dest_rect(i as u32, j))?;
                            self.draw_mino(
                                canvas,
                                &self.sprites_ghost,
                                i as u32,
                                j,
                                *shape,
                                *rotation,
                                *mino_id,
                                false,
                            )?;
                        }
                        BlockState::Stack(shape, rotation, mino_id) => {
                            self.draw_mino(
                                canvas,
                                &self.sprites,
                                i as u32,
                                j,
                                *shape,
                                *rotation,
                                *mino_id,
                                true,
                            )?;
                        }
                        BlockState::Garbage => {
                            self.draw_garbage(canvas, i as u32, j)?;
                        }
                    }
                }

                // post draw animate
                let animate_line = animate_lines
                    .iter()
                    .find(|(line, _)| *line == j)
                    .map(|(_, animate)| animate);
                match animate_line {
                    None => {}
                    Some(animate) => {
                        match animate {
                            TextureAnimate::SetAlpha => {
                                // simulate alpha by copying over the board background
                                canvas.copy(
                                    &self.board_texture,
                                    self.options.src_row_rect(j),
                                    self.options.row_rect(j),
                                )?;
                            }
                            TextureAnimate::FillAlphaRectangle { width } => {
                                // simulate alpha by copying over the board background
                                let row_rect = self.options.row_rect(j);
                                let rect_width = (row_rect.width() as f64 * width).round() as u32;
                                let dst_rect = Rect::from_center(
                                    row_rect.center(),
                                    rect_width,
                                    row_rect.height(),
                                );
                                let src_rect = Rect::from_center(
                                    self.options.src_row_rect(j).center(),
                                    rect_width,
                                    row_rect.height(),
                                );
                                canvas.copy(&self.board_texture, src_rect, dst_rect)?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            let game_over_query = self.game_over.query();
            let game_snip = Rect::new(
                self.options.game_point.x(),
                self.options.game_point.y(),
                self.options.board_width,
                self.options.board_height,
            );
            let game_over_snip = Rect::from_center(
                game_snip.center(),
                game_over_query.width,
                game_over_query.height,
            );
            canvas.copy(&self.game_over, None, game_over_snip)?;
        }

        for j in curtain_range {
            for x in 0..BOARD_WIDTH {
                self.draw_garbage(canvas, x, j)?;
            }
        }

        Ok(())
    }

    fn line_snip(&self, j: u32) -> Rect {
        self.options.row_rect(j)
    }

    fn destroy_animation_type(&self) -> DestroyAnimationType {
        self.options.destroy_animation
    }

    fn game_over_animation_type(&self) -> GameOverAnimationType {
        self.options.game_over_animation
    }

    fn music(&self) -> &Music {
        &self.music
    }

    fn receive_event(&mut self, event: GameEvent) -> Result<(), String> {
        match event {
            GameEvent::Move => play_sound(&self.move_sound),
            GameEvent::Rotate => play_sound(&self.rotate_sound),
            GameEvent::Lock => play_sound(&self.lock_sound),
            GameEvent::Destroy(lines) => {
                let mut count = 0;
                for line in lines {
                    if line.is_some() {
                        count += 1;
                    } else {
                        break;
                    }
                }
                if count >= 4 {
                    play_sound(&self.tetris_sound)
                } else if count > 0 {
                    play_sound(&self.line_clear_sound)
                } else {
                    Ok(())
                }
            }
            GameEvent::Destroyed { level_up, .. } => {
                if level_up {
                    play_sound(&self.level_up_sound)
                } else if self.stack_drop_sound.is_some() {
                    play_sound(self.stack_drop_sound.as_ref().unwrap())
                } else {
                    Ok(())
                }
            }
            GameEvent::ReceivedGarbage => play_sound(&self.send_garbage_sound),
            GameEvent::GameOver(_) => play_sound(&self.game_over_sound),
            GameEvent::Victory => play_sound(&self.victory_sound),
            GameEvent::Paused => play_sound(&self.pause_sound),
            _ => Ok(()),
        }
    }

    fn mino_rects(&self, minos: Minos) -> [Rect; 4] {
        minos.map(|mino| self.options.mino_dest_rect(mino.x as u32, mino.y as u32))
    }
}
