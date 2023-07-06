use std::collections::HashMap;
use bitflags::Flags;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;
use crate::game::geometry::Rotation;
use crate::game::tetromino::{Corner, Minos, Perimeter, TetrominoShape};

struct PerimeterTexture<'a> {
    texture: Texture<'a>,
    minos: [Rect; 4]
}

pub struct PerimeterRender<'a> {
    textures: HashMap<TetrominoShape, PerimeterTexture<'a>>
}

impl<'a> PerimeterRender<'a> {
    pub fn new(canvas: &mut WindowCanvas, texture_creator: &'a TextureCreator<WindowContext>, block_size: u32) -> Result<Self, String> {
        let mut textures = HashMap::new();
        for shape in TetrominoShape::ALL.iter().copied() {
            let meta = shape.meta();
            let size = meta.bounding_box() * block_size;
            let mut texture = texture_creator
                .create_texture_target(None, size, size)
                .map_err(|e| e.to_string())?;
            texture.set_blend_mode(BlendMode::Blend);
            texture.set_alpha_mod(0x60);
            let rects = meta.minos().iter()
                .map(|p| Rect::new(
                    p.x * block_size as i32, p.y * block_size as i32,
                    block_size, block_size
                ))
                .collect::<Vec<Rect>>();
            canvas.with_texture_canvas(&mut texture, |c| {
                c.set_draw_color(Color::RGBA(0, 0, 0, 0));
                c.clear();
                c.set_draw_color(Color::WHITE);
                for ((rect, perimeter), outside_corner) in rects.iter().zip(meta.perimeter()).zip(meta.outside_corners()) {
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
            textures.insert(shape, PerimeterTexture { texture, minos: rects.try_into().unwrap() });
        }
        Ok(Self { textures })
    }

    pub fn draw_mino(&self, canvas: &mut WindowCanvas, shape: TetrominoShape, rotation: Rotation, mino_id: u32, dest: Rect) -> Result<(), String> {
        let pt = self.textures.get(&shape).unwrap();
        // TODO rotation
        let rect = pt.minos[mino_id as usize];
        // canvas.copy(&pt.texture, rect, dest)
        canvas.copy_ex(&pt.texture, rect, dest, rotation.angle(), None, false, false)
    }
}