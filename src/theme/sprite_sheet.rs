use std::collections::HashMap;
use bitflags::Flags;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::game::block::BlockState;
use crate::game::Game;
use crate::game::geometry::Rotation;
use crate::game::tetromino::{Corner, Perimeter, TetrominoShape};
use crate::theme::geometry::{BoardGeometry, VISIBLE_BOARD_HEIGHT};

struct TetrominoTexture<'a> {
    perimeter: Texture<'a>,
    normal: Texture<'a>,
    ghost: Texture<'a>,
    stack: Texture<'a>,
    snips: [Rect; 4],
    width: u32,
    height: u32,
    is_symmetrical: bool
}

impl<'a> TetrominoTexture<'a> {
    fn texture(&self, mino_type: MinoType) -> &Texture {
        match mino_type {
            MinoType::Normal => &self.normal,
            MinoType::Ghost => &self.ghost,
            MinoType::Stack => &self.stack,
            MinoType::Perimeter => &self.perimeter,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MinoType {
    Normal,
    Ghost,
    Stack,
    Perimeter,
    Garbage
}

#[derive(Debug, Clone)]
pub struct TetrominoSpriteSheetMeta {
    sprite_file: String,
    source_block_size: u32,
    i: MinoPoints,
    j: MinoPoints,
    l: MinoPoints,
    o: MinoPoints,
    s: MinoPoints,
    t: MinoPoints,
    z: MinoPoints,
    garbage: Point,
    ghost_alpha: u8
}

#[derive(Debug, Copy, Clone)]
struct MinoPoints {
    normal: [Point; 4],
    stack: [Point; 4],
    is_symmetrical: bool
}

impl MinoPoints {
    fn of_type(&self, mino_type: MinoType) -> [Point; 4] {
        match mino_type {
            MinoType::Normal => self.normal,
            MinoType::Stack => self.stack,
            _ => unreachable!(),
        }
    }
}

impl From<Point> for MinoPoints {
    fn from(value: Point) -> Self {
        Self { normal: [value; 4], stack: [value; 4], is_symmetrical: true }
    }
}

impl From<(Point, Point)> for MinoPoints {
    fn from((mino, stack): (Point, Point)) -> Self {
        Self { normal: [mino; 4], stack: [stack; 4], is_symmetrical: true }
    }
}

impl From<[Point; 4]> for MinoPoints {
    fn from(value: [Point; 4]) -> Self {
        Self { normal: value.clone(), stack: value.clone(), is_symmetrical: false }
    }
}

impl From<([Point; 4], [Point; 4])> for MinoPoints {
    fn from((minos, stack): ([Point; 4], [Point; 4])) -> Self {
        Self { normal: minos, stack, is_symmetrical: false }
    }
}

impl TetrominoSpriteSheetMeta {
    pub fn new<I, J, L, O, S, T, Z, G>(
        sprite_file: &str,
        source_block_size: u32,
        i: I,
        j: J,
        l: L,
        o: O,
        s: S,
        t: T,
        z: Z,
        garbage: G,
        ghost_alpha: u8
    ) -> Self
    where I : Into<MinoPoints>,
          J : Into<MinoPoints>,
          L : Into<MinoPoints>,
          O : Into<MinoPoints>,
          S : Into<MinoPoints>,
          T : Into<MinoPoints>,
          Z : Into<MinoPoints>,
          G : Into<Point> {
        Self {
            sprite_file: sprite_file.to_string(),
            source_block_size,
            i: i.into(),
            j: j.into(),
            l: l.into(),
            o: o.into(),
            s: s.into(),
            t: t.into(),
            z: z.into(),
            garbage: garbage.into(),
            ghost_alpha
        }
    }

    fn snips(&self, shape: TetrominoShape, mino_type: MinoType) -> [Rect; 4] {
        let points = self.shape(shape).of_type(mino_type);
        points.iter()
            .map(|p| Rect::new(p.x(), p.y(), self.source_block_size, self.source_block_size))
            .collect::<Vec<Rect>>()
            .try_into()
            .unwrap()
    }

    fn shape(&self, shape: TetrominoShape) -> &MinoPoints {
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

    fn garbage_snip(&self) -> Rect {
        Rect::new(self.garbage.x(), self.garbage.y(), self.source_block_size, self.source_block_size)
    }

    pub fn block_size(&self) -> u32 {
        self.source_block_size
    }
}

pub struct FlatTetrominoSpriteSheet<'a> {
    texture: Texture<'a>,
    snips: HashMap<TetrominoShape, Rect>
}

impl<'a> FlatTetrominoSpriteSheet<'a> {
    pub fn texture(&self) -> &Texture<'a> {
        &self.texture
    }

    pub fn snip(&self, shape: TetrominoShape) -> Rect {
        self.snips[&shape]
    }
}

pub struct TetrominoSpriteSheet<'a> {
    tetrominos: HashMap<TetrominoShape, TetrominoTexture<'a>>,
    block_size: u32,
    garbage: Texture<'a>
}

fn draw_sprites<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    sprites: &Texture,
    src_snips: [Rect; 4],
    mino_rects: [Rect; 4],
    width: u32,
    height: u32
) -> Result<Texture<'a>, String> {
    let mut texture = texture_creator
        .create_texture_target(None, width, height)
        .map_err(|e| e.to_string())?;
    texture.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut texture, |c| {
        c.set_draw_color(Color::RGBA(0, 0, 0, 0));
        c.clear();
        for (src, dest) in src_snips.iter().copied().zip(mino_rects) {
            c.copy(sprites, src, dest).unwrap();
        }
    }).map_err(|e| e.to_string())?;
    Ok(texture)
}

fn draw_perimeter<'a>(
    canvas: &mut WindowCanvas,
    texture_creator: &'a TextureCreator<WindowContext>,
    shape: TetrominoShape,
    mino_rects: [Rect; 4],
    width: u32,
    height: u32
) -> Result<Texture<'a>, String> {
    let meta = shape.meta();
    let mut texture = texture_creator
        .create_texture_target(None, width, height)
        .map_err(|e| e.to_string())?;
    texture.set_blend_mode(BlendMode::Blend);
    canvas.with_texture_canvas(&mut texture, |c| {
        c.set_draw_color(Color::RGBA(0, 0, 0, 0));
        c.clear();
        c.set_draw_color(Color::WHITE);
        for ((rect, perimeter), outside_corner) in mino_rects.iter().zip(meta.perimeter()).zip(meta.outside_corners()) {
            let top_left = rect.top_left();
            let top_right = rect.top_right() - Point::new(1, 0);
            let bottom_right = rect.bottom_right() - Point::new(1, 1);
            let bottom_left = rect.bottom_left() - Point::new(0, 1);

            if outside_corner.contains(Corner::TopLeft) {
                c.draw_point(top_left).unwrap();
            }
            if outside_corner.contains(Corner::TopRight) {
                c.draw_point(top_right).unwrap();
            }
            if outside_corner.contains(Corner::BottomRight) {
                c.draw_point(bottom_right).unwrap();
            }
            if outside_corner.contains(Corner::BottomLeft) {
                c.draw_point(bottom_left).unwrap();
            }

            if perimeter.contains(Perimeter::Top) {
                c.draw_line(top_left, top_right).unwrap();
            }
            if perimeter.contains(Perimeter::Right) {
                c.draw_line(top_right, bottom_right).unwrap();
            }
            if perimeter.contains(Perimeter::Bottom) {
                c.draw_line(bottom_left, bottom_right).unwrap();
            }
            if perimeter.contains(Perimeter::Left) {
                c.draw_line(top_left, bottom_left).unwrap();
            }
        }
    }).map_err(|e| e.to_string())?;
    Ok(texture)
}

impl<'a> TetrominoSpriteSheet<'a> {
    pub fn new(
        canvas: &mut WindowCanvas,
        texture_creator: &'a TextureCreator<WindowContext>,
        meta: TetrominoSpriteSheetMeta,
        block_size: u32,
    ) -> Result<Self, String> {
        let sprite_src = texture_creator.load_texture(&meta.sprite_file)?;

        let mut textures = HashMap::new();
        for shape in TetrominoShape::ALL.iter().copied() {
            let normal_minos = shape.meta().normal_minos();
            let width = (normal_minos.iter().map(|p| p.x).max().unwrap() + 1) as u32 * block_size;
            let height_j = (normal_minos.iter().map(|p| p.y).max().unwrap() + 1) as u32;
            let height = height_j * block_size;

            let mino_rects: [Rect; 4] = normal_minos.iter()
                .map(|p| Rect::new(
                    p.x * block_size as i32, (height_j as i32 - p.y - 1) * block_size as i32,
                    block_size, block_size
                ))
                .collect::<Vec<Rect>>()
                .try_into()
                .unwrap();

            let normal = draw_sprites(
                canvas,
                texture_creator,
                &sprite_src,
                meta.snips(shape, MinoType::Normal),
                mino_rects.clone(),
                width,
                height
            )?;

            let mut ghost = texture_creator
                .create_texture_target(None, width, height)
                .map_err(|e| e.to_string())?;
            ghost.set_blend_mode(BlendMode::Blend);
            ghost.set_alpha_mod(meta.ghost_alpha);
            canvas.with_texture_canvas(&mut ghost, |c| {
                c.copy(&normal, None, None).unwrap();
            }).map_err(|e| e.to_string())?;

            textures.insert(shape, TetrominoTexture {
                perimeter: draw_perimeter(canvas, texture_creator, shape, mino_rects.clone(), width, height)?,
                normal: draw_sprites(
                    canvas,
                    texture_creator,
                    &sprite_src,
                    meta.snips(shape, MinoType::Normal),
                    mino_rects.clone(),
                    width,
                    height
                )?,
                ghost,
                stack: draw_sprites(
                    canvas,
                    texture_creator,
                    &sprite_src,
                    meta.snips(shape, MinoType::Stack),
                    mino_rects.clone(),
                    width,
                    height
                )?,
                snips: mino_rects,
                width,
                height,
                is_symmetrical: meta.shape(shape).is_symmetrical
            });
        }

        let mut garbage = texture_creator
            .create_texture_target(None, block_size, block_size)
            .map_err(|e| e.to_string())?;
        garbage.set_blend_mode(BlendMode::Blend);
        canvas.with_texture_canvas(&mut garbage, |c| {
            c.copy(&sprite_src, meta.garbage_snip(), None).unwrap();
        }).map_err(|e| e.to_string())?;

        Ok(Self { tetrominos: textures, block_size, garbage })
    }

    pub fn draw_perimeter(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, rotation: Rotation, mino_id: u32, dest: Point) -> Result<(), String> {
        self.draw(canvas, shape, rotation, mino_id, dest, MinoType::Perimeter)
    }

    pub fn draw_mino(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, rotation: Rotation, mino_id: u32, dest: Point) -> Result<(), String> {
        self.draw(canvas, shape, rotation, mino_id, dest, MinoType::Normal)
    }

    pub fn draw_ghost(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, rotation: Rotation, mino_id: u32, dest: Point) -> Result<(), String> {
        self.draw(canvas, shape, rotation, mino_id, dest, MinoType::Ghost)
    }

    pub fn draw_stack(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, rotation: Rotation, mino_id: u32, dest: Point) -> Result<(), String> {
        self.draw(canvas, shape, rotation, mino_id, dest, MinoType::Stack)
    }

    pub fn draw_garbage(&self, canvas: &mut WindowCanvas, dest: Point) -> Result<(), String> {
        canvas.copy(&self.garbage, None, self.mino_rect(dest))
    }

    pub fn draw_tetromino_in_center(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, mino_type: MinoType, dest: Point) -> Result<(), String> {
        let tetromino = self.tetrominos.get(&shape).unwrap();
        let rect = Rect::from_center(dest, tetromino.width, tetromino.height);
        canvas.copy(tetromino.texture(mino_type), None, rect)
    }

    pub fn draw_tetromino_fill(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, mino_type: MinoType, dest: Rect, max_scale: f64) -> Result<(), String> {
        let tetromino = self.tetrominos.get(&shape).unwrap();
        let scale_x = dest.width() as f64 / tetromino.width as f64;
        let scale_y = dest.height() as f64 / tetromino.height as f64;
        let scale = scale_x.min(scale_y).min(max_scale);
        let rect = Rect::from_center(
            dest.center(),
            (scale * tetromino.width as f64).round() as u32,
            (scale * tetromino.height as f64).round() as u32
        );
        canvas.copy(tetromino.texture(mino_type), None, rect)
    }

    pub fn draw_board(&self, canvas: &mut WindowCanvas, game: &Game, geometry: &BoardGeometry, ghost_type: MinoType) -> Result<(), String> {
        for j in 0..VISIBLE_BOARD_HEIGHT {
            for (i, block) in game.row(j).iter().copied().enumerate() {
                let point = geometry.mino_point(i as u32, j);
                match block {
                    BlockState::Empty => {}
                    BlockState::Tetromino(shape, rotation, mino_id) => {
                        self.draw_mino(canvas, shape, rotation, mino_id, point)?;
                    }
                    BlockState::Ghost(shape, rotation, mino_id) => {
                        self.draw(canvas, shape, rotation, mino_id, point, ghost_type)?;
                    }
                    BlockState::Stack(shape, rotation, mino_id) => {
                        self.draw_stack(canvas, shape, rotation, mino_id, point)?;
                    }
                    BlockState::Garbage => {
                        self.draw_garbage(canvas, point)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, rotation: Rotation, mino_id: u32, dest: Point, mino_type: MinoType) -> Result<(), String> {
        let tetromino = self.tetrominos.get(&shape).unwrap();
        let snip = tetromino.snips[mino_id as usize];
        let dest = self.mino_rect(dest);
        let texture = tetromino.texture(mino_type);
        if mino_type != MinoType::Perimeter && tetromino.is_symmetrical {
            canvas.copy(texture, snip, dest)
        } else {
            canvas.copy_ex(texture, snip, dest, rotation.angle(), None, false, false)
        }
    }

    fn mino_rect(&self, point: Point) -> Rect {
        Rect::new(point.x(), point.y(), self.block_size, self.block_size)
    }

    pub fn flatten<'b>(
        &self,
        canvas: &mut WindowCanvas,
        texture_creator: &'b TextureCreator<WindowContext>,
        mino_type: MinoType
    ) -> Result<FlatTetrominoSpriteSheet<'b>, String> {
        let sizes = TetrominoShape::ALL.map(|s| {
            let tetromino = self.tetrominos.get(&s).unwrap();
            (tetromino.width, tetromino.height)
        });
        let width = sizes.map(|(w, _)| w).into_iter().sum::<u32>();
        let height = sizes.map(|(_, h)| h).into_iter().max().unwrap();
        let mut texture = texture_creator.create_texture_target(None, width, height)
            .map_err(|e| e.to_string())?;
        texture.set_blend_mode(BlendMode::Blend);

        let mut snips = HashMap::new();
        canvas.with_texture_canvas(&mut texture, |c| {
            let mut x = 0;
            for (shape, tetromino) in self.tetrominos.iter() {
                let rect = Rect::new(x, 0, tetromino.width, tetromino.height);
                snips.insert(*shape, rect);
                c.copy(tetromino.texture(mino_type), None, rect).unwrap();
                x += tetromino.width as i32;
            }
        }).map_err(|e| e.to_string())?;

        Ok(FlatTetrominoSpriteSheet { texture, snips })
    }
}


